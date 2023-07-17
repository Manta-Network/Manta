// Copyright 2020-2023 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

use super::{
    assets_config::CalamariAssetConfig, AssetManager, Assets, Balances, DmpQueue,
    EnsureRootOrMoreThanHalfCouncil, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
    RuntimeCall, RuntimeEvent, RuntimeOrigin, Treasury, XcmpQueue, MAXIMUM_BLOCK_WEIGHT,
};
use codec::{Decode, Encode};
use core::marker::PhantomData;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use frame_support::{
    match_types, parameter_types,
    traits::{Contains, Currency, Everything, Nothing},
    weights::Weight,
};
use frame_system::EnsureRoot;
use manta_primitives::{
    assets::AssetIdLocationConvert,
    types::{AccountId, Balance, CalamariAssetId},
    xcm::{
        AccountIdToMultiLocation, AllowTopLevelPaidExecutionDescendOriginFirst, FirstAssetTrader,
        IsNativeConcrete, MultiAssetAdapter, MultiNativeAsset,
    },
};
use orml_traits::location::AbsoluteReserveProvider;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use scale_info::TypeInfo;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;
use xcm::latest::prelude::*;
use xcm_builder::{
    Account32Hash, AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId,
    EnsureXcmOrigin, FixedRateOfFungible, LocationInverter, ParentAsSuperuser, ParentIsPreset,
    RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue,
    TakeWeightCredit, WeightInfoBounds,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type OnSystemEvent = ();
    type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub SelfReserve: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
    // Converts multilocation into a 32 byte hash for local `AccountId`s
    Account32Hash<RelayNetwork, AccountId>,
);

/// This is the type to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`.
/// It uses some Rust magic macro to do the pattern matching sequentially.
/// There is an `OriginKind` which can biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognised.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognised.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<RuntimeOrigin>,
    // If the incoming XCM origin is of type `AccountId32` and the Network is Network::Any
    // or `RelayNetwork`, convert it to a Native 32 byte account.
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    /// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: u64 = 1_000_000_000;
    /// Used in native traders
    /// This might be able to skipped.
    /// We have to use `here()` because of reanchoring logic
    pub ParaTokenPerSecond: (xcm::v2::AssetId, u128) = (Concrete(MultiLocation::here()), 1_000_000_000);
    pub const MaxInstructions: u32 = 100;
}

/// Transactor for the native asset which implements `fungible` trait, as well as
/// Transactor for assets in pallet-assets, i.e. implements `fungibles` trait
pub type MultiAssetTransactor = MultiAssetAdapter<
    Runtime,
    // Used to find the query the native asset id of the chain.
    CalamariAssetConfig,
    // "default" implementation of converting a `MultiLocation` to an `AccountId`
    LocationToAccountId,
    // Used when the incoming asset is a fungible concrete asset matching the given location or name:
    IsNativeConcrete<SelfReserve>,
    // Used to match incoming assets which are not the native asset.
    ConvertedConcreteAssetId<
        CalamariAssetId,
        Balance,
        AssetIdLocationConvert<AssetManager>,
        JustTry,
    >,
>;

match_types! {
    pub type ParentLocation: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here }
    };
}
match_types! {
    pub type ParentOrSiblings: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here } |
        MultiLocation { parents: 1, interior: X1(_) }
    };
}

pub type Barrier = (
    // Allows local origin messages which call weight_credit >= weight_limit.
    TakeWeightCredit,
    // Allows execution of Transact XCM instruction from configurable set of origins
    // as long as the message is in the format DescendOrigin + WithdrawAsset + BuyExecution
    AllowTopLevelPaidExecutionDescendOriginFirst<Everything>,
    // Allows non-local origin messages, for example from from the xcmp queue,
    // which have the ability to deposit assets and pay for their own execution.
    AllowTopLevelPaidExecutionFrom<Everything>,
    // Parent root gets free execution
    AllowUnpaidExecutionFrom<ParentLocation>,
    // Expected responses are OK.
    // Allows `Pending` or `VersionNotifier` query responses.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    // Allows execution of `SubscribeVersion` or `UnsubscribeVersion` instruction,
    // from parent or sibling chains.
    AllowSubscriptionsFrom<ParentOrSiblings>,
);

parameter_types! {
    pub XcmFeesAccount: AccountId = Treasury::account_id();
}

/// Xcm fee of native token
pub struct XcmNativeFeeToTreasury;

impl TakeRevenue for XcmNativeFeeToTreasury {
    #[inline]
    fn take_revenue(revenue: MultiAsset) {
        if let MultiAsset {
            id: Concrete(location),
            fun: Fungible(amount),
        } = revenue
        {
            if location == MultiLocation::here() {
                let _ = Balances::deposit_creating(&XcmFeesAccount::get(), amount);
            }
        }
    }
}

pub type XcmFeesToAccount = manta_primitives::xcm::XcmFeesToAccount<
    AccountId,
    Assets,
    ConvertedConcreteAssetId<
        CalamariAssetId,
        Balance,
        AssetIdLocationConvert<AssetManager>,
        JustTry,
    >,
    XcmFeesAccount,
>;

pub struct XcmExecutorConfig;
impl Config for XcmExecutorConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    // Defines how to Withdraw and Deposit instruction work
    type AssetTransactor = MultiAssetTransactor;
    type OriginConverter = XcmOriginToCallOrigin;
    // Combinations of (Location, Asset) pairs which we trust as reserves.
    type IsReserve = MultiNativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = WeightInfoBounds<
        crate::weights::xcm::CalamariXcmWeight<RuntimeCall>,
        RuntimeCall,
        MaxInstructions,
    >;
    // Trader is the means to purchasing weight credit for XCM execution.
    // We define two traders:
    // The first one will charge parachain's native currency, who's `MultiLocation`
    // is defined in `SelfReserve`.
    // The second one will charge the first asset in the MultiAssets with pre-defined rate
    // i.e. units_per_second in `AssetManager`
    type Trader = (
        FixedRateOfFungible<ParaTokenPerSecond, XcmNativeFeeToTreasury>,
        FirstAssetTrader<AssetManager, XcmFeesToAccount>,
    );
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    // This is needed for the version change notifier work
    type SubscriptionService = PolkadotXcm;
}

/// Converts a Signed Local Origin into a MultiLocation
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
    /// This means that no location will pass XcmExecuteFilter, so a dispatched `execute` message will be filtered.
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = WeightInfoBounds<
        crate::weights::xcm::CalamariXcmWeight<RuntimeCall>,
        RuntimeCall,
        MaxInstructions,
    >;
    type LocationInverter = LocationInverter<Ancestry>;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRootOrMoreThanHalfCouncil;
    type ControllerOriginConverter = XcmOriginToCallOrigin;
    type WeightInfo = crate::weights::cumulus_pallet_xcmp_queue::SubstrateWeight<Runtime>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

/// We wrap AssetId for XToken
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
    /// Assets registered in pallet-assets
    MantaCurrency(CalamariAssetId),
}

/// Maps a xTokens CurrencyId to a xcm MultiLocation implemented by some asset manager
pub struct CurrencyIdtoMultiLocation<AssetXConverter>(PhantomData<AssetXConverter>);

impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
    for CurrencyIdtoMultiLocation<AssetXConverter>
where
    AssetXConverter: xcm_executor::traits::Convert<MultiLocation, CalamariAssetId>,
{
    fn convert(currency: CurrencyId) -> Option<MultiLocation> {
        match currency {
            CurrencyId::MantaCurrency(asset_id) => match AssetXConverter::reverse_ref(asset_id) {
                Ok(location) => Some(location),
                Err(_) => None,
            },
        }
    }
}

parameter_types! {
    pub const BaseXcmWeight: u64 = 100_000_000;
    pub const MaxAssetsForTransfer: usize = 2;
}

impl Contains<CurrencyId> for AssetManager {
    fn contains(id: &CurrencyId) -> bool {
        let asset_id =
            CurrencyIdtoMultiLocation::<AssetIdLocationConvert<AssetManager>>::convert(id.clone());
        Self::check_outgoing_assets_filter(&asset_id)
    }
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type CurrencyIdConvert = CurrencyIdtoMultiLocation<AssetIdLocationConvert<AssetManager>>;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type SelfLocation = SelfReserve;

    /// Weigher Configuration
    ///
    /// Take note that this pallet does not have the typical configurable WeightInfo.
    /// It uses the Weigher configuration to calculate weights for the user callable
    /// extrinsics on this chain, as well as weights for execution on the destination
    /// chain. Both based on the composed xcm messages.
    type Weigher = WeightInfoBounds<
        crate::weights::xcm::CalamariXcmWeight<RuntimeCall>,
        RuntimeCall,
        MaxInstructions,
    >;

    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = AssetManager;
    type MultiLocationsFilter = AssetManager;
    type OutgoingAssetsFilter = AssetManager;
    type ReserveProvider = AbsoluteReserveProvider;
}
