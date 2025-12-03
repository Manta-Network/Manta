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

//! Tests for Chameleon Staking Pallet

use crate::{
    mock::*,
    types::*,
    Error, Event,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, OnInitialize},
};
use sp_runtime::Perbill;

#[test]
fn delegation_works() {
    new_test_ext().execute_with(|| {
        // Setup validator
        let validator = 1;
        let delegator = 2;
        let validator_stake = 1750 * CHML;
        let delegation_amount = 100 * CHML;

        // Give accounts some balance
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        let _ = Balances::deposit_creating(&delegator, delegation_amount * 2);

        // Validator joins
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

        // Check delegation was recorded
        let delegation = ChameleonStaking::delegations(delegator, validator).unwrap();
        assert_eq!(delegation.amount, delegation_amount);
        assert_eq!(delegation.delegator, delegator);
        assert_eq!(delegation.validator, validator);

        // Check validator info updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.total_stake, validator_stake + delegation_amount);
        assert_eq!(validator_info.delegator_count, 1);

        // Check total staked updated
        assert_eq!(ChameleonStaking::total_staked(), validator_stake + delegation_amount);
    });
}

#[test]
fn undelegation_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 1750 * CHML;
        let delegation_amount = 100 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        let _ = Balances::deposit_creating(&delegator, delegation_amount * 2);
        
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

        // Check delegation removed
        assert!(ChameleonStaking::delegations(delegator, validator).is_none());

        // Check unbonding request created
        let unbonding_requests = ChameleonStaking::unbonding_requests(delegator);
        assert_eq!(unbonding_requests.len(), 1);
        assert_eq!(unbonding_requests[0].amount, delegation_amount);

        // Check validator info updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.total_stake, validator_stake);
        assert_eq!(validator_info.delegator_count, 0);
    });
}

#[test]
fn reward_distribution_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 10_000 * CHML;
        let delegation_amount = 5_000 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        let _ = Balances::deposit_creating(&delegator, delegation_amount * 2);
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));

        // Set commission to 10%
        assert_ok!(ChameleonStaking::set_commission(
            RuntimeOrigin::signed(validator),
            Perbill::from_percent(10)
        ));

        let initial_validator_balance = Balances::free_balance(&validator);
        let initial_delegator_balance = Balances::free_balance(&delegator);

        // Distribute rewards
        let era = 0;
        assert_ok!(ChameleonStaking::distribute_era_rewards(era));

        // Check balances increased (rewards distributed)
        let final_validator_balance = Balances::free_balance(&validator);
        let final_delegator_balance = Balances::free_balance(&delegator);

        assert!(final_validator_balance > initial_validator_balance);
        assert!(final_delegator_balance > initial_delegator_balance);
    });
}

#[test]
fn slashing_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 10_000 * CHML;
        let delegation_amount = 5_000 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        let _ = Balances::deposit_creating(&delegator, delegation_amount * 2);
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));
        
        assert_ok!(ChameleonStaking::delegate(
            RuntimeOrigin::signed(delegator),
            validator,
            delegation_amount
        ));

        let initial_total_stake = ChameleonStaking::total_staked();

        // Slash for downtime (0.1%)
        assert_ok!(ChameleonStaking::slash_validator(
            &validator,
            SlashingOffense::ExtendedDowntime
        ));

        // Check total stake reduced
        let final_total_stake = ChameleonStaking::total_staked();
        assert!(final_total_stake < initial_total_stake);

        // Check validator info updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert!(validator_info.total_stake < validator_stake + delegation_amount);
    });
}

#[test]
fn commission_setting_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let validator_stake = 1750 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));

        // Set commission to 15%
        let commission = Perbill::from_percent(15);
        assert_ok!(ChameleonStaking::set_commission(
            RuntimeOrigin::signed(validator),
            commission
        ));

        // Check commission updated
        let validator_info = ChameleonStaking::validators(validator).unwrap();
        assert_eq!(validator_info.commission, commission);

        // Test invalid commission (>100%)
        assert_noop!(
            ChameleonStaking::set_commission(
                RuntimeOrigin::signed(validator),
                Perbill::from_percent(101)
            ),
            Error::<Test>::InvalidCommission
        );
    });
}

#[test]
fn unbonding_period_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 1750 * CHML;
        let delegation_amount = 100 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        let _ = Balances::deposit_creating(&delegator, delegation_amount * 2);
        
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

        // Try to withdraw immediately (should fail)
        assert_noop!(
            ChameleonStaking::withdraw_unbonded(RuntimeOrigin::signed(delegator)),
            Error::<Test>::UnbondingRequestNotFound
        );

        // Advance blocks to unbonding period
        let unbonding_period = 14 * 24 * 600; // 14 days in blocks
        System::set_block_number(unbonding_period + 1);

        // Now withdrawal should work
        assert_ok!(ChameleonStaking::withdraw_unbonded(
            RuntimeOrigin::signed(delegator)
        ));

        // Check unbonding request removed
        let unbonding_requests = ChameleonStaking::unbonding_requests(delegator);
        assert_eq!(unbonding_requests.len(), 0);
    });
}

#[test]
fn validator_limits_work() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let insufficient_stake = 1000 * CHML; // Below minimum

        // Give account balance
        let _ = Balances::deposit_creating(&validator, insufficient_stake * 2);

        // Try to join with insufficient stake
        assert_noop!(
            ChameleonStaking::join_candidates(
                RuntimeOrigin::signed(validator),
                insufficient_stake
            ),
            Error::<Test>::ValidatorStakeBelowMin
        );

        // Join with sufficient stake
        let sufficient_stake = 1750 * CHML;
        let _ = Balances::deposit_creating(&validator, sufficient_stake);
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            sufficient_stake
        ));

        // Try to join again (should fail)
        assert_noop!(
            ChameleonStaking::join_candidates(
                RuntimeOrigin::signed(validator),
                sufficient_stake
            ),
            Error::<Test>::ValidatorAlreadyExists
        );
    });
}

#[test]
fn delegation_limits_work() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let delegator = 2;
        let validator_stake = 1750 * CHML;
        let delegation_amount = 100 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        let _ = Balances::deposit_creating(&delegator, delegation_amount);
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));

        // Try to delegate more than balance
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(delegator),
                validator,
                delegation_amount * 2
            ),
            Error::<Test>::InsufficientBalance
        );

        // Try to delegate zero amount
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(delegator),
                validator,
                0
            ),
            Error::<Test>::InvalidAmount
        );

        // Try to delegate to non-existent validator
        assert_noop!(
            ChameleonStaking::delegate(
                RuntimeOrigin::signed(delegator),
                999, // Non-existent validator
                delegation_amount
            ),
            Error::<Test>::ValidatorNotFound
        );
    });
}

#[test]
fn apy_calculation_works() {
    new_test_ext().execute_with(|| {
        let validator = 1;
        let validator_stake = 1750 * CHML;

        // Setup
        let _ = Balances::deposit_creating(&validator, validator_stake * 2);
        
        assert_ok!(ChameleonStaking::join_candidates(
            RuntimeOrigin::signed(validator),
            validator_stake
        ));

        // Calculate APY
        let apy = ChameleonStaking::calculate_validator_apy(&validator).unwrap();
        
        // APY should be reasonable (not zero, not too high)
        assert!(apy > Perbill::zero());
        assert!(apy < Perbill::one()); // Less than 100%
    });
}

// Helper function to create test externalities
fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    
    // Add some initial balance to accounts
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000 * CHML),
            (2, 1_000_000 * CHML),
            (3, 1_000_000 * CHML),
        ],
    }
    .assimilate_storage(&mut ext)
    .unwrap();
    
    ext.into()
}

// Constants for tests
const CHML: u128 = 1_000_000_000_000_000_000; // 1 CHML with 18 decimals