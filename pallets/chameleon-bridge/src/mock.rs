// Copyright 2020-2024 Manta Network.
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

//! Mock runtime for testing the Chameleon Bridge pallet

use crate as pallet_chameleon_bridge;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64},
    PalletId,
};
use frame_system as system;
use manta_primitives::{
    chameleon_constants::CHAMELEON_BRIDGE_PALLET_ID,
    types::{Balance, BlockNumber},
};
use sp_core::{H256, ConstU16};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u64;
pub type AssetId = u32;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        ChameleonBridge: pallet_chameleon_bridge,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
}

parameter_types! {
    pub const AssetDeposit: Balance = 100;
    pub const ApprovalDeposit: Balance = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 10;
    pub const MetadataDepositPerByte: Balance = 1;
}

impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = AssetId;
    type AssetIdParameter = codec::Compact<AssetId>;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureRoot<AccountId>>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = ConstU128<1>;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<1000>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const BridgePalletId: PalletId = CHAMELEON_BRIDGE_PALLET_ID;
    pub const ConfirmationBlocks: u32 = 12;
    pub const ValidatorThreshold: u32 = 5;
    pub const ChallengePeriod: BlockNumber = 3600; // 6 hours in blocks
}

impl pallet_chameleon_bridge::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Assets = Assets;
    type PalletId = BridgePalletId;
    type ConfirmationBlocks = ConfirmationBlocks;
    type ValidatorThreshold = ValidatorThreshold;
    type ChallengePeriod = ChallengePeriod;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::<Test>::default().build_storage().unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1000000000000000000000), // 1000 CHML
            (2, 1000000000000000000000), // 1000 CHML
            (3, 1000000000000000000000), // 1000 CHML
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    
    pallet_chameleon_bridge::GenesisConfig::<Test> {
        validators: vec![
            // Initial validators will be added in tests
        ],
        bridge_config: pallet_chameleon_bridge::types::BridgeConfig {
            min_confirmations: 12,
            signature_threshold: 5,
            max_single_deposit: 1000 * 1_000_000_000_000_000_000, // 1000 ETH equivalent
            daily_limit: 10000 * 1_000_000_000_000_000_000,       // 10000 ETH equivalent
            is_paused: false,
            challenge_period: 3600, // 6 hours in blocks
        },
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    
    storage.into()
}
