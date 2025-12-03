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

//! # Chameleon pDEX (Privacy-preserving Decentralized Exchange)
//!
//! A privacy-preserving decentralized exchange implementation for Chameleon Network
//! featuring automated market maker (AMM) pools with constant product formula.
//!
//! ## Overview
//!
//! The pDEX enables private token swaps using:
//! - Constant product AMM (x × y = k)
//! - 0.25% swap fees (90% to LPs, 10% to Treasury)
//! - LP rewards from 30% of validator emissions
//! - Slippage protection and MEV resistance
//!
//! ## Key Features
//!
//! - **AMM Pools**: Uniswap v2-style constant product formula
//! - **LP Rewards**: 19.5M CHML distributed over 20 years
//! - **Fee Distribution**: 90% to LPs, 10% to Treasury
//! - **Privacy**: Future zkSNARK integration for private amounts

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::{fungibles, Preservation},
        Get, StorageVersion,
    },
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};
use sp_std::vec::Vec;

// Re-export pallet items
pub use pallet::*;

// Import modules
mod types;
mod amm;
mod rewards;

pub use types::*;
pub use amm::*;
pub use rewards::*;

// Import chameleon constants
use manta_primitives::chameleon_constants::CHAMELEON_PDEX_PALLET_ID;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

mod weights;
pub use weights::WeightInfo;

/// Current storage version
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency used for staking and rewards
        type Currency: fungibles::Inspect<Self::AccountId>
            + fungibles::Mutate<Self::AccountId>
            + fungibles::Create<Self::AccountId>;

        /// Asset ID type
        type AssetId: Parameter + Copy + Ord + Default;

        /// Balance type
        type Balance: Parameter + Copy + Ord + Zero + Saturating + From<u128>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// Pallet ID for the pDEX
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Maximum number of pools
        #[pallet::constant]
        type MaxPools: Get<u32>;

        /// Minimum liquidity for pool creation
        #[pallet::constant]
        type MinimumLiquidity: Get<Self::Balance>;
    }

    /// Liquidity pools storage
    /// Maps (AssetA, AssetB) -> LiquidityPool
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        Blake2_128Concat,
        T::AssetId,
        LiquidityPool<T::AssetId, T::Balance>,
        OptionQuery,
    >;

    /// LP token positions for users
    /// Maps (AccountId, PoolId) -> LpPosition
    #[pallet::storage]
    #[pallet::getter(fn lp_positions)]
    pub type LpPositions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        PoolId<T::AssetId>,
        LpPosition<T::AccountId, T::Balance>,
        OptionQuery,
    >;

    /// Pool rewards accumulator
    /// Maps PoolId -> RewardInfo
    #[pallet::storage]
    #[pallet::getter(fn pool_rewards)]
    pub type PoolRewards<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PoolId<T::AssetId>,
        RewardInfo<T::Balance>,
        ValueQuery,
    >;

    /// User claimable rewards
    /// Maps (AccountId, PoolId) -> Balance
    #[pallet::storage]
    #[pallet::getter(fn user_rewards)]
    pub type UserRewards<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        PoolId<T::AssetId>,
        T::Balance,
        ValueQuery,
    >;

    /// Total volume per pool (24h rolling)
    #[pallet::storage]
    #[pallet::getter(fn pool_volume)]
    pub type PoolVolume<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PoolId<T::AssetId>,
        T::Balance,
        ValueQuery,
    >;

    /// Pool creation nonce for unique pool IDs
    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub type NextPoolId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Pool created [pool_id, asset_a, asset_b, creator]
        PoolCreated {
            pool_id: PoolId<T::AssetId>,
            asset_a: T::AssetId,
            asset_b: T::AssetId,
            creator: T::AccountId,
        },
        /// Liquidity added [pool_id, provider, amount_a, amount_b, lp_tokens]
        LiquidityAdded {
            pool_id: PoolId<T::AssetId>,
            provider: T::AccountId,
            amount_a: T::Balance,
            amount_b: T::Balance,
            lp_tokens: T::Balance,
        },
        /// Liquidity removed [pool_id, provider, amount_a, amount_b, lp_tokens]
        LiquidityRemoved {
            pool_id: PoolId<T::AssetId>,
            provider: T::AccountId,
            amount_a: T::Balance,
            amount_b: T::Balance,
            lp_tokens: T::Balance,
        },
        /// Swap executed [pool_id, trader, asset_in, asset_out, amount_in, amount_out]
        Swap {
            pool_id: PoolId<T::AssetId>,
            trader: T::AccountId,
            asset_in: T::AssetId,
            asset_out: T::AssetId,
            amount_in: T::Balance,
            amount_out: T::Balance,
        },
        /// LP rewards claimed [pool_id, claimer, amount]
        RewardsClaimed {
            pool_id: PoolId<T::AssetId>,
            claimer: T::AccountId,
            amount: T::Balance,
        },
        /// Pool rewards distributed [pool_id, total_amount]
        RewardsDistributed {
            pool_id: PoolId<T::AssetId>,
            total_amount: T::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool already exists
        PoolAlreadyExists,
        /// Pool does not exist
        PoolNotFound,
        /// Insufficient liquidity in pool
        InsufficientLiquidity,
        /// Insufficient balance
        InsufficientBalance,
        /// Slippage tolerance exceeded
        SlippageExceeded,
        /// Invalid asset pair (same asset)
        InvalidAssetPair,
        /// Amount too small
        AmountTooSmall,
        /// LP position not found
        LpPositionNotFound,
        /// No rewards to claim
        NoRewardsToClaim,
        /// Pool creation limit reached
        TooManyPools,
        /// Minimum liquidity not met
        MinimumLiquidityNotMet,
        /// Mathematical overflow
        Overflow,
        /// Division by zero
        DivisionByZero,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new liquidity pool
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_pool())]
        pub fn create_pool(
            origin: OriginFor<T>,
            asset_a: T::AssetId,
            asset_b: T::AssetId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure assets are different
            ensure!(asset_a != asset_b, Error::<T>::InvalidAssetPair);

            // Ensure pool doesn't already exist (check both directions)
            ensure!(
                !Pools::<T>::contains_key(&asset_a, &asset_b) &&
                !Pools::<T>::contains_key(&asset_b, &asset_a),
                Error::<T>::PoolAlreadyExists
            );

            // Check pool limit
            let pool_count = NextPoolId::<T>::get();
            ensure!(pool_count < T::MaxPools::get(), Error::<T>::TooManyPools);

            // Create pool ID
            let pool_id = PoolId::new(asset_a, asset_b);

            // Create new pool
            let pool = LiquidityPool::new(asset_a, asset_b);

            // Store pool (always store with asset_a < asset_b for consistency)
            let (first_asset, second_asset) = if asset_a < asset_b {
                (asset_a, asset_b)
            } else {
                (asset_b, asset_a)
            };
            Pools::<T>::insert(&first_asset, &second_asset, &pool);

            // Initialize pool rewards
            PoolRewards::<T>::insert(&pool_id, RewardInfo::default());

            // Increment pool counter
            NextPoolId::<T>::put(pool_count + 1);

            // Emit event
            Self::deposit_event(Event::PoolCreated {
                pool_id,
                asset_a,
                asset_b,
                creator: who,
            });

            Ok(())
        }

        /// Add liquidity to a pool
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            asset_a: T::AssetId,
            asset_b: T::AssetId,
            amount_a_desired: T::Balance,
            amount_b_desired: T::Balance,
            amount_a_min: T::Balance,
            amount_b_min: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get pool (check both directions)
            let (pool, is_reversed) = Self::get_pool_with_direction(&asset_a, &asset_b)
                .ok_or(Error::<T>::PoolNotFound)?;

            // Calculate optimal amounts
            let (amount_a, amount_b) = if pool.total_lp_tokens.is_zero() {
                // First liquidity provision
                ensure!(
                    amount_a_desired >= T::MinimumLiquidity::get() &&
                    amount_b_desired >= T::MinimumLiquidity::get(),
                    Error::<T>::MinimumLiquidityNotMet
                );
                (amount_a_desired, amount_b_desired)
            } else {
                // Subsequent liquidity provision - maintain ratio
                let (reserve_a, reserve_b) = if is_reversed {
                    (pool.reserve_b, pool.reserve_a)
                } else {
                    (pool.reserve_a, pool.reserve_b)
                };

                let amount_b_optimal = Self::quote(amount_a_desired, reserve_a, reserve_b)?;
                if amount_b_optimal <= amount_b_desired {
                    ensure!(amount_b_optimal >= amount_b_min, Error::<T>::SlippageExceeded);
                    (amount_a_desired, amount_b_optimal)
                } else {
                    let amount_a_optimal = Self::quote(amount_b_desired, reserve_b, reserve_a)?;
                    ensure!(amount_a_optimal <= amount_a_desired, Error::<T>::SlippageExceeded);
                    ensure!(amount_a_optimal >= amount_a_min, Error::<T>::SlippageExceeded);
                    (amount_a_optimal, amount_b_desired)
                }
            };

            // Calculate LP tokens to mint
            let lp_tokens = Self::calculate_lp_tokens_to_mint(&pool, amount_a, amount_b, is_reversed)?;

            // Transfer tokens from user to pool account
            let pool_account = Self::pool_account_id(&asset_a, &asset_b);
            T::Currency::transfer(
                asset_a,
                &who,
                &pool_account,
                amount_a,
                Preservation::Expendable,
            )?;
            T::Currency::transfer(
                asset_b,
                &who,
                &pool_account,
                amount_b,
                Preservation::Expendable,
            )?;

            // Update pool state
            let mut updated_pool = pool;
            if is_reversed {
                updated_pool.reserve_a = updated_pool.reserve_a.saturating_add(amount_b);
                updated_pool.reserve_b = updated_pool.reserve_b.saturating_add(amount_a);
            } else {
                updated_pool.reserve_a = updated_pool.reserve_a.saturating_add(amount_a);
                updated_pool.reserve_b = updated_pool.reserve_b.saturating_add(amount_b);
            }
            updated_pool.total_lp_tokens = updated_pool.total_lp_tokens.saturating_add(lp_tokens);

            // Store updated pool
            let (first_asset, second_asset) = if asset_a < asset_b {
                (asset_a, asset_b)
            } else {
                (asset_b, asset_a)
            };
            Pools::<T>::insert(&first_asset, &second_asset, &updated_pool);

            // Update LP position
            let pool_id = PoolId::new(asset_a, asset_b);
            let current_position = LpPositions::<T>::get(&who, &pool_id)
                .unwrap_or_else(|| LpPosition::new(who.clone(), pool_id.clone()));
            
            let updated_position = LpPosition {
                lp_tokens: current_position.lp_tokens.saturating_add(lp_tokens),
                ..current_position
            };
            LpPositions::<T>::insert(&who, &pool_id, &updated_position);

            // Emit event
            Self::deposit_event(Event::LiquidityAdded {
                pool_id,
                provider: who,
                amount_a,
                amount_b,
                lp_tokens,
            });

            Ok(())
        }

        /// Remove liquidity from a pool
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::remove_liquidity())]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            asset_a: T::AssetId,
            asset_b: T::AssetId,
            lp_tokens: T::Balance,
            amount_a_min: T::Balance,
            amount_b_min: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get pool and LP position
            let (pool, is_reversed) = Self::get_pool_with_direction(&asset_a, &asset_b)
                .ok_or(Error::<T>::PoolNotFound)?;
            
            let pool_id = PoolId::new(asset_a, asset_b);
            let position = LpPositions::<T>::get(&who, &pool_id)
                .ok_or(Error::<T>::LpPositionNotFound)?;

            // Ensure user has enough LP tokens
            ensure!(position.lp_tokens >= lp_tokens, Error::<T>::InsufficientBalance);

            // Calculate amounts to return
            let (amount_a, amount_b) = Self::calculate_remove_liquidity_amounts(
                &pool, lp_tokens, is_reversed
            )?;

            // Check slippage
            ensure!(amount_a >= amount_a_min, Error::<T>::SlippageExceeded);
            ensure!(amount_b >= amount_b_min, Error::<T>::SlippageExceeded);

            // Transfer tokens back to user
            let pool_account = Self::pool_account_id(&asset_a, &asset_b);
            T::Currency::transfer(
                asset_a,
                &pool_account,
                &who,
                amount_a,
                Preservation::Expendable,
            )?;
            T::Currency::transfer(
                asset_b,
                &pool_account,
                &who,
                amount_b,
                Preservation::Expendable,
            )?;

            // Update pool state
            let mut updated_pool = pool;
            if is_reversed {
                updated_pool.reserve_a = updated_pool.reserve_a.saturating_sub(amount_b);
                updated_pool.reserve_b = updated_pool.reserve_b.saturating_sub(amount_a);
            } else {
                updated_pool.reserve_a = updated_pool.reserve_a.saturating_sub(amount_a);
                updated_pool.reserve_b = updated_pool.reserve_b.saturating_sub(amount_b);
            }
            updated_pool.total_lp_tokens = updated_pool.total_lp_tokens.saturating_sub(lp_tokens);

            // Store updated pool
            let (first_asset, second_asset) = if asset_a < asset_b {
                (asset_a, asset_b)
            } else {
                (asset_b, asset_a)
            };
            Pools::<T>::insert(&first_asset, &second_asset, &updated_pool);

            // Update LP position
            let updated_position = LpPosition {
                lp_tokens: position.lp_tokens.saturating_sub(lp_tokens),
                ..position
            };
            
            if updated_position.lp_tokens.is_zero() {
                LpPositions::<T>::remove(&who, &pool_id);
            } else {
                LpPositions::<T>::insert(&who, &pool_id, &updated_position);
            }

            // Emit event
            Self::deposit_event(Event::LiquidityRemoved {
                pool_id,
                provider: who,
                amount_a,
                amount_b,
                lp_tokens,
            });

            Ok(())
        }

        /// Swap exact tokens for tokens
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap_exact_tokens_for_tokens(
            origin: OriginFor<T>,
            amount_in: T::Balance,
            amount_out_min: T::Balance,
            path: Vec<T::AssetId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate path
            ensure!(path.len() >= 2, Error::<T>::InvalidAssetPair);
            ensure!(path.len() <= 3, Error::<T>::InvalidAssetPair); // Support direct and single-hop swaps

            let asset_in = path[0];
            let asset_out = path[path.len() - 1];

            // For now, only support direct swaps (single pool)
            ensure!(path.len() == 2, Error::<T>::InvalidAssetPair);

            // Get pool
            let (pool, is_reversed) = Self::get_pool_with_direction(&asset_in, &asset_out)
                .ok_or(Error::<T>::PoolNotFound)?;

            // Calculate output amount
            let amount_out = Self::calculate_swap_output(
                amount_in,
                &pool,
                asset_in,
                is_reversed,
            )?;

            // Check slippage
            ensure!(amount_out >= amount_out_min, Error::<T>::SlippageExceeded);

            // Execute swap
            let pool_account = Self::pool_account_id(&asset_in, &asset_out);
            
            // Transfer input tokens from user to pool
            T::Currency::transfer(
                asset_in,
                &who,
                &pool_account,
                amount_in,
                Preservation::Expendable,
            )?;

            // Transfer output tokens from pool to user
            T::Currency::transfer(
                asset_out,
                &pool_account,
                &who,
                amount_out,
                Preservation::Expendable,
            )?;

            // Update pool reserves
            let mut updated_pool = pool;
            if is_reversed {
                if asset_in == updated_pool.asset_b {
                    updated_pool.reserve_b = updated_pool.reserve_b.saturating_add(amount_in);
                    updated_pool.reserve_a = updated_pool.reserve_a.saturating_sub(amount_out);
                } else {
                    updated_pool.reserve_a = updated_pool.reserve_a.saturating_add(amount_in);
                    updated_pool.reserve_b = updated_pool.reserve_b.saturating_sub(amount_out);
                }
            } else {
                if asset_in == updated_pool.asset_a {
                    updated_pool.reserve_a = updated_pool.reserve_a.saturating_add(amount_in);
                    updated_pool.reserve_b = updated_pool.reserve_b.saturating_sub(amount_out);
                } else {
                    updated_pool.reserve_b = updated_pool.reserve_b.saturating_add(amount_in);
                    updated_pool.reserve_a = updated_pool.reserve_a.saturating_sub(amount_out);
                }
            }

            // Store updated pool
            let (first_asset, second_asset) = if asset_in < asset_out {
                (asset_in, asset_out)
            } else {
                (asset_out, asset_in)
            };
            Pools::<T>::insert(&first_asset, &second_asset, &updated_pool);

            // Update volume tracking
            let pool_id = PoolId::new(asset_in, asset_out);
            let current_volume = PoolVolume::<T>::get(&pool_id);
            PoolVolume::<T>::insert(&pool_id, current_volume.saturating_add(amount_in));

            // Emit event
            Self::deposit_event(Event::Swap {
                pool_id,
                trader: who,
                asset_in,
                asset_out,
                amount_in,
                amount_out,
            });

            Ok(())
        }

        /// Claim LP rewards
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::claim_rewards())]
        pub fn claim_lp_rewards(
            origin: OriginFor<T>,
            pool_id: PoolId<T::AssetId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get claimable rewards
            let reward_amount = UserRewards::<T>::get(&who, &pool_id);
            ensure!(!reward_amount.is_zero(), Error::<T>::NoRewardsToClaim);

            // Clear user rewards
            UserRewards::<T>::remove(&who, &pool_id);

            // Transfer rewards to user (this would be CHML tokens)
            // For now, we'll assume the reward token is the first asset in the pool
            // In a real implementation, this would be the native CHML token
            let reward_asset = pool_id.asset_a; // Placeholder
            let pallet_account = T::PalletId::get().into_account_truncating();
            
            T::Currency::transfer(
                reward_asset,
                &pallet_account,
                &who,
                reward_amount,
                Preservation::Expendable,
            )?;

            // Emit event
            Self::deposit_event(Event::RewardsClaimed {
                pool_id,
                claimer: who,
                amount: reward_amount,
            });

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Get pool account ID
        pub fn pool_account_id(asset_a: &T::AssetId, asset_b: &T::AssetId) -> T::AccountId {
            let (first, second) = if asset_a < asset_b {
                (asset_a, asset_b)
            } else {
                (asset_b, asset_a)
            };
            T::PalletId::get().into_sub_account_truncating((first, second))
        }

        /// Get pool with direction handling
        pub fn get_pool_with_direction(
            asset_a: &T::AssetId,
            asset_b: &T::AssetId,
        ) -> Option<(LiquidityPool<T::AssetId, T::Balance>, bool)> {
            if let Some(pool) = Pools::<T>::get(asset_a, asset_b) {
                Some((pool, false))
            } else if let Some(pool) = Pools::<T>::get(asset_b, asset_a) {
                Some((pool, true))
            } else {
                None
            }
        }

        /// Quote function for calculating optimal amounts
        pub fn quote(
            amount_a: T::Balance,
            reserve_a: T::Balance,
            reserve_b: T::Balance,
        ) -> Result<T::Balance, Error<T>> {
            ensure!(!amount_a.is_zero(), Error::<T>::AmountTooSmall);
            ensure!(!reserve_a.is_zero() && !reserve_b.is_zero(), Error::<T>::InsufficientLiquidity);
            
            // amount_b = amount_a * reserve_b / reserve_a
            let amount_b = amount_a.saturating_mul(reserve_b) / reserve_a;
            Ok(amount_b)
        }

        /// Calculate LP tokens to mint
        pub fn calculate_lp_tokens_to_mint(
            pool: &LiquidityPool<T::AssetId, T::Balance>,
            amount_a: T::Balance,
            amount_b: T::Balance,
            is_reversed: bool,
        ) -> Result<T::Balance, Error<T>> {
            if pool.total_lp_tokens.is_zero() {
                // First liquidity provision: LP tokens = sqrt(amount_a * amount_b)
                let product = amount_a.saturating_mul(amount_b);
                // Simplified sqrt for demo - in production use proper sqrt implementation
                let lp_tokens = Self::integer_sqrt(product);
                Ok(lp_tokens)
            } else {
                // Subsequent provisions: LP tokens proportional to existing ratio
                let (reserve_a, reserve_b) = if is_reversed {
                    (pool.reserve_b, pool.reserve_a)
                } else {
                    (pool.reserve_a, pool.reserve_b)
                };
                
                let lp_tokens_a = amount_a.saturating_mul(pool.total_lp_tokens) / reserve_a;
                let lp_tokens_b = amount_b.saturating_mul(pool.total_lp_tokens) / reserve_b;
                
                // Take minimum to maintain ratio
                Ok(lp_tokens_a.min(lp_tokens_b))
            }
        }

        /// Calculate amounts when removing liquidity
        pub fn calculate_remove_liquidity_amounts(
            pool: &LiquidityPool<T::AssetId, T::Balance>,
            lp_tokens: T::Balance,
            is_reversed: bool,
        ) -> Result<(T::Balance, T::Balance), Error<T>> {
            ensure!(!pool.total_lp_tokens.is_zero(), Error::<T>::DivisionByZero);
            
            let (reserve_a, reserve_b) = if is_reversed {
                (pool.reserve_b, pool.reserve_a)
            } else {
                (pool.reserve_a, pool.reserve_b)
            };
            
            let amount_a = lp_tokens.saturating_mul(reserve_a) / pool.total_lp_tokens;
            let amount_b = lp_tokens.saturating_mul(reserve_b) / pool.total_lp_tokens;
            
            Ok((amount_a, amount_b))
        }

        /// Calculate swap output using constant product formula
        pub fn calculate_swap_output(
            amount_in: T::Balance,
            pool: &LiquidityPool<T::AssetId, T::Balance>,
            asset_in: T::AssetId,
            is_reversed: bool,
        ) -> Result<T::Balance, Error<T>> {
            let (reserve_in, reserve_out) = if is_reversed {
                if asset_in == pool.asset_b {
                    (pool.reserve_b, pool.reserve_a)
                } else {
                    (pool.reserve_a, pool.reserve_b)
                }
            } else {
                if asset_in == pool.asset_a {
                    (pool.reserve_a, pool.reserve_b)
                } else {
                    (pool.reserve_b, pool.reserve_a)
                }
            };
            
            Self::get_amount_out(amount_in, reserve_in, reserve_out)
        }

        /// Get amount out with fee calculation
        pub fn get_amount_out(
            amount_in: T::Balance,
            reserve_in: T::Balance,
            reserve_out: T::Balance,
        ) -> Result<T::Balance, Error<T>> {
            ensure!(!amount_in.is_zero(), Error::<T>::AmountTooSmall);
            ensure!(!reserve_in.is_zero() && !reserve_out.is_zero(), Error::<T>::InsufficientLiquidity);
            
            // Apply fee (0.25% = 2.5/1000 = 997/1000 after fee)
            let amount_in_with_fee = amount_in.saturating_mul(997u128.into());
            let numerator = amount_in_with_fee.saturating_mul(reserve_out);
            let denominator = reserve_in.saturating_mul(1000u128.into()).saturating_add(amount_in_with_fee);
            
            ensure!(!denominator.is_zero(), Error::<T>::DivisionByZero);
            Ok(numerator / denominator)
        }

        /// Simple integer square root implementation
        pub fn integer_sqrt(n: T::Balance) -> T::Balance {
            if n.is_zero() {
                return n;
            }
            
            let mut x = n;
            let mut y = (n + 1u128.into()) / 2u128.into();
            
            while y < x {
                x = y;
                y = (x + n / x) / 2u128.into();
            }
            
            x
        }
    }
}
