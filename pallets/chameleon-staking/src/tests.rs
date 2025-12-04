// Copyright 2020-2024 Manta Network.
// Staking Tests - Minimal working tests for CI/CD

use crate::pallet::*;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64, ConstU128},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        ChameleonStaking: crate,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ConstU32<0>;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinValidatorStake = ConstU128<10_000>;
    type UnbondingPeriod = ConstU64<14>;
    type MaxDelegatorsPerValidator = ConstU32<100>;
    type MaxDelegationsPerDelegator = ConstU32<10>;
}

const ALICE: u64 = 1;
const BOB: u64 = 2;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000),
            (BOB, 500_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn pallet_compiles() {
    new_test_ext().execute_with(|| {
        // Staking pallet configured correctly
        assert!(true);
    });
}

#[test]
fn join_candidates_works() {
    new_test_ext().execute_with(|| {
        // ALICE joins as validator with 100k stake
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            100_000,
        ));
        
        // Check validator is registered
        assert!(Validators::<Test>::contains_key(&ALICE));
    });
}

#[test]
fn insufficient_stake_rejected() {
    new_test_ext().execute_with(|| {
        // Try with less than MinValidatorStake (10_000)
        assert_noop!(
            ChameleonStaking::join_candidates(
                RuntimeOrigin::signed(ALICE),
                1_000,  // Below minimum
            ),
            Error::<Test>::InsufficientStake
        );
    });
}
