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

use core::marker::PhantomData;
use frame_support::{
	migration::{have_storage_value, remove_storage_prefix},
	pallet_prelude::Weight,
	traits::{Get, OnRuntimeUpgrade},
};

pub struct RemoveSudo<T>(PhantomData<T>);
impl<T: frame_system::Config> OnRuntimeUpgrade for RemoveSudo<T> {
	fn on_runtime_upgrade() -> Weight {
		if have_storage_value(b"Sudo", b"", b"") {
			remove_storage_prefix(b"Sudo", b"", b"");
			T::DbWeight::get()
				.reads(1 as Weight)
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
		} else {
			T::DbWeight::get().reads(1 as Weight)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		if have_storage_value(b"Sudo", b"", b"") {
			let sudo_key = frame_support::migration::storage_iter::<T::AccountId>(b"Sudo", b"")
				.collect::<Vec<_>>();
			log::info!(target, "OnRuntimeUpgrade", "Here's sudo key: ", sudo_key);
			Ok(())
		} else {
			Err("Sudo doesn't exist.")
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		if !have_storage_value(b"Sudo", b"", b"") {
			Err("Failed to remove sudo module.")
		} else {
			Ok(())
		}
	}
}
