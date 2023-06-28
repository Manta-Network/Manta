// Copyright 2020-2022 Manta Network.
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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency},
    transactional,
};
use frame_system::pallet_prelude::*;
use safe_regex::{regex, Matcher0};
use sp_runtime::{
    traits::{AccountIdConversion, Hash, Saturating},
    DispatchResult,
};
use sp_std::vec::Vec;

mod mock;
mod tests;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

pub type ZkAddressType = [u8; 32];

pub type UserName = Vec<u8>;

pub const NAME_MAX_LEN: usize = 64;
pub const NAME_MIN_LEN: usize = 3;

/// Type alias for currency balance.
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{traits::StorageVersion, PalletId};

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Pallet ID
        type PalletId: Get<PalletId>;

        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type RegisterWaitingPeriod: Get<Self::BlockNumber>;

        #[pallet::constant]
        type RegisterPrice: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Username exists
        NameAlreadyRegistered,
        /// Username not registered
        NotRegistered,
        /// Username not owned
        NotOwned,
        /// The Registration time not reached
        RegisterTimeNotReached,
        /// Username invalid
        InvalidUsernameFormat,
        /// Already pending Register
        AlreadyPendingRegister,
        /// Not Found (used in cases of canceling)
        UsernameNotFound,
        /// Username registered but is not primary (transfers)
        UsernameNotPrimary,
        /// Not enough balance for Register payment
        InsufficientBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NameRegistered {
            username: UserName,
            owner: ZkAddressType,
        },
        NameQueuedForRegister {
            hash_username: T::Hash,
            hash_owner: T::Hash,
        },
        NameSetAsPrimary {
            owner: ZkAddressType,
            username: UserName,
        },
        RegisterCanceled {
            hash_username: T::Hash,
            hash_owner: T::Hash,
        },
        RegisterRemoved {
            username: UserName,
            owner: ZkAddressType,
        },
    }

    /// All registered Names
    #[pallet::storage]
    #[pallet::getter(fn username_records)]
    pub type UsernameRecords<T: Config> =
        StorageMap<_, Twox64Concat, UserName, ZkAddressType, OptionQuery>;

    /// Names pending to be registered with the given blocknumber(wait time) [username,(registrant,blocknumber)]
    #[pallet::storage]
    #[pallet::getter(fn pending_register)]
    pub type PendingRegister<T: Config> =
        StorageMap<_, Twox64Concat, T::Hash, (T::Hash, T::BlockNumber), OptionQuery>;

    /// Primary Records, 1 AccountID may have only one primary name
    #[pallet::storage]
    #[pallet::getter(fn primary_records)]
    pub type PrimaryRecords<T: Config> =
        StorageMap<_, Twox64Concat, ZkAddressType, UserName, OptionQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Queue Username for Register if it has not been registered or queued yet
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register())]
        #[transactional]
        pub fn register(
            origin: OriginFor<T>,
            username: UserName,
            registrant: ZkAddressType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <T as pallet::Config>::Currency::transfer(
                &who,
                &Self::account_id(),
                T::RegisterPrice::get(),
                ExistenceRequirement::KeepAlive,
            )?;

            Self::do_register(&username, registrant)
        }

        /// After Pending Register has passed its block wait time, finish regiser
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::accept_register())]
        #[transactional]
        pub fn accept_register(
            origin: OriginFor<T>,
            username: UserName,
            registrant: ZkAddressType,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Self::do_accept_register(username, registrant)?;

            Ok(())
        }

        /// Set a registered and owned username as Primary
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_primary_name())]
        #[transactional]
        pub fn set_primary_name(
            origin: OriginFor<T>,
            username: UserName,
            registrant: ZkAddressType,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Self::try_set_primary_name(username, registrant)?;

            Ok(())
        }

        /// Cancel pending name for register
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_pending_register())]
        #[transactional]
        pub fn cancel_pending_register(
            origin: OriginFor<T>,
            username: UserName,
            registrant: ZkAddressType,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Self::try_cancel_pending_register(username, registrant)?;

            Ok(())
        }

        /// Remove Already Registered Name
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::remove_register())]
        #[transactional]
        pub fn remove_register(
            origin: OriginFor<T>,
            username: UserName,
            registrant: ZkAddressType,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Self::try_remove_register(username, registrant)?;

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Returns the account ID of this pallet.
    #[inline]
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    /// Queue username for regiser
    fn do_register(username: &UserName, registrant: ZkAddressType) -> DispatchResult {
        // Username checks
        username_validation(username).ok_or(Error::<T>::InvalidUsernameFormat)?;

        let (hash_user, hash_address) = (
            T::Hashing::hash_of(username),
            T::Hashing::hash_of(&registrant),
        );

        // Check if already Pending Register
        ensure!(
            !PendingRegister::<T>::contains_key(hash_user),
            Error::<T>::AlreadyPendingRegister
        );

        // Check if already registered
        ensure!(
            !UsernameRecords::<T>::contains_key(username),
            Error::<T>::NameAlreadyRegistered
        );

        PendingRegister::<T>::insert(
            hash_user,
            (
                hash_address,
                frame_system::Pallet::<T>::block_number()
                    .saturating_add(T::RegisterWaitingPeriod::get()),
            ),
        );

        Self::deposit_event(Event::NameQueuedForRegister {
            hash_username: hash_user,
            hash_owner: hash_address,
        });
        Ok(())
    }

    /// Finish Register after block time has passed
    fn do_accept_register(username: UserName, registrant: ZkAddressType) -> DispatchResult {
        // Username checks
        username_validation(&username).ok_or(Error::<T>::InvalidUsernameFormat)?;

        let (hash_user, hash_address) = (
            T::Hashing::hash_of(&username),
            T::Hashing::hash_of(&registrant),
        );

        // check if block number has been passed
        let pending_register_data =
            PendingRegister::<T>::get(hash_user).ok_or(Error::<T>::UsernameNotFound)?;
        ensure!(
            frame_system::Pallet::<T>::block_number() > pending_register_data.1,
            Error::<T>::RegisterTimeNotReached
        );

        ensure!(
            pending_register_data.0 == hash_address,
            Error::<T>::NotOwned
        );

        // Move from pending into records
        PendingRegister::<T>::remove(hash_user);
        UsernameRecords::<T>::insert(&username, registrant);

        Self::deposit_event(Event::NameRegistered {
            username,
            owner: registrant,
        });
        Ok(())
    }

    /// Set primary name if register and owned
    fn try_set_primary_name(username: UserName, registrant: ZkAddressType) -> DispatchResult {
        // Username checks
        username_validation(&username).ok_or(Error::<T>::InvalidUsernameFormat)?;

        //check if name is registered
        ensure!(
            UsernameRecords::<T>::contains_key(&username),
            Error::<T>::NotRegistered
        );
        // check if its owned (can unwrap because of previous checks for the key)
        ensure!(
            UsernameRecords::<T>::get(&username).unwrap() == registrant,
            Error::<T>::NotOwned
        );

        // check if we already have a primary
        if PrimaryRecords::<T>::contains_key(registrant) {
            PrimaryRecords::<T>::mutate(registrant, |old_username| {
                *old_username = Some(username.clone())
            });
        } else {
            PrimaryRecords::<T>::insert(registrant, &username);
        }

        Self::deposit_event(Event::NameSetAsPrimary {
            owner: registrant,
            username,
        });
        Ok(())
    }

    fn try_cancel_pending_register(
        username: UserName,
        registrant: ZkAddressType,
    ) -> DispatchResult {
        // Username checks
        username_validation(&username).ok_or(Error::<T>::InvalidUsernameFormat)?;

        let (hash_user, hash_address) = (
            T::Hashing::hash_of(&username),
            T::Hashing::hash_of(&registrant),
        );

        let pending_register_data =
            PendingRegister::<T>::get(hash_user).ok_or(Error::<T>::UsernameNotFound)?;

        ensure!(
            pending_register_data.0 == hash_address,
            Error::<T>::NotOwned
        );

        PendingRegister::<T>::remove(hash_user);

        Self::deposit_event(Event::RegisterCanceled {
            hash_username: hash_user,
            hash_owner: hash_address,
        });
        Ok(())
    }

    fn try_remove_register(username: UserName, registrant: ZkAddressType) -> DispatchResult {
        // Username checks
        username_validation(&username).ok_or(Error::<T>::InvalidUsernameFormat)?;

        ensure!(
            UsernameRecords::<T>::contains_key(&username),
            Error::<T>::NotRegistered
        );

        ensure!(
            UsernameRecords::<T>::get(&username).unwrap() == registrant,
            Error::<T>::NotOwned
        );

        UsernameRecords::<T>::remove(&username);

        // check if the name we are removing is a primary name to keep storage synced
        if let Ok(primary_username) = PrimaryRecords::<T>::try_get(registrant) {
            if primary_username == username {
                PrimaryRecords::<T>::remove(registrant);
            }
        }

        Self::deposit_event(Event::RegisterRemoved {
            username,
            owner: registrant,
        });
        Ok(())
    }
}

/// username validation
fn username_validation(username: &Vec<u8>) -> Option<()> {
    if !(NAME_MIN_LEN..=NAME_MAX_LEN).contains(&username.len()) {
        return None;
    }

    let username_format: Matcher0<_> = regex!(br"[a-zA-Z][-a-zA-Z0-9\._]*[a-zA-Z0-9]");
    if username_format.is_match(username.as_slice()) {
        return Some(());
    }
    None
}
