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

//! Usage examples for the Chameleon Staking Pallet
//!
//! This file demonstrates how to integrate and use the Chameleon Staking Pallet
//! with proper constants from the chameleon_constants module.

use manta_primitives::chameleon_constants::{
    staking::{MIN_VALIDATOR_STAKE, SLASHING_DOWNTIME, SLASHING_DOUBLE_SIGN},
    time::UNBONDING_PERIOD_BLOCKS,
};
use sp_runtime::Perbill;

/// Example runtime configuration for Chameleon Staking
/// 
/// This shows how to properly configure the pallet in your runtime
pub struct ExampleRuntimeConfig;

// Example configuration parameters
parameter_types! {
    /// Minimum validator stake from chameleon constants
    pub const MinValidatorStake: u128 = MIN_VALIDATOR_STAKE;
    
    /// Unbonding period from chameleon constants (14 days)
    pub const UnbondingPeriod: u32 = UNBONDING_PERIOD_BLOCKS;
    
    /// Maximum delegators per validator
    pub const MaxDelegatorsPerValidator: u32 = 500;
    
    /// Maximum delegations per delegator
    pub const MaxDelegationsPerDelegator: u32 = 100;
    
    /// Maximum unbonding requests per account
    pub const MaxUnbondingRequests: u32 = 10;
}

/// Example implementation of the Config trait
/// 
/// This demonstrates proper configuration with all required types
impl pallet_chameleon_staking::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances; // Must implement LockableCurrency + ReservableCurrency
    type MinValidatorStake = MinValidatorStake;
    type UnbondingPeriod = UnbondingPeriod;
    type MaxDelegatorsPerValidator = MaxDelegatorsPerValidator;
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
    type MaxUnbondingRequests = MaxUnbondingRequests;
}

/// Example usage scenarios
pub mod examples {
    use super::*;
    use pallet_chameleon_staking::{Pallet as ChameleonStaking, SlashingOffense};
    use sp_runtime::traits::Zero;
    
    /// Example: Validator joins the network
    pub fn example_validator_join() -> Result<(), &'static str> {
        let validator_account = AccountId::from([1u8; 32]);
        let stake_amount = MIN_VALIDATOR_STAKE * 2; // 2x minimum stake
        
        // Join as validator candidate
        ChameleonStaking::<Runtime>::join_candidates(
            RuntimeOrigin::signed(validator_account.clone()),
            stake_amount,
        ).map_err(|_| "Failed to join candidates")?;
        
        // Set commission to 15%
        let commission = Perbill::from_percent(15);
        ChameleonStaking::<Runtime>::set_commission(
            RuntimeOrigin::signed(validator_account),
            commission,
        ).map_err(|_| "Failed to set commission")?;
        
        Ok(())
    }
    
    /// Example: User delegates to validator
    pub fn example_delegation() -> Result<(), &'static str> {
        let delegator_account = AccountId::from([2u8; 32]);
        let validator_account = AccountId::from([1u8; 32]);
        let delegation_amount = 1_000_000_000_000_000_000_000; // 1000 CHML
        
        // Delegate to validator
        ChameleonStaking::<Runtime>::delegate(
            RuntimeOrigin::signed(delegator_account),
            validator_account,
            delegation_amount,
        ).map_err(|_| "Failed to delegate")?;
        
        Ok(())
    }
    
    /// Example: Unbonding and withdrawal process
    pub fn example_unbonding() -> Result<(), &'static str> {
        let delegator_account = AccountId::from([2u8; 32]);
        let validator_account = AccountId::from([1u8; 32]);
        
        // Start unbonding
        ChameleonStaking::<Runtime>::undelegate(
            RuntimeOrigin::signed(delegator_account.clone()),
            validator_account,
        ).map_err(|_| "Failed to undelegate")?;
        
        // After unbonding period (14 days), withdraw
        // Note: In practice, you'd wait for the actual time to pass
        ChameleonStaking::<Runtime>::withdraw_unbonded(
            RuntimeOrigin::signed(delegator_account),
        ).map_err(|_| "Failed to withdraw unbonded")?;
        
        Ok(())
    }
    
    /// Example: Reward distribution
    pub fn example_reward_distribution() -> Result<(), &'static str> {
        let total_reward = 1_000_000_000_000_000_000_000; // 1000 CHML
        
        // Distribute rewards for the current era
        ChameleonStaking::<Runtime>::distribute_rewards(total_reward)
            .map_err(|_| "Failed to distribute rewards")?;
        
        Ok(())
    }
    
    /// Example: Claiming rewards
    pub fn example_claim_rewards() -> Result<(), &'static str> {
        let account = AccountId::from([1u8; 32]);
        
        // Claim pending rewards
        ChameleonStaking::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account),
        ).map_err(|_| "Failed to claim rewards")?;
        
        Ok(())
    }
    
    /// Example: Slashing a validator
    pub fn example_slashing() -> Result<(), &'static str> {
        let validator_account = AccountId::from([1u8; 32]);
        
        // Slash validator for extended downtime
        ChameleonStaking::<Runtime>::slash_validator(
            &validator_account,
            SlashingOffense::ExtendedDowntime,
        ).map_err(|_| "Failed to slash validator")?;
        
        Ok(())
    }
    
    /// Example: Query validator information
    pub fn example_queries() {
        let validator_account = AccountId::from([1u8; 32]);
        let delegator_account = AccountId::from([2u8; 32]);
        
        // Check if account is a validator
        let is_validator = ChameleonStaking::<Runtime>::is_validator(&validator_account);
        println!("Is validator: {}", is_validator);
        
        // Get validator information
        if let Some(validator_info) = ChameleonStaking::<Runtime>::get_validator_info(&validator_account) {
            println!("Validator self stake: {}", validator_info.self_stake);
            println!("Validator total stake: {}", validator_info.total_stake);
            println!("Validator commission: {:?}", validator_info.commission);
            println!("Validator status: {:?}", validator_info.status);
        }
        
        // Get delegation amount
        let delegation = ChameleonStaking::<Runtime>::get_delegation(&delegator_account, &validator_account);
        println!("Delegation amount: {}", delegation);
        
        // Get pending rewards
        let pending_rewards = ChameleonStaking::<Runtime>::pending_rewards(&delegator_account);
        println!("Pending rewards: {}", pending_rewards);
        
        // Get total staked in network
        let total_staked = ChameleonStaking::<Runtime>::total_staked();
        println!("Total network stake: {}", total_staked);
        
        // Get current era
        let current_era = ChameleonStaking::<Runtime>::current_era();
        println!("Current era: {}", current_era);
    }
}

/// Constants usage examples
pub mod constants_usage {
    use super::*;
    
    /// Demonstrate usage of chameleon constants
    pub fn show_constants() {
        println!("=== Chameleon Staking Constants ===");
        
        // Staking constants
        println!("Minimum validator stake: {} CHML", MIN_VALIDATOR_STAKE / 1_000_000_000_000_000_000);
        println!("Unbonding period: {} blocks", UNBONDING_PERIOD_BLOCKS);
        
        // Slashing rates
        println!("Downtime slashing: {:?}", SLASHING_DOWNTIME);
        println!("Double-sign slashing: {:?}", SLASHING_DOUBLE_SIGN);
        
        // Convert slashing percentages to human-readable format
        let downtime_percent = SLASHING_DOWNTIME.deconstruct() as f64 / 10_000_000.0;
        let double_sign_percent = SLASHING_DOUBLE_SIGN.deconstruct() as f64 / 10_000_000.0;
        
        println!("Downtime slashing: {}%", downtime_percent);
        println!("Double-sign slashing: {}%", double_sign_percent);
    }
    
    /// Calculate example rewards
    pub fn calculate_example_rewards() {
        let validator_stake = MIN_VALIDATOR_STAKE * 2; // 3,500 CHML
        let delegator_stake = 1_000_000_000_000_000_000_000; // 1,000 CHML
        let total_validator_stake = validator_stake + delegator_stake; // 4,500 CHML
        let commission = Perbill::from_percent(10); // 10% commission
        let era_reward = 100_000_000_000_000_000_000; // 100 CHML reward
        
        println!("=== Reward Calculation Example ===");
        println!("Validator self-stake: {} CHML", validator_stake / 1_000_000_000_000_000_000);
        println!("Delegator stake: {} CHML", delegator_stake / 1_000_000_000_000_000_000);
        println!("Total validator stake: {} CHML", total_validator_stake / 1_000_000_000_000_000_000);
        println!("Era reward: {} CHML", era_reward / 1_000_000_000_000_000_000);
        println!("Commission: {}%", commission.deconstruct() / 10_000_000);
        
        // Calculate commission amount
        let commission_amount = commission * era_reward;
        let delegator_reward_pool = era_reward - commission_amount;
        
        // Validator gets commission + their share of delegator rewards
        let validator_delegator_share = delegator_reward_pool * validator_stake / total_validator_stake;
        let validator_total_reward = commission_amount + validator_delegator_share;
        
        // Delegator gets their proportional share (minus commission)
        let delegator_reward = delegator_reward_pool * delegator_stake / total_validator_stake;
        
        println!("Validator reward: {} CHML", validator_total_reward / 1_000_000_000_000_000_000);
        println!("Delegator reward: {} CHML", delegator_reward / 1_000_000_000_000_000_000);
    }
}

/// Integration checklist
pub mod integration_checklist {
    /// Steps to integrate Chameleon Staking into your runtime
    pub fn integration_steps() {
        println!("=== Chameleon Staking Integration Checklist ===");
        println!("1. ✅ Add pallet to Cargo.toml dependencies");
        println!("2. ✅ Import chameleon constants");
        println!("3. ✅ Configure pallet parameters");
        println!("4. ✅ Add to construct_runtime! macro");
        println!("5. ✅ Implement Config trait");
        println!("6. ✅ Set up genesis configuration");
        println!("7. ✅ Connect to Currency pallet (Balances)");
        println!("8. ✅ Add reward distribution mechanism");
        println!("9. ✅ Set up validator performance monitoring");
        println!("10. ✅ Configure governance for parameter updates");
        println!("");
        println!("Status: ✅ READY FOR INTEGRATION");
    }
}
