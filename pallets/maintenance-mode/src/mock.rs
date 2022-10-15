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

// Mocks for the maintenance mode pallet.

#![cfg(test)]

use super::*;
use crate as pallet_maintenance_mode;
use cumulus_primitives_core::{relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler};
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    dispatch::RawOrigin,
    traits::{
        Contains, Everything, GenesisBuild, OffchainWorker, OnFinalize, OnIdle, OnInitialize,
        OnRuntimeUpgrade,
    },
    weights::Weight,
};
use frame_system::EnsureRoot;
use manta_primitives::types::{AccountId, AssetId, Balance, BlockNumber};
use sp_core::H256;
use sp_runtime::{traits::{BlakeTwo256, ConstU32, IdentityLookup}, Perbill, DispatchResult};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        MaintenanceMode: pallet_maintenance_mode::{Pallet, Call, Storage, Event, Config},
        MockPalletMaintenanceHooks: mock_pallet_maintenance_hooks::{Pallet, Call, Event},
        Assets: pallet_assets::{Pallet, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
    type BaseCallFilter = MaintenanceMode;
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

ord_parameter_types! {
    pub const ListingOrigin: AccountId = ALICE;
}
impl Config for Test {
    type Event = Event;
    type NormalCallFilter = Everything;
    type MaintenanceCallFilter = MaintenanceCallFilter;
    type MaintenanceOrigin = EnsureRoot<AccountId>;
    // type MaintenanceOrigin = EnsureSignedBy<ListingOrigin, AccountId>;
    type XcmExecutionManager = ();
    type NormalDmpHandler = NormalDmpHandler;
    type MaintenanceDmpHandler = MaintenanceDmpHandler;
    type NormalExecutiveHooks = NormalHooks;
    type MaintenanceExecutiveHooks = MaintenanceHooks;
    type AssetFreezer = AssetsFreezer;
    type AssetIdInParachain = Everything;
}

pub struct AssetsFreezer;
impl AssetFreezer for AssetsFreezer {
    fn freeze_asset(asset_id: AssetId) -> DispatchResult {
        Assets::freeze_asset(RawOrigin::Signed(ListingOrigin::get()).into(), asset_id)
        // Assets::freeze_asset(Origin::signed(ListingOrigin::get()), asset_id)
    }

    fn freeze(asset_id: AssetId, account: AccountId) -> DispatchResult {
        Assets::freeze(RawOrigin::Signed(ListingOrigin::get()).into(), asset_id, account)
    }
}

parameter_types! {
    pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type AssetId = AssetId;
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
    type WeightInfo = ();
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
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

/// During maintenance mode we will not allow any calls.
pub struct MaintenanceCallFilter;
impl Contains<Call> for MaintenanceCallFilter {
    fn contains(_: &Call) -> bool {
        false
    }
}

pub struct MaintenanceDmpHandler;
impl DmpMessageHandler for MaintenanceDmpHandler {
    // This implementation makes messages be queued
    // Since the limit is 0, messages are queued for next iteration
    fn handle_dmp_messages(
        _iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
        _limit: Weight,
    ) -> Weight {
        return 1;
    }
}

pub struct NormalDmpHandler;
impl DmpMessageHandler for NormalDmpHandler {
    // This implementation makes messages be queued
    // Since the limit is 0, messages are queued for next iteration
    fn handle_dmp_messages(
        _iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
        _limit: Weight,
    ) -> Weight {
        return 0;
    }
}

impl mock_pallet_maintenance_hooks::Config for Test {
    type Event = Event;
}

// Pallet to throw events, used to test maintenance mode hooks
#[frame_support::pallet]
pub mod mock_pallet_maintenance_hooks {
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event {
        MaintenanceOnIdle,
        MaintenanceOnInitialize,
        MaintenanceOffchainWorker,
        MaintenanceOnFinalize,
        MaintenanceOnRuntimeUpgrade,
        NormalOnIdle,
        NormalOnInitialize,
        NormalOffchainWorker,
        NormalOnFinalize,
        NormalOnRuntimeUpgrade,
    }
}

pub struct MaintenanceHooks;

impl OnInitialize<BlockNumber> for MaintenanceHooks {
    fn on_initialize(_n: BlockNumber) -> Weight {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::MaintenanceOnInitialize,
        );
        return 1;
    }
}

impl OnIdle<BlockNumber> for MaintenanceHooks {
    fn on_idle(_n: BlockNumber, _max_weight: Weight) -> Weight {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::MaintenanceOnIdle,
        );
        return 1;
    }
}

impl OnRuntimeUpgrade for MaintenanceHooks {
    fn on_runtime_upgrade() -> Weight {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::MaintenanceOnRuntimeUpgrade,
        );
        return 1;
    }
}

impl OnFinalize<BlockNumber> for MaintenanceHooks {
    fn on_finalize(_n: BlockNumber) {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::MaintenanceOnFinalize,
        );
    }
}

impl OffchainWorker<BlockNumber> for MaintenanceHooks {
    fn offchain_worker(_n: BlockNumber) {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::MaintenanceOffchainWorker,
        );
    }
}

pub struct NormalHooks;

impl OnInitialize<BlockNumber> for NormalHooks {
    fn on_initialize(_n: BlockNumber) -> Weight {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::NormalOnInitialize,
        );
        return 0;
    }
}

impl OnIdle<BlockNumber> for NormalHooks {
    fn on_idle(_n: BlockNumber, _max_weight: Weight) -> Weight {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::NormalOnIdle,
        );
        return 0;
    }
}

impl OnRuntimeUpgrade for NormalHooks {
    fn on_runtime_upgrade() -> Weight {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::NormalOnRuntimeUpgrade,
        );

        return 0;
    }
}

impl OnFinalize<BlockNumber> for NormalHooks {
    fn on_finalize(_n: BlockNumber) {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::NormalOnFinalize,
        );
    }
}

impl OffchainWorker<BlockNumber> for NormalHooks {
    fn offchain_worker(_n: BlockNumber) {
        MockPalletMaintenanceHooks::deposit_event(
            mock_pallet_maintenance_hooks::Event::NormalOffchainWorker,
        );
    }
}

/// Externality builder for pallet maintenance mode's mock runtime
pub(crate) struct ExtBuilder {
    maintenance_mode: bool,
}

impl Default for ExtBuilder {
    fn default() -> ExtBuilder {
        ExtBuilder {
            maintenance_mode: false,
        }
    }
}

impl ExtBuilder {
    pub(crate) fn with_maintenance_mode(mut self, m: bool) -> Self {
        self.maintenance_mode = m;
        self
    }

    pub(crate) fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .expect("Frame system builds valid default genesis config");

        GenesisBuild::<Test>::assimilate_storage(
            &pallet_maintenance_mode::GenesisConfig {
                start_in_maintenance_mode: self.maintenance_mode,
            },
            &mut t,
        )
        .expect("Pallet maintenance mode storage can be assimilated");

        // GenesisBuild::<Test>::assimilate_storage(
        //     &pallet_assets::GenesisConfig {
        //         assets: vec![
        //             // id, owner, is_sufficient, min_balance
        //             (999, 0, true, 1),
        //             (888, 0, true, 1),
        //         ],
        //         metadata: vec![
        //             // id, name, symbol, decimals
        //             (999, "Token Name".into(), "TOKEN".into(), 10),
        //             (888, "Token Name".into(), "TOKEN".into(), 10),
        //         ],
        //         accounts: vec![
        //             // id, account_id, balance
        //             (999, 1, 100),
        //             (888, 1, 100),
        //         ],
        //     },
        //     &mut t,
        // )
        // .expect("Pallet maintenance mode storage can be assimilated");

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub(crate) fn events() -> Vec<pallet_maintenance_mode::Event> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::MaintenanceMode(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub(crate) fn mock_events() -> Vec<mock_pallet_maintenance_hooks::Event> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::MockPalletMaintenanceHooks(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
