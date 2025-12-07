// Copyright 2020-2024 Manta Network.
// pDEX Tests - Simplified for CI/CD

use crate::pallet::*;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64, ConstU128},
    parameter_types,
    PalletId,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup, AccountIdConversion},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        ChameleonPdex: crate,
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

parameter_types! {
    pub const PdexPalletId: PalletId = PalletId(*b"pdex/amm");
    pub const MaxPools: u32 = 100;
    pub const MinimumLiquidity: u128 = 1000;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u128;
    type Balance = u128;
    type Assets = Assets;
    type PalletId = PdexPalletId;
    type MaxPools = MaxPools;
    type MinimumLiquidity = MinimumLiquidity;
}

const CHML: u128 = 1;
const ETH: u128 = 2;
const ALICE: u64 = 1;
const BOB: u64 = 2;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 10_000_000_000_000_000),
            (BOB, 10_000_000_000_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        
        // Create assets
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), CHML, ALICE, true, 1));
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), ETH, ALICE, true, 1));
        
        // Create LP token assets (1000-1009)
        let pallet_acc: u64 = PdexPalletId::get().into_account_truncating();
        for lp_id in 1000u128..1010u128 {
            assert_ok!(Assets::force_create(RuntimeOrigin::root(), lp_id, pallet_acc, true, 1));
        }
        
        // Mint tokens
        let balance = 1_000_000_000_000_000u128;
        for acc in [ALICE, BOB] {
            assert_ok!(Assets::mint(RuntimeOrigin::signed(ALICE), CHML, acc, balance));
            assert_ok!(Assets::mint(RuntimeOrigin::signed(ALICE), ETH, acc, balance));
        }
        
        // Fund pool accounts
        for pool_id in 0u32..10u32 {
            let pool_acc = ChameleonPdex::pool_account(pool_id);
            assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), pool_acc, 10_000_000_000_000u128));
        }
    });
    
    ext
}

#[test]
fn test_create_pool() {
    new_test_ext().execute_with(|| {
        assert_ok!(ChameleonPdex::create_pool(RuntimeOrigin::signed(ALICE), CHML, ETH));
        assert_eq!(ChameleonPdex::pool_count(), 1);
    });
}

#[test]
fn test_cannot_create_same_asset_pool() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ChameleonPdex::create_pool(RuntimeOrigin::signed(ALICE), CHML, CHML),
            Error::<Test>::SameAsset
        );
    });
}

#[test]
fn test_add_liquidity() {
    new_test_ext().execute_with(|| {
        assert_ok!(ChameleonPdex::create_pool(RuntimeOrigin::signed(ALICE), CHML, ETH));
        
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            0,
            100_000_000_000u128,
            50_000_000_000u128,
            0,
        ));
        
        let pool = ChameleonPdex::pools(0).unwrap();
        assert_eq!(pool.reserve_a, 100_000_000_000u128);
        assert_eq!(pool.reserve_b, 50_000_000_000u128);
    });
}

#[test]
fn test_swap() {
    new_test_ext().execute_with(|| {
        // Create pool and add liquidity
        assert_ok!(ChameleonPdex::create_pool(RuntimeOrigin::signed(ALICE), CHML, ETH));
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE), 0,
            1_000_000_000_000u128, 1_000_000_000_000u128, 0,
        ));
        
        let bob_eth_before = Assets::balance(ETH, &BOB);
        
        // BOB swaps CHML for ETH
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB), 0, CHML, 10_000_000_000u128, 0,
        ));
        
        // BOB should have more ETH now
        assert!(Assets::balance(ETH, &BOB) > bob_eth_before);
    });
}
