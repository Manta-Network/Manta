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

//! Tests for the Chameleon pDEX pallet

use crate::{
    mock::*,
    types::*,
    amm::*,
    rewards::*,
    Error,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{fungibles::Inspect, Get},
};
use sp_runtime::Perbill;

// Mock runtime for testing
mod mock {
    use super::*;
    use frame_support::{
        construct_runtime, parameter_types,
        traits::{ConstU32, ConstU64, Everything},
        weights::Weight,
        PalletId,
    };
    use frame_system as system;
    use sp_core::H256;
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };

    type Block = frame_system::mocking::MockBlock<Test>;
    type AccountId = u64;
    type AssetId = u32;
    type Balance = u128;

    // Configure a mock runtime to test the pallet.
    construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Assets: pallet_assets,
            Balances: pallet_balances,
            ChameleonPdex: crate,
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
        type MaxHolds = ConstU32<1>;
        type MaxFreezes = ConstU32<1>;
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
        type AssetIdParameter = AssetId;
        type Currency = Balances;
        type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
        type ForceOrigin = frame_system::EnsureRoot<AccountId>;
        type AssetDeposit = AssetDeposit;
        type AssetAccountDeposit = ConstU64<1>;
        type MetadataDepositBase = MetadataDepositBase;
        type MetadataDepositPerByte = MetadataDepositPerByte;
        type ApprovalDeposit = ApprovalDeposit;
        type StringLimit = StringLimit;
        type Freezer = ();
        type Extra = ();
        type CallbackHandle = ();
        type WeightInfo = ();
        type RemoveItemsLimit = ConstU32<1000>;
    }

    parameter_types! {
        pub const PdexPalletId: PalletId = PalletId(*b"chmlpdex");
        pub const MaxPools: u32 = 1000;
        pub const MinimumLiquidity: Balance = 1000;
    }

    impl crate::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type AssetId = AssetId;
        type Balance = Balance;
        type WeightInfo = ();
        type PalletId = PdexPalletId;
        type MaxPools = MaxPools;
        type MinimumLiquidity = MinimumLiquidity;
        type Currency = Assets;
    }

    // Build genesis storage according to the mock runtime.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = system::GenesisConfig::<Test>::default().build_storage().unwrap();
        
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 1_000_000_000_000), // Alice
                (2, 1_000_000_000_000), // Bob
                (3, 1_000_000_000_000), // Charlie
            ],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        storage.into()
    }

    pub fn create_asset(asset_id: AssetId, admin: AccountId, min_balance: Balance) {
        assert_ok!(Assets::create(
            RuntimeOrigin::signed(admin),
            asset_id.into(),
            admin.into(),
            min_balance
        ));
    }

    pub fn mint_asset(asset_id: AssetId, to: AccountId, amount: Balance) {
        assert_ok!(Assets::mint(
            RuntimeOrigin::signed(1), // Admin
            asset_id.into(),
            to.into(),
            amount
        ));
    }
}

// Test constants
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const CHML: u32 = 1;
const ETH: u32 = 2;
const USDC: u32 = 3;

#[test]
fn create_pool_works() {
    new_test_ext().execute_with(|| {
        // Create assets
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);

        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));

        // Check pool exists
        let pool = ChameleonPdex::pools(CHML, ETH).unwrap();
        assert_eq!(pool.asset_a, CHML);
        assert_eq!(pool.asset_b, ETH);
        assert_eq!(pool.reserve_a, 0);
        assert_eq!(pool.reserve_b, 0);
        assert_eq!(pool.total_lp_tokens, 0);
    });
}

#[test]
fn create_pool_fails_with_same_asset() {
    new_test_ext().execute_with(|| {
        create_asset(CHML, ALICE, 1);

        // Try to create pool with same asset
        assert_noop!(
            ChameleonPdex::create_pool(
                RuntimeOrigin::signed(ALICE),
                CHML,
                CHML
            ),
            Error::<Test>::InvalidAssetPair
        );
    });
}

#[test]
fn create_pool_fails_if_already_exists() {
    new_test_ext().execute_with(|| {
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);

        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));

        // Try to create same pool again
        assert_noop!(
            ChameleonPdex::create_pool(
                RuntimeOrigin::signed(BOB),
                CHML,
                ETH
            ),
            Error::<Test>::PoolAlreadyExists
        );

        // Try to create reversed pool
        assert_noop!(
            ChameleonPdex::create_pool(
                RuntimeOrigin::signed(BOB),
                ETH,
                CHML
            ),
            Error::<Test>::PoolAlreadyExists
        );
    });
}

#[test]
fn add_liquidity_initial_works() {
    new_test_ext().execute_with(|| {
        // Setup
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);
        mint_asset(CHML, ALICE, 10_000);
        mint_asset(ETH, ALICE, 100);

        // Create pool
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));

        // Add initial liquidity
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
            1000, // amount_a_desired
            10,   // amount_b_desired
            1000, // amount_a_min
            10,   // amount_b_min
        ));

        // Check pool state
        let pool = ChameleonPdex::pools(CHML, ETH).unwrap();
        assert_eq!(pool.reserve_a, 1000);
        assert_eq!(pool.reserve_b, 10);
        // LP tokens = sqrt(1000 * 10) = sqrt(10000) = 100
        assert_eq!(pool.total_lp_tokens, 100);

        // Check LP position
        let pool_id = PoolId::new(CHML, ETH);
        let position = ChameleonPdex::lp_positions(ALICE, pool_id).unwrap();
        assert_eq!(position.lp_tokens, 100);
    });
}

#[test]
fn add_liquidity_subsequent_works() {
    new_test_ext().execute_with(|| {
        // Setup
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);
        mint_asset(CHML, ALICE, 10_000);
        mint_asset(ETH, ALICE, 100);
        mint_asset(CHML, BOB, 10_000);
        mint_asset(ETH, BOB, 100);

        // Create pool and add initial liquidity
        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
            1000, 10, 1000, 10
        ));

        // Bob adds liquidity (should maintain ratio)
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(BOB),
            CHML,
            ETH,
            500, // amount_a_desired
            5,   // amount_b_desired
            500, // amount_a_min
            5,   // amount_b_min
        ));

        // Check pool state
        let pool = ChameleonPdex::pools(CHML, ETH).unwrap();
        assert_eq!(pool.reserve_a, 1500); // 1000 + 500
        assert_eq!(pool.reserve_b, 15);   // 10 + 5
        assert_eq!(pool.total_lp_tokens, 150); // 100 + 50

        // Check Bob's LP position
        let pool_id = PoolId::new(CHML, ETH);
        let position = ChameleonPdex::lp_positions(BOB, pool_id).unwrap();
        assert_eq!(position.lp_tokens, 50);
    });
}

#[test]
fn remove_liquidity_works() {
    new_test_ext().execute_with(|| {
        // Setup with liquidity
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);
        mint_asset(CHML, ALICE, 10_000);
        mint_asset(ETH, ALICE, 100);

        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
            1000, 10, 1000, 10
        ));

        // Remove half the liquidity
        assert_ok!(ChameleonPdex::remove_liquidity(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
            50, // lp_tokens
            500, // amount_a_min
            5,   // amount_b_min
        ));

        // Check pool state
        let pool = ChameleonPdex::pools(CHML, ETH).unwrap();
        assert_eq!(pool.reserve_a, 500); // 1000 - 500
        assert_eq!(pool.reserve_b, 5);   // 10 - 5
        assert_eq!(pool.total_lp_tokens, 50); // 100 - 50

        // Check LP position
        let pool_id = PoolId::new(CHML, ETH);
        let position = ChameleonPdex::lp_positions(ALICE, pool_id).unwrap();
        assert_eq!(position.lp_tokens, 50);
    });
}

#[test]
fn swap_exact_tokens_for_tokens_works() {
    new_test_ext().execute_with(|| {
        // Setup with liquidity
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);
        mint_asset(CHML, ALICE, 10_000);
        mint_asset(ETH, ALICE, 100);
        mint_asset(CHML, BOB, 1_000);

        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
            1000, 10, 1000, 10
        ));

        // Bob swaps CHML for ETH
        let initial_chml = Assets::balance(CHML, &BOB);
        let initial_eth = Assets::balance(ETH, &BOB);

        assert_ok!(ChameleonPdex::swap_exact_tokens_for_tokens(
            RuntimeOrigin::signed(BOB),
            100, // amount_in
            0,   // amount_out_min (no slippage protection for test)
            vec![CHML, ETH], // path
        ));

        // Check balances changed
        let final_chml = Assets::balance(CHML, &BOB);
        let final_eth = Assets::balance(ETH, &BOB);

        assert_eq!(final_chml, initial_chml - 100); // Spent 100 CHML
        assert!(final_eth > initial_eth); // Received some ETH

        // Check pool reserves updated
        let pool = ChameleonPdex::pools(CHML, ETH).unwrap();
        assert_eq!(pool.reserve_a, 1100); // 1000 + 100
        assert!(pool.reserve_b < 10); // Less than 10 ETH remaining
    });
}

#[test]
fn swap_fails_with_insufficient_liquidity() {
    new_test_ext().execute_with(|| {
        // Setup empty pool
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);
        mint_asset(CHML, BOB, 1_000);

        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));

        // Try to swap in empty pool
        assert_noop!(
            ChameleonPdex::swap_exact_tokens_for_tokens(
                RuntimeOrigin::signed(BOB),
                100,
                0,
                vec![CHML, ETH],
            ),
            Error::<Test>::PoolNotFound // Pool exists but has no liquidity
        );
    });
}

#[test]
fn swap_fails_with_slippage_exceeded() {
    new_test_ext().execute_with(|| {
        // Setup with liquidity
        create_asset(CHML, ALICE, 1);
        create_asset(ETH, ALICE, 1);
        mint_asset(CHML, ALICE, 10_000);
        mint_asset(ETH, ALICE, 100);
        mint_asset(CHML, BOB, 1_000);

        assert_ok!(ChameleonPdex::create_pool(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH
        ));
        assert_ok!(ChameleonPdex::add_liquidity(
            RuntimeOrigin::signed(ALICE),
            CHML,
            ETH,
            1000, 10, 1000, 10
        ));

        // Try to swap with unrealistic minimum output
        assert_noop!(
            ChameleonPdex::swap_exact_tokens_for_tokens(
                RuntimeOrigin::signed(BOB),
                100,
                10, // Expecting 10 ETH for 100 CHML (impossible)
                vec![CHML, ETH],
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}

// AMM Math Tests
#[test]
fn test_amm_swap_calculation() {
    // Test constant product formula
    let input_amount = 100u128;
    let input_reserve = 1000u128;
    let output_reserve = 1000u128;
    let fee_bps = 25u32; // 0.25%

    let output = calculate_swap_output(
        input_amount,
        input_reserve,
        output_reserve,
        fee_bps,
    ).unwrap();

    // With 0.25% fee: input_after_fee = 100 * 0.9975 = 99.75
    // output = 1000 * 99.75 / (1000 + 99.75) ≈ 90.7
    assert!(output >= 90 && output <= 92);

    // Verify constant product increases (due to fees)
    let k_before = input_reserve * output_reserve;
    let k_after = (input_reserve + input_amount) * (output_reserve - output);
    assert!(k_after >= k_before);
}

#[test]
fn test_lp_token_calculation() {
    // Initial liquidity
    let lp_tokens = calculate_lp_tokens_mint(
        1000u128, // amount_a
        1000u128, // amount_b
        0u128,    // reserve_a (empty pool)
        0u128,    // reserve_b (empty pool)
        0u128,    // total_lp_tokens
    ).unwrap();
    
    // LP tokens = sqrt(1000 * 1000) = 1000
    assert_eq!(lp_tokens, 1000);

    // Subsequent liquidity
    let lp_tokens_2 = calculate_lp_tokens_mint(
        500u128,  // amount_a
        500u128,  // amount_b
        1000u128, // reserve_a
        1000u128, // reserve_b
        1000u128, // total_lp_tokens
    ).unwrap();
    
    // LP tokens = min(500 * 1000 / 1000, 500 * 1000 / 1000) = 500
    assert_eq!(lp_tokens_2, 500);
}

#[test]
fn test_quote_function() {
    let amount_b = quote(
        100u128,  // amount_a
        1000u128, // reserve_a
        2000u128, // reserve_b
    ).unwrap();
    
    // amount_b = 100 * 2000 / 1000 = 200
    assert_eq!(amount_b, 200);
}

#[test]
fn test_fee_distribution() {
    let total_fee = 1000u128;
    let (lp_fee, treasury_fee) = distribute_swap_fee(total_fee);
    
    // 90% to LPs, 10% to treasury
    assert_eq!(lp_fee, 900);
    assert_eq!(treasury_fee, 100);
}

// Reward System Tests
#[test]
fn test_reward_distribution_calculation() {
    use crate::rewards::*;
    use sp_std::collections::btree_map::BTreeMap;

    let distributor = RewardDistributor::<u32, u128>::new(0, 0);
    
    // Test period rewards for year 1
    let period_rewards = distributor.calculate_period_rewards().unwrap();
    assert!(period_rewards > 0);

    // Test LP reward calculation
    let mut lp_positions = BTreeMap::new();
    lp_positions.insert([1u8; 32], 600u128); // 60% share
    lp_positions.insert([2u8; 32], 400u128); // 40% share
    
    let lp_rewards = distributor.calculate_lp_rewards(
        1000u128, // pool_reward
        &lp_positions,
        1000u128, // total_lp_tokens
    ).unwrap();
    
    assert_eq!(lp_rewards[&[1u8; 32]], 600); // 60% of 1000
    assert_eq!(lp_rewards[&[2u8; 32]], 400); // 40% of 1000
}

#[test]
fn test_vesting_schedule() {
    use crate::types::VestingSchedule;
    
    let schedule = VestingSchedule::new(
        1000u128, // total_amount
        100,      // start_block
        1000,     // duration_blocks
    );
    
    // At start block: 0% vested
    assert_eq!(schedule.vested_amount(100), 0);
    
    // At 50% completion: 50% vested
    assert_eq!(schedule.vested_amount(600), 500);
    
    // At completion: 100% vested
    assert_eq!(schedule.vested_amount(1100), 1000);
    
    // After completion: still 100% vested
    assert_eq!(schedule.vested_amount(2000), 1000);
}

#[test]
fn test_integer_sqrt() {
    assert_eq!(integer_sqrt(0u128), 0);
    assert_eq!(integer_sqrt(1u128), 1);
    assert_eq!(integer_sqrt(4u128), 2);
    assert_eq!(integer_sqrt(9u128), 3);
    assert_eq!(integer_sqrt(16u128), 4);
    assert_eq!(integer_sqrt(1000000u128), 1000);
}
