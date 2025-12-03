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
