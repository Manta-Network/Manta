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

//! Parachain runtime mock.

#![cfg(test)]

use codec::{Decode, Encode};
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use frame_support::{
    assert_ok, construct_runtime, match_types,
    pallet_prelude::DispatchResult,
    parameter_types,
    traits::{ConstU32, Currency, Everything, Nothing},
    weights::Weight,
    PalletId,
};
use frame_system::EnsureRoot;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, Hash, IdentityLookup},
    AccountId32,
};
use sp_std::prelude::*;

use manta_primitives::{
    assets::{
        AssetConfig, AssetIdLocationConvert, AssetIdType, AssetLocation, AssetRegistry,
        AssetRegistryMetadata, AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::{ASSET_MANAGER_PALLET_ID, MANTA_DECIMAL, WEIGHT_PER_SECOND},
    types::{BlockNumber, Header, MantaAssetId},
    xcm::{FirstAssetTrader, IsNativeConcrete, MultiAssetAdapter, MultiNativeAsset},
};
use pallet_xcm::XcmPassthrough;
use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use polkadot_parachain::primitives::{
    DmpMessageHandler, Id as ParaId, Sibling, XcmpMessageFormat, XcmpMessageHandler,
};
use xcm::{latest::prelude::*, Version as XcmVersion, VersionedMultiLocation, VersionedXcm};
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId,
    EnsureXcmOrigin, FixedRateOfFungible, LocationInverter, ParentIsPreset,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit, WeightInfoBounds,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};
use xcm_simulator::{DmpMessageHandlerT, Get, TestExt, XcmpMessageHandlerT};

pub type AccountId = AccountId32;
pub type Balance = u128;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
    pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

parameter_types! {
    pub const DotLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub SelfReserve: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

parameter_types! {
    pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = MantaAssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
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
    // If the incoming XCM origin is of type `AccountId32` and the Network is Network::Any
    // or `RelayNetwork`, convert it to a Native 32 byte account.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognised.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

parameter_types! {
    pub const UnitWeightCost: Weight = 1_000_000_000;
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
    // Used to find the query the native asset id of the chain.
    ParachainAssetConfig,
    // "default" implementation of converting a `MultiLocation` to an `AccountId`
    LocationToAccountId,
    // Used when the incoming asset is a fungible concrete asset matching the given location or name:
    IsNativeConcrete<SelfReserve>,
    // Used to match incoming assets which are not the native asset.
    ConvertedConcreteAssetId<MantaAssetId, Balance, AssetIdLocationConvert<AssetManager>, JustTry>,
>;

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;

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
    /// Xcm fees will go to the asset manager (we don't implement treasury yet for mock parachain)
    pub XcmFeesAccount: AccountId = AssetManager::account_id();
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

/// Xcm fee of non native token
pub type XcmFeesToAccount = manta_primitives::xcm::XcmFeesToAccount<
    AccountId,
    Assets,
    ConvertedConcreteAssetId<MantaAssetId, Balance, AssetIdLocationConvert<AssetManager>, JustTry>,
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
    type Weigher =
        WeightInfoBounds<manta_runtime::weights::xcm::MantaXcmWeight<Call>, Call, MaxInstructions>;
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

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToCallOrigin;
    type WeightInfo = ();
}

#[frame_support::pallet]
pub mod mock_msg_queue {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type XcmExecutor: ExecuteXcm<Self::Call>;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    // without storage info is a work around
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn parachain_id)]
    pub(super) type ParachainId<T: Config> = StorageValue<_, ParaId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn received_dmp)]
    /// A queue of received DMP messages
    pub(super) type ReceivedDmp<T: Config> = StorageValue<_, Vec<Xcm<T::Call>>, ValueQuery>;

    impl<T: Config> Get<ParaId> for Pallet<T> {
        fn get() -> ParaId {
            Self::parachain_id()
        }
    }

    pub type MessageId = [u8; 32];

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // XCMP
        /// Some XCM was executed OK.
        Success(Option<T::Hash>),
        /// Some XCM failed.
        Fail(Option<T::Hash>, XcmError),
        /// Bad XCM version used.
        BadVersion(Option<T::Hash>),
        /// Bad XCM format used.
        BadFormat(Option<T::Hash>),

        // DMP
        /// Downward message is invalid XCM.
        InvalidFormat(MessageId),
        /// Downward message is unsupported version of XCM.
        UnsupportedVersion(MessageId),
        /// Downward message executed with the given outcome.
        ExecutedDownward(MessageId, Outcome),
    }

    impl<T: Config> Pallet<T> {
        pub fn set_para_id(para_id: ParaId) {
            ParachainId::<T>::put(para_id);
        }

        fn handle_xcmp_message(
            sender: ParaId,
            _sent_at: RelayBlockNumber,
            xcm: VersionedXcm<T::Call>,
            max_weight: Weight,
        ) -> Result<Weight, XcmError> {
            let hash = Encode::using_encoded(&xcm, T::Hashing::hash);
            let (result, event) = match Xcm::<T::Call>::try_from(xcm) {
                Ok(xcm) => {
                    let location = (1, Parachain(sender.into()));
                    match T::XcmExecutor::execute_xcm(location, xcm, max_weight) {
                        Outcome::Error(e) => (Err(e), Event::Fail(Some(hash), e)),
                        Outcome::Complete(w) => (Ok(w), Event::Success(Some(hash))),
                        // As far as the caller is concerned, this was dispatched without error, so
                        // we just report the weight used.
                        Outcome::Incomplete(w, e) => (Ok(w), Event::Fail(Some(hash), e)),
                    }
                }
                Err(()) => (
                    Err(XcmError::UnhandledXcmVersion),
                    Event::BadVersion(Some(hash)),
                ),
            };
            Self::deposit_event(event);
            result
        }
    }

    impl<T: Config> XcmpMessageHandler for Pallet<T> {
        fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
            iter: I,
            max_weight: Weight,
        ) -> Weight {
            for (sender, sent_at, data) in iter {
                let mut data_ref = data;
                let _ = XcmpMessageFormat::decode(&mut data_ref)
                    .expect("Simulator encodes with versioned xcm format; qed");

                let mut remaining_fragments = data_ref;
                while !remaining_fragments.is_empty() {
                    if let Ok(xcm) = VersionedXcm::<T::Call>::decode(&mut remaining_fragments) {
                        let _ = Self::handle_xcmp_message(sender, sent_at, xcm, max_weight);
                    } else {
                        debug_assert!(false, "Invalid incoming XCMP message data");
                    }
                }
            }
            max_weight
        }
    }

    impl<T: Config> DmpMessageHandler for Pallet<T> {
        fn handle_dmp_messages(
            iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
            limit: Weight,
        ) -> Weight {
            for (_i, (_sent_at, data)) in iter.enumerate() {
                let id = sp_io::hashing::blake2_256(&data[..]);
                let maybe_msg =
                    VersionedXcm::<T::Call>::decode(&mut &data[..]).map(Xcm::<T::Call>::try_from);
                match maybe_msg {
                    Err(_) => {
                        Self::deposit_event(Event::InvalidFormat(id));
                    }
                    Ok(Err(())) => {
                        Self::deposit_event(Event::UnsupportedVersion(id));
                    }
                    Ok(Ok(x)) => {
                        let outcome = T::XcmExecutor::execute_xcm(Parent, x.clone(), limit);
                        <ReceivedDmp<T>>::append(x);
                        Self::deposit_event(Event::ExecutedDownward(id, outcome));
                    }
                }
            }
            limit
        }
    }
}

impl mock_msg_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type DmpMessageHandler = MsgQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type OnSystemEvent = ();
    type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

pub type LocalOriginToLocation = ();

impl pallet_xcm::Config for Runtime {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    // Do not allow teleports
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Nothing;
    type Weigher =
        WeightInfoBounds<manta_runtime::weights::xcm::MantaXcmWeight<Call>, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type Origin = Origin;
    type Call = Call;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = CurrentXcmVersion;
}

parameter_types! {
    /// An implementation of `Get<u32>` which just returns the latest XCM version which we can
    /// support.
    pub static CurrentXcmVersion: u32 = 0;
}

pub(crate) fn set_current_xcm_version(version: XcmVersion) {
    CurrentXcmVersion::set(version);
}

pub struct MantaAssetRegistry;
impl BalanceType for MantaAssetRegistry {
    type Balance = Balance;
}
impl AssetIdType for MantaAssetRegistry {
    type AssetId = MantaAssetId;
}
impl AssetRegistry for MantaAssetRegistry {
    type Metadata = AssetStorageMetadata;
    type Error = sp_runtime::DispatchError;

    fn create_asset(
        asset_id: MantaAssetId,
        metadata: AssetStorageMetadata,
        min_balance: Balance,
        is_sufficient: bool,
    ) -> DispatchResult {
        Assets::force_create(
            Origin::root(),
            asset_id,
            AssetManager::account_id(),
            is_sufficient,
            min_balance,
        )?;

        Assets::force_set_metadata(
            Origin::root(),
            asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }

    fn update_asset_metadata(
        asset_id: &MantaAssetId,
        metadata: AssetStorageMetadata,
    ) -> DispatchResult {
        Assets::force_set_metadata(
            Origin::root(),
            *asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }
}

parameter_types! {
    pub const StartNonNativeAssetId: MantaAssetId = 8;
    pub const NativeAssetId: MantaAssetId = 1;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(SelfReserve::get()));
    pub NativeAssetMetadata: AssetRegistryMetadata<Balance> = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"ParaAToken".to_vec(),
            symbol: b"ParaA".to_vec(),
            decimals: MANTA_DECIMAL,
            is_frozen: false,
        },
        min_balance: 1,
        is_sufficient: true,
    };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

/// AssetConfig implementations for this runtime
#[derive(Clone, Eq, PartialEq)]
pub struct ParachainAssetConfig;
impl LocationType for ParachainAssetConfig {
    type Location = AssetLocation;
}
impl AssetIdType for ParachainAssetConfig {
    type AssetId = MantaAssetId;
}
impl BalanceType for ParachainAssetConfig {
    type Balance = Balance;
}
impl AssetConfig<Runtime> for ParachainAssetConfig {
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type NativeAssetId = NativeAssetId;
    type AssetRegistryMetadata = AssetRegistryMetadata<Balance>;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata = AssetStorageMetadata;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = NativeAndNonNative<Runtime, ParachainAssetConfig, Balances, Assets>;
}

impl pallet_asset_manager::Config for Runtime {
    type Event = Event;
    type AssetId = MantaAssetId;
    type Balance = Balance;
    type Location = AssetLocation;
    type AssetConfig = ParachainAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId>;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = ();
}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

// We wrap AssetId for XToken
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
    MantaCurrency(MantaAssetId),
}

/// Maps a xTokens CurrencyId to a xcm MultiLocation implemented by some asset manager
pub struct CurrencyIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
    for CurrencyIdtoMultiLocation<AssetXConverter>
where
    AssetXConverter: xcm_executor::traits::Convert<MultiLocation, MantaAssetId>,
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
    pub const BaseXcmWeight: Weight = 100_000_000;
    pub const MaxAssetsForTransfer: usize = 3;
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type AccountIdToMultiLocation = manta_primitives::xcm::AccountIdToMultiLocation;
    type CurrencyIdConvert = CurrencyIdtoMultiLocation<AssetIdLocationConvert<AssetManager>>;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type SelfLocation = SelfReserve;
    type Weigher =
        WeightInfoBounds<manta_runtime::weights::xcm::MantaXcmWeight<Call>, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = AssetManager;
    type MultiLocationsFilter = AssetManager;
    type ReserveProvider = orml_traits::location::AbsoluteReserveProvider;
}

impl parachain_info::Config for Runtime {}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub const PALLET_ASSET_INDEX: u8 = 1;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
        Assets: pallet_assets::{Pallet, Storage, Event<T>} = 1,
        AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 2,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
        MsgQueue: mock_msg_queue::{Pallet, Storage, Event<T>} = 4,
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 5,
        XTokens: orml_xtokens::{Pallet, Call, Event<T>, Storage} = 6,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 7,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 8,
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 9,
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned} = 10,
    }
);

pub(crate) fn para_events() -> Vec<Event> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(Some)
        .collect::<Vec<_>>()
}

use frame_support::traits::{OnFinalize, OnInitialize, OnRuntimeUpgrade};
pub(crate) fn on_runtime_upgrade() {
    PolkadotXcm::on_runtime_upgrade();
}

pub(crate) fn para_roll_to(n: u32) {
    while System::block_number() < n {
        PolkadotXcm::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        PolkadotXcm::on_initialize(System::block_number());
    }
}

pub(crate) fn create_asset_metadata(
    name: &str,
    symbol: &str,
    decimals: u8,
    min_balance: u128,
    is_frozen: bool,
    is_sufficient: bool,
) -> AssetRegistryMetadata<Balance> {
    AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: name.as_bytes().to_vec(),
            symbol: symbol.as_bytes().to_vec(),
            decimals,
            is_frozen,
        },
        min_balance,
        is_sufficient,
    }
}

pub(crate) fn create_asset_location(parents: u8, para_id: u32) -> AssetLocation {
    AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        parents,
        X1(Parachain(para_id)),
    )))
}

fn insert_dummy_data(
    dummy_mult_loc: MultiLocation,
    dummy_asset_metadata: &AssetRegistryMetadata<Balance>,
    start_from: MantaAssetId,
    insert_until: MantaAssetId,
) {
    let mut next_asset_id = start_from;
    let mut next_dummy_mult_loc = dummy_mult_loc;
    while next_asset_id < insert_until {
        assert_ok!(AssetManager::register_asset(
            self::Origin::root(),
            AssetLocation::from(next_dummy_mult_loc.clone()),
            dummy_asset_metadata.clone()
        ));
        next_dummy_mult_loc.parents += 1;
        next_asset_id += 1;
    }
}

pub(crate) fn register_assets_on_parachain<P>(
    source_location: &AssetLocation,
    asset_metadata: &AssetRegistryMetadata<Balance>,
    units_per_second: Option<u128>,
    mint_asset: Option<(AccountId, Balance, bool, bool)>,
) -> MantaAssetId
where
    P: XcmpMessageHandlerT + DmpMessageHandlerT + TestExt,
{
    let mut asset_id = 0;
    let mut dummy_mult_loc = match source_location.0.clone() {
        VersionedMultiLocation::V1(some_location) => some_location,
        _ => MultiLocation::default(),
    };
    // Use some fake location as dummy to fill in gaps between Native and Non-Native assets
    dummy_mult_loc.parents = 50;
    let native_asset_id = <ParachainAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
    let non_native_asset_id_start =
        <ParachainAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();

    P::execute_with(|| {
        asset_id = AssetManager::next_asset_id();

        if asset_id < native_asset_id {
            insert_dummy_data(dummy_mult_loc, asset_metadata, asset_id, native_asset_id);
        } else if asset_id < non_native_asset_id_start {
            insert_dummy_data(
                dummy_mult_loc,
                asset_metadata,
                asset_id,
                non_native_asset_id_start,
            );
        }

        asset_id = AssetManager::next_asset_id();

        assert_ok!(AssetManager::register_asset(
            self::Origin::root(),
            source_location.clone(),
            asset_metadata.clone()
        ));

        if let Some((owner, min_balance, is_sufficient, is_frozen)) = mint_asset {
            assert_ok!(self::Assets::force_asset_status(
                self::Origin::root(),
                asset_id,
                owner.clone(),
                owner.clone(),
                owner.clone(),
                owner,
                min_balance,
                is_sufficient,
                is_frozen,
            ));
        }

        if let Some(ups) = units_per_second {
            assert_ok!(AssetManager::set_units_per_second(
                self::Origin::root(),
                asset_id,
                ups,
            ));
        }

        assert_eq!(
            Some(asset_id),
            AssetManager::location_asset_id(source_location.clone())
        );
    });

    asset_id
}
