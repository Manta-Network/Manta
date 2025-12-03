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

//! Mock runtime for Chameleon Staking tests

use crate as pallet_chameleon_staking;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64, Everything},
    weights::Weight,
    PalletId,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
type AccountId = u64;
type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        ChameleonStaking: pallet_chameleon_staking,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
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
}

parameter_types! {
    pub const ChameleonStakingPalletId: PalletId = PalletId(*b"chmlstak");
    pub const MaxDelegationsPerDelegator: u32 = 100;
    pub const MaxDelegatorsPerValidator: u32 = 500;
    pub const MinValidatorStake: Balance = 1_750_000_000_000_000_000_000; // 1,750 CHML
    pub const UnbondingPeriod: BlockNumber = 201_600; // 14 days in blocks
    pub const SlashingDowntime: Perbill = Perbill::from_parts(1_000_000); // 0.1%
    pub const SlashingDoubleSign: Perbill = Perbill::from_parts(50_000_000); // 5%
}

// Mock parachain staking config (minimal implementation)
parameter_types! {
    pub const MinCandidateStk: Balance = 1_750_000_000_000_000_000_000;
    pub const MinDelegatorStk: Balance = 1_000_000_000_000_000_000; // 1 CHML
    pub const MaxTopDelegationsPerCandidate: u32 = 300;
    pub const MaxBottomDelegationsPerCandidate: u32 = 50;
    pub const MaxDelegationsPerDelegator: u32 = 100;
    pub const DefaultBlocksPerRound: u32 = 600; // 1 hour
    pub const LeaveCandidatesDelay: u32 = 28; // rounds
    pub const CandidateBondLessDelay: u32 = 28; // rounds
    pub const LeaveDelegatorsDelay: u32 = 28; // rounds
    pub const RevokeDelegationDelay: u32 = 28; // rounds
    pub const DelegationBondLessDelay: u32 = 28; // rounds
    pub const RewardPaymentDelay: u32 = 2; // rounds
    pub const MinSelectedCandidates: u32 = 5;
    pub const MaxSelectedCandidates: u32 = 200;
}

// Minimal parachain staking implementation for testing
impl pallet_parachain_staking::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MonetaryGovernanceOrigin = frame_system::EnsureRoot<AccountId>;
    type MinCandidateStk = MinCandidateStk;
    type MinDelegatorStk = MinDelegatorStk;
    type MinSelectedCandidates = MinSelectedCandidates;
    type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
    type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
    type DefaultBlocksPerRound = DefaultBlocksPerRound;
    type LeaveCandidatesDelay = LeaveCandidatesDelay;
    type CandidateBondLessDelay = CandidateBondLessDelay;
    type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
    type RevokeDelegationDelay = RevokeDelegationDelay;
    type DelegationBondLessDelay = DelegationBondLessDelay;
    type RewardPaymentDelay = RewardPaymentDelay;
    type MaxSelectedCandidates = MaxSelectedCandidates;
    type WeightInfo = ();
}

impl pallet_chameleon_staking::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
    type PalletId = ChameleonStakingPalletId;
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
    type MaxDelegatorsPerValidator = MaxDelegatorsPerValidator;
    type MinValidatorStake = MinValidatorStake;
    type UnbondingPeriod = UnbondingPeriod;
    type SlashingDowntime = SlashingDowntime;
    type SlashingDoubleSign = SlashingDoubleSign;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000_000_000_000), // 1M CHML
            (2, 1_000_000_000_000_000_000_000_000), // 1M CHML
            (3, 1_000_000_000_000_000_000_000_000), // 1M CHML
        ],
    }
    .assimilate_storage(&mut ext)
    .unwrap();
    
    ext.into()
}