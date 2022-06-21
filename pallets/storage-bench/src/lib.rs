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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
    pallet_prelude::*,
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

pub(crate) const CIPHER_TEXT_LENGTH: usize = 68;
pub(crate) const EPHEMERAL_PUBLIC_KEY_LENGTH: usize = 32;
pub(crate) const UTXO_ACCUMULATOR_OUTPUT_LENGTH: usize = 32;
pub(crate) const UTXO_LENGTH: usize = 32;
pub(crate) const VOID_NUMBER_LENGTH: usize = 32;
pub(crate) const PROOF_LENGTH: usize = 192;

/// Group Type
pub type Group = [u8; EPHEMERAL_PUBLIC_KEY_LENGTH];

/// UTXO Type
pub type Utxo = [u8; UTXO_LENGTH];

/// Void Number Type
pub type VoidNumber = [u8; VOID_NUMBER_LENGTH];

/// UTXO Accumulator Output Type
pub type UtxoAccumulatorOutput = [u8; UTXO_ACCUMULATOR_OUTPUT_LENGTH];

/// Ciphertext Type
pub type Ciphertext = [u8; CIPHER_TEXT_LENGTH];

/// Transfer Proof Type
pub type Proof = [u8; PROOF_LENGTH];

/// Encrypted Note
#[cfg_attr(
    feature = "rpc",
    derive(Deserialize, Serialize),
    serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct EncryptedNote {
    /// Ephemeral Public Key
    pub ephemeral_public_key: Group,

    /// Ciphertext
    #[cfg_attr(
        feature = "rpc",
        serde(
            with = "manta_util::serde_with::As::<[manta_util::serde_with::Same; CIPHER_TEXT_LENGTH]>"
        )
    )]
    pub ciphertext: Ciphertext,
}

impl Default for EncryptedNote {
    #[inline]
    fn default() -> Self {
        Self {
            ephemeral_public_key: [0; EPHEMERAL_PUBLIC_KEY_LENGTH],
            ciphertext: [0; CIPHER_TEXT_LENGTH],
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which may set filter.
        type UpdateOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// read a shard element: (UTXO, EncryptedNote)
        ShardElementRead(Utxo, EncryptedNote),
        /// read a range of shard element
        ShardElementBatchRead(u64),
        /// write a shard element: (UTXO, EncryptedNote)
        ShardElementWritten(Utxo, EncryptedNote),
        /// read a void number
        VoidNumberRead(VoidNumber),
        /// write a void number
        VoidNumberWritten(VoidNumber),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// pull a null storage item
        NullStorageItem,
    }

    #[pallet::storage]
    pub type ShardsIdentityIdentity<T: Config> =
        StorageDoubleMap<_, Identity, u8, Identity, u64, (Utxo, EncryptedNote), ValueQuery>;

    #[pallet::storage]
    pub type ShardsIdentityIdentityIndices<T: Config> =
        StorageMap<_, Twox64Concat, u8, u64, ValueQuery>;

    #[pallet::storage]
    pub type ShardsTwoxIdentity<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u8, Identity, u64, (Utxo, EncryptedNote), ValueQuery>;

    #[pallet::storage]
    pub type ShardsTwoxIdentityIndices<T: Config> =
        StorageMap<_, Twox64Concat, u8, u64, ValueQuery>;

    #[pallet::storage]
    pub type ShardsTwoxTwox<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u8, Twox64Concat, u64, (Utxo, EncryptedNote), ValueQuery>;

    #[pallet::storage]
    pub type ShardsTwoxTwoxIndices<T: Config> = StorageMap<_, Twox64Concat, u8, u64, ValueQuery>;

    /// Void Number Set
    #[pallet::storage]
    pub(super) type VoidNumberSetIdentity<T: Config> =
        StorageMap<_, Identity, VoidNumber, (), ValueQuery>;

    /// Void Number Set
    #[pallet::storage]
    pub(super) type VoidNumberSetTwox<T: Config> =
        StorageMap<_, Twox64Concat, VoidNumber, (), ValueQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(500_000)]
        #[transactional]
        pub fn insert_shard_element_identity_identity(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo: Utxo,
            encrypted_note: EncryptedNote,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let utxo_idx = ShardsIdentityIdentityIndices::<T>::get(shard_idx);
            ShardsIdentityIdentity::<T>::insert(shard_idx, Self::encode(utxo_idx), (utxo, encrypted_note.clone()));
            ShardsIdentityIdentityIndices::<T>::insert(shard_idx, utxo_idx + 1);
            Self::deposit_event(Event::ShardElementWritten(utxo, encrypted_note));
            Ok(())
        }

        #[pallet::weight(500_000)]
        #[transactional]
        pub fn insert_shard_element_twox_identity(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo: Utxo,
            encrypted_note: EncryptedNote,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let utxo_idx = ShardsIdentityIdentityIndices::<T>::get(shard_idx);
            ShardsTwoxIdentity::<T>::insert(shard_idx, Self::encode(utxo_idx), (utxo, encrypted_note.clone()));
            ShardsTwoxIdentityIndices::<T>::insert(shard_idx, utxo_idx + 1);
            Self::deposit_event(Event::ShardElementWritten(utxo, encrypted_note));
            Ok(())
        }

        #[pallet::weight(500_000)]
        #[transactional]
        pub fn insert_shard_element_twox_twox(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo: Utxo,
            encrypted_note: EncryptedNote,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let utxo_idx = ShardsIdentityIdentityIndices::<T>::get(shard_idx);
            ShardsTwoxTwox::<T>::insert(shard_idx, Self::encode(utxo_idx), (utxo, encrypted_note.clone()));
            ShardsTwoxIdentityIndices::<T>::insert(shard_idx, utxo_idx + 1);
            Self::deposit_event(Event::ShardElementWritten(utxo, encrypted_note));
            Ok(())
        }

        #[pallet::weight(500_000)]
        #[transactional]
        pub fn point_read_shard_element_identity_identity(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo_idx: u64,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let (utxo, encrypted_note) =
                ShardsIdentityIdentity::<T>::get(shard_idx, Self::encode(utxo_idx));
            Self::deposit_event(Event::ShardElementRead(utxo, encrypted_note));
            Ok(())
        }

        #[pallet::weight(500_000)]
        #[transactional]
        pub fn point_read_shard_element_twox_identity(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo_idx: u64,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let (utxo, encrypted_note) =
                ShardsTwoxIdentity::<T>::get(shard_idx, Self::encode(utxo_idx));
            Self::deposit_event(Event::ShardElementRead(utxo, encrypted_note));
            Ok(())
        }

        #[pallet::weight(500_000)]
        #[transactional]
        pub fn point_read_shard_element_twox_twox(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo_idx: u64,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let (utxo, encrypted_note) =
                ShardsTwoxTwox::<T>::get(shard_idx, Self::encode(utxo_idx));
            Self::deposit_event(Event::ShardElementRead(utxo, encrypted_note));
            Ok(())
        }

        #[pallet::weight(500_000)]
        pub fn range_read_shard_element_identity_identity(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo_idx: u64,
            amount: u64,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let mut receivers = Vec::new();
            let mut iter = if utxo_idx == 0 {
                ShardsIdentityIdentity::<T>::iter_prefix(shard_idx)
            } else {
                let raw_key = ShardsIdentityIdentity::<T>::hashed_key_for(
                    shard_idx,
                    Self::encode(utxo_idx as u64 - 1),
                );
                ShardsIdentityIdentity::<T>::iter_prefix_from(shard_idx, raw_key)
            };
            for _ in 0..amount {
                match iter.next() {
                    Some((_, next)) => receivers.push(next),
                    _ => return Err(Error::<T>::NullStorageItem.into()),
                }
            }
            Self::deposit_event(Event::ShardElementBatchRead(amount));
            Ok(())
        }

        #[pallet::weight(500_000)]
        pub fn range_read_shard_element_twox_identity(
            origin: OriginFor<T>,
            shard_idx: u8,
            utxo_idx: u64,
            amount: u64,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            let mut receivers = Vec::new();
            let mut iter = if utxo_idx == 0 {
                ShardsTwoxIdentity::<T>::iter_prefix(shard_idx)
            } else {
                let raw_key = ShardsTwoxIdentity::<T>::hashed_key_for(
                    shard_idx,
                    Self::encode(utxo_idx as u64 - 1),
                );
                ShardsTwoxIdentity::<T>::iter_prefix_from(shard_idx, raw_key)
            };
            for _ in 0..amount {
                match iter.next() {
                    Some((_, next)) => receivers.push(next),
                    _ => return Err(Error::<T>::NullStorageItem.into()),
                }
            }
            Self::deposit_event(Event::ShardElementBatchRead(amount));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn encode(n: u64) -> u64 {
            let mut bytes = n.to_ne_bytes();
            bytes.reverse();
            u64::from_ne_bytes(bytes)
        }       
    }
}
