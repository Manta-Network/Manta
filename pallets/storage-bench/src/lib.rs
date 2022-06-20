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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
	dispatch::{CallMetadata, GetCallMetadata},
	pallet_prelude::*,
	traits::{Contains, PalletInfoAccess},
	transactional,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};

mod mock;
mod tests;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

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
 
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The origin which may set filter.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for the extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// can not pause
		CannotPause,
		/// invalid character encoding
		InvalidCharacter,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// read a shard element: (UTXO, EncryptedNote)
		ShardElementRead(Utxo, EncryptedNote),
		/// write a shard element: (UTXO, EncryptedNote)
		ShardElementWritten(Utxo, EncryptedNote),
        /// read a void number
        VoidNumberRead(VoidNumber);
        /// write a void number
        VoidNumberWritten(VoidNumber);
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
    pub type ShardsTwoxTwoxIndices<T: Config> = 
        StorageMap<_, Twox64Concat, u8, u64, ValueQuery>;
    
    /// Void Number Set
	#[pallet::storage]
	pub(super) type VoidNumberSetIdentity<T: Config> = StorageMap<_, Identity, VoidNumber, (), ValueQuery>;

    /// Void Number Set
	#[pallet::storage]
	pub(super) type VoidNumberSetTwox<T: Config> = StorageMap<_, Twox64Concat, VoidNumber, (), ValueQuery>;

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
            ShardsIdentityIdentity::<T>::insert(shard_idx, utxo_idx, (utxo, encrypted_note));
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
            ShardsTwoxIdentity::<T>::insert(shard_idx, utxo_idx, (utxo, encrypted_note));
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
            ShardsTwoxTwox::<T>::insert(shard_idx, utxo_idx, (utxo, encrypted_note));
            ShardsTwoxTwox::<T>::insert(shard_idx, utxo_idx + 1);
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
            let (utxo, encrypted_Note) = ShardsIdentityIdentity::<T>::get(shard_idx, utxo_idx);
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
            let (utxo, encrypted_Note) = ShardsTwoxIdentity::<T>::get(shard_idx, utxo_idx);
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
            let (utxo, encrypted_Note) = ShardsTwoxTwox::<T>::get(shard_idx, utxo_idx);
            Self::deposit_event(Event::ShardElementRead(utxo, encrypted_note));
			Ok(())
		}

        #[pallet::weight(500_000)]
		#[transactional]
		pub fn range_read_shard_element_identity_identity(
			origin: OriginFor<T>,
            shard_idx: u8,
			utxo_idx: u64,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
            let (utxo, encrypted_Note) = ShardsTwoxTwox::<T>::get(shard_idx, utxo_idx);
            Self::deposit_event(Event::ShardElementRead(utxo, encrypted_note));
			Ok(())
		}

        #[pallet::weight(500_000)]
		#[transactional]
		pub fn range_read_shard_element_twox_identity(
			origin: OriginFor<T>,
            shard_idx: u8,
			utxo_idx: u64,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
            let (utxo, encrypted_Note) = ShardsTwoxTwox::<T>::get(shard_idx, utxo_idx);
            Self::deposit_event(Event::ShardElementRead(utxo, encrypted_note));
			Ok(())
		}

	}

    
}

