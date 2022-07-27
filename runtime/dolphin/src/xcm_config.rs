// Copyright 2020-2022 Manta Network.
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
    assets_config::{DolphinAssetConfig, DolphinConcreteFungibleLedger},
    AssetManager, Assets, Call, DmpQueue, EnsureRootOrMoreThanHalfCouncil, Event, Origin,
    ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, Treasury, XcmpQueue,
    MAXIMUM_BLOCK_WEIGHT,
};

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use sp_std::prelude::*;

use frame_support::{
    match_types, parameter_types,
    traits::{Everything, Nothing},
    weights::Weight,
};
use frame_system::EnsureRoot;
use manta_primitives::{
    assets::{AssetIdLocationConvert, AssetLocation},
    types::{AccountId, AssetId, Balance},
    xcm::{
        AccountIdToMultiLocation, FirstAssetTrader, IsNativeConcrete, MultiAssetAdapter,
        MultiNativeAsset,
    },
};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;

use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId,
    EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds, LocationInverter, ParentAsSuperuser,
    ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
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

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
    pub const KsmLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
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
);

/// This is the type to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`.
/// It uses some Rust magic macro to do the pattern matching sequentially.
/// There is an `OriginKind` which can biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognised.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognised.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<Origin>,
    // If the incoming XCM origin is of type `AccountId32` and the Network is Network::Any
    // or `RelayNetwork`, convert it to a Native 32 byte account.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

parameter_types! {
    // One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: Weight = 1_000_000_000;
    // Used in native traders
    // This might be able to skipped.
    // We have to use `here()` because of reanchoring logic
    pub ParaTokenPerSecond: (xcm::v2::AssetId, u128) = (Concrete(MultiLocation::here()), 1_000_000_000);
    pub const MaxInstructions: u32 = 100;
}

/// Transactor for the native asset which implements `fungible` trait, as well as
/// Transactor for assets in pallet-assets, i.e. implements `fungibles` trait
pub type MultiAssetTransactor = MultiAssetAdapter<
    Runtime,
    // "default" implementation of converting a `MultiLocation` to an `AccountId`
    LocationToAccountId,
    // Used when the incoming asset is a fungible concrete asset matching the given location or name:
    IsNativeConcrete<SelfReserve>,
    // Used to match incoming assets which are not the native asset.
    ConvertedConcreteAssetId<
        AssetId,
        Balance,
        AssetIdLocationConvert<AssetLocation, AssetManager>,
        JustTry,
    >,
    // Precondition checks and actual implementations of mint and burn logic.
    DolphinConcreteFungibleLedger,
    // Used to find the query the native asset id of the chain.
    DolphinAssetConfig,
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

pub type XcmFeesToAccount = manta_primitives::xcm::XcmFeesToAccount<
    Assets,
    ConvertedConcreteAssetId<
        AssetId,
        Balance,
        AssetIdLocationConvert<AssetLocation, AssetManager>,
        JustTry,
    >,
    AccountId,
    XcmFeesAccount,
>;

pub struct XcmExecutorConfig;
impl Config for XcmExecutorConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    // Defines how to Withdraw and Deposit instruction work
    type AssetTransactor = MultiAssetTransactor;
    type OriginConverter = XcmOriginToCallOrigin;
    // Combinations of (Location, Asset) pairs which we trust as reserves.
    type IsReserve = MultiNativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    // Trader is the means to purchasing weight credit for XCM execution.
    // We define two traders:
    // The first one will charge parachain's native currency, who's `MultiLocation`
    // is defined in `SelfReserve`.
    // The second one will charge the first asset in the MultiAssets with pre-defined rate
    // i.e. units_per_second in `AssetManager`
    type Trader = (
        FixedRateOfFungible<ParaTokenPerSecond, ()>,
        FirstAssetTrader<AssetId, AssetLocation, AssetManager, XcmFeesToAccount>,
    );
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    // This is needed for the version change notifier work
    type SubscriptionService = PolkadotXcm;
}

/// No one is allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = ();

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

    type Origin = Origin;
    type Call = Call;
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    /// This means that no location will pass XcmExecuteFilter, so a dispatched `execute` message will be filtered.
    /// This shouldn't be reachable since `LocalOriginToLocation = ();`, but let's be on the safe side.
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRootOrMoreThanHalfCouncil;
    type ControllerOriginConverter = XcmOriginToCallOrigin;
    type WeightInfo = crate::weights::cumulus_pallet_xcmp_queue::SubstrateWeight<Runtime>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

// We wrap AssetId for XToken
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
    MantaCurrency(AssetId),
}

pub struct CurrencyIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
    for CurrencyIdtoMultiLocation<AssetXConverter>
where
    AssetXConverter: xcm_executor::traits::Convert<MultiLocation, AssetId>,
{
    fn convert(currency: CurrencyId) -> Option<MultiLocation> {
        match currency {
            CurrencyId::MantaCurrency(asset_id) => match AssetXConverter::reverse_ref(&asset_id) {
                Ok(location) => Some(location),
                Err(_) => None,
            },
        }
    }
}

parameter_types! {
    pub const BaseXcmWeight: Weight = 100_000_000;
    pub const MaxAssetsForTransfer: usize = 2;
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
    type CurrencyIdConvert =
        CurrencyIdtoMultiLocation<AssetIdLocationConvert<AssetLocation, AssetManager>>;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type SelfLocation = SelfReserve;
    // Take note that this pallet does not have the typical configurable WeightInfo.
    // It uses the Weigher configuration to calculate weights for the user callable extrinsics on this chain,
    // as well as weights for execution on the destination chain. Both based on the composed xcm messages.
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = AssetManager;
    type MultiLocationsFilter = AssetManager;
    type ReserveProvider = orml_traits::location::AbsoluteReserveProvider;
}
