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

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::{pallet_prelude::*, traits::Len, transactional};
use frame_system::pallet_prelude::*;
use manta_primitives::types::{Balance, BlockNumber};
use sp_runtime::{
    traits::{AccountIdConversion, Hash, Saturating},
    DispatchResult,
};
use sp_std::{prelude::*, vec::Vec};

mod mock;
mod tests;
// pub mod weights;

pub use pallet::*;
// pub use weights::WeightInfo;

pub type ZkAddressType = [u8; 32];

pub type UserName = Vec<u8>;

pub const NAME_MAX_LEN: usize = 63;
pub const NAME_MIN_LEN: usize = 3;

/// Username Record
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct NameRecord {
    pub registrant: ZkAddressType,
    pub price: Balance,
    pub status: Status,
}

/// Username Status
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum Status {
    /// The username isn't registered
    AVAILABLE,
    /// The username being registered, but not set as primary name
    REGISTERED,
    /// The username being set as primary name
    TRANSFERABLE,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{traits::StorageVersion, PalletId};
    // use manta_primitives::assets::AssetConfig;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // type Call: Parameter;

        /// Pallet ID
        type PalletId: Get<PalletId>;

        type RegisterWaitingPeriod: Get<Self::BlockNumber>;

        // Asset Configuration
        // type AssetConfig: AssetConfig<Self, AssetId = StandardAssetId, Balance = AssetValue>;

        // type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Username exists
        NameExists,
        /// Username not registered
        NotRegistered,
        /// The Registration time not reached
        RegisterTimeNotReach,
        /// Username invalid
        ValidationFailed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NameRegistered,
    }

    ///
    #[pallet::storage]
    #[pallet::getter(fn username_records)]
    pub type UsernameRecords<T: Config> =
        StorageMap<_, Twox64Concat, Vec<u8>, NameRecord, OptionQuery>;

    ///
    #[pallet::storage]
    #[pallet::getter(fn pending_register)]
    pub type PendingRegister<T: Config> =
        StorageMap<_, Twox64Concat, (T::Hash, T::Hash), T::BlockNumber, OptionQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        ///
        #[pallet::call_index(0)]
        #[pallet::weight(1000)]
        #[transactional]
        pub fn register(
            origin: OriginFor<T>,
            username: Vec<u8>,
            registrant: ZkAddressType,
        ) -> DispatchResult {
            let _origin = ensure_signed(origin)?;

            Self::do_register(&username, registrant)
        }

        ///
        #[pallet::call_index(1)]
        #[pallet::weight(1000)]
        #[transactional]
        pub fn accept_register(
            origin: OriginFor<T>,
            username: Vec<u8>,
            registrant: ZkAddressType,
            price: Balance,
        ) -> DispatchResult {
            let _origin = ensure_signed(origin)?;

            Self::do_accept_register(&username, registrant, price)?;

            // TODO: usernames as NFTs making them tradeble
            Self::mint_username_as_nft(&username);

            Ok(())
        }

        ///
        #[pallet::call_index(2)]
        #[pallet::weight(1000)]
        #[transactional]
        pub fn set_primary_name(origin: OriginFor<T>, username: Vec<u8>) -> DispatchResult {
            let _origin = ensure_signed(origin)?;

            Ok(())
        }

        ///
        #[pallet::call_index(3)]
        #[pallet::weight(1000)]
        #[transactional]
        pub fn transfer_to_username(
            origin: OriginFor<T>,
            username: Vec<u8>,
            registrant: ZkAddressType,
            price: Balance,
        ) -> DispatchResult {
            let _origin = ensure_signed(origin)?;

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

    ///
    fn do_register(username: &Vec<u8>, registrant: ZkAddressType) -> DispatchResult {
        PendingRegister::<T>::insert(
            (T::Hashing::hash_of(username), T::Hashing::hash_of(&registrant)),
            frame_system::Pallet::<T>::block_number()
                .saturating_add(T::RegisterWaitingPeriod::get()),
        );
        Ok(())
    }

    ///
    fn do_accept_register(
        username: &Vec<u8>,
        registrant: ZkAddressType,
        price: Balance,
    ) -> DispatchResult {
        ///
        username_validation(username).ok_or(Error::<T>::ValidationFailed)?;

        ///
        let (hash_user, hash_address) = (
            T::Hashing::hash_of(username),
            T::Hashing::hash_of(&registrant),
        );
        let creation_block = PendingRegister::<T>::get((hash_user, hash_address))
            .ok_or(Error::<T>::NotRegistered)?;
        ensure!(
            frame_system::Pallet::<T>::block_number() > creation_block,
            Error::<T>::RegisterTimeNotReach
        );

        let record = NameRecord {
            registrant,
            price,
            status: Status::REGISTERED,
        };
        UsernameRecords::<T>::insert(username, record);

        Self::deposit_event(Event::NameRegistered);
        Ok(())
    }

    fn mint_username_as_nft(username: &Vec<u8>) {}
}

/// username validation
fn username_validation(username: &Vec<u8>) -> Option<()> {
    let label = core::str::from_utf8(username.as_slice())
        .map(|label| label.to_ascii_lowercase())
        .ok()?;

    if !(NAME_MIN_LEN..=NAME_MAX_LEN).contains(&label.len()) {
        return None;
    }

    let label_chars = label.chars().collect::<Vec<_>>();

    match label_chars.as_slice() {
        [first, middle @ .., last]
            if first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric() =>
        {
            for (i, &c) in middle.iter().enumerate() {
                match c {
                    c if c.is_ascii_alphanumeric() => continue,
                    c if c == '-' => {
                        if i == 1 || i == 2 {
                            return None;
                        }
                        continue;
                    }
                    _ => return None,
                }
            }
        }
        _ => return None,
    }

    Some(())
}
