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

//! Delegation logic for Chameleon Staking

use crate::{
    pallet::*,
    types::*,
};
use frame_support::{
    ensure,
    traits::{Currency, LockableCurrency, WithdrawReasons},
};
use sp_runtime::{
    traits::{Saturating, Zero},
    DispatchError, DispatchResult,
};

impl<T: Config> Pallet<T> {
    /// Delegate CHML to a validator
    pub fn do_delegate(
        delegator: T::AccountId,
        validator: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // Ensure amount is not zero
        ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

        // Ensure validator exists and is active
        let mut validator_info = Validators::<T>::get(&validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        // Check delegation limits
        ensure!(
            validator_info.delegator_count < T::MaxDelegatorsPerValidator::get(),
            Error::<T>::TooManyDelegators
        );

        // Check if delegator already has too many delegations
        let delegator_delegations = Delegations::<T>::iter_prefix(&delegator).count() as u32;
        ensure!(
            delegator_delegations < T::MaxDelegationsPerDelegator::get(),
            Error::<T>::TooManyDelegations
        );

        // Ensure delegator has sufficient balance
        ensure!(
            T::Currency::free_balance(&delegator) >= amount,
            Error::<T>::InsufficientBalance
        );

        // Lock delegator's tokens
        T::Currency::set_lock(
            STAKING_ID,
            &delegator,
            amount,
            WithdrawReasons::all(),
        );

        // Update validator's total stake
        validator_info.total_stake = validator_info.total_stake.saturating_add(amount);
        
        // If this is a new delegator, increment count
        if !Delegations::<T>::contains_key(&delegator, &validator) {
            validator_info.delegator_count = validator_info.delegator_count.saturating_add(1);
        }
        
        Validators::<T>::insert(&validator, &validator_info);

        // Record delegation
        let current_block = <frame_system::Pallet<T>>::block_number();
        let delegation = Delegation {
            delegator: delegator.clone(),
            validator: validator.clone(),
            amount,
            delegated_at: current_block.saturated_into::<u32>(),
        };
        Delegations::<T>::insert(&delegator, &validator, delegation);

        // Update total staked
        TotalStaked::<T>::mutate(|total| *total = total.saturating_add(amount));

        Self::deposit_event(Event::Delegated {
            delegator,
            validator,
            amount,
        });

        Ok(())
    }

    /// Remove delegation from a validator
    pub fn do_undelegate(
        delegator: T::AccountId,
        validator: T::AccountId,
    ) -> DispatchResult {
        // Get delegation
        let delegation = Delegations::<T>::get(&delegator, &validator)
            .ok_or(Error::<T>::DelegationNotFound)?;

        // Get validator info
        let mut validator_info = Validators::<T>::get(&validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        // Update validator's total stake and delegator count
        validator_info.total_stake = validator_info.total_stake.saturating_sub(delegation.amount);
        validator_info.delegator_count = validator_info.delegator_count.saturating_sub(1);
        Validators::<T>::insert(&validator, validator_info);

        // Start unbonding period
        let current_block = <frame_system::Pallet<T>>::block_number();
        let unlock_at = current_block.saturating_add(T::UnbondingPeriod::get());
        
        let unbonding_request = UnbondingRequest {
            amount: delegation.amount,
            unlock_at,
        };
        
        UnbondingRequests::<T>::mutate(&delegator, |requests| {
            requests.push(unbonding_request);
        });

        // Remove delegation
        Delegations::<T>::remove(&delegator, &validator);

        // Update total staked
        TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(delegation.amount));

        Self::deposit_event(Event::Undelegated {
            delegator: delegator.clone(),
            validator,
            amount: delegation.amount,
        });

        Self::deposit_event(Event::UnbondingStarted {
            who: delegator,
            amount: delegation.amount,
            unlock_at,
        });

        Ok(())
    }

    /// Start unbonding process for validator's own stake
    pub fn do_unbond(
        who: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // Ensure amount is not zero
        ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

        // If this is a validator, ensure they maintain minimum stake
        if let Some(mut validator_info) = Validators::<T>::get(&who) {
            let remaining_stake = validator_info.self_stake.saturating_sub(amount);
            ensure!(
                remaining_stake >= T::MinValidatorStake::get(),
                Error::<T>::ValidatorStakeBelowMin
            );
            
            // Update validator's self stake
            validator_info.self_stake = remaining_stake;
            validator_info.total_stake = validator_info.total_stake.saturating_sub(amount);
            Validators::<T>::insert(&who, validator_info);
        }

        let current_block = <frame_system::Pallet<T>>::block_number();
        let unlock_at = current_block.saturating_add(T::UnbondingPeriod::get());
        
        let unbonding_request = UnbondingRequest {
            amount,
            unlock_at,
        };
        
        UnbondingRequests::<T>::mutate(&who, |requests| {
            requests.push(unbonding_request);
        });

        // Update total staked
        TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(amount));

        Self::deposit_event(Event::UnbondingStarted {
            who,
            amount,
            unlock_at,
        });

        Ok(())
    }

    /// Withdraw unbonded funds
    pub fn do_withdraw_unbonded(who: T::AccountId) -> DispatchResult {
        let current_block = <frame_system::Pallet<T>>::block_number();
        let mut total_withdrawn = BalanceOf::<T>::zero();
        
        UnbondingRequests::<T>::mutate(&who, |requests| {
            requests.retain(|request| {
                if request.unlock_at <= current_block {
                    total_withdrawn = total_withdrawn.saturating_add(request.amount);
                    false // Remove this request
                } else {
                    true // Keep this request
                }
            });
        });

        ensure!(!total_withdrawn.is_zero(), Error::<T>::UnbondingRequestNotFound);

        // Remove lock if no more unbonding requests
        if UnbondingRequests::<T>::get(&who).is_empty() {
            T::Currency::remove_lock(STAKING_ID, &who);
        }

        Self::deposit_event(Event::UnbondingCompleted {
            who,
            amount: total_withdrawn,
        });

        Ok(())
    }

    /// Set validator commission rate
    pub fn do_set_commission(
        validator: T::AccountId,
        commission: sp_runtime::Perbill,
    ) -> DispatchResult {
        // Ensure commission is not more than 100%
        ensure!(
            commission <= sp_runtime::Perbill::one(),
            Error::<T>::InvalidCommission
        );

        // Get validator info
        let mut validator_info = Validators::<T>::get(&validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        // Update commission
        validator_info.commission = commission;
        Validators::<T>::insert(&validator, validator_info);

        Self::deposit_event(Event::CommissionSet {
            validator,
            commission,
        });

        Ok(())
    }

    /// Join as validator candidate
    pub fn do_join_candidates(
        candidate: T::AccountId,
        bond: BalanceOf<T>,
    ) -> DispatchResult {
        // Ensure not already a validator
        ensure!(
            !Validators::<T>::contains_key(&candidate),
            Error::<T>::ValidatorAlreadyExists
        );

        // Ensure bond meets minimum requirement
        ensure!(
            bond >= T::MinValidatorStake::get(),
            Error::<T>::ValidatorStakeBelowMin
        );

        // Ensure candidate has sufficient balance
        ensure!(
            T::Currency::free_balance(&candidate) >= bond,
            Error::<T>::InsufficientBalance
        );

        // Lock candidate's tokens
        T::Currency::set_lock(
            STAKING_ID,
            &candidate,
            bond,
            WithdrawReasons::all(),
        );

        // Create validator info
        let validator_info = ValidatorInfo {
            controller: candidate.clone(),
            stash: candidate.clone(),
            self_stake: bond,
            total_stake: bond,
            delegator_count: 0,
            commission: sp_runtime::Perbill::from_percent(10), // Default 10% commission
            status: ValidatorStatus::Active,
            performance: ValidatorPerformance::default(),
        };

        Validators::<T>::insert(&candidate, validator_info);

        // Update total staked
        TotalStaked::<T>::mutate(|total| *total = total.saturating_add(bond));

        Ok(())
    }

    /// Leave validator candidate pool
    pub fn do_leave_candidates(candidate: T::AccountId) -> DispatchResult {
        // Get validator info
        let validator_info = Validators::<T>::get(&candidate)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        // Start unbonding for validator's self stake
        Self::do_unbond(candidate.clone(), validator_info.self_stake)?;

        // Remove validator
        Validators::<T>::remove(&candidate);

        Ok(())
    }

    /// Process unbonding requests that are ready
    pub fn process_unbonding_requests(current_block: BlockNumberOf<T>) -> DispatchResult {
        // This would be called in on_initialize to process ready unbonding requests
        // For now, we'll keep it simple and let users call withdraw_unbonded manually
        Ok(())
    }

    /// Get active validators
    pub fn active_validators() -> Vec<ValidatorInfo<T::AccountId, BalanceOf<T>>> {
        Validators::<T>::iter()
            .filter_map(|(_, validator_info)| {
                if matches!(validator_info.status, ValidatorStatus::Active) {
                    Some(validator_info)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if account is a validator
    pub fn is_validator(account: &T::AccountId) -> bool {
        Validators::<T>::contains_key(account)
    }

    /// Get validator commission
    pub fn get_validator_commission(validator: &T::AccountId) -> Option<sp_runtime::Perbill> {
        Validators::<T>::get(validator).map(|info| info.commission)
    }
}