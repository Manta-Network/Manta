// Copyright 2020-2023 Manta Network.
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
//
//! # Native Barrier Pallet

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

use codec::{Codec, MaxEncodedLen};
use core::time::Duration;
#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use frame_support::{ensure, pallet_prelude::DispatchResult, traits::UnixTime};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Zero},
    FixedPointOperand, Saturating,
};
use sp_std::{fmt::Debug, prelude::*};
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The balance of an account.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo
            + FixedPointOperand;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Timestamp provider
        type UnixTime: UnixTime;
    }

    /// The current storage version.
    const STORAGE_VERSION: frame_support::traits::StorageVersion =
        frame_support::traits::StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NativeBarrierInitialized {
            init: Option<(T::Balance, Duration)>,
        },
        AccountsAddedToBarrier {
            accounts: Vec<T::AccountId>,
        },
        AccountsRemovedFromBarrier {
            accounts: Vec<T::AccountId>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        NativeBarrierLimitExceeded,
        NativeBarrierNotInitialized,
    }

    /// Stores daily limit value
    #[pallet::storage]
    #[pallet::getter(fn get_configurations)]
    pub type Configurations<T: Config> = StorageValue<_, (T::Balance, Duration), OptionQuery>;

    /// Stores remaining limit for each account. Skipped days are accumulated.
    #[pallet::storage]
    #[pallet::getter(fn get_remaining_limit)]
    pub type RemainingLimit<T: Config> =
        StorageMap<_, Identity, T::AccountId, T::Balance, OptionQuery>;

    /// Caches the last processed day, used to check for start of new days
    #[pallet::storage]
    #[pallet::getter(fn get_last_day_processed)]
    pub type LastDayProcessed<T: Config> = StorageValue<_, u64, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub barrier_accounts: Vec<T::AccountId>,
        pub daily_limit: T::Balance,
        pub start_unix_time: Duration,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                barrier_accounts: vec![],
                daily_limit: T::Balance::zero(),
                start_unix_time: Duration::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Configurations::<T>::set(Some((self.daily_limit, self.start_unix_time)));
            for account_id in self.barrier_accounts.iter() {
                RemainingLimit::<T>::insert(account_id, T::Balance::zero());
            }
        }
    }

    #[cfg(feature = "std")]
    impl<T: Config> GenesisConfig<T> {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
            <Self as GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
            <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            if let Some((_, start_unix_time)) = Configurations::<T>::get() {
                let now = T::UnixTime::now();
                if start_unix_time <= now {
                    let days_since_start =
                        (now.as_secs() - start_unix_time.as_secs()) / (24 * 60 * 60);

                    if let Some(last_day_processed) = <LastDayProcessed<T>>::get() {
                        if days_since_start > last_day_processed {
                            Self::reset_remaining_limit(days_since_start - last_day_processed);
                            <LastDayProcessed<T>>::put(days_since_start);
                        }
                    } else {
                        <LastDayProcessed<T>>::put(0);
                    }
                }
            }

            T::WeightInfo::on_initialize()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets the start unix time and daily limit for the barrier logic.
        /// Can be in the past or the future. Can be disabled by setting to None.      
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::initialize_native_barrier())]
        pub fn initialize_native_barrier(
            origin: OriginFor<T>,
            init: Option<(T::Balance, Duration)>,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <Configurations<T>>::set(init);
            if <LastDayProcessed<T>>::get().is_none() {
                <LastDayProcessed<T>>::put(0);
            }

            if init != None {
                for account_id in accounts.iter() {
                    if !RemainingLimit::<T>::contains_key(account_id) {
                        RemainingLimit::<T>::insert(account_id, T::Balance::zero());
                    }
                }

                Self::deposit_event(Event::AccountsAddedToBarrier { accounts });
            }

            Self::deposit_event(Event::NativeBarrierInitialized { init });
            Ok(())
        }

        /// Add `accounts` to barrier to make them have limited native transfers
        /// Sets the <accounts> in the RemainingLimits storage item,
        /// and sets their limit to the amount of one daily limit. Can be used multiple times.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::add_accounts_to_native_barrier())]
        pub fn add_accounts_to_native_barrier(
            origin: OriginFor<T>,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            if Configurations::<T>::get().is_some() {
                for account_id in accounts.iter() {
                    if !RemainingLimit::<T>::contains_key(account_id) {
                        RemainingLimit::<T>::insert(account_id, T::Balance::zero());
                    }
                }

                Self::deposit_event(Event::AccountsAddedToBarrier { accounts });
            } else {
                return Err(Error::<T>::NativeBarrierNotInitialized.into());
            }

            Ok(().into())
        }

        /// Remove `accounts` from the barrier's RemainingLimit storage
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::remove_accounts_from_native_barrier())]
        pub fn remove_accounts_from_native_barrier(
            origin: OriginFor<T>,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            for account_id in accounts.iter() {
                RemainingLimit::<T>::remove(account_id);
            }

            Self::deposit_event(Event::AccountsRemovedFromBarrier { accounts });

            Ok(().into())
        }
    }
}

impl<T: Config> orml_traits::native_barrier::NativeBarrier<T::AccountId, T::Balance> for Pallet<T> {
    fn ensure_limit_not_exceeded(account_id: &T::AccountId, amount: T::Balance) -> DispatchResult {
        if let Some((_, start_unix_time)) = <Configurations<T>>::get() {
            let now = T::UnixTime::now();
            if start_unix_time <= now {
                if let Some(remaining_limit) = RemainingLimit::<T>::get(account_id) {
                    ensure!(
                        amount <= remaining_limit,
                        Error::<T>::NativeBarrierLimitExceeded
                    );

                    // If the ensure didn't return an error, update the native transfers
                    Self::update_native_barrier(account_id, amount);
                }
            }
        }

        Ok(())
    }

    fn update_native_barrier(account_id: &T::AccountId, amount: T::Balance) {
        <RemainingLimit<T>>::mutate_exists(account_id, |maybe_remainder| match maybe_remainder {
            Some(remainder) => {
                *remainder = remainder.saturating_sub(amount);
            }
            None => {}
        });
    }
}

impl<T: Config> Pallet<T> {
    fn reset_remaining_limit(unprocessed_days: u64) {
        if let Some((daily_limit, _)) = <Configurations<T>>::get() {
            for (account_id, mut remaining_limit) in RemainingLimit::<T>::iter() {
                for _ in 0..unprocessed_days {
                    remaining_limit += daily_limit;
                }
                <RemainingLimit<T>>::insert(&account_id, remaining_limit);
            }
        }
    }
}
