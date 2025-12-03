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

//! Reward distribution logic for Chameleon Staking

use crate::{
    pallet::*,
    types::*,
};
use frame_support::{
    traits::{Currency, Imbalance},
};
use manta_primitives::chameleon_constants::emission::*;
use sp_runtime::{
    traits::{Saturating, Zero},
    DispatchError, DispatchResult, Perbill, SaturatedConversion,
};
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// Distribute rewards at the end of an era
    pub fn distribute_era_rewards(era: u32) -> DispatchResult {
        // Calculate era reward from emission schedule
        let era_reward = Self::calculate_era_reward(era)?;
        
        if era_reward.is_zero() {
            return Ok(());
        }

        // Get all active validators
        let validators = Self::active_validators();
        if validators.is_empty() {
            return Ok(());
        }

        let total_stake: BalanceOf<T> = validators.iter()
            .map(|v| v.total_stake)
            .fold(BalanceOf::<T>::zero(), |acc, stake| acc.saturating_add(stake));

        if total_stake.is_zero() {
            return Ok(());
        }

        // Distribute rewards to each validator
        for validator in validators {
            let validator_reward = Self::calculate_validator_reward(&validator, era_reward, total_stake)?;
            Self::distribute_validator_reward(&validator, validator_reward)?;
        }

        Self::deposit_event(Event::RewardsDistributed {
            era,
            total_reward: era_reward,
        });

        Ok(())
    }

    /// Calculate the total reward for an era based on emission schedule
    pub fn calculate_era_reward(era: u32) -> Result<BalanceOf<T>, DispatchError> {
        // Determine which year we're in (assuming ~365 eras per year)
        let year = (era / 365).min(19) as usize; // Cap at year 19 (0-indexed)
        
        // Get yearly emission and divide by ~365 eras
        let yearly_emission = YEARLY_EMISSIONS.get(year)
            .copied()
            .unwrap_or(0u128);
        
        // Convert to validator rewards (70% of total)
        let validator_yearly_reward = yearly_emission
            .saturating_mul(VALIDATOR_REWARD_PERCENT as u128)
            .saturating_div(100u128);
        
        // Divide by number of eras in a year
        let era_reward = validator_yearly_reward.saturating_div(365u128);
        
        // Convert to Balance type
        Ok(era_reward.saturated_into())
    }

    /// Calculate reward for a specific validator
    pub fn calculate_validator_reward(
        validator: &ValidatorInfo<T::AccountId, BalanceOf<T>>,
        base_reward: BalanceOf<T>,
        total_network_stake: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        // Calculate stake weight (validator's share of total network stake)
        let stake_weight = if total_network_stake.is_zero() {
            Perbill::zero()
        } else {
            Perbill::from_rational(validator.total_stake, total_network_stake)
        };

        // Apply performance multipliers
        let uptime_multiplier = validator.performance.uptime_percent;
        let performance_multiplier = Self::calculate_performance_multiplier(&validator.performance);
        
        // Governance participation bonus
        let governance_bonus = if validator.performance.blocks_produced > 10 {
            Perbill::from_percent(110) // 10% bonus
        } else {
            Perbill::one()
        };

        // Calculate final reward
        let mut reward = stake_weight * base_reward;
        reward = uptime_multiplier * reward;
        reward = performance_multiplier * reward;
        reward = governance_bonus * reward;

        Ok(reward)
    }

    /// Calculate performance multiplier based on block production
    fn calculate_performance_multiplier(performance: &ValidatorPerformance) -> Perbill {
        let total_blocks = performance.blocks_produced.saturating_add(performance.blocks_missed);
        
        if total_blocks == 0 {
            return Perbill::one();
        }
        
        // Performance = blocks_produced / total_blocks
        Perbill::from_rational(performance.blocks_produced, total_blocks)
    }

    /// Distribute reward to a validator and their delegators
    pub fn distribute_validator_reward(
        validator: &ValidatorInfo<T::AccountId, BalanceOf<T>>,
        total_reward: BalanceOf<T>,
    ) -> DispatchResult {
        if total_reward.is_zero() {
            return Ok(());
        }

        // Calculate commission (validator's cut of delegator rewards)
        let commission_rate = validator.commission;
        
        // Calculate validator's self-stake reward
        let self_stake_ratio = if validator.total_stake.is_zero() {
            Perbill::zero()
        } else {
            Perbill::from_rational(validator.self_stake, validator.total_stake)
        };
        
        let self_stake_reward = self_stake_ratio * total_reward;
        
        // Calculate delegator pool reward
        let delegator_pool_reward = total_reward.saturating_sub(self_stake_reward);
        
        // Calculate commission from delegator rewards
        let commission_amount = commission_rate * delegator_pool_reward;
        
        // Net delegator rewards after commission
        let net_delegator_reward = delegator_pool_reward.saturating_sub(commission_amount);
        
        // Total validator reward = self-stake reward + commission
        let validator_total_reward = self_stake_reward.saturating_add(commission_amount);

        // Pay validator
        if !validator_total_reward.is_zero() {
            let _ = T::Currency::deposit_creating(&validator.stash, validator_total_reward);
        }

        // Distribute to delegators
        if !net_delegator_reward.is_zero() {
            Self::distribute_to_delegators(&validator.stash, net_delegator_reward, validator.total_stake)?;
        }

        Ok(())
    }

    /// Distribute rewards to delegators proportionally
    pub fn distribute_to_delegators(
        validator: &T::AccountId,
        total_delegator_reward: BalanceOf<T>,
        validator_total_stake: BalanceOf<T>,
    ) -> DispatchResult {
        if total_delegator_reward.is_zero() || validator_total_stake.is_zero() {
            return Ok(());
        }

        // Get all delegations for this validator
        let delegations: Vec<_> = Delegations::<T>::iter_prefix(validator)
            .collect();

        for (delegator, delegation) in delegations {
            // Calculate delegator's share
            let delegator_ratio = Perbill::from_rational(delegation.amount, validator_total_stake);
            let delegator_reward = delegator_ratio * total_delegator_reward;

            if !delegator_reward.is_zero() {
                let _ = T::Currency::deposit_creating(&delegator, delegator_reward);
            }
        }

        Ok(())
    }

    /// Update validator performance metrics
    pub fn update_validator_performance(
        validator: &T::AccountId,
        blocks_produced: u32,
        blocks_missed: u32,
        uptime_percent: Perbill,
    ) -> DispatchResult {
        Validators::<T>::mutate(validator, |validator_info| {
            if let Some(info) = validator_info {
                info.performance.blocks_produced = info.performance.blocks_produced.saturating_add(blocks_produced);
                info.performance.blocks_missed = info.performance.blocks_missed.saturating_add(blocks_missed);
                info.performance.uptime_percent = uptime_percent;
                info.performance.last_active_era = Self::current_era();
            }
        });
        
        Ok(())
    }

    /// Calculate APY for a validator
    pub fn calculate_validator_apy(
        validator: &T::AccountId,
    ) -> Result<Perbill, DispatchError> {
        let validator_info = Validators::<T>::get(validator)
            .ok_or(Error::<T>::ValidatorNotFound)?;

        // Get current year's emission
        let current_era = Self::current_era();
        let year = (current_era / 365).min(19) as usize;
        let yearly_emission = YEARLY_EMISSIONS.get(year)
            .copied()
            .unwrap_or(0u128);
        
        let validator_yearly_reward = yearly_emission
            .saturating_mul(VALIDATOR_REWARD_PERCENT as u128)
            .saturating_div(100u128);

        let total_staked = Self::total_staked();
        
        if total_staked.is_zero() {
            return Ok(Perbill::zero());
        }

        // Calculate validator's expected yearly reward
        let stake_ratio = Perbill::from_rational(validator_info.total_stake, total_staked);
        let expected_reward: BalanceOf<T> = (stake_ratio * validator_yearly_reward.saturated_into());
        
        // APY = expected_reward / validator_total_stake
        let apy = Perbill::from_rational(expected_reward, validator_info.total_stake);
        
        Ok(apy)
    }

    /// Get validator reward history (placeholder for future implementation)
    pub fn get_validator_reward_history(
        _validator: &T::AccountId,
        _eras: u32,
    ) -> Vec<EraReward<BalanceOf<T>>> {
        // This would return historical reward data
        // For now, return empty vec as this requires additional storage
        Vec::new()
    }

    /// Calculate total rewards distributed in an era
    pub fn calculate_total_era_rewards(era: u32) -> BalanceOf<T> {
        Self::calculate_era_reward(era).unwrap_or_else(|_| BalanceOf::<T>::zero())
    }

    /// Get current reward rate for the network
    pub fn current_reward_rate() -> Perbill {
        let current_era = Self::current_era();
        let year = (current_era / 365).min(19) as usize;
        
        // Get this year's emission as percentage of total supply
        let yearly_emission = YEARLY_EMISSIONS.get(year)
            .copied()
            .unwrap_or(0u128);
        
        let total_supply = 100_000_000_000_000_000_000_000_000u128; // 100M CHML
        
        if total_supply == 0 {
            return Perbill::zero();
        }
        
        // Return as percentage
        Perbill::from_rational(yearly_emission, total_supply)
    }
}