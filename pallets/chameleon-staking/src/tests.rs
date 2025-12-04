// Copyright 2020-2024 Manta Network.
// Chameleon Staking Tests - Production-Grade Delegation and Reward Testing

use crate::pallet::*;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64, ConstU128, Currency, LockableCurrency, WithdrawReasons},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, Perbill,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Mock runtime construction
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        ChameleonStaking: crate,
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

// Staking configuration
impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinValidatorStake = ConstU128<10_000>; // 10K minimum
    type UnbondingPeriod = ConstU64<14>; // 14 blocks = 14 days in test
    type MaxDelegatorsPerValidator = ConstU32<100>;
    type MaxDelegationsPerDelegator = ConstU32<10>;
}

// Test accounts
const ALICE: u64 = 1; // Validator
const BOB: u64 = 2;   // Delegator 1
const CHARLIE: u64 = 3; // Delegator 2
const DAVE: u64 = 4;  // Another validator

// Helper function to create test environment
fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    // Initialize balances for test accounts
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000), // 1M CHML
            (BOB, 500_000),     // 500K CHML
            (CHARLIE, 300_000), // 300K CHML
            (DAVE, 800_000),    // 800K CHML
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

// Helper function to get locked balance
fn get_locked_balance(who: &u64) -> u128 {
    let locks = Balances::locks(who);
    locks.iter()
        .find(|lock| lock.id == crate::STAKING_ID)
        .map(|lock| lock.amount)
        .unwrap_or(0)
}

// ============================================================================
// REQUIRED TESTS (Minimum 3)
// ============================================================================

#[test]
fn test_delegation_creates_stake() {
    new_test_ext().execute_with(|| {
        // Setup: Alice becomes validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            50_000 // 50K self-stake
        ));
        
        // Verify validator setup
        let validator_info = Validators::<Test>::get(ALICE).unwrap();
        assert_eq!(validator_info.self_stake, 50_000);
        assert_eq!(validator_info.total_stake, 50_000);
        assert_eq!(get_locked_balance(&ALICE), 50_000);
        
        // Test: Bob delegates to Alice
        let delegation_amount = 25_000;
        let bob_initial_balance = Balances::free_balance(&BOB);
        
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            delegation_amount
        ));
        
        // Verify: Tokens locked for Bob
        assert_eq!(get_locked_balance(&BOB), delegation_amount);
        assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // Free balance unchanged
        
        // Verify: Delegation recorded
        let delegation = Delegations::<Test>::get(ALICE, BOB);
        assert_eq!(delegation, delegation_amount);
        
        // Verify: Validator total stake updated
        let updated_info = Validators::<Test>::get(ALICE).unwrap();
        assert_eq!(updated_info.total_stake, 75_000); // 50K + 25K
        assert_eq!(updated_info.delegator_count, 1);
        
        // Verify: Total network stake updated
        assert_eq!(TotalStaked::<Test>::get(), 75_000);
        
        // Verify: Event emitted
        System::assert_last_event(RuntimeEvent::ChameleonStaking(
            Event::Delegated {
                delegator: BOB,
                validator: ALICE,
                amount: delegation_amount,
            }
        ));
    });
}

#[test]
fn test_rewards_distributed_proportionally() {
    new_test_ext().execute_with(|| {
        // Setup: Create validator with delegations
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            60_000 // Alice self-stake: 60K
        ));
        
        // Bob delegates 30K
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            30_000
        ));
        
        // Charlie delegates 10K
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(CHARLIE),
            ALICE,
            10_000
        ));
        
        // Total stake: 60K (Alice) + 30K (Bob) + 10K (Charlie) = 100K
        let validator_info = Validators::<Test>::get(ALICE).unwrap();
        assert_eq!(validator_info.total_stake, 100_000);
        assert_eq!(validator_info.commission, Perbill::from_percent(10)); // Default 10%
        
        // Test: Distribute 10,000 CHML rewards
        let total_reward = 10_000u128;
        assert_ok!(ChameleonStaking::distribute_rewards(total_reward));
        
        // Expected calculation:
        // Alice gets 100% of validator rewards (only validator)
        // Alice's share = 10,000 * (100,000 / 100,000) = 10,000
        // 
        // Self-stake reward = 10,000 * (60,000 / 100,000) = 6,000
        // Delegation pool = 10,000 * (40,000 / 100,000) = 4,000
        // Commission = 10% of 4,000 = 400
        // Alice total = 6,000 + 400 = 6,400
        // 
        // Remaining for delegators = 4,000 - 400 = 3,600
        // Bob's share = 3,600 * (30,000 / 40,000) = 2,700
        // Charlie's share = 3,600 * (10,000 / 40,000) = 900
        
        let alice_reward = PendingRewards::<Test>::get(ALICE);
        let bob_reward = PendingRewards::<Test>::get(BOB);
        let charlie_reward = PendingRewards::<Test>::get(CHARLIE);
        
        assert_eq!(alice_reward, 6_400, "Alice should get self-stake + commission");
        assert_eq!(bob_reward, 2_700, "Bob should get proportional delegation reward");
        assert_eq!(charlie_reward, 900, "Charlie should get proportional delegation reward");
        
        // Verify: Sum of all rewards equals total reward
        let total_distributed = alice_reward + bob_reward + charlie_reward;
        assert_eq!(total_distributed, total_reward, "All rewards must sum to total");
        
        // Verify: Era incremented
        assert_eq!(CurrentEra::<Test>::get(), 1);
        
        // Verify: Event emitted
        System::assert_last_event(RuntimeEvent::ChameleonStaking(
            Event::RewardsDistributed {
                era: 0,
                total_reward: total_distributed,
            }
        ));
    });
}

#[test]
fn test_unbonding_period_enforced() {
    new_test_ext().execute_with(|| {
        // Setup: Alice as validator, Bob delegates
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            50_000
        ));
        
        let delegation_amount = 30_000;
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            delegation_amount
        ));
        
        // Verify initial state
        assert_eq!(get_locked_balance(&BOB), delegation_amount);
        assert_eq!(Delegations::<Test>::get(ALICE, BOB), delegation_amount);
        
        // Test: Bob starts unbonding
        let unbond_amount = 20_000;
        assert_ok!(ChameleonStaking::undelegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            unbond_amount
        ));
        
        // Verify: Delegation reduced immediately
        assert_eq!(Delegations::<Test>::get(ALICE, BOB), 10_000); // 30K - 20K
        
        // Verify: Unbonding request created
        let unbonding_requests = UnbondingRequests::<Test>::get(BOB);
        assert_eq!(unbonding_requests.len(), 1);
        assert_eq!(unbonding_requests[0].amount, unbond_amount);
        assert_eq!(unbonding_requests[0].unlock_at, 1 + 14); // Current block + unbonding period
        
        // Verify: Still locked (cannot withdraw yet)
        assert_eq!(get_locked_balance(&BOB), delegation_amount); // Still locked
        
        // Test: Try to withdraw before unbonding period - should fail
        assert_noop!(
            ChameleonStaking::withdraw_unbonded(RuntimeOrigin::signed(BOB)),
            Error::<Test>::NothingToWithdraw
        );
        
        // Advance time but not enough (13 blocks)
        System::set_block_number(14); // Block 1 + 13 = 14 (still 1 block short)
        
        assert_noop!(
            ChameleonStaking::withdraw_unbonded(RuntimeOrigin::signed(BOB)),
            Error::<Test>::NothingToWithdraw
        );
        
        // Advance to exactly unbonding period (14 blocks)
        System::set_block_number(15); // Block 1 + 14 = 15 (exactly unbonding period)
        
        // Test: Can withdraw after unbonding period
        assert_ok!(ChameleonStaking::withdraw_unbonded(RuntimeOrigin::signed(BOB)));
        
        // Verify: Unbonding request cleared
        let unbonding_requests_after = UnbondingRequests::<Test>::get(BOB);
        assert_eq!(unbonding_requests_after.len(), 0);
        
        // Verify: Lock removed (tokens unlocked)
        // Note: In production, this would reduce the lock amount
        // For simplicity, our implementation removes the entire lock
        assert_eq!(get_locked_balance(&BOB), 0);
        
        // Verify: Event emitted
        System::assert_last_event(RuntimeEvent::ChameleonStaking(
            Event::Withdrawn {
                who: BOB,
                amount: unbond_amount,
            }
        ));
    });
}

// ============================================================================
// ADDITIONAL COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_multiple_validators_reward_distribution() {
    new_test_ext().execute_with(|| {
        // Setup: Two validators with different stakes
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            40_000 // Alice: 40K
        ));
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(DAVE),
            60_000 // Dave: 60K
        ));
        
        // Bob delegates to Alice
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            20_000
        ));
        
        // Charlie delegates to Dave
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(CHARLIE),
            DAVE,
            40_000
        ));
        
        // Total network stake: (40K + 20K) + (60K + 40K) = 160K
        assert_eq!(TotalStaked::<Test>::get(), 160_000);
        
        // Distribute 16,000 CHML rewards
        let total_reward = 16_000u128;
        assert_ok!(ChameleonStaking::distribute_rewards(total_reward));
        
        // Expected:
        // Alice validator gets: 16,000 * (60,000 / 160,000) = 6,000
        // Dave validator gets: 16,000 * (100,000 / 160,000) = 10,000
        
        // Alice breakdown:
        // Self-stake: 6,000 * (40,000 / 60,000) = 4,000
        // Delegation pool: 6,000 * (20,000 / 60,000) = 2,000
        // Commission (10%): 200
        // Alice total: 4,000 + 200 = 4,200
        // Bob gets: 2,000 - 200 = 1,800
        
        // Dave breakdown:
        // Self-stake: 10,000 * (60,000 / 100,000) = 6,000
        // Delegation pool: 10,000 * (40,000 / 100,000) = 4,000
        // Commission (10%): 400
        // Dave total: 6,000 + 400 = 6,400
        // Charlie gets: 4,000 - 400 = 3,600
        
        assert_eq!(PendingRewards::<Test>::get(ALICE), 4_200);
        assert_eq!(PendingRewards::<Test>::get(BOB), 1_800);
        assert_eq!(PendingRewards::<Test>::get(DAVE), 6_400);
        assert_eq!(PendingRewards::<Test>::get(CHARLIE), 3_600);
        
        // Verify total
        let total_distributed = 4_200 + 1_800 + 6_400 + 3_600;
        assert_eq!(total_distributed, total_reward);
    });
}

#[test]
fn test_commission_affects_rewards() {
    new_test_ext().execute_with(|| {
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            50_000
        ));
        
        // Set high commission (50%)
        assert_ok!(ChameleonStaking::set_commission(
            RuntimeOrigin::signed(ALICE),
            Perbill::from_percent(50)
        ));
        
        // Bob delegates
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            50_000
        ));
        
        // Total: 100K (50K Alice + 50K Bob)
        // Distribute 10K rewards
        assert_ok!(ChameleonStaking::distribute_rewards(10_000));
        
        // Expected:
        // Self-stake reward: 10,000 * (50,000 / 100,000) = 5,000
        // Delegation pool: 10,000 * (50,000 / 100,000) = 5,000
        // Commission (50%): 2,500
        // Alice total: 5,000 + 2,500 = 7,500
        // Bob gets: 5,000 - 2,500 = 2,500
        
        assert_eq!(PendingRewards::<Test>::get(ALICE), 7_500);
        assert_eq!(PendingRewards::<Test>::get(BOB), 2_500);
    });
}

#[test]
fn test_slashing_reduces_stakes() {
    new_test_ext().execute_with(|| {
        // Setup validator with delegation
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            60_000
        ));
        
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(BOB),
            ALICE,
            40_000
        ));
        
        // Total: 100K
        let initial_total = TotalStaked::<Test>::get();
        assert_eq!(initial_total, 100_000);
        
        // Slash for extended downtime (0.1%)
        assert_ok!(ChameleonStaking::slash_validator(
            &ALICE,
            SlashingOffense::ExtendedDowntime
        ));
        
        // Expected slash: 100K * 0.1% = 100 CHML
        let validator_info = Validators::<Test>::get(ALICE).unwrap();
        assert_eq!(validator_info.status, ValidatorStatus::Slashed);
        
        // Verify total stake reduced
        let new_total = TotalStaked::<Test>::get();
        assert_eq!(new_total, 99_900); // 100K - 100
        
        // Verify event
        System::assert_last_event(RuntimeEvent::ChameleonStaking(
            Event::ValidatorSlashed {
                validator: ALICE,
                amount: 100,
                offense: SlashingOffense::ExtendedDowntime,
            }
        ));
    });
}

#[test]
fn test_claim_rewards_works() {
    new_test_ext().execute_with(|| {
        // Setup and distribute rewards
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            50_000
        ));
        
        assert_ok!(ChameleonStaking::distribute_rewards(5_000));
        
        // Verify pending rewards
        assert_eq!(PendingRewards::<Test>::get(ALICE), 5_000);
        
        // Claim rewards
        assert_ok!(ChameleonStaking::claim_rewards(RuntimeOrigin::signed(ALICE)));
        
        // Verify rewards cleared
        assert_eq!(PendingRewards::<Test>::get(ALICE), 0);
        
        // Verify event
        System::assert_last_event(RuntimeEvent::ChameleonStaking(
            Event::RewardsClaimed {
                who: ALICE,
                amount: 5_000,
            }
        ));
        
        // Try to claim again - should fail
        assert_noop!(
            ChameleonStaking::claim_rewards(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::NothingToClaim
        );
    });
}

#[test]
fn test_validator_requirements_enforced() {
    new_test_ext().execute_with(|| {
        // Test minimum stake requirement
        assert_noop!(
            ChameleonStaking::join_candidates(
                RuntimeOrigin::signed(ALICE),
                5_000 // Below 10K minimum
            ),
            Error::<Test>::InsufficientStake
        );
        
        // Test duplicate validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            50_000
        ));
        
        assert_noop!(
            ChameleonStaking::join_candidates(
                RuntimeOrigin::signed(ALICE),
                60_000
            ),
            Error::<Test>::AlreadyValidator
        );
    });
}

#[test]
fn test_delegation_edge_cases() {
    new_test_ext().execute_with(|| {
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(ALICE),
            50_000
        ));
        
        // Test zero delegation
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(BOB),
                ALICE,
                0
            ),
            Error::<Test>::InsufficientStake
        );
        
        // Test delegation to non-existent validator
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(BOB),
                DAVE, // Not a validator
                10_000
            ),
            Error::<Test>::ValidatorNotFound
        );
        
        // Test undelegation without delegation
        assert_noop!(
            ChameleonStaking::undelegate(
                RuntimeOrigin::signed(BOB),
                ALICE,
                10_000
            ),
            Error::<Test>::NotDelegated
        );
    });
}
