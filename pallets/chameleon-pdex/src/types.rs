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

//! Types for the Chameleon pDEX pallet

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, Perbill};
use sp_std::cmp::Ordering;

/// Pool identifier that ensures consistent ordering
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct PoolId<AssetId> {
    pub asset_a: AssetId,
    pub asset_b: AssetId,
}

impl<AssetId: Ord + Copy> PoolId<AssetId> {
    /// Create a new pool ID with consistent ordering (asset_a < asset_b)
    pub fn new(asset_1: AssetId, asset_2: AssetId) -> Self {
        if asset_1 < asset_2 {
            Self {
                asset_a: asset_1,
                asset_b: asset_2,
            }
        } else {
            Self {
                asset_a: asset_2,
                asset_b: asset_1,
            }
        }
    }
}

impl<AssetId: Ord> Ord for PoolId<AssetId> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.asset_a.cmp(&other.asset_a) {
            Ordering::Equal => self.asset_b.cmp(&other.asset_b),
            other => other,
        }
    }
}

impl<AssetId: Ord> PartialOrd for PoolId<AssetId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Liquidity pool structure implementing constant product AMM
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct LiquidityPool<AssetId, Balance> {
    /// First asset in the pair (always the smaller AssetId)
    pub asset_a: AssetId,
    /// Second asset in the pair (always the larger AssetId)
    pub asset_b: AssetId,
    /// Reserve of asset A
    pub reserve_a: Balance,
    /// Reserve of asset B
    pub reserve_b: Balance,
    /// Total LP tokens issued for this pool
    pub total_lp_tokens: Balance,
    /// Swap fee rate (default 0.25%)
    pub fee: Perbill,
}

impl<AssetId: Copy + Ord, Balance: Zero + Copy> LiquidityPool<AssetId, Balance> {
    /// Create a new empty liquidity pool
    pub fn new(asset_1: AssetId, asset_2: AssetId) -> Self {
        let (asset_a, asset_b) = if asset_1 < asset_2 {
            (asset_1, asset_2)
        } else {
            (asset_2, asset_1)
        };
        
        Self {
            asset_a,
            asset_b,
            reserve_a: Balance::zero(),
            reserve_b: Balance::zero(),
            total_lp_tokens: Balance::zero(),
            fee: Perbill::from_parts(2_500_000), // 0.25%
        }
    }
    
    /// Check if the pool is empty (no liquidity)
    pub fn is_empty(&self) -> bool {
        self.reserve_a.is_zero() || self.reserve_b.is_zero()
    }
    
    /// Get the pool ID
    pub fn pool_id(&self) -> PoolId<AssetId> {
        PoolId {
            asset_a: self.asset_a,
            asset_b: self.asset_b,
        }
    }
}

/// LP position for a user in a specific pool
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct LpPosition<AccountId, Balance> {
    /// Owner of the LP position
    pub owner: AccountId,
    /// Pool identifier
    pub pool_id: PoolId<u32>, // Assuming AssetId is u32 for now
    /// Amount of LP tokens owned
    pub lp_tokens: Balance,
    /// Block number when position was created
    pub deposited_at: u32, // BlockNumber
}

impl<AccountId: Clone, Balance: Zero + Copy> LpPosition<AccountId, Balance> {
    /// Create a new LP position
    pub fn new(owner: AccountId, pool_id: PoolId<u32>) -> Self {
        Self {
            owner,
            pool_id,
            lp_tokens: Balance::zero(),
            deposited_at: 0, // Will be set when actually depositing
        }
    }
}

/// Reward information for a pool
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Default)]
pub struct RewardInfo<Balance> {
    /// Total rewards accumulated for this pool
    pub total_rewards: Balance,
    /// Rewards per LP token (scaled by 1e18 for precision)
    pub rewards_per_lp_token: Balance,
    /// Last update block
    pub last_update_block: u32,
    /// 24h trading volume for reward calculation
    pub volume_24h: Balance,
    /// Total value locked (TVL) in the pool
    pub tvl: Balance,
}

impl<Balance: Zero + Copy> RewardInfo<Balance> {
    /// Create new reward info
    pub fn new() -> Self {
        Self {
            total_rewards: Balance::zero(),
            rewards_per_lp_token: Balance::zero(),
            last_update_block: 0,
            volume_24h: Balance::zero(),
            tvl: Balance::zero(),
        }
    }
}

/// Swap path for multi-hop swaps (future feature)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SwapPath<AssetId> {
    /// Assets in the swap path
    pub assets: sp_std::vec::Vec<AssetId>,
}

/// Swap information for tracking
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SwapInfo<AssetId, Balance> {
    /// Input asset
    pub asset_in: AssetId,
    /// Output asset
    pub asset_out: AssetId,
    /// Input amount
    pub amount_in: Balance,
    /// Output amount
    pub amount_out: Balance,
    /// Fee paid
    pub fee_paid: Balance,
    /// Price impact percentage (scaled by 1e6)
    pub price_impact: u32,
}

/// Pool statistics for analytics
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Default)]
pub struct PoolStats<Balance> {
    /// 24h trading volume
    pub volume_24h: Balance,
    /// 7d trading volume
    pub volume_7d: Balance,
    /// Total fees collected
    pub total_fees: Balance,
    /// Number of swaps
    pub swap_count: u32,
    /// Number of LP providers
    pub lp_count: u32,
    /// All-time high TVL
    pub ath_tvl: Balance,
}

/// Vesting schedule for LP rewards
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct VestingSchedule<Balance> {
    /// Total amount to vest
    pub total_amount: Balance,
    /// Amount already claimed
    pub claimed_amount: Balance,
    /// Vesting start block
    pub start_block: u32,
    /// Vesting duration in blocks (90 days = ~1,296,000 blocks)
    pub duration_blocks: u32,
}

impl<Balance: Zero + Copy> VestingSchedule<Balance> {
    /// Create new vesting schedule
    pub fn new(total_amount: Balance, start_block: u32, duration_blocks: u32) -> Self {
        Self {
            total_amount,
            claimed_amount: Balance::zero(),
            start_block,
            duration_blocks,
        }
    }
    
    /// Calculate vested amount at given block
    pub fn vested_amount(&self, current_block: u32) -> Balance
    where
        Balance: From<u128> + sp_std::ops::Mul<Output = Balance> + sp_std::ops::Div<Output = Balance>,
    {
        if current_block <= self.start_block {
            return Balance::zero();
        }
        
        let elapsed_blocks = current_block - self.start_block;
        if elapsed_blocks >= self.duration_blocks {
            return self.total_amount;
        }
        
        // Linear vesting: vested = total * elapsed / duration
        self.total_amount * Balance::from(elapsed_blocks as u128) / Balance::from(self.duration_blocks as u128)
    }
    
    /// Calculate claimable amount (vested - claimed)
    pub fn claimable_amount(&self, current_block: u32) -> Balance
    where
        Balance: From<u128> + sp_std::ops::Mul<Output = Balance> + sp_std::ops::Div<Output = Balance> + sp_std::ops::Sub<Output = Balance>,
    {
        let vested = self.vested_amount(current_block);
        if vested > self.claimed_amount {
            vested - self.claimed_amount
        } else {
            Balance::zero()
        }
    }
}

/// Price oracle data (for future price feeds)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct PriceData<Balance> {
    /// Price in base units (scaled by 1e18)
    pub price: Balance,
    /// Timestamp of last update
    pub timestamp: u64,
    /// Confidence interval (0-100)
    pub confidence: u8,
}

/// Pool configuration parameters
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct PoolConfig {
    /// Swap fee percentage (in Perbill)
    pub swap_fee: Perbill,
    /// Minimum liquidity required
    pub min_liquidity: u128,
    /// Maximum price impact allowed (in Perbill)
    pub max_price_impact: Perbill,
    /// Whether the pool is active
    pub is_active: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            swap_fee: Perbill::from_parts(2_500_000), // 0.25%
            min_liquidity: 1000, // Minimum 1000 units
            max_price_impact: Perbill::from_parts(100_000_000), // 10%
            is_active: true,
        }
    }
}

/// Fee distribution configuration
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct FeeDistribution {
    /// Percentage to LP providers (90%)
    pub lp_share: Perbill,
    /// Percentage to treasury (10%)
    pub treasury_share: Perbill,
}

impl Default for FeeDistribution {
    fn default() -> Self {
        Self {
            lp_share: Perbill::from_parts(900_000_000), // 90%
            treasury_share: Perbill::from_parts(100_000_000), // 10%
        }
    }
}

/// Slippage tolerance configuration
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SlippageTolerance {
    /// Maximum allowed slippage (in Perbill)
    pub max_slippage: Perbill,
}

impl Default for SlippageTolerance {
    fn default() -> Self {
        Self {
            max_slippage: Perbill::from_parts(10_000_000), // 1%
        }
    }
}
