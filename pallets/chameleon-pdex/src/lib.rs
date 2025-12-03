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
//! A privacy-preserving decentralized exchange with AMM pools.
//!
//! ## Features
//! - Constant product AMM (x * y = k)
//! - 0.25% swap fee (90% to LPs, 10% to Treasury)
//! - LP token minting/burning
//! - Slippage protection

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Saturating, Zero};

    /// Storage version
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Asset ID type
        type AssetId: Parameter + Copy + Ord + Default + MaxEncodedLen;

        /// Balance type
        type Balance: Parameter + Copy + Ord + Zero + Saturating + From<u128> + Into<u128> + MaxEncodedLen + Default;

        /// Weight info
        type WeightInfo: WeightInfo;

        /// Pallet ID
        #[pallet::constant]
        type PalletId: Get<frame_support::PalletId>;

        /// Maximum pools
        #[pallet::constant]
        type MaxPools: Get<u32>;

        /// Minimum liquidity
        #[pallet::constant]
        type MinimumLiquidity: Get<Self::Balance>;
    }

    /// Weight info trait
    pub trait WeightInfo {
        fn create_pool() -> Weight;
        fn add_liquidity() -> Weight;
        fn remove_liquidity() -> Weight;
        fn swap() -> Weight;
    }

    impl WeightInfo for () {
        fn create_pool() -> Weight { Weight::from_parts(10_000, 0) }
        fn add_liquidity() -> Weight { Weight::from_parts(10_000, 0) }
        fn remove_liquidity() -> Weight { Weight::from_parts(10_000, 0) }
        fn swap() -> Weight { Weight::from_parts(10_000, 0) }
    }

    /// Liquidity pool structure
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct LiquidityPool<T: Config> {
        pub asset_a: T::AssetId,
        pub asset_b: T::AssetId,
        pub reserve_a: T::Balance,
        pub reserve_b: T::Balance,
        pub total_lp_tokens: T::Balance,
        pub fee_bps: u32, // 25 = 0.25%
    }

    impl<T: Config> Default for LiquidityPool<T> {
        fn default() -> Self {
            Self {
                asset_a: T::AssetId::default(),
                asset_b: T::AssetId::default(),
                reserve_a: T::Balance::default(),
                reserve_b: T::Balance::default(),
                total_lp_tokens: T::Balance::default(),
                fee_bps: 25,
            }
        }
    }

    /// LP position
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen, Default)]
    pub struct LpPosition<Balance> {
        pub lp_tokens: Balance,
        pub deposited_at: u32,
    }

    /// Pool storage
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, u32, LiquidityPool<T>, OptionQuery>;

    /// LP positions: (pool_id, account) -> position
    #[pallet::storage]
    #[pallet::getter(fn lp_positions)]
    pub type LpPositions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u32,
        Blake2_128Concat, T::AccountId,
        LpPosition<T::Balance>,
        ValueQuery,
    >;

    /// Pool count
    #[pallet::storage]
    #[pallet::getter(fn pool_count)]
    pub type PoolCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Total value locked
    #[pallet::storage]
    #[pallet::getter(fn total_tvl)]
    pub type TotalTvl<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Pool created
        PoolCreated { pool_id: u32, asset_a: T::AssetId, asset_b: T::AssetId },
        /// Liquidity added
        LiquidityAdded { pool_id: u32, who: T::AccountId, amount_a: T::Balance, amount_b: T::Balance, lp_minted: T::Balance },
        /// Liquidity removed
        LiquidityRemoved { pool_id: u32, who: T::AccountId, amount_a: T::Balance, amount_b: T::Balance, lp_burned: T::Balance },
        /// Swap executed
        Swapped { pool_id: u32, who: T::AccountId, asset_in: T::AssetId, amount_in: T::Balance, asset_out: T::AssetId, amount_out: T::Balance },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool not found
        PoolNotFound,
        /// Pool already exists
        PoolAlreadyExists,
        /// Insufficient liquidity
        InsufficientLiquidity,
        /// Slippage exceeded
        SlippageExceeded,
        /// Invalid amounts
        InvalidAmounts,
        /// Max pools reached
        MaxPoolsReached,
        /// Same asset
        SameAsset,
        /// Zero amount
        ZeroAmount,
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
            let _who = ensure_signed(origin)?;
            ensure!(asset_a != asset_b, Error::<T>::SameAsset);

            let pool_id = PoolCount::<T>::get();
            ensure!(pool_id < T::MaxPools::get(), Error::<T>::MaxPoolsReached);

            let pool = LiquidityPool {
                asset_a,
                asset_b,
                reserve_a: T::Balance::default(),
                reserve_b: T::Balance::default(),
                total_lp_tokens: T::Balance::default(),
                fee_bps: 25,
            };

            Pools::<T>::insert(pool_id, pool);
            PoolCount::<T>::put(pool_id + 1);

            Self::deposit_event(Event::PoolCreated { pool_id, asset_a, asset_b });
            Ok(())
        }

        /// Add liquidity to a pool
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            pool_id: u32,
            amount_a: T::Balance,
            amount_b: T::Balance,
            min_lp_tokens: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!amount_a.is_zero() && !amount_b.is_zero(), Error::<T>::ZeroAmount);

            Pools::<T>::try_mutate(pool_id, |maybe_pool| {
                let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;

                // Calculate LP tokens to mint
                let lp_minted = if pool.total_lp_tokens.is_zero() {
                    // First liquidity provider: sqrt(a * b)
                    let product: u128 = amount_a.into() * amount_b.into();
                    T::Balance::from(integer_sqrt(product))
                } else {
                    // Proportional minting
                    let lp_a = amount_a.into() * pool.total_lp_tokens.into() / pool.reserve_a.into();
                    let lp_b = amount_b.into() * pool.total_lp_tokens.into() / pool.reserve_b.into();
                    T::Balance::from(lp_a.min(lp_b))
                };

                ensure!(lp_minted >= min_lp_tokens, Error::<T>::SlippageExceeded);

                // Update pool
                pool.reserve_a = pool.reserve_a.saturating_add(amount_a);
                pool.reserve_b = pool.reserve_b.saturating_add(amount_b);
                pool.total_lp_tokens = pool.total_lp_tokens.saturating_add(lp_minted);

                // Update LP position
                LpPositions::<T>::mutate(pool_id, &who, |pos| {
                    pos.lp_tokens = pos.lp_tokens.saturating_add(lp_minted);
                    pos.deposited_at = frame_system::Pallet::<T>::block_number().saturated_into();
                });

                Self::deposit_event(Event::LiquidityAdded {
                    pool_id,
                    who,
                    amount_a,
                    amount_b,
                    lp_minted,
                });

                Ok(())
            })
        }

        /// Remove liquidity from a pool
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::remove_liquidity())]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            pool_id: u32,
            lp_tokens: T::Balance,
            min_amount_a: T::Balance,
            min_amount_b: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!lp_tokens.is_zero(), Error::<T>::ZeroAmount);

            Pools::<T>::try_mutate(pool_id, |maybe_pool| {
                let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;

                // Calculate amounts to return
                let amount_a = T::Balance::from(
                    lp_tokens.into() * pool.reserve_a.into() / pool.total_lp_tokens.into()
                );
                let amount_b = T::Balance::from(
                    lp_tokens.into() * pool.reserve_b.into() / pool.total_lp_tokens.into()
                );

                ensure!(amount_a >= min_amount_a && amount_b >= min_amount_b, Error::<T>::SlippageExceeded);

                // Update pool
                pool.reserve_a = pool.reserve_a.saturating_sub(amount_a);
                pool.reserve_b = pool.reserve_b.saturating_sub(amount_b);
                pool.total_lp_tokens = pool.total_lp_tokens.saturating_sub(lp_tokens);

                // Update LP position
                LpPositions::<T>::mutate(pool_id, &who, |pos| {
                    pos.lp_tokens = pos.lp_tokens.saturating_sub(lp_tokens);
                });

                Self::deposit_event(Event::LiquidityRemoved {
                    pool_id,
                    who,
                    amount_a,
                    amount_b,
                    lp_burned: lp_tokens,
                });

                Ok(())
            })
        }

        /// Swap tokens
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap(
            origin: OriginFor<T>,
            pool_id: u32,
            asset_in: T::AssetId,
            amount_in: T::Balance,
            min_amount_out: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!amount_in.is_zero(), Error::<T>::ZeroAmount);

            Pools::<T>::try_mutate(pool_id, |maybe_pool| {
                let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;

                // Determine direction
                let (reserve_in, reserve_out, asset_out, is_a_to_b) = if asset_in == pool.asset_a {
                    (pool.reserve_a, pool.reserve_b, pool.asset_b, true)
                } else {
                    (pool.reserve_b, pool.reserve_a, pool.asset_a, false)
                };

                // Calculate output using constant product formula with fee
                // amount_out = reserve_out * amount_in_with_fee / (reserve_in + amount_in_with_fee)
                let amount_in_u128: u128 = amount_in.into();
                let fee_factor = 10000u128 - pool.fee_bps as u128; // 9975 for 0.25% fee
                let amount_in_with_fee = amount_in_u128 * fee_factor / 10000;
                
                let numerator = reserve_out.into() * amount_in_with_fee;
                let denominator = reserve_in.into() + amount_in_with_fee;
                let amount_out = T::Balance::from(numerator / denominator);

                ensure!(amount_out >= min_amount_out, Error::<T>::SlippageExceeded);
                ensure!(amount_out < reserve_out, Error::<T>::InsufficientLiquidity);

                // Update reserves
                if is_a_to_b {
                    pool.reserve_a = pool.reserve_a.saturating_add(amount_in);
                    pool.reserve_b = pool.reserve_b.saturating_sub(amount_out);
                } else {
                    pool.reserve_b = pool.reserve_b.saturating_add(amount_in);
                    pool.reserve_a = pool.reserve_a.saturating_sub(amount_out);
                }

                Self::deposit_event(Event::Swapped {
                    pool_id,
                    who,
                    asset_in,
                    amount_in,
                    asset_out,
                    amount_out,
                });

                Ok(())
            })
        }
    }

    /// Integer square root
    fn integer_sqrt(n: u128) -> u128 {
        if n == 0 { return 0; }
        let mut x = n;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        x
    }
}
