// Copyright 2020-2024 Manta Network.
// pDEX Tests - Production-Grade AMM Testing

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

// Mock runtime construction
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        ChameleonPdex: crate,
    }
);

// System configuration
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

// Balances configuration
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

// Assets configuration
impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u128;
    type AssetIdParameter = u128;
    type Currency = Balances;
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU128<0>;  // No deposit for easier testing
    type AssetAccountDeposit = ConstU128<0>;  // No deposit for easier testing
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

// pDEX configuration
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

// Test asset IDs
const CHML: u128 = 1;
const ETH: u128 = 2;
const USDC: u128 = 3;

// Test accounts
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

// Helper function to get pallet account
fn get_pallet_account() -> u64 {
    PdexPalletId::get().into_account_truncating()
}

// Helper function to create test environment
fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    // Initialize native balances for test accounts
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 10_000_000_000_000_000),
            (BOB, 10_000_000_000_000_000),
            (CHARLIE, 10_000_000_000_000_000),
            (get_pallet_account(), 10_000_000_000_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        
        // Create test assets with force_create (no deposit required)
        // Using is_sufficient = true means accounts don't need native balance to hold assets
        for asset_id in [CHML, ETH, USDC] {
            assert_ok!(Assets::force_create(
                RuntimeOrigin::root(),
                asset_id,
                ALICE,  // Admin
                true,   // is_sufficient
                1,      // min_balance
            ));
        }
        
        // Pre-create LP token assets (IDs 1000+)
        // The pallet account needs to be admin to mint LP tokens
        let pallet_acc = get_pallet_account();
        for lp_id in 1000u128..1020u128 {
            assert_ok!(Assets::force_create(
                RuntimeOrigin::root(),
                lp_id,
                pallet_acc,  // Pallet is admin - can mint
                true,        // is_sufficient
                1,           // min_balance
            ));
        }
        
        // Mint tokens to test accounts
        let initial_balance = 1_000_000_000_000_000u128;
        for account in [ALICE, BOB, CHARLIE] {
            for asset in [CHML, ETH, USDC] {
                assert_ok!(Assets::mint(
                    RuntimeOrigin::signed(ALICE),
                    asset,
                    account,
                    initial_balance,
                ));
            }
        }
        
        // Fund pool accounts (pool 0-9) with native balance
        for pool_id in 0u32..10u32 {
            let pool_acc = ChameleonPdex::pool_account(pool_id);
            // Give native balance
            assert_ok!(Balances::force_set_balance(
                RuntimeOrigin::root(),
                pool_acc,
                10_000_000_000_000u128,
            ));
        }
    });
    
    ext
}

// Helper to setup a pool with initial liquidity
fn setup_pool_with_liquidity(asset_a: u128, asset_b: u128, amount_a: u128, amount_b: u128) -> u32 {
    // Create pool
    assert_ok!(ChameleonPdex::create_pool(
        RuntimeOrigin::signed(ALICE),
        asset_a,
        asset_b,
    ));
    
    let pool_id = ChameleonPdex::pool_count() - 1;
    
    // Add initial liquidity
    assert_ok!(ChameleonPdex::add_liquidity(
        RuntimeOrigin::signed(ALICE),
        pool_id,
        amount_a,
        amount_b,
        0,  // min_lp_tokens
    ));
    
    pool_id
}

// ============================================================================
// CORE TESTS
// ============================================================================

#[test]
fn test_create_pool_works() {
    new_test_ext().execute_with(|| {
        // Create a pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        // Verify pool exists
        assert_eq!(ChameleonPdex::pool_count(), 1);
        
        // Check pool data
        let pool = ChameleonPdex::pools(0).unwrap();
        assert_eq!(pool.asset_a, CHML);
        assert_eq!(pool.asset_b, ETH);
        assert_eq!(pool.reserve_a, 0);
        assert_eq!(pool.reserve_b, 0);
    });
}

#[test]
fn test_cannot_create_pool_with_same_asset() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ChameleonPdex::create_pool(
                RuntimeOrigin::signed(ALICE),
                CHML,
                CHML,
            ),
            Error::<Test>::SameAsset
        );
    });
}

#[test]
fn test_add_liquidity_transfers_tokens() {
    new_test_ext().execute_with(|| {
        let amount_chml = 100_000_000_000u128;
        let amount_eth = 50_000_000_000u128;
        
        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        let pool_acc = ChameleonPdex::pool_account(pool_id);
        
        // Check initial balances
        let alice_chml_before = Assets::balance(CHML, &ALICE);
        let alice_eth_before = Assets::balance(ETH, &ALICE);
        let pool_chml_before = Assets::balance(CHML, &pool_acc);
        let pool_eth_before = Assets::balance(ETH, &pool_acc);
        
        // Add liquidity
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            amount_chml,
            amount_eth,
            0,  // min_lp_tokens
        ));
        
        // Verify ALICE's balances decreased
        assert_eq!(Assets::balance(CHML, &ALICE), alice_chml_before - amount_chml);
        assert_eq!(Assets::balance(ETH, &ALICE), alice_eth_before - amount_eth);
        
        // Verify pool's balances increased
        assert_eq!(Assets::balance(CHML, &pool_acc), pool_chml_before + amount_chml);
        assert_eq!(Assets::balance(ETH, &pool_acc), pool_eth_before + amount_eth);
        
        // Verify pool reserves updated
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        assert_eq!(pool.reserve_a, amount_chml);
        assert_eq!(pool.reserve_b, amount_eth);
    });
}

#[test]
fn test_swap_updates_reserves() {
    new_test_ext().execute_with(|| {
        // Setup pool with liquidity
        let pool_id = setup_pool_with_liquidity(CHML, ETH, 1_000_000_000_000u128, 500_000_000_000u128);
        
        let pool_before = ChameleonPdex::pools(pool_id).unwrap();
        let k_before = pool_before.reserve_a as u128 * pool_before.reserve_b as u128;
        
        // Perform swap
        let swap_amount = 10_000_000_000u128;
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_id,
            CHML,
            swap_amount,
            0,  // min_amount_out
        ));
        
        // Verify reserves updated
        let pool_after = ChameleonPdex::pools(pool_id).unwrap();
        
        // Reserve A should increase (CHML added)
        assert!(pool_after.reserve_a > pool_before.reserve_a);
        // Reserve B should decrease (ETH removed)
        assert!(pool_after.reserve_b < pool_before.reserve_b);
        
        // K should remain constant or increase (due to fees)
        let k_after = pool_after.reserve_a as u128 * pool_after.reserve_b as u128;
        assert!(k_after >= k_before);
    });
}

#[test]
fn test_remove_liquidity_returns_tokens() {
    new_test_ext().execute_with(|| {
        // Setup pool with liquidity
        let amount_chml = 1_000_000_000_000u128;
        let amount_eth = 500_000_000_000u128;
        
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        
        // Add liquidity
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            amount_chml,
            amount_eth,
            0,
        ));
        
        // Get LP balance
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        let lp_balance = Assets::balance(pool.lp_asset_id, &ALICE);
        
        // Remove half liquidity
        let remove_amount = lp_balance / 2;
        let alice_chml_before = Assets::balance(CHML, &ALICE);
        let alice_eth_before = Assets::balance(ETH, &ALICE);
        
        assert_ok!(ChameleonPdex::remove_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            remove_amount,
            0,  // min_amount_a
            0,  // min_amount_b
        ));
        
        // Verify tokens returned
        assert!(Assets::balance(CHML, &ALICE) > alice_chml_before);
        assert!(Assets::balance(ETH, &ALICE) > alice_eth_before);
        
        // Verify LP tokens burned
        assert_eq!(Assets::balance(pool.lp_asset_id, &ALICE), lp_balance - remove_amount);
    });
}

#[test]
fn test_slippage_protection() {
    new_test_ext().execute_with(|| {
        // Setup pool
        let pool_id = setup_pool_with_liquidity(CHML, ETH, 1_000_000_000_000u128, 1_000_000_000_000u128);
        
        // Try swap with very high minimum output (should fail)
        let swap_amount = 10_000_000_000u128;
        let unrealistic_min_out = 100_000_000_000u128;  // More than possible
        
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                CHML,
                swap_amount,
                unrealistic_min_out,
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}

#[test]
fn test_pool_not_found() {
    new_test_ext().execute_with(|| {
        // Try to add liquidity to non-existent pool
        assert_noop!(
            ChameleonPdex::add_liquidity(
                RuntimeOrigin::signed(ALICE),
                999,  // Non-existent pool
                1000,
                1000,
                0,
            ),
            Error::<Test>::PoolNotFound
        );
    });
}

#[test]
fn test_zero_amount_rejected() {
    new_test_ext().execute_with(|| {
        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        // Try to add zero liquidity
        assert_noop!(
            ChameleonPdex::add_liquidity(
                RuntimeOrigin::signed(ALICE),
                0,
                0,  // Zero amount
                1000,
                0,
            ),
            Error::<Test>::InvalidAmounts
        );
    });
}

#[test]
fn test_multiple_liquidity_providers() {
    new_test_ext().execute_with(|| {
        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        
        // ALICE adds initial liquidity
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            1_000_000_000_000u128,
            500_000_000_000u128,
            0,
        ));
        
        // BOB adds more liquidity
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(BOB),
            pool_id,
            500_000_000_000u128,
            250_000_000_000u128,
            0,
        ));
        
        // Verify both have LP tokens
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        assert!(Assets::balance(pool.lp_asset_id, &ALICE) > 0);
        assert!(Assets::balance(pool.lp_asset_id, &BOB) > 0);
        
        // Verify total reserves
        assert_eq!(pool.reserve_a, 1_500_000_000_000u128);
        assert_eq!(pool.reserve_b, 750_000_000_000u128);
    });
}

#[test]
fn test_swap_both_directions() {
    new_test_ext().execute_with(|| {
        // Setup pool
        let pool_id = setup_pool_with_liquidity(CHML, ETH, 1_000_000_000_000u128, 1_000_000_000_000u128);
        
        // Swap CHML -> ETH
        let bob_eth_before = Assets::balance(ETH, &BOB);
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_id,
            CHML,
            10_000_000_000u128,
            0,
        ));
        assert!(Assets::balance(ETH, &BOB) > bob_eth_before);
        
        // Swap ETH -> CHML
        let charlie_chml_before = Assets::balance(CHML, &CHARLIE);
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(CHARLIE),
            pool_id,
            ETH,
            10_000_000_000u128,
            0,
        ));
        assert!(Assets::balance(CHML, &CHARLIE) > charlie_chml_before);
    });
}

#[test]
fn test_constant_product_maintained() {
    new_test_ext().execute_with(|| {
        // Setup pool
        let pool_id = setup_pool_with_liquidity(CHML, ETH, 1_000_000_000_000u128, 1_000_000_000_000u128);
        
        let pool_before = ChameleonPdex::pools(pool_id).unwrap();
        let k_before = pool_before.reserve_a * pool_before.reserve_b;
        
        // Multiple swaps
        for _ in 0..5 {
            assert_ok!(ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                CHML,
                1_000_000_000u128,
                0,
            ));
        }
        
        let pool_after = ChameleonPdex::pools(pool_id).unwrap();
        let k_after = pool_after.reserve_a * pool_after.reserve_b;
        
        // K should be maintained or increase (due to fees)
        assert!(k_after >= k_before);
    });
}

#[test]
fn test_insufficient_liquidity() {
    new_test_ext().execute_with(|| {
        // Setup pool with small liquidity
        let pool_id = setup_pool_with_liquidity(CHML, ETH, 1_000_000u128, 1_000_000u128);
        
        // Try to swap more than available
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                CHML,
                10_000_000_000_000u128,  // Way too much
                1,  // Expecting at least 1
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}
