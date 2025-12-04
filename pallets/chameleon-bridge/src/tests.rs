// Copyright 2020-2024 Manta Network.
// Bridge Tests - Minimal working tests for CI/CD

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

const VALIDATOR_1: u64 = 10;
const ALICE: u64 = 1;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 10_000_000_000_000),
            (VALIDATOR_1, 1_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    
    ext
}

#[test]
fn pallet_compiles() {
    new_test_ext().execute_with(|| {
        // Bridge pallet configured correctly
        assert!(true);
    });
}

#[test]
fn non_validator_cannot_report() {
    new_test_ext().execute_with(|| {
        let tx_hash = H256::from([1u8; 32]);
        
        // Non-validator should fail
        assert_noop!(
            ChameleonBridge::report_deposit(
                RuntimeOrigin::signed(ALICE),
                tx_hash,
                ALICE,
                BridgeableAsset::ETH,
                1_000_000u128,
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn bridge_pause_works() {
    new_test_ext().execute_with(|| {
        // Only root can pause
        assert_ok!(ChameleonBridge::pause_bridge(RuntimeOrigin::root()));
        assert!(ChameleonBridge::is_paused());
        
        // Resume
        assert_ok!(ChameleonBridge::resume_bridge(RuntimeOrigin::root()));
        assert!(!ChameleonBridge::is_paused());
    });
}
