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

//! Relay chain runtime mock.

#![cfg(test)]

use core::cell::RefCell;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, Everything, Nothing, ProcessMessage, ProcessMessageError},
    weights::{IdentityFee, WeightMeter},
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};

use manta_primitives::{types::BlockNumber, xcm::AllowTopLevelPaidExecutionFrom};
use polkadot_parachain::primitives::Id as ParaId;
use polkadot_runtime_parachains::{
    configuration,
    inclusion::{AggregateMessageOrigin, UmpQueueId},
    origin, shared,
};
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowUnpaidExecutionFrom,
    ChildParachainAsNative, ChildParachainConvertsVia, ChildSystemParachainAsSuperuser,
    CurrencyAdapter as XcmCurrencyAdapter, FixedRateOfFungible, FixedWeightBounds, IsConcrete,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};

pub type AccountId = AccountId32;
pub type Balance = u128;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
}

impl frame_system::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Block = Block;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
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
    type OnSetCode = ();
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
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<1>;
    type MaxHolds = ConstU32<1>;
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
    type PalletsOrigin = OriginCaller;
}

impl shared::Config for Runtime {}

impl configuration::Config for Runtime {
    type WeightInfo = configuration::TestWeightInfo;
}

parameter_types! {
    pub const KsmLocation: MultiLocation = MultiLocation::here();
    pub const KusamaNetwork: NetworkId = NetworkId::Kusama;
    pub const AnyNetwork: NetworkId = NetworkId::ByGenesis([1u8; 32]);
    pub Ancestry: MultiLocation = Here.into();
    pub UnitWeightCost: u64 = 1_000;
}

pub type SovereignAccountOf = (
    ChildParachainConvertsVia<ParaId, AccountId>,
    AccountId32Aliases<KusamaNetwork, AccountId>,
);

pub type LocalAssetTransactor =
    XcmCurrencyAdapter<Balances, IsConcrete<KsmLocation>, SovereignAccountOf, AccountId, ()>;

type LocalOriginConverter = (
    SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
    ChildParachainAsNative<origin::Origin, RuntimeOrigin>,
    SignedAccountId32AsNative<KusamaNetwork, RuntimeOrigin>,
    ChildSystemParachainAsSuperuser<ParaId, RuntimeOrigin>,
);

parameter_types! {
    pub const BaseXcmWeight: Weight = Weight::from_parts(1_000, 10);
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub KsmPerSecond: (xcm::v3::AssetId, u128, u128) = (Concrete(MultiLocation::here()), 1, 1);
    pub const MaxInstructions: u32 = 100;
    pub const UniversalLocation: InteriorMultiLocation = Here;
    pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct TestSendXcm;
impl SendXcm for TestSendXcm {
    type Ticket = (MultiLocation, Xcm<()>, XcmHash);
    fn validate(
        dest: &mut Option<MultiLocation>,
        msg: &mut Option<Xcm<()>>,
    ) -> SendResult<(MultiLocation, Xcm<()>, XcmHash)> {
        let msg = msg.take().unwrap();
        let hash = fake_message_hash(&msg);
        let triplet = (dest.take().unwrap(), msg, hash);
        Ok((triplet, MultiAssets::new()))
    }
    fn deliver(triplet: (MultiLocation, Xcm<()>, XcmHash)) -> Result<XcmHash, SendError> {
        let hash = triplet.2;
        SENT_XCM.with(|q| q.borrow_mut().push(triplet));
        Ok(hash)
    }
}

pub type XcmRouter = TestSendXcm;
pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    // Expected responses are OK.
    AllowKnownQueryResponses<XcmPallet>,
    // Subscriptions for version tracking are OK.
    AllowSubscriptionsFrom<Everything>,
    // The following is purely for testing ump
    AllowUnpaidExecutionFrom<Everything>,
);

pub struct XcmExecutorConfig;
impl Config for XcmExecutorConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    type IsReserve = ();
    type IsTeleporter = ();
    type UniversalLocation = UniversalLocation;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type Trader = FixedRateOfFungible<KsmPerSecond, ()>;
    type ResponseHandler = XcmPallet;
    type AssetTrap = XcmPallet;
    type AssetClaims = XcmPallet;
    type SubscriptionService = XcmPallet;
    type Aliasers = Nothing;
    type AssetLocker = XcmPallet;
    type AssetExchanger = ();
    type PalletInstancesInfo = ();
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
    type FeeManager = ();
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, KusamaNetwork>;

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    // Anyone can execute XCM messages locally...
    type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = ();
    type UniversalLocation = UniversalLocation;
    type AdminOrigin = EnsureRoot<AccountId>;
    type SovereignAccountOf = ();
    type MaxRemoteLockConsumers = ConstU32<0>;
    type MaxLockers = ConstU32<8>;
    type RemoteLockConsumerIdentifier = ();
    #[cfg(feature = "runtime-benchmarks")]
    type ReachableDest = ReachableDest;
    #[cfg(feature = "manta")]
    type WeightInfo = manta_runtime::weights::pallet_xcm::WeightInfo<Runtime>;
    #[cfg(feature = "calamari")]
    type WeightInfo = calamari_runtime::weights::pallet_xcm::WeightInfo<Runtime>;
}

parameter_types! {
    pub const FirstMessageFactorPercent: u64 = 100;
}

impl origin::Config for Runtime {}

pub struct MessageProcessor;
impl ProcessMessage for MessageProcessor {
    type Origin = AggregateMessageOrigin;

    fn process_message(
        message: &[u8],
        origin: Self::Origin,
        meter: &mut WeightMeter,
        id: &mut [u8; 32],
    ) -> Result<bool, ProcessMessageError> {
        let para = match origin {
            AggregateMessageOrigin::Ump(UmpQueueId::Para(para)) => para,
        };
        xcm_builder::ProcessXcmMessage::<
            Junction,
            xcm_executor::XcmExecutor<XcmExecutorConfig>,
            RuntimeCall,
        >::process_message(message, Junction::Parachain(para.into()), meter, id)
    }
}

parameter_types! {
    pub MessageQueueServiceWeight: Weight = Weight::from_parts(1_000_000_000, 1_000_000);
    pub const MessageQueueHeapSize: u32 = 65_536;
    pub const MessageQueueMaxStale: u32 = 16;
}

impl pallet_message_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Size = u32;
    type HeapSize = MessageQueueHeapSize;
    type MaxStale = MessageQueueMaxStale;
    type ServiceWeight = MessageQueueServiceWeight;
    type MessageProcessor = MessageProcessor;
    type QueueChangeHandler = ();
    type QueuePausedQuery = ();
    type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub struct Runtime
    {
        System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        ParasOrigin: origin::{Pallet, Origin},
        XcmPallet: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin},
        Utility: pallet_utility::{Pallet, Call, Event},
        MessageQueue: pallet_message_queue,
    }
);

pub(crate) fn relay_events() -> Vec<RuntimeEvent> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(Some)
        .collect::<Vec<_>>()
}

use frame_support::traits::{OnFinalize, OnInitialize};
pub(crate) fn relay_roll_to(n: BlockNumber) {
    while System::block_number() < n as u64 {
        XcmPallet::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        XcmPallet::on_initialize(System::block_number());
    }
}

thread_local! {
    pub static SENT_XCM: RefCell<Vec<(MultiLocation, opaque::Xcm, XcmHash)>> = RefCell::new(Vec::new());
}
pub fn sent_xcm() -> Vec<(MultiLocation, opaque::Xcm, XcmHash)> {
    SENT_XCM.with(|q| (*q.borrow()).clone())
}

pub fn fake_message_hash<T>(message: &Xcm<T>) -> XcmHash {
    use codec::Encode;
    message.using_encoded(sp_io::hashing::blake2_256)
}
