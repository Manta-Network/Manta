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

//! Tests for the Chameleon Staking Pallet

use super::*;
use crate::mock::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, LockableCurrency, WithdrawReasons},
};
use sp_runtime::Perbill;

#[test]
fn join_candidates_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let stake = 2000; // Above minimum
        
        // Should work with sufficient stake
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            stake
        ));
        
        // Check validator info is stored
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.self_stake, stake);
        assert_eq!(validator_info.total_stake, stake);
        assert_eq!(validator_info.delegator_count, 0);
        assert_eq!(validator_info.status, ValidatorStatus::Waiting);
        
        // Check global state
        assert_eq!(ChameleonStaking::total_staked(), stake);
        assert_eq!(ChameleonStaking::validator_count(), 1);
        
        // Should fail if already a validator
        assert_noop!(
            ChameleonStaking::join_candidates(RuntimeOrigin::signed(validator), stake),
            Error::<Test>::AlreadyValidator
        );
    });
}

#[test]
fn join_candidates_fails_with_insufficient_stake() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let insufficient_stake = 1000; // Below minimum of 1750
        
        assert_noop!(
            ChameleonStaking::join_candidates(
                RuntimeOrigin::signed(validator),
                insufficient_stake
            ),
            Error::<Test>::InsufficientStake
        );
    });
}

#[test]
fn delegate_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // First, validator joins
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        // Delegator delegates
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        // Check delegation is stored
        assert_eq!(ChameleonStaking::delegations(delegator, validator), delegation_amount);
        
        // Check validator info is updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.total_stake, validator_stake + delegation_amount);
        assert_eq!(validator_info.delegator_count, 1);
        
        // Check global state
        assert_eq!(ChameleonStaking::total_staked(), validator_stake + delegation_amount);
        assert_eq!(ChameleonStaking::delegator_count(validator), 1);
        assert_eq!(ChameleonStaking::delegation_count(delegator), 1);
    });
}

#[test]
fn delegate_fails_with_invalid_conditions() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // Cannot delegate to non-existent validator
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(delegator),
                validator,
                delegation_amount
            ),
            Error::<Test>::ValidatorNotFound
        );
        
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        // Cannot delegate zero amount
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(delegator),
                validator,
                0
            ),
            Error::<Test>::InsufficientStake
        );
        
        // Cannot delegate to self
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(validator),
                validator,
                delegation_amount
            ),
            Error::<Test>::CannotDelegateToSelf
        );
        
        // Successful delegation
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        // Cannot delegate again to same validator
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(delegator),
                validator,
                delegation_amount
            ),
            Error::<Test>::AlreadyDelegated
        );
    });
}

#[test]
fn undelegate_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // Setup
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        // Undelegate
        assert_ok!(ChameleonStaking::undelegate(
            RuntimeOrigin::signed(delegator),
            validator
        ));
        
        // Check delegation is removed
        assert_eq!(ChameleonStaking::delegations(delegator, validator), 0);
        
        // Check validator info is updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.total_stake, validator_stake);
        assert_eq!(validator_info.delegator_count, 0);
        
        // Check unbonding request is created
        let unbonding_requests = ChameleonStaking::unbonding_requests(delegator);
        assert_eq!(unbonding_requests.len(), 1);
        assert_eq!(unbonding_requests[0].amount, delegation_amount);
        
        // Check global state
        assert_eq!(ChameleonStaking::total_staked(), validator_stake);
        assert_eq!(ChameleonStaking::delegator_count(validator), 0);
        assert_eq!(ChameleonStaking::delegation_count(delegator), 0);
    });
}

#[test]
fn undelegate_fails_when_not_delegated() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        
        // Setup validator only
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        // Try to undelegate without delegation
        assert_noop!(
            ChameleonStaking::undelegate(
                RuntimeOrigin::signed(delegator),
                validator
            ),
            Error::<Test>::NotDelegated
        );
    });
}

#[test]
fn withdraw_unbonded_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // Setup and delegate
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        // Undelegate to start unbonding
        assert_ok!(ChameleonStaking::undelegate(
            RuntimeOrigin::signed(delegator),
            validator
        ));
        
        // Cannot withdraw before unbonding period
        assert_noop!(
            ChameleonStaking::withdraw_unbonded(RuntimeOrigin::signed(delegator)),
            Error::<Test>::NoUnbondedTokens
        );
        
        // Fast forward past unbonding period
        System::set_block_number(System::block_number() + UnbondingPeriod::get() + 1);
        
        // Now withdrawal should work
        assert_ok!(ChameleonStaking::withdraw_unbonded(
            RuntimeOrigin::signed(delegator)
        ));
        
        // Check unbonding requests are cleared
        let unbonding_requests = ChameleonStaking::unbonding_requests(delegator);
        assert_eq!(unbonding_requests.len(), 0);
    });
}

#[test]
fn set_commission_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let validator_stake = 2000;
        let new_commission = Perbill::from_percent(15);
        
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        // Set commission
        assert_ok!(ChameleonStaking::set_commission(
            RuntimeOrigin::signed(validator),
            new_commission
        ));
        
        // Check commission is updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.commission, new_commission);
    });
}

#[test]
fn set_commission_fails_for_non_validator() {
    new_test_ext().execute_with(|| {
        let non_validator = 1;
        let commission = Perbill::from_percent(15);
        
        assert_noop!(
            ChameleonStaking::set_commission(
                RuntimeOrigin::signed(non_validator),
                commission
            ),
            Error::<Test>::ValidatorNotFound
        );
    });
}

#[test]
fn set_commission_fails_with_invalid_rate() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let validator_stake = 2000;
        let invalid_commission = Perbill::from_parts(1_100_000_000); // > 100%
        
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        assert_noop!(
            ChameleonStaking::set_commission(
                RuntimeOrigin::signed(validator),
                invalid_commission
            ),
            Error::<Test>::InvalidCommission
        );
    });
}

#[test]
fn leave_candidates_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let validator_stake = 2000;
        
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        // Leave candidates
        assert_ok!(ChameleonStaking::leave_candidates(
            RuntimeOrigin::signed(validator)
        ));
        
        // Check validator is removed
        assert!(ChameleonStaking::validators(validator).is_none());
        
        // Check unbonding request is created
        let unbonding_requests = ChameleonStaking::unbonding_requests(validator);
        assert_eq!(unbonding_requests.len(), 1);
        assert_eq!(unbonding_requests[0].amount, validator_stake);
        
        // Check global state
        assert_eq!(ChameleonStaking::total_staked(), 0);
        assert_eq!(ChameleonStaking::validator_count(), 0);
    });
}

#[test]
fn leave_candidates_fails_with_delegators() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // Setup validator and delegation
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        // Cannot leave with active delegators
        assert_noop!(
            ChameleonStaking::leave_candidates(RuntimeOrigin::signed(validator)),
            Error::<Test>::TooManyDelegators
        );
    });
}

#[test]
fn reward_distribution_works() {
    new_test_ext().execute_with(|| {
        let validator1 = 1;
        let validator2 = 2;
        let delegator = 3;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        let total_reward = 1000;
        
        // Setup validators
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator1),
            validator_stake
        ));
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator2),
            validator_stake
        ));
        
        // Add delegation to validator1
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator1,
            delegation_amount
        ));
        
        // Set validators as active
        Validators::<Test>::mutate(validator1, |info| {
            if let Some(ref mut validator_info) = info {
                validator_info.status = ValidatorStatus::Active;
            }
        });
        Validators::<Test>::mutate(validator2, |info| {
            if let Some(ref mut validator_info) = info {
                validator_info.status = ValidatorStatus::Active;
            }
        });
        
        // Distribute rewards
        assert_ok!(ChameleonStaking::distribute_rewards(total_reward));
        
        // Check that rewards were distributed
        // Validator1 has 3000 total stake (2000 self + 1000 delegated)
        // Validator2 has 2000 total stake
        // Total network stake: 5000
        // Validator1 should get: 1000 * 3000 / 5000 = 600
        // Validator2 should get: 1000 * 2000 / 5000 = 400
        
        let validator1_rewards = ChameleonStaking::pending_rewards(validator1);
        let validator2_rewards = ChameleonStaking::pending_rewards(validator2);
        let delegator_rewards = ChameleonStaking::pending_rewards(delegator);
        
        // Validator1 gets commission + their share of delegator rewards
        // Validator2 gets all their rewards (no delegators)
        assert!(validator1_rewards > 0);
        assert!(validator2_rewards > 0);
        assert!(delegator_rewards > 0);
        
        // Era should be incremented
        assert_eq!(ChameleonStaking::current_era(), 1);
    });
}

#[test]
fn claim_rewards_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let reward_amount = 1000;
        
        // Manually set pending rewards
        PendingRewards::<Test>::insert(validator, reward_amount);
        
        let initial_balance = Balances::free_balance(validator);
        
        // Claim rewards
        assert_ok!(ChameleonStaking::claim_rewards(
            RuntimeOrigin::signed(validator)
        ));
        
        // Check rewards are cleared
        assert_eq!(ChameleonStaking::pending_rewards(validator), 0);
        
        // Check balance increased
        assert_eq!(Balances::free_balance(validator), initial_balance + reward_amount);
    });
}

#[test]
fn claim_rewards_fails_when_no_rewards() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        
        assert_noop!(
            ChameleonStaking::claim_rewards(RuntimeOrigin::signed(validator)),
            Error::<Test>::NoRewardsToClaim
        );
    });
}

#[test]
fn slashing_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // Setup validator and delegation
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        let initial_total_stake = validator_stake + delegation_amount;
        
        // Slash for downtime (0.1%)
        assert_ok!(ChameleonStaking::slash_validator(
            &validator,
            SlashingOffense::ExtendedDowntime
        ));
        
        // Check validator info is updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.status, ValidatorStatus::Slashed);
        
        // Check total stake is reduced
        let expected_slash = Perbill::from_parts(1_000_000) * initial_total_stake; // 0.1%
        assert_eq!(validator_info.total_stake, initial_total_stake - expected_slash);
        
        // Check global total is updated
        assert_eq!(ChameleonStaking::total_staked(), initial_total_stake - expected_slash);
    });
}

#[test]
fn helper_functions_work() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 2000;
        let delegation_amount = 1000;
        
        // Initially not a validator
        assert!(!ChameleonStaking::is_validator(&validator));
        assert!(ChameleonStaking::get_validator_info(&validator).is_none());
        
        // Setup validator
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        // Now is a validator
        assert!(ChameleonStaking::is_validator(&validator));
        assert!(ChameleonStaking::get_validator_info(&validator).is_some());
        
        // No delegation initially
        assert_eq!(ChameleonStaking::get_delegation(&delegator, &validator), 0);
        
        // Add delegation
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));
        
        // Now has delegation
        assert_eq!(ChameleonStaking::get_delegation(&delegator, &validator), delegation_amount);
    });
}
