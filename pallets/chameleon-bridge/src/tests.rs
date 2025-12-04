// Copyright 2020-2024 Manta Network.
// Bridge Tests - Simplified for CI/CD

use crate::pallet::*;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64, ConstU128},
};
use sp_core::{H160, H256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        ChameleonBridge: crate,
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

impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u128;
    type AssetIdParameter = u128;
    type Currency = Balances;
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU128<0>;
    type AssetAccountDeposit = ConstU128<0>;
    type MetadataDepositBase = ConstU128<0>;
    type MetadataDepositPerByte = ConstU128<0>;
    type ApprovalDeposit = ConstU128<0>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<1000>;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u128;
    type Balance = u128;
    type Assets = Assets;
    type MinConfirmations = ConstU32<3>;
    type SignatureThreshold = ConstU32<5>;
}

const ALICE: u64 = 1;
const VALIDATOR_1: u64 = 10;
const VALIDATOR_2: u64 = 11;
const VALIDATOR_3: u64 = 12;
const VALIDATOR_4: u64 = 13;
const VALIDATOR_5: u64 = 14;
const ETH_ADDRESS: H160 = H160([1u8; 20]);
const TX_HASH: H256 = H256([2u8; 32]);
const WRAPPED_ETH: u128 = 1;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 10_000_000_000_000),
            (VALIDATOR_1, 1_000_000),
            (VALIDATOR_2, 1_000_000),
            (VALIDATOR_3, 1_000_000),
            (VALIDATOR_4, 1_000_000),
            (VALIDATOR_5, 1_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        
        // Create wrapped asset with bridge pallet as admin
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), WRAPPED_ETH, ALICE, true, 1));
    });
    
    ext
}

#[test]
fn test_report_lock_event() {
    new_test_ext().execute_with(|| {
        let amount = 1_000_000_000u128;
        
        assert_ok!(ChameleonBridge::report_lock_event(
            RuntimeOrigin::signed(VALIDATOR_1),
            TX_HASH,
            ETH_ADDRESS,
            ALICE,
            amount,
            WRAPPED_ETH,
        ));
        
        // Check vote was recorded
        let votes = ChameleonBridge::deposit_votes(TX_HASH);
        assert_eq!(votes.len(), 1);
    });
}

#[test]
fn test_threshold_required() {
    new_test_ext().execute_with(|| {
        let amount = 1_000_000_000u128;
        
        // 4 validators vote (below 5 threshold)
        for validator in [VALIDATOR_1, VALIDATOR_2, VALIDATOR_3, VALIDATOR_4] {
            assert_ok!(ChameleonBridge::report_lock_event(
                RuntimeOrigin::signed(validator),
                TX_HASH,
                ETH_ADDRESS,
                ALICE,
                amount,
                WRAPPED_ETH,
            ));
        }
        
        // No tokens minted yet (below threshold)
        assert_eq!(Assets::balance(WRAPPED_ETH, &ALICE), 0);
    });
}

#[test]
fn test_cannot_double_vote() {
    new_test_ext().execute_with(|| {
        let amount = 1_000_000_000u128;
        
        assert_ok!(ChameleonBridge::report_lock_event(
            RuntimeOrigin::signed(VALIDATOR_1),
            TX_HASH,
            ETH_ADDRESS,
            ALICE,
            amount,
            WRAPPED_ETH,
        ));
        
        // Same validator cannot vote twice
        assert_noop!(
            ChameleonBridge::report_lock_event(
                RuntimeOrigin::signed(VALIDATOR_1),
                TX_HASH,
                ETH_ADDRESS,
                ALICE,
                amount,
                WRAPPED_ETH,
            ),
            Error::<Test>::AlreadyVoted
        );
    });
}
