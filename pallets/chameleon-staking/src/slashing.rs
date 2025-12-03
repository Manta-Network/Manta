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

//! Slashing logic for Chameleon Staking

use crate::{
    pallet::*,
    types::*,
};
use frame_support::{
    traits::{Currency, Imbalance},
};
use sp_runtime::{
    traits::{Saturating, Zero},
    DispatchResult, Perbill,
};
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// Slash validator for offense
    pub fn slash_validator(
        validator: &T::AccountId,
        offense: SlashingOffense,
    ) -> DispatchResult {
        let mut validator_info = Validators::<T>::get(validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        let slash_percent = match offense {
            SlashingOffense::ExtendedDowntime => T::SlashingDowntime::get(),
            SlashingOffense::DoubleSigning => T::SlashingDoubleSign::get(),
        };

        // Calculate total slash amount
        let total_slash_amount = slash_percent * validator_info.total_stake;

        // Slash validator's self-stake first
        let validator_slash = total_slash_amount.min(validator_info.self_stake);
        
        if !validator_slash.is_zero() {
            // Slash from validator's balance
            let (imbalance, _) = T::Currency::slash(&validator_info.stash, validator_slash);
            // Burn the slashed amount
            drop(imbalance);
            
            validator_info.self_stake = validator_info.self_stake.saturating_sub(validator_slash);
            validator_info.total_stake = validator_info.total_stake.saturating_sub(validator_slash);
        }

        // Calculate remaining slash amount for delegators
        let remaining_slash = total_slash_amount.saturating_sub(validator_slash);
        
        if !remaining_slash.is_zero() {
            Self::slash_delegators(validator, remaining_slash)?;
        }

        // Update validator status based on offense severity
        match offense {
            SlashingOffense::ExtendedDowntime => {
                // For downtime, just mark as slashed but keep active
                // They can recover by improving performance
            },
            SlashingOffense::DoubleSigning => {
                // For double signing, mark as slashed and inactive
                validator_info.status = ValidatorStatus::Slashed;
            },
        }

        Validators::<T>::insert(validator, validator_info);

        // Update total staked amount
        TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(total_slash_amount));

        Self::deposit_event(Event::ValidatorSlashed {
            validator: validator.clone(),
            amount: total_slash_amount,
            offense,
        });

        Ok(())
    }

    /// Slash delegators proportionally
    pub fn slash_delegators(
        validator: &T::AccountId,
        total_slash_amount: BalanceOf<T>,
    ) -> DispatchResult {
        if total_slash_amount.is_zero() {
            return Ok(());
        }

        // Get validator info to calculate delegator stake
        let validator_info = Validators::<T>::get(validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;
        
        let delegator_stake = validator_info.total_stake.saturating_sub(validator_info.self_stake);
        
        if delegator_stake.is_zero() {
            return Ok(());
        }

        // Get all delegations for this validator
        let delegations: Vec<_> = Delegations::<T>::iter_prefix(validator)
            .collect();

        let mut total_slashed = BalanceOf::<T>::zero();

        for (delegator, mut delegation) in delegations {
            // Calculate this delegator's share of the slash
            let delegator_slash_ratio = Perbill::from_rational(delegation.amount, delegator_stake);
            let delegator_slash = delegator_slash_ratio * total_slash_amount;
            
            if !delegator_slash.is_zero() {
                // Slash from delegator's locked balance
                let actual_slash = delegator_slash.min(delegation.amount);
                
                // Update delegation amount
                delegation.amount = delegation.amount.saturating_sub(actual_slash);
                
                if delegation.amount.is_zero() {
                    // Remove delegation if fully slashed
                    Delegations::<T>::remove(&delegator, validator);
                } else {
                    // Update delegation
                    Delegations::<T>::insert(&delegator, validator, delegation);
                }
                
                // Reduce locked balance
                let current_lock = T::Currency::free_balance(&delegator);
                let new_lock = current_lock.saturating_sub(actual_slash);
                
                if new_lock.is_zero() {
                    T::Currency::remove_lock(STAKING_ID, &delegator);
                } else {
                    T::Currency::set_lock(
                        STAKING_ID,
                        &delegator,
                        new_lock,
                        frame_support::traits::WithdrawReasons::all(),
                    );
                }
                
                total_slashed = total_slashed.saturating_add(actual_slash);
            }
        }

        // Update validator's total stake
        Validators::<T>::mutate(validator, |validator_info| {
            if let Some(info) = validator_info {
                info.total_stake = info.total_stake.saturating_sub(total_slashed);
            }
        });

        Ok(())
    }

    /// Check for slashing conditions
    pub fn check_slashing_conditions(
        validator: &T::AccountId,
        current_block: BlockNumberOf<T>,
    ) -> Result<Option<SlashingOffense>, DispatchError> {
        let validator_info = Validators::<T>::get(validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        // Check for extended downtime (>12 hours = 7200 blocks at 6s per block)
        let downtime_threshold = 7200u32.into();
        let last_active_block = validator_info.performance.last_active_era.saturated_into::<u32>().into();
        
        if current_block.saturating_sub(last_active_block) > downtime_threshold {
            return Ok(Some(SlashingOffense::ExtendedDowntime));
        }

        // Check for poor performance (less than 50% block production success rate)
        let total_blocks = validator_info.performance.blocks_produced
            .saturating_add(validator_info.performance.blocks_missed);
        
        if total_blocks > 100 { // Only check after sufficient sample size
            let success_rate = Perbill::from_rational(
                validator_info.performance.blocks_produced,
                total_blocks
            );
            
            if success_rate < Perbill::from_percent(50) {
                return Ok(Some(SlashingOffense::ExtendedDowntime));
            }
        }

        // Double signing would be detected by consensus mechanism
        // and reported separately

        Ok(None)
    }

    /// Report double signing offense
    pub fn report_double_signing(
        reporter: T::AccountId,
        validator: T::AccountId,
        _proof: Vec<u8>, // Proof of double signing
    ) -> DispatchResult {
        // Verify proof (simplified for now)
        // In a real implementation, this would verify cryptographic proof
        
        // Slash for double signing
        Self::slash_validator(&validator, SlashingOffense::DoubleSigning)?;
        
        // Reward reporter (optional)
        // Could give a portion of slashed funds to the reporter
        
        Ok(())
    }

    /// Automatic slashing check (called in on_initialize)
    pub fn automatic_slashing_check(current_block: BlockNumberOf<T>) -> DispatchResult {
        // Check all active validators for slashing conditions
        let validators: Vec<_> = Validators::<T>::iter()
            .filter(|(_, info)| matches!(info.status, ValidatorStatus::Active))
            .map(|(account, _)| account)
            .collect();

        for validator in validators {
            if let Ok(Some(offense)) = Self::check_slashing_conditions(&validator, current_block) {
                let _ = Self::slash_validator(&validator, offense);
            }
        }

        Ok(())
    }

    /// Calculate slashing impact for a validator
    pub fn calculate_slashing_impact(
        validator: &T::AccountId,
        offense: SlashingOffense,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
        let validator_info = Validators::<T>::get(validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        let slash_percent = match offense {
            SlashingOffense::ExtendedDowntime => T::SlashingDowntime::get(),
            SlashingOffense::DoubleSigning => T::SlashingDoubleSign::get(),
        };

        let total_slash = slash_percent * validator_info.total_stake;
        let validator_slash = total_slash.min(validator_info.self_stake);
        let delegator_slash = total_slash.saturating_sub(validator_slash);

        Ok((validator_slash, delegator_slash))
    }

    /// Get slashing history for a validator (placeholder)
    pub fn get_slashing_history(
        _validator: &T::AccountId,
    ) -> Vec<(SlashingOffense, BalanceOf<T>, BlockNumberOf<T>)> {
        // This would return historical slashing events
        // Requires additional storage to implement
        Vec::new()
    }

    /// Check if validator is slashed
    pub fn is_validator_slashed(validator: &T::AccountId) -> bool {
        Validators::<T>::get(validator)
            .map(|info| matches!(info.status, ValidatorStatus::Slashed))
            .unwrap_or(false)
    }

    /// Rehabilitate a slashed validator (governance call)
    pub fn rehabilitate_validator(
        validator: T::AccountId,
    ) -> DispatchResult {
        Validators::<T>::mutate(&validator, |validator_info| {
            if let Some(info) = validator_info {
                if matches!(info.status, ValidatorStatus::Slashed) {
                    info.status = ValidatorStatus::Active;
                    // Reset performance metrics
                    info.performance = ValidatorPerformance::default();
                }
            }
        });

        Ok(())
    }
}