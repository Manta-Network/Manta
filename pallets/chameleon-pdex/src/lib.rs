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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{Get, StorageVersion},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};

// Re-export pallet items
pub use pallet::*;

// Import modules
mod types;
mod amm;
mod rewards;

pub use types::*;
pub use amm::*;
pub use rewards::*;

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

        /// Asset ID type
        type AssetId: Parameter + Copy + Ord + Default + MaxEncodedLen;

        /// Balance type
        type Balance: Parameter + Copy + Ord + Zero + Saturating + From<u128> + MaxEncodedLen + Default +
                     sp_std::ops::Div<Output = Self::Balance> +
                     sp_std::ops::Mul<Output = Self::Balance> +
                     sp_std::ops::Add<Output = Self::Balance> +
                     sp_std::ops::Sub<Output = Self::Balance> +
                     PartialOrd;

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

        /// Currency trait for asset operations
        type Currency: frame_support::traits::fungibles::Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::Balance> +
                      frame_support::traits::fungibles::Mutate<Self::AccountId> +
                      frame_support::traits::fungibles::Transfer<Self::AccountId>;
    }

    /// Liquidity pools storage
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
    #[pallet::storage]
    #[pallet::getter(fn pool_rewards)]
    pub type PoolRewards<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PoolId<T::AssetId>,
        RewardInfo<T::Balance>,
        OptionQuery,
    >;

    /// User claimable rewards
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

    /// Pool creation nonce for unique pool IDs
    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub type NextPoolId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Pool volume tracking for rewards calculation
    #[pallet::storage]
    #[pallet::getter(fn pool_volume)]
    pub type PoolVolume<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PoolId<T::AssetId>,
        T::Balance,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Pool created
        PoolCreated {
            pool_id: PoolId<T::AssetId>,
            asset_a: T::AssetId,
            asset_b: T::AssetId,
            creator: T::AccountId,
        },
        /// Liquidity added to pool
        LiquidityAdded {
            pool_id: PoolId<T::AssetId>,
            provider: T::AccountId,
            amount_a: T::Balance,
            amount_b: T::Balance,
            lp_tokens_minted: T::Balance,
        },
        /// Liquidity removed from pool
        LiquidityRemoved {
            pool_id: PoolId<T::AssetId>,
            provider: T::AccountId,
            amount_a: T::Balance,
            amount_b: T::Balance,
            lp_tokens_burned: T::Balance,
        },
        /// Swap executed
        SwapExecuted {
            pool_id: PoolId<T::AssetId>,
            trader: T::AccountId,
            asset_in: T::AssetId,
            amount_in: T::Balance,
            asset_out: T::AssetId,
            amount_out: T::Balance,
            fee_paid: T::Balance,
        },
        /// Rewards claimed
        RewardsClaimed {
            pool_id: PoolId<T::AssetId>,
            claimer: T::AccountId,
            amount: T::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool already exists
        PoolAlreadyExists,
        /// Pool does not exist
        PoolNotFound,
        /// Invalid asset pair (same asset)
        InvalidAssetPair,
        /// Pool creation limit reached
        TooManyPools,
        /// Insufficient liquidity in pool
        InsufficientLiquidity,
        /// Slippage tolerance exceeded
        SlippageExceeded,
        /// Insufficient balance
        InsufficientBalance,
        /// Invalid amount (zero or too small)
        InvalidAmount,
        /// Mathematical overflow
        Overflow,
        /// LP position not found
        LpPositionNotFound,
        /// No rewards to claim
        NoRewardsToClaim,
        /// Invalid path for multi-hop swap
        InvalidPath,
        /// Pool is empty (no reserves)
        EmptyPool,
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
            PoolRewards::<T>::insert(&pool_id, RewardInfo::new());

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

        /// Add liquidity to an existing pool
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

            // Ensure amounts are valid
            ensure!(!amount_a_desired.is_zero() && !amount_b_desired.is_zero(), Error::<T>::InvalidAmount);

            // Get pool (ensure consistent ordering)
            let (first_asset, second_asset) = if asset_a < asset_b {
                (asset_a, asset_b)
            } else {
                (asset_b, asset_a)
            };

            let mut pool = Pools::<T>::get(&first_asset, &second_asset)
                .ok_or(Error::<T>::PoolNotFound)?;

            // Calculate optimal amounts and LP tokens
            let (amount_a, amount_b, lp_tokens) = if pool.is_empty() {
                // First liquidity provision
                let lp_tokens = calculate_lp_tokens_mint(
                    amount_a_desired,
                    amount_b_desired,
                    T::Balance::zero(),
                    T::Balance::zero(),
                    T::Balance::zero(),
                ).map_err(|_| Error::<T>::InvalidAmount)?;

                (amount_a_desired, amount_b_desired, lp_tokens)
            } else {
                // Subsequent liquidity provision - maintain ratio
                let amount_b_optimal = quote(amount_a_desired, pool.reserve_a, pool.reserve_b)
                    .map_err(|_| Error::<T>::InvalidAmount)?;

                let (final_a, final_b) = if amount_b_optimal <= amount_b_desired {
                    (amount_a_desired, amount_b_optimal)
                } else {
                    let amount_a_optimal = quote(amount_b_desired, pool.reserve_b, pool.reserve_a)
                        .map_err(|_| Error::<T>::InvalidAmount)?;
                    (amount_a_optimal, amount_b_desired)
                };

                // Check slippage protection
                ensure!(final_a >= amount_a_min, Error::<T>::SlippageExceeded);
                ensure!(final_b >= amount_b_min, Error::<T>::SlippageExceeded);

                let lp_tokens = calculate_lp_tokens_mint(
                    final_a,
                    final_b,
                    pool.reserve_a,
                    pool.reserve_b,
                    pool.total_lp_tokens,
                ).map_err(|_| Error::<T>::InvalidAmount)?;

                (final_a, final_b, lp_tokens)
            };

            // Ensure minimum liquidity
            ensure!(lp_tokens >= T::MinimumLiquidity::get(), Error::<T>::InvalidAmount);

            // Transfer tokens from user to pool account
            let pool_account = Self::pool_account_id(&first_asset, &second_asset);
            
            T::Currency::transfer(asset_a, &who, &pool_account, amount_a, false)
                .map_err(|_| Error::<T>::InsufficientBalance)?;
            T::Currency::transfer(asset_b, &who, &pool_account, amount_b, false)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // Update pool reserves
            pool.reserve_a = pool.reserve_a.saturating_add(amount_a);
            pool.reserve_b = pool.reserve_b.saturating_add(amount_b);
            pool.total_lp_tokens = pool.total_lp_tokens.saturating_add(lp_tokens);

            // Update or create LP position
            let pool_id = PoolId::new(asset_a, asset_b);
            let mut position = LpPositions::<T>::get(&who, &pool_id)
                .unwrap_or_else(|| LpPosition::new(who.clone(), PoolId::new(first_asset, second_asset)));
            
            position.lp_tokens = position.lp_tokens.saturating_add(lp_tokens);
            position.deposited_at = frame_system::Pallet::<T>::block_number().saturated_into();

            // Store updates
            Pools::<T>::insert(&first_asset, &second_asset, &pool);
            LpPositions::<T>::insert(&who, &pool_id, &position);

            // Emit event
            Self::deposit_event(Event::LiquidityAdded {
                pool_id,
                provider: who,
                amount_a,
                amount_b,
                lp_tokens_minted: lp_tokens,
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

            // Ensure valid amount
            ensure!(!lp_tokens.is_zero(), Error::<T>::InvalidAmount);

            // Get pool
            let (first_asset, second_asset) = if asset_a < asset_b {
                (asset_a, asset_b)
            } else {
                (asset_b, asset_a)
            };

            let mut pool = Pools::<T>::get(&first_asset, &second_asset)
                .ok_or(Error::<T>::PoolNotFound)?;

            // Check LP position
            let pool_id = PoolId::new(asset_a, asset_b);
            let mut position = LpPositions::<T>::get(&who, &pool_id)
                .ok_or(Error::<T>::LpPositionNotFound)?;

            ensure!(position.lp_tokens >= lp_tokens, Error::<T>::InsufficientBalance);

            // Calculate amounts to return
            let (amount_a, amount_b) = calculate_lp_tokens_burn(
                lp_tokens,
                pool.reserve_a,
                pool.reserve_b,
                pool.total_lp_tokens,
            ).map_err(|_| Error::<T>::InvalidAmount)?;

            // Check slippage protection
            ensure!(amount_a >= amount_a_min, Error::<T>::SlippageExceeded);
            ensure!(amount_b >= amount_b_min, Error::<T>::SlippageExceeded);

            // Transfer tokens back to user
            let pool_account = Self::pool_account_id(&first_asset, &second_asset);
            
            T::Currency::transfer(asset_a, &pool_account, &who, amount_a, false)
                .map_err(|_| Error::<T>::InsufficientLiquidity)?;
            T::Currency::transfer(asset_b, &pool_account, &who, amount_b, false)
                .map_err(|_| Error::<T>::InsufficientLiquidity)?;

            // Update pool reserves
            pool.reserve_a = pool.reserve_a.saturating_sub(amount_a);
            pool.reserve_b = pool.reserve_b.saturating_sub(amount_b);
            pool.total_lp_tokens = pool.total_lp_tokens.saturating_sub(lp_tokens);

            // Update LP position
            position.lp_tokens = position.lp_tokens.saturating_sub(lp_tokens);

            // Store updates
            Pools::<T>::insert(&first_asset, &second_asset, &pool);
            
            if position.lp_tokens.is_zero() {
                LpPositions::<T>::remove(&who, &pool_id);
            } else {
                LpPositions::<T>::insert(&who, &pool_id, &position);
            }

            // Emit event
            Self::deposit_event(Event::LiquidityRemoved {
                pool_id,
                provider: who,
                amount_a,
                amount_b,
                lp_tokens_burned: lp_tokens,
            });

            Ok(())
        }

        /// Execute a swap between two assets
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap_exact_tokens_for_tokens(
            origin: OriginFor<T>,
            amount_in: T::Balance,
            amount_out_min: T::Balance,
            path: sp_std::vec::Vec<T::AssetId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate path
            ensure!(path.len() >= 2, Error::<T>::InvalidPath);
            ensure!(!amount_in.is_zero(), Error::<T>::InvalidAmount);

            // For now, only support direct swaps (path length = 2)
            ensure!(path.len() == 2, Error::<T>::InvalidPath);

            let asset_in = path[0];
            let asset_out = path[1];

            // Get pool
            let (first_asset, second_asset) = if asset_in < asset_out {
                (asset_in, asset_out)
            } else {
                (asset_out, asset_in)
            };

            let mut pool = Pools::<T>::get(&first_asset, &second_asset)
                .ok_or(Error::<T>::PoolNotFound)?;

            // Ensure pool has liquidity
            ensure!(!pool.is_empty(), Error::<T>::EmptyPool);

            // Calculate swap output
            let (input_reserve, output_reserve) = if asset_in == pool.asset_a {
                (pool.reserve_a, pool.reserve_b)
            } else {
                (pool.reserve_b, pool.reserve_a)
            };

            let amount_out = calculate_swap_output(
                amount_in,
                input_reserve,
                output_reserve,
                pool.fee_bps,
            ).map_err(|_| Error::<T>::InsufficientLiquidity)?;

            // Check slippage protection
            ensure!(amount_out >= amount_out_min, Error::<T>::SlippageExceeded);

            // Calculate and distribute fees
            let fee_amount = calculate_swap_fee(amount_in, pool.fee_bps);
            let (lp_fee, treasury_fee) = distribute_swap_fee(fee_amount);

            // Transfer tokens
            let pool_account = Self::pool_account_id(&first_asset, &second_asset);
            
            T::Currency::transfer(asset_in, &who, &pool_account, amount_in, false)
                .map_err(|_| Error::<T>::InsufficientBalance)?;
            T::Currency::transfer(asset_out, &pool_account, &who, amount_out, false)
                .map_err(|_| Error::<T>::InsufficientLiquidity)?;

            // Update pool reserves (including LP fees)
            if asset_in == pool.asset_a {
                pool.reserve_a = pool.reserve_a.saturating_add(amount_in.saturating_sub(treasury_fee));
                pool.reserve_b = pool.reserve_b.saturating_sub(amount_out);
            } else {
                pool.reserve_b = pool.reserve_b.saturating_add(amount_in.saturating_sub(treasury_fee));
                pool.reserve_a = pool.reserve_a.saturating_sub(amount_out);
            }

            // Store updated pool
            Pools::<T>::insert(&first_asset, &second_asset, &pool);

            // Update volume tracking for rewards
            let pool_id = PoolId::new(asset_in, asset_out);
            let current_volume = PoolVolume::<T>::get(&pool_id);
            PoolVolume::<T>::insert(&pool_id, current_volume.saturating_add(amount_in));

            // TODO: Transfer treasury fee to treasury account
            // For now, treasury fee stays in pool (will be implemented with treasury integration)

            // Emit event
            let pool_id = PoolId::new(asset_in, asset_out);
            Self::deposit_event(Event::SwapExecuted {
                pool_id,
                trader: who,
                asset_in,
                amount_in,
                asset_out,
                amount_out,
                fee_paid: fee_amount,
            });

            Ok(())
        }

        /// Claim accumulated LP rewards
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::claim_rewards())]
        pub fn claim_rewards(
            origin: OriginFor<T>,
            pool_id: PoolId<T::AssetId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if user has rewards to claim
            let reward_amount = UserRewards::<T>::get(&who, &pool_id);
            ensure!(!reward_amount.is_zero(), Error::<T>::NoRewardsToClaim);

            // Remove claimed rewards
            UserRewards::<T>::remove(&who, &pool_id);

            // TODO: Transfer CHML rewards to user
            // For now, this is a placeholder - will be implemented with CHML token integration

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
    }
}
