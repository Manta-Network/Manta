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

pub mod weights;

use codec::{Codec, MaxEncodedLen};
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
        frame_support::traits::StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<(T)>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        ///   #</weight>
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_start_unix_time())]
        pub fn set_start_unix_time(
            origin: OriginFor<T>,
            start_unix_time: core::time::Duration,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <StartUnixTime<T>>::set(Some(start_unix_time));
            Self::deposit_event(Event::StartUnixTime { start_unix_time });
            Ok(())
        }

        ///   #</weight>
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_daily_xcm_limit())]
        pub fn set_daily_xcm_limit(
            origin: OriginFor<T>,
            daily_xcm_limit: T::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <DailyXcmLimit<T>>::set(Some(daily_xcm_limit));
            Self::deposit_event(Event::DailyLimitSet {
                daily_limit: daily_xcm_limit,
            });
            Ok(())
        }

        /// Add `accounts` to barrier to make them have limited xcm native transfers
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::add_accounts_to_native_barrier())]
        pub fn add_accounts_to_native_barrier(
            origin: OriginFor<T>,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            // TODO: unwrap()
            let start_time = <StartUnixTime<T>>::get().unwrap().as_secs();
            for account_id in accounts {
                if !XcmNativeTransfers::<T>::contains_key(&account_id) {
                    XcmNativeTransfers::<T>::insert(&account_id, (T::Balance::zero(), start_time));
                }

                if !RemainingXcmLimit::<T>::contains_key(&account_id) {
                    RemainingXcmLimit::<T>::insert(account_id, T::Balance::zero());
                }
            }

            Self::deposit_event(Event::BarrierListUpdated);

            Ok(().into())
        }

        /// Remove `accounts` from barrier
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::remove_accounts_to_native_barrier())]
        pub fn remove_accounts_to_native_barrier(
            origin: OriginFor<T>,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            for account_id in accounts {
                XcmNativeTransfers::<T>::remove(account_id);
            }

            Self::deposit_event(Event::BarrierListUpdated);

            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// TODO: docs
        StartUnixTime {
            start_unix_time: core::time::Duration,
        },
        /// TODO: docs
        DailyLimitSet { daily_limit: T::Balance },
        /// TODO: docs
        BarrierListUpdated,
    }

    #[pallet::error]
    pub enum Error<T> {
        ///
        XcmTransfersLimitExceeded,
        ///
        XcmTransfersNotAllowedForAccount,
    }
    /// Stores amount of native asset XCM transfers and timestamp of last transfer
    #[pallet::storage]
    pub type XcmNativeTransfers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (T::Balance, u64), OptionQuery>;

    /// Stores limit value
    #[pallet::storage]
    pub type DailyXcmLimit<T: Config> = StorageValue<_, T::Balance, OptionQuery>;

    #[pallet::storage]
    pub type RemainingXcmLimit<T: Config> =
        StorageMap<_, Identity, T::AccountId, T::Balance, OptionQuery>;

    /// Stores limit value
    #[pallet::storage]
    pub type LastDayProcessed<T: Config> = StorageValue<_, u64, OptionQuery>;

    /// Stores limit value
    #[pallet::storage]
    pub type StartUnixTime<T: Config> = StorageValue<_, core::time::Duration, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        // TODO: real genesis
        pub balances: Vec<(T::AccountId, T::Balance)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                balances: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            if let Some(start_unix_time) = StartUnixTime::<T>::get() {
                let now = T::UnixTime::now();
                if start_unix_time <= now {
                    let days_since_start =
                        (now.as_secs() - start_unix_time.as_secs()) / (24 * 60 * 60);

                    // Default is ok, it would only be used the first time
                    let last_day_processed = <LastDayProcessed<T>>::get().unwrap_or(0);

                    if days_since_start > last_day_processed {
                        Self::reset_remaining_xcm_limit();
                        <LastDayProcessed<T>>::put(days_since_start);
                    }
                }
            }

            //T::WeightInfo::on_initialize()
            Weight::from_ref_time(0) // TODO: use the commented out line
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

const XCM_LIMIT_PERIOD_IN_SEC: u64 = 86400; // 1 day

impl<T: Config> orml_traits::xcm_transfer::NativeBarrier<T::AccountId, T::Balance> for Pallet<T> {
    fn ensure_xcm_transfer_limit_not_exceeded(
        account_id: &T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult {
        if let Some(transfer_limit) = DailyXcmLimit::<T>::get() {
            // The address is not in the barrier list, so we don't care about it
            if <XcmNativeTransfers<T>>::get(account_id).is_none() {
                return Ok(());
            }

            let now = T::UnixTime::now().as_secs();
            let current_period = (now / XCM_LIMIT_PERIOD_IN_SEC) * XCM_LIMIT_PERIOD_IN_SEC;
            let (mut transferred, last_transfer) = XcmNativeTransfers::<T>::get(account_id)
                .ok_or(Error::<T>::XcmTransfersNotAllowedForAccount)?;
            let remaining_limit = RemainingXcmLimit::<T>::get(account_id).unwrap_or(transfer_limit);

            if last_transfer < current_period {
                transferred = T::Balance::zero();
                XcmNativeTransfers::<T>::insert(account_id, (transferred, now));
            };

            ensure!(
                transferred + amount <= remaining_limit,
                Error::<T>::XcmTransfersLimitExceeded
            );

            // If the ensure didn't return an error, update the native transfers
            Self::update_xcm_native_transfers(account_id, amount);

            // TODO: maybe add event here that the transfers were updated
        }

        Ok(())
    }

    fn update_xcm_native_transfers(account_id: &T::AccountId, amount: T::Balance) {
        <XcmNativeTransfers<T>>::mutate_exists(account_id, |maybe_transfer| match maybe_transfer {
            Some((current_amount, last_transfer)) => {
                *current_amount = *current_amount + amount;
                *last_transfer = T::UnixTime::now().as_secs();
            }
            None => {}
        });

        <RemainingXcmLimit<T>>::mutate_exists(
            account_id,
            |maybe_remainder| match maybe_remainder {
                Some(remainder) => {
                    *remainder = remainder.saturating_sub(amount);
                }
                None => {}
            },
        );
    }
}

impl<T: Config> Pallet<T> {
    fn reset_remaining_xcm_limit() {
        if let Some(daily_limit) = <DailyXcmLimit<T>>::get() {
            for (account_id, _) in XcmNativeTransfers::<T>::iter() {
                let mut remaining_limit =
                    <RemainingXcmLimit<T>>::get(&account_id).unwrap_or(daily_limit);
                remaining_limit += daily_limit;
                <RemainingXcmLimit<T>>::insert(&account_id, remaining_limit);
            }
        }
    }
}
