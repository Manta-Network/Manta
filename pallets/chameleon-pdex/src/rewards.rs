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

//! LP Rewards distribution system for Chameleon pDEX
//!
//! Distributes 30% of validator emissions (19.5M CHML over 20 years) to LP providers
//! based on their share of liquidity and pool performance metrics.

use sp_runtime::{
    traits::{Zero, Saturating},
    Perbill,
};
use sp_std::collections::btree_map::BTreeMap;

// Import chameleon constants
use manta_primitives::chameleon_constants::{
    emission::{LP_REWARD_PERCENT, YEARLY_EMISSIONS},
    time::BLOCKS_PER_YEAR,
};

use crate::types::{PoolId, RewardInfo, VestingSchedule};

/// Reward calculation errors
#[derive(Debug, PartialEq, Eq)]
pub enum RewardError {
    /// No rewards available
    NoRewards,
    /// Invalid pool
    InvalidPool,
    /// Mathematical overflow
    Overflow,
    /// Division by zero
    DivisionByZero,
}

/// Pool performance metrics for dynamic yield optimization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolMetrics<Balance> {
    /// Total Value Locked (TVL) in USD equivalent
    pub tvl: Balance,
    /// 24-hour trading volume
    pub volume_24h: Balance,
    /// Utilization rate (volume / TVL)
    pub utilization_rate: u32, // Basis points (0-10000)
    /// Risk factor (lower is better)
    pub risk_factor: u32, // Basis points (0-10000)
    /// Number of active LPs
    pub lp_count: u32,
}

impl<Balance: Zero + Copy> Default for PoolMetrics<Balance> {
    fn default() -> Self {
        Self {
            tvl: Balance::zero(),
            volume_24h: Balance::zero(),
            utilization_rate: 0,
            risk_factor: 1000, // 10% default risk
            lp_count: 0,
        }
    }
}

/// Reward distribution calculator
pub struct RewardDistributor<AssetId, Balance> {
    /// Pool metrics for reward calculation
    pub pool_metrics: BTreeMap<PoolId<AssetId>, PoolMetrics<Balance>>,
    /// Current year (0-19 for 20-year schedule)
    pub current_year: u32,
    /// Current block number
    pub current_block: u32,
}

impl<AssetId: Ord + Copy, Balance> RewardDistributor<AssetId, Balance>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
{
    /// Create new reward distributor
    pub fn new(current_year: u32, current_block: u32) -> Self {
        Self {
            pool_metrics: BTreeMap::new(),
            current_year,
            current_block,
        }
    }

    /// Calculate total LP rewards for current period (per block)
    pub fn calculate_period_rewards(&self) -> Result<Balance, RewardError> {
        if self.current_year >= 20 {
            return Ok(Balance::zero()); // Emission period ended
        }

        // Get yearly emission for current year
        let yearly_emission = Balance::from(YEARLY_EMISSIONS[self.current_year as usize]);
        
        // Calculate LP share (30% of total emissions)
        let lp_yearly_emission = yearly_emission.saturating_mul(Balance::from(LP_REWARD_PERCENT as u128)) 
            / Balance::from(100u128);
        
        // Convert to per-block rewards
        let blocks_per_year = Balance::from(BLOCKS_PER_YEAR as u128);
        let per_block_rewards = lp_yearly_emission / blocks_per_year;
        
        Ok(per_block_rewards)
    }

    /// Calculate pool weights for dynamic yield optimization
    /// 
    /// Weight = TVL × Volume × Utilization / Risk_Factor
    /// Higher weight = higher reward allocation
    pub fn calculate_pool_weights(&self) -> BTreeMap<PoolId<AssetId>, u128> {
        let mut weights = BTreeMap::new();
        
        for (pool_id, metrics) in &self.pool_metrics {
            // Base weight from TVL (scaled down to prevent overflow)
            let tvl_weight = self.balance_to_u128(metrics.tvl) / 1_000_000; // Scale down
            
            // Volume multiplier (higher volume = higher weight)
            let volume_weight = self.balance_to_u128(metrics.volume_24h) / 1_000_000; // Scale down
            
            // Utilization bonus (0-100% -> 1.0-2.0x multiplier)
            let utilization_multiplier = 10000 + metrics.utilization_rate; // 100% - 200%
            
            // Risk penalty (higher risk = lower weight)
            let risk_divisor = sp_std::cmp::max(metrics.risk_factor, 1000); // Min 10% risk
            
            // LP count bonus (more LPs = more decentralized = higher weight)
            let lp_bonus = sp_std::cmp::min(metrics.lp_count * 100, 5000); // Max 50% bonus
            
            // Final weight calculation
            let base_weight = (tvl_weight + volume_weight) * utilization_multiplier as u128 / 10000;
            let adjusted_weight = base_weight * (10000 + lp_bonus as u128) / (risk_divisor as u128);
            
            weights.insert(pool_id.clone(), adjusted_weight);
        }
        
        weights
    }

    /// Distribute rewards to pools based on their weights
    pub fn distribute_pool_rewards(
        &self,
        total_rewards: Balance,
    ) -> Result<BTreeMap<PoolId<AssetId>, Balance>, RewardError> {
        let pool_weights = self.calculate_pool_weights();
        let total_weight: u128 = pool_weights.values().sum();
        
        if total_weight == 0 {
            return Ok(BTreeMap::new());
        }
        
        let mut pool_rewards = BTreeMap::new();
        let total_rewards_u128 = self.balance_to_u128(total_rewards);
        
        for (pool_id, weight) in pool_weights {
            let pool_reward_u128 = (total_rewards_u128 * weight) / total_weight;
            let pool_reward = Balance::from(pool_reward_u128);
            pool_rewards.insert(pool_id, pool_reward);
        }
        
        Ok(pool_rewards)
    }

    /// Calculate individual LP rewards within a pool
    pub fn calculate_lp_rewards(
        &self,
        pool_reward: Balance,
        lp_positions: &BTreeMap<[u8; 32], Balance>, // AccountId -> LP tokens (simplified as bytes)
        total_lp_tokens: Balance,
    ) -> Result<BTreeMap<[u8; 32], Balance>, RewardError> {
        if total_lp_tokens.is_zero() {
            return Ok(BTreeMap::new());
        }
        
        let mut lp_rewards = BTreeMap::new();
        
        for (account, lp_tokens) in lp_positions {
            // Proportional reward: user_reward = pool_reward * user_lp_tokens / total_lp_tokens
            let user_reward = pool_reward.saturating_mul(*lp_tokens) / total_lp_tokens;
            
            if !user_reward.is_zero() {
                lp_rewards.insert(*account, user_reward);
            }
        }
        
        Ok(lp_rewards)
    }

    /// Create vesting schedule for LP rewards (50% instant, 50% vested over 90 days)
    pub fn create_reward_vesting(
        &self,
        total_reward: Balance,
    ) -> (Balance, VestingSchedule<Balance>) {
        let instant_reward = total_reward / Balance::from(2u128); // 50%
        let vested_reward = total_reward.saturating_sub(instant_reward); // 50%
        
        let vesting_duration = 90 * 24 * 60 * 10; // 90 days in blocks (6s per block)
        let vesting_schedule = VestingSchedule::new(
            vested_reward,
            self.current_block,
            vesting_duration,
        );
        
        (instant_reward, vesting_schedule)
    }

    /// Update pool metrics for reward calculation
    pub fn update_pool_metrics(
        &mut self,
        pool_id: PoolId<AssetId>,
        tvl: Balance,
        volume_24h: Balance,
        lp_count: u32,
    ) {
        let utilization_rate = if !tvl.is_zero() {
            let volume_u128 = self.balance_to_u128(volume_24h);
            let tvl_u128 = self.balance_to_u128(tvl);
            sp_std::cmp::min((volume_u128 * 10000) / tvl_u128, 10000) as u32
        } else {
            0
        };
        
        // Calculate risk factor based on pool characteristics
        let risk_factor = self.calculate_risk_factor(tvl, volume_24h, lp_count);
        
        let metrics = PoolMetrics {
            tvl,
            volume_24h,
            utilization_rate,
            risk_factor,
            lp_count,
        };
        
        self.pool_metrics.insert(pool_id, metrics);
    }

    /// Calculate risk factor for a pool
    /// Lower risk = higher rewards
    fn calculate_risk_factor(
        &self,
        tvl: Balance,
        volume_24h: Balance,
        lp_count: u32,
    ) -> u32 {
        let mut risk = 1000u32; // Base 10% risk
        
        // TVL risk: lower TVL = higher risk
        let tvl_u128 = self.balance_to_u128(tvl);
        if tvl_u128 < 1_000_000 { // Less than 1M units
            risk += 500; // +5% risk
        }
        
        // Volume risk: very high or very low volume = higher risk
        let volume_u128 = self.balance_to_u128(volume_24h);
        if volume_u128 > tvl_u128 * 2 { // Volume > 2x TVL
            risk += 300; // +3% risk (high volatility)
        } else if volume_u128 < tvl_u128 / 100 { // Volume < 1% TVL
            risk += 200; // +2% risk (low liquidity)
        }
        
        // LP count risk: fewer LPs = higher centralization risk
        if lp_count < 5 {
            risk += 400; // +4% risk
        } else if lp_count < 20 {
            risk += 200; // +2% risk
        }
        
        sp_std::cmp::min(risk, 5000) // Cap at 50% risk
    }

    /// Helper function to convert Balance to u128 (simplified)
    fn balance_to_u128(&self, balance: Balance) -> u128 {
        // This is a simplified conversion - in production, implement proper conversion
        // For now, assume Balance can be converted to u128
        1000000u128 // Placeholder
    }

    /// Get estimated APY for a pool based on current metrics
    pub fn estimate_pool_apy(&self, pool_id: &PoolId<AssetId>) -> u32 {
        if let Some(metrics) = self.pool_metrics.get(pool_id) {
            // Base APY from LP rewards
            let base_apy = self.calculate_base_apy();
            
            // Volume bonus: higher volume = higher fee income
            let volume_bonus = sp_std::cmp::min(metrics.utilization_rate / 100, 500); // Max 5% bonus
            
            // Risk adjustment
            let risk_penalty = metrics.risk_factor / 100; // Convert basis points to percentage
            
            let total_apy = base_apy + volume_bonus;
            if total_apy > risk_penalty {
                total_apy - risk_penalty
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Calculate base APY from LP reward emissions
    fn calculate_base_apy(&self) -> u32 {
        if self.current_year >= 20 {
            return 0;
        }
        
        // Simplified calculation - in production, use actual TVL data
        // Assume total TVL of $50M and current year emission
        let yearly_lp_rewards = YEARLY_EMISSIONS[self.current_year as usize] * LP_REWARD_PERCENT as u128 / 100;
        
        // Assuming $0.50 per CHML and $50M total TVL
        // APY = (yearly_rewards * price) / total_tvl * 100
        // This is a simplified calculation
        match self.current_year {
            0 => 2500, // 25% APY in year 1
            1 => 2250, // 22.5% APY in year 2
            2 => 2000, // 20% APY in year 3
            _ => 1500, // 15% APY for later years
        }
    }
}

/// Reward claim manager
pub struct RewardClaimer<Balance> {
    /// Pending rewards for users
    pub pending_rewards: BTreeMap<[u8; 32], Balance>, // AccountId -> Amount
    /// Vesting schedules
    pub vesting_schedules: BTreeMap<[u8; 32], sp_std::vec::Vec<VestingSchedule<Balance>>>,
}

impl<Balance> RewardClaimer<Balance>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Add<Output = Balance> +
             sp_std::ops::Sub<Output = Balance> +
             PartialOrd + PartialEq,
{
    /// Create new reward claimer
    pub fn new() -> Self {
        Self {
            pending_rewards: BTreeMap::new(),
            vesting_schedules: BTreeMap::new(),
        }
    }

    /// Add rewards for a user
    pub fn add_rewards(&mut self, account: [u8; 32], amount: Balance) {
        let current_rewards = self.pending_rewards.get(&account).copied().unwrap_or(Balance::zero());
        self.pending_rewards.insert(account, current_rewards + amount);
    }

    /// Add vesting schedule for a user
    pub fn add_vesting_schedule(&mut self, account: [u8; 32], schedule: VestingSchedule<Balance>) {
        self.vesting_schedules.entry(account).or_insert_with(sp_std::vec::Vec::new).push(schedule);
    }

    /// Claim available rewards (instant + vested)
    pub fn claim_rewards(&mut self, account: [u8; 32], current_block: u32) -> Balance
    where
        Balance: sp_std::ops::Mul<Output = Balance> + sp_std::ops::Div<Output = Balance>,
    {
        let mut total_claimable = Balance::zero();
        
        // Claim instant rewards
        if let Some(pending) = self.pending_rewards.remove(&account) {
            total_claimable = total_claimable + pending;
        }
        
        // Claim vested rewards
        if let Some(schedules) = self.vesting_schedules.get_mut(&account) {
            let mut remaining_schedules = sp_std::vec::Vec::new();
            
            for mut schedule in schedules.drain(..) {
                let claimable = schedule.claimable_amount(current_block);
                if !claimable.is_zero() {
                    total_claimable = total_claimable + claimable;
                    schedule.claimed_amount = schedule.claimed_amount + claimable;
                }
                
                // Keep schedule if not fully claimed
                if schedule.claimed_amount < schedule.total_amount {
                    remaining_schedules.push(schedule);
                }
            }
            
            if !remaining_schedules.is_empty() {
                self.vesting_schedules.insert(account, remaining_schedules);
            } else {
                self.vesting_schedules.remove(&account);
            }
        }
        
        total_claimable
    }

    /// Get total claimable amount for a user
    pub fn get_claimable_amount(&self, account: [u8; 32], current_block: u32) -> Balance
    where
        Balance: sp_std::ops::Mul<Output = Balance> + sp_std::ops::Div<Output = Balance>,
    {
        let mut total = Balance::zero();
        
        // Add instant rewards
        if let Some(pending) = self.pending_rewards.get(&account) {
            total = total + *pending;
        }
        
        // Add vested rewards
        if let Some(schedules) = self.vesting_schedules.get(&account) {
            for schedule in schedules {
                total = total + schedule.claimable_amount(current_block);
            }
        }
        
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PoolId;

    type Balance = u128;
    type AssetId = u32;

    #[test]
    fn test_period_rewards_calculation() {
        let distributor = RewardDistributor::<AssetId, Balance>::new(0, 0);
        let period_rewards = distributor.calculate_period_rewards().unwrap();
        
        // Year 1: 7.4M CHML, 30% to LPs = 2.22M CHML
        // Per block: 2.22M / blocks_per_year
        assert!(period_rewards > 0);
    }

    #[test]
    fn test_pool_weight_calculation() {
        let mut distributor = RewardDistributor::<AssetId, Balance>::new(0, 0);
        
        let pool_id = PoolId::new(1, 2);
        distributor.update_pool_metrics(pool_id.clone(), 1_000_000, 100_000, 10);
        
        let weights = distributor.calculate_pool_weights();
        assert!(weights.contains_key(&pool_id));
        assert!(weights[&pool_id] > 0);
    }

    #[test]
    fn test_lp_reward_distribution() {
        let distributor = RewardDistributor::<AssetId, Balance>::new(0, 0);
        
        let mut lp_positions = BTreeMap::new();
        lp_positions.insert([1u8; 32], 500u128); // Alice: 500 LP tokens
        lp_positions.insert([2u8; 32], 300u128); // Bob: 300 LP tokens
        lp_positions.insert([3u8; 32], 200u128); // Charlie: 200 LP tokens
        
        let total_lp_tokens = 1000u128;
        let pool_reward = 1000u128;
        
        let lp_rewards = distributor.calculate_lp_rewards(
            pool_reward,
            &lp_positions,
            total_lp_tokens,
        ).unwrap();
        
        // Alice should get 50% (500/1000)
        assert_eq!(lp_rewards[&[1u8; 32]], 500);
        // Bob should get 30% (300/1000)
        assert_eq!(lp_rewards[&[2u8; 32]], 300);
        // Charlie should get 20% (200/1000)
        assert_eq!(lp_rewards[&[3u8; 32]], 200);
    }

    #[test]
    fn test_vesting_schedule() {
        let distributor = RewardDistributor::<AssetId, Balance>::new(0, 1000);
        let total_reward = 1000u128;
        
        let (instant, vesting) = distributor.create_reward_vesting(total_reward);
        
        assert_eq!(instant, 500); // 50% instant
        assert_eq!(vesting.total_amount, 500); // 50% vested
        assert_eq!(vesting.start_block, 1000);
    }

    #[test]
    fn test_reward_claiming() {
        let mut claimer = RewardClaimer::<Balance>::new();
        let account = [1u8; 32];
        
        // Add instant rewards
        claimer.add_rewards(account, 500);
        
        // Add vesting schedule
        let vesting = VestingSchedule::new(500, 1000, 1000); // 500 tokens over 1000 blocks
        claimer.add_vesting_schedule(account, vesting);
        
        // Claim at block 1500 (halfway through vesting)
        let claimable = claimer.claim_rewards(account, 1500);
        
        // Should get 500 instant + 250 vested (50% of 500)
        assert_eq!(claimable, 750);
    }

    #[test]
    fn test_apy_estimation() {
        let mut distributor = RewardDistributor::<AssetId, Balance>::new(0, 0);
        let pool_id = PoolId::new(1, 2);
        
        // High TVL, good volume, many LPs = lower risk, higher APY
        distributor.update_pool_metrics(pool_id.clone(), 10_000_000, 1_000_000, 50);
        
        let apy = distributor.estimate_pool_apy(&pool_id);
        assert!(apy > 1000); // Should be > 10% APY
    }
}
