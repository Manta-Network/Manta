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

use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, Everything, Nothing},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};

use manta_primitives::types::{BlockNumber, Header};
use polkadot_parachain::primitives::Id as ParaId;
use polkadot_runtime_parachains::{configuration, origin, shared, ump};
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, ChildParachainAsNative,
    ChildParachainConvertsVia, ChildSystemParachainAsSuperuser,
    CurrencyAdapter as XcmCurrencyAdapter, FixedRateOfFungible, FixedWeightBounds, IsConcrete,
    LocationInverter, SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation,
    TakeWeightCredit,
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
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
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
    pub const KsmLocation: MultiLocation = Here.into();
    pub const KusamaNetwork: NetworkId = NetworkId::Kusama;
    pub const AnyNetwork: NetworkId = NetworkId::Any;
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
    pub const BaseXcmWeight: u64 = 1_000;
    pub KsmPerSecond: (AssetId, u128) = (Concrete(KsmLocation::get()), 1);
    pub const MaxInstructions: u32 = 100;
}

pub type XcmRouter = super::RelayChainXcmRouter;
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
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type Trader = FixedRateOfFungible<KsmPerSecond, ()>;
    type ResponseHandler = XcmPallet;
    type AssetTrap = XcmPallet;
    type AssetClaims = XcmPallet;
    type SubscriptionService = XcmPallet;
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
    type LocationInverter = LocationInverter<Ancestry>;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

parameter_types! {
    pub const FirstMessageFactorPercent: u64 = 100;
}

impl ump::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type UmpSink = ump::XcmSink<XcmExecutor<XcmExecutorConfig>, Runtime>;
    type FirstMessageFactorPercent = FirstMessageFactorPercent;
    type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = polkadot_runtime_parachains::ump::TestWeightInfo;
}

impl origin::Config for Runtime {}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        ParasOrigin: origin::{Pallet, Origin},
        ParasUmp: ump::{Pallet, Call, Storage, Event},
        XcmPallet: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin},
        Utility: pallet_utility::{Pallet, Call, Event},
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
    while System::block_number() < n {
        XcmPallet::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        XcmPallet::on_initialize(System::block_number());
    }
}
