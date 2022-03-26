// Copyright 2020-2021 Manta Network.
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

use super::*;
use frame_support::{
	dispatch::GetStorageVersion,
	pallet_prelude::Weight,
	traits::{Get, PalletInfoAccess, StorageVersion},
	Twox64Concat,
};
///  This migrates the pallet from the standard version by parity to our modified storage
impl<T: Config> Pallet<T> {
	pub fn migrate_to_v2() -> frame_support::weights::Weight {
		use frame_support::migration::{
			have_storage_value, remove_storage_prefix, storage_key_iter,
		};
		// Storage migrations should use storage versions for safety.
		if Self::on_chain_storage_version() < 2 {
			log::info!(" >>> Executing collator-selection V1->V2 migration!");

			// Drain all keys from the old (now unused) map
			let iter_map = storage_key_iter::<T::AccountId, T::BlockNumber, Twox64Concat>(
				Self::name().as_bytes(),
				b"LastAuthoredBlock",
			)
			.drain();
			let mut dropcount = 0;
			for _ in iter_map {
				dropcount += 1;
			}
			log::info!(" >>> Cleaned {} keys from LastAuthoredBlock", dropcount);
			remove_storage_prefix(Self::name().as_bytes(), b"LastAuthoredBlock", &[]);
			log::info!(" >>> Removed LastAuthoredBlock from storage" );

			// Update storage version.
			StorageVersion::new(2).put::<Self>();

			// Remove KickThreshold if customized
			if have_storage_value(Self::name().as_bytes(), b"KickThreshold", &[]) {
				remove_storage_prefix(Self::name().as_bytes(), b"KickThreshold", &[]);
				log::info!(" >>> Removed KickThreshold");
			} else {
				log::warn!(" !!! Chain did not have KickThreshold in storage. This is uenexpected but is possible if the genesis config was never changed");
			}

			// Return the weight consumed by the migration.
			T::DbWeight::get().reads_writes(1, dropcount as Weight + 1)
		} else {
			log::debug!(" >>> collator-selection V1->V2 migration not needed!");
			0
		}
	}
	pub fn pre_migrate_to_v2() -> Result<(), &'static str> {
		use frame_support::migration::{have_storage_value, storage_key_iter};
		let chainver = Self::on_chain_storage_version();
		if !(chainver < 2) {
			return Err("Migration to V2 does not apply");
		}

		if !have_storage_value(Self::name().as_bytes(), b"KickThreshold", &[]) {
			log::warn!("Precheck: KickThreshold does not exist");
		}

		if storage_key_iter::<T::AccountId, T::BlockNumber, Twox64Concat>(
			Self::name().as_bytes(),
			b"LastAuthoredBlock",
		)
		.count() == 0
		{
			return Err("LastAuthoredBlock does not exist");
		}
		Ok(())
	}

	pub fn post_migrate_to_v2() -> Result<(), &'static str> {
		use frame_support::migration::{have_storage_value, storage_key_iter};
		if !(Self::on_chain_storage_version() == 2) {
			return Err("storage version not upgraded");
		}

		if have_storage_value(Self::name().as_bytes(), b"KickThreshold", &[]) {
			return Err("KickThreshold wasn't removed");
		}

		if storage_key_iter::<T::AccountId, T::BlockNumber, Twox64Concat>(
			Self::name().as_bytes(),
			b"LastAuthoredBlock",
		)
		.count() > 0
		{
			return Err("LastAuthoredBlock wasn't removed");
		}
		Ok(())
	}
}
