// Copyright 2020-2024 Manta Network.
// pDEX Tests - Production-Grade AMM Testing

use crate::pallet::*;
use frame_support::
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64, ConstU128, tokens::fungibles::{Inspect, Mutate}},
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
}

// Assets configuration
impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u32;
    type AssetIdParameter = u32;
    type Currency = Balances;
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU128<100>;
    type AssetAccountDeposit = ConstU128<10>;
    type MetadataDepositBase = ConstU128<10>;
    type MetadataDepositPerByte = ConstU128<1>;
    type ApprovalDeposit = ConstU128<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<1000>;
}

// pDEX configuration
impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u32;
    type Balance = u128;
    type Assets = Assets;
    type PalletId = ConstPalletId;
    type MaxPools = ConstU32<100>;
    type MinimumLiquidity = ConstU128<1000>;
}

// Constants
parameter_types! {
    pub const ConstPalletId: PalletId = PalletId(*b"pdex/amm");
}

// Test asset IDs
const CHML: u32 = 1;
const ETH: u32 = 2;
const USDC: u32 = 3;

// Test accounts
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

// Helper function to create test environment
fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    // Initialize balances for test accounts
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000_000_000), // 1M native tokens
            (BOB, 1_000_000_000_000),
            (CHARLIE, 1_000_000_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        
        // Create test assets
        assert_ok!(Assets::create(
            RuntimeOrigin::signed(ALICE),
            CHML.into(),
            ALICE,
            1000, // min_balance
        ));
        
        assert_ok!(Assets::create(
            RuntimeOrigin::signed(ALICE),
            ETH.into(),
            ALICE,
            1000,
        ));
        
        assert_ok!(Assets::create(
            RuntimeOrigin::signed(ALICE),
            USDC.into(),
            ALICE,
            1000,
        ));
        
        // Mint tokens to test accounts
        let initial_balance = 1_000_000_000_000u128; // 1M tokens
        
        for account in [ALICE, BOB, CHARLIE] {
            for asset in [CHML, ETH, USDC] {
                assert_ok!(Assets::mint(
                    RuntimeOrigin::signed(ALICE),
                    asset.into(),
                    account,
                    initial_balance,
                ));
            }
        }
    });
    
    ext
}

// Helper function to get pool account
fn pool_account(pool_id: u32) -> u64 {
    ChameleonPdex::pool_account(pool_id)
}

// ============================================================================
// REQUIRED TESTS (Minimum 4)
// ============================================================================

#[test]
fn test_add_liquidity_transfers_tokens() {
    new_test_ext().execute_with(|| {
        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        let pool_acc = pool_account(pool_id);
        
        // Check initial balances
        let alice_chml_before = Assets::balance(CHML, &ALICE);
        let alice_eth_before = Assets::balance(ETH, &ALICE);
        let pool_chml_before = Assets::balance(CHML, &pool_acc);
        let pool_eth_before = Assets::balance(ETH, &pool_acc);
        
        let amount_chml = 100_000_000u128; // 100 CHML
        let amount_eth = 10_000_000u128;   // 10 ETH
        
        // Add liquidity
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            amount_chml,
            amount_eth,
            0, // min_lp_tokens
        ));
        
        // Verify token transfers
        // Alice balance should decrease
        assert_eq!(
            Assets::balance(CHML, &ALICE),
            alice_chml_before - amount_chml,
            "Alice CHML balance should decrease"
        );
        assert_eq!(
            Assets::balance(ETH, &ALICE),
            alice_eth_before - amount_eth,
            "Alice ETH balance should decrease"
        );
        
        // Pool balance should increase
        assert_eq!(
            Assets::balance(CHML, &pool_acc),
            pool_chml_before + amount_chml,
            "Pool CHML balance should increase"
        );
        assert_eq!(
            Assets::balance(ETH, &pool_acc),
            pool_eth_before + amount_eth,
            "Pool ETH balance should increase"
        );
        
        // Verify LP tokens minted
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        let expected_lp = crate::pallet::integer_sqrt(amount_chml * amount_eth);
        assert_eq!(
            Assets::balance(pool.lp_asset_id, &ALICE),
            expected_lp,
            "LP tokens should be minted to Alice"
        );
        
        // Verify pool reserves updated
        assert_eq!(pool.reserve_a, amount_chml, "Pool reserve A should match");
        assert_eq!(pool.reserve_b, amount_eth, "Pool reserve B should match");
        assert_eq!(pool.total_lp_tokens, expected_lp, "Total LP tokens should match");
        
        // Verify event emitted
        System::assert_has_event(
            Event::LiquidityAdded {
                pool_id,
                provider: ALICE,
                amount_a: amount_chml,
                amount_b: amount_eth,
                lp_minted: expected_lp,
            }.into()
        );
    });
}

#[test]
fn test_swap_updates_reserves() {
    new_test_ext().execute_with(|| {
        // Create pool and add liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        let initial_chml = 1_000_000u128; // 1M CHML
        let initial_eth = 100_000u128;    // 100K ETH (1 ETH = 10 CHML)
        
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            initial_chml,
            initial_eth,
            0,
        ));
        
        // Get initial k value (constant product)
        let pool_before = ChameleonPdex::pools(pool_id).unwrap();
        let k_before = pool_before.reserve_a * pool_before.reserve_b;
        
        // Perform swap: 10,000 CHML -> ETH
        let swap_amount_in = 10_000u128;
        
        // Calculate expected output using AMM formula
        // amount_out = reserve_out * amount_in_with_fee / (reserve_in + amount_in_with_fee)
        let fee_factor = 10000u128 - 25; // 0.25% fee = 25 basis points
        let amount_in_with_fee = swap_amount_in * fee_factor / 10000;
        let expected_amount_out = initial_eth * amount_in_with_fee / (initial_chml + amount_in_with_fee);
        
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_id,
            CHML,
            swap_amount_in,
            0, // min_amount_out (no slippage protection for test)
        ));
        
        // Verify reserves updated correctly
        let pool_after = ChameleonPdex::pools(pool_id).unwrap();
        
        // Reserve A (CHML) should increase by swap amount
        assert_eq!(
            pool_after.reserve_a,
            initial_chml + swap_amount_in,
            "CHML reserve should increase by swap amount"
        );
        
        // Reserve B (ETH) should decrease by output amount
        assert_eq!(
            pool_after.reserve_b,
            initial_eth - expected_amount_out,
            "ETH reserve should decrease by output amount"
        );
        
        // Verify constant product formula (k should increase due to fees)
        let k_after = pool_after.reserve_a * pool_after.reserve_b;
        assert!(
            k_after >= k_before,
            "Constant product should increase due to fees: {} >= {}",
            k_after,
            k_before
        );
        
        // Verify actual token transfers
        let pool_acc = pool_account(pool_id);
        assert_eq!(
            Assets::balance(CHML, &pool_acc),
            initial_chml + swap_amount_in,
            "Pool CHML balance should match reserve"
        );
        assert_eq!(
            Assets::balance(ETH, &pool_acc),
            initial_eth - expected_amount_out,
            "Pool ETH balance should match reserve"
        );
        
        // Verify Bob received ETH
        assert_eq!(
            Assets::balance(ETH, &BOB),
            1_000_000_000_000u128 + expected_amount_out, // initial + received
            "Bob should receive ETH from swap"
        );
    });
}

#[test]
fn test_remove_liquidity_returns_tokens() {
    new_test_ext().execute_with(|| {
        // Create pool and add liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        let amount_chml = 100_000u128;
        let amount_eth = 10_000u128;
        
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            amount_chml,
            amount_eth,
            0,
        ));
        
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        let lp_tokens = Assets::balance(pool.lp_asset_id, &ALICE);
        
        // Remove half the liquidity
        let lp_to_remove = lp_tokens / 2;
        
        // Calculate expected returns
        let expected_chml = lp_to_remove * amount_chml / lp_tokens;
        let expected_eth = lp_to_remove * amount_eth / lp_tokens;
        
        // Get balances before removal
        let alice_chml_before = Assets::balance(CHML, &ALICE);
        let alice_eth_before = Assets::balance(ETH, &ALICE);
        let alice_lp_before = Assets::balance(pool.lp_asset_id, &ALICE);
        
        assert_ok!(ChameleonPdex::remove_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            lp_to_remove,
            0, // min_amount_a
            0, // min_amount_b
        ));
        
        // Verify LP tokens burned
        assert_eq!(
            Assets::balance(pool.lp_asset_id, &ALICE),
            alice_lp_before - lp_to_remove,
            "LP tokens should be burned"
        );
        
        // Verify tokens returned to Alice
        assert_eq!(
            Assets::balance(CHML, &ALICE),
            alice_chml_before + expected_chml,
            "Alice should receive CHML back"
        );
        assert_eq!(
            Assets::balance(ETH, &ALICE),
            alice_eth_before + expected_eth,
            "Alice should receive ETH back"
        );
        
        // Verify pool reserves decreased
        let pool_after = ChameleonPdex::pools(pool_id).unwrap();
        assert_eq!(
            pool_after.reserve_a,
            amount_chml - expected_chml,
            "Pool CHML reserve should decrease"
        );
        assert_eq!(
            pool_after.reserve_b,
            amount_eth - expected_eth,
            "Pool ETH reserve should decrease"
        );
        assert_eq!(
            pool_after.total_lp_tokens,
            lp_tokens - lp_to_remove,
            "Total LP tokens should decrease"
        );
    });
}

#[test]
fn test_slippage_protection() {
    new_test_ext().execute_with(|| {
        // Create pool with liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            1_000_000u128, // 1M CHML
            100_000u128,   // 100K ETH
            0,
        ));
        
        // Calculate expected output for 10K CHML swap
        let swap_amount = 10_000u128;
        let fee_factor = 10000u128 - 25; // 0.25% fee
        let amount_in_with_fee = swap_amount * fee_factor / 10000;
        let expected_out = 100_000u128 * amount_in_with_fee / (1_000_000u128 + amount_in_with_fee);
        
        // Test 1: Swap should succeed with reasonable slippage tolerance
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_id,
            CHML,
            swap_amount,
            expected_out - 10, // Allow small slippage
        ));
        
        // Test 2: Swap should fail if slippage tolerance too tight
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                CHML,
                swap_amount,
                expected_out + 1000, // Unrealistic expectation
            ),
            Error::<Test>::SlippageExceeded
        );
        
        // Test 3: Add liquidity slippage protection
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        let expected_lp = 1000u128 * pool.total_lp_tokens / pool.reserve_a; // Proportional LP tokens
        
        assert_noop!(
            ChameleonPdex::add_liquidity(
                RuntimeOrigin::signed(BOB),
                pool_id,
                1000u128,
                100u128,
                expected_lp + 1000, // Unrealistic LP expectation
            ),
            Error::<Test>::SlippageExceeded
        );
        
        // Test 4: Remove liquidity slippage protection
        let lp_balance = Assets::balance(pool.lp_asset_id, &ALICE);
        let lp_to_remove = lp_balance / 10; // Remove 10%
        
        assert_noop!(
            ChameleonPdex::remove_liquidity(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                lp_to_remove,
                1_000_000u128, // Unrealistic CHML expectation
                1_000_000u128, // Unrealistic ETH expectation
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}

// ============================================================================
// ADDITIONAL COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_create_pool_works() {
    new_test_ext().execute_with(|| {
        // Test successful pool creation
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool = ChameleonPdex::pools(0).unwrap();
        assert_eq!(pool.asset_a, CHML);
        assert_eq!(pool.asset_b, ETH);
        assert_eq!(pool.reserve_a, 0);
        assert_eq!(pool.reserve_b, 0);
        assert_eq!(pool.total_lp_tokens, 0);
        
        // Verify event
        System::assert_has_event(
            Event::PoolCreated {
                pool_id: 0,
                asset_a: CHML,
                asset_b: ETH,
                lp_asset_id: pool.lp_asset_id,
            }.into()
        );
        
        // Test error cases
        assert_noop!(
            ChameleonPdex::create_pool(
                RuntimeOrigin::signed(ALICE),
                CHML,
                CHML, // Same asset
            ),
            Error::<Test>::SameAsset
        );
    });
}

#[test]
fn test_proportional_liquidity_provision() {
    new_test_ext().execute_with(|| {
        // Create pool and add initial liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            1000u128,
            100u128,
            0,
        ));
        
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        let initial_lp = pool.total_lp_tokens;
        
        // Bob adds proportional liquidity (2x the amounts)
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(BOB),
            pool_id,
            2000u128, // 2x CHML
            200u128,  // 2x ETH
            0,
        ));
        
        let pool_after = ChameleonPdex::pools(pool_id).unwrap();
        let bob_lp = Assets::balance(pool_after.lp_asset_id, &BOB);
        
        // Bob should get 2x the LP tokens Alice got
        assert_eq!(bob_lp, initial_lp * 2, "Bob should get proportional LP tokens");
        
        // Total reserves should be 3x original
        assert_eq!(pool_after.reserve_a, 3000u128);
        assert_eq!(pool_after.reserve_b, 300u128);
        assert_eq!(pool_after.total_lp_tokens, initial_lp * 3);
    });
}

#[test]
fn test_large_swap_price_impact() {
    new_test_ext().execute_with(|| {
        // Create pool with moderate liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            100_000u128, // 100K CHML
            10_000u128,  // 10K ETH (1 ETH = 10 CHML)
            0,
        ));
        
        // Small swap (1% of liquidity) - should have minimal price impact
        let small_swap = 1_000u128; // 1K CHML (1% of pool)
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_id,
            CHML,
            small_swap,
            0,
        ));
        
        // Large swap (50% of liquidity) - should have significant price impact
        let large_swap = 50_000u128; // 50K CHML (50% of pool)
        
        // This should work but with poor exchange rate due to price impact
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(CHARLIE),
            pool_id,
            CHML,
            large_swap,
            0, // No slippage protection for test
        ));
        
        let pool_after = ChameleonPdex::pools(pool_id).unwrap();
        
        // Pool should still maintain some liquidity
        assert!(pool_after.reserve_b > 0, "Pool should maintain ETH liquidity");
        assert!(pool_after.reserve_a > 100_000u128, "Pool CHML should increase");
    });
}

#[test]
fn test_zero_amount_rejections() {
    new_test_ext().execute_with(|| {
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        
        // Zero amounts should be rejected
        assert_noop!(
            ChameleonPdex::add_liquidity(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                0, // Zero CHML
                1000u128,
                0,
            ),
            Error::<Test>::ZeroAmount
        );
        
        assert_noop!(
            ChameleonPdex::add_liquidity(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                1000u128,
                0, // Zero ETH
                0,
            ),
            Error::<Test>::ZeroAmount
        );
        
        // Add some liquidity first
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            1000u128,
            100u128,
            0,
        ));
        
        // Zero swap amount should be rejected
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                CHML,
                0, // Zero amount
                0,
            ),
            Error::<Test>::ZeroAmount
        );
        
        // Zero LP removal should be rejected
        assert_noop!(
            ChameleonPdex::remove_liquidity(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                0, // Zero LP tokens
                0,
                0,
            ),
            Error::<Test>::ZeroAmount
        );
    });
}

#[test]
fn test_pool_not_found_errors() {
    new_test_ext().execute_with(|| {
        let non_existent_pool = 999u32;
        
        assert_noop!(
            ChameleonPdex::add_liquidity(
                RuntimeOrigin::signed(ALICE),
                non_existent_pool,
                1000u128,
                100u128,
                0,
            ),
            Error::<Test>::PoolNotFound
        );
        
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(ALICE),
                non_existent_pool,
                CHML,
                1000u128,
                0,
            ),
            Error::<Test>::PoolNotFound
        );
        
        assert_noop!(
            ChameleonPdex::remove_liquidity(
                RuntimeOrigin::signed(ALICE),
                non_existent_pool,
                1000u128,
                0,
                0,
            ),
            Error::<Test>::PoolNotFound
        );
    });
}

#[test]
fn test_insufficient_liquidity_protection() {
    new_test_ext().execute_with(|| {
        // Create pool with minimal liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            1000u128,
            100u128,
            0,
        ));
        
        // Try to swap more than available liquidity
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                ETH,
                200u128, // More than 100 ETH in pool
                0,
            ),
            Error::<Test>::InsufficientLiquidity
        );
        
        // Try to swap from empty pool (remove all liquidity first)
        let pool = ChameleonPdex::pools(pool_id).unwrap();
        let all_lp = Assets::balance(pool.lp_asset_id, &ALICE);
        
        assert_ok!(ChameleonPdex::remove_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            all_lp,
            0,
            0,
        ));
        
        // Now pool is empty, swaps should fail
        assert_noop!(
            ChameleonPdex::swap(
                RuntimeOrigin::signed(BOB),
                pool_id,
                CHML,
                1000u128,
                0,
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn test_fee_calculation_and_treasury() {
    new_test_ext().execute_with(|| {
        // Create pool and add liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        let pool_id = 0;
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            1_000_000u128,
            100_000u128,
            0,
        ));
        
        let swap_amount = 10_000u128;
        let expected_fee = swap_amount * 25 / 10000; // 0.25% fee
        let expected_treasury_fee = expected_fee * 1000 / 10000; // 10% to treasury
        
        // Check treasury fees before swap
        let treasury_before = ChameleonPdex::treasury_fees(CHML);
        
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_id,
            CHML,
            swap_amount,
            0,
        ));
        
        // Check treasury fees after swap
        let treasury_after = ChameleonPdex::treasury_fees(CHML);
        assert_eq!(
            treasury_after,
            treasury_before + expected_treasury_fee,
            "Treasury should receive 10% of swap fees"
        );
        
        // Verify event includes fee information
        let events = System::events();
        let swap_event = events.iter().find(|e| {
            matches!(e.event, RuntimeEvent::ChameleonPdex(Event::Swapped { .. }))
        }).unwrap();
        
        if let RuntimeEvent::ChameleonPdex(Event::Swapped { fee, .. }) = &swap_event.event {
            assert_eq!(*fee, expected_fee, "Event should include correct fee amount");
        } else {
            panic!("Expected Swapped event");
        }
    });
}

#[test]
fn test_multiple_pools_independence() {
    new_test_ext().execute_with(|| {
        // Create multiple pools
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
        ));
        
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            USDC,
        ));
        
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            ETH,
            USDC,
        ));
        
        let pool_0 = 0u32; // CHML/ETH
        let pool_1 = 1u32; // CHML/USDC
        let pool_2 = 2u32; // ETH/USDC
        
        // Add liquidity to each pool with different ratios
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_0,
            1000u128, // 1 ETH = 10 CHML
            100u128,
            0,
        ));
        
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_1,
            2000u128, // 1 USDC = 2 CHML
            1000u128,
            0,
        ));
        
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            pool_2,
            50u128,   // 1 ETH = 5 USDC
            250u128,
            0,
        ));
        
        // Verify pools are independent
        let pool_0_data = ChameleonPdex::pools(pool_0).unwrap();
        let pool_1_data = ChameleonPdex::pools(pool_1).unwrap();
        let pool_2_data = ChameleonPdex::pools(pool_2).unwrap();
        
        assert_ne!(pool_0_data.lp_asset_id, pool_1_data.lp_asset_id);
        assert_ne!(pool_1_data.lp_asset_id, pool_2_data.lp_asset_id);
        
        // Swap in one pool shouldn't affect others
        let pool_1_reserves_before = (pool_1_data.reserve_a, pool_1_data.reserve_b);
        let pool_2_reserves_before = (pool_2_data.reserve_a, pool_2_data.reserve_b);
        
        assert_ok!(ChameleonPdex::swap(
            RuntimeOrigin::signed(BOB),
            pool_0,
            CHML,
            100u128,
            0,
        ));
        
        let pool_1_after = ChameleonPdex::pools(pool_1).unwrap();
        let pool_2_after = ChameleonPdex::pools(pool_2).unwrap();
        
        // Other pools should be unchanged
        assert_eq!(
            (pool_1_after.reserve_a, pool_1_after.reserve_b),
            pool_1_reserves_before,
            "Pool 1 should be unaffected by Pool 0 swap"
        );
        assert_eq!(
            (pool_2_after.reserve_a, pool_2_after.reserve_b),
            pool_2_reserves_before,
            "Pool 2 should be unaffected by Pool 0 swap"
        );
    });
}
