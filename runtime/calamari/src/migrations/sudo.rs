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
        if have_storage_value(b"Sudo", b"Key", b"") {
            remove_storage_prefix(b"Sudo", b"Key", b"");
            remove_storage_prefix(b"Sudo", b":__STORAGE_VERSION__:", b"");
            log::info!(target: "OnRuntimeUpgrade", "✅ Sudo key has been removed.");
            log::info!(target: "OnRuntimeUpgrade", "✅ The pallet version has been removed.");
            T::DbWeight::get()
                .reads(1 as Weight)
                .saturating_add(T::DbWeight::get().writes(1 as Weight))
        } else {
            T::DbWeight::get().reads(1 as Weight)
        }
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        if have_storage_value(b"Sudo", b"Key", b"") {
            log::info!(target: "OnRuntimeUpgrade", "✅ Sudo key will be removed soon.");
            log::info!(target: "OnRuntimeUpgrade", "✅ The pallet version will be removed soon.");
            Ok(())
        } else {
            Err("Sudo doesn't exist.")
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        if have_storage_value(b"Sudo", b"Key", b"") {
            Err("Failed to remove sudo module.")
        } else {
            Ok(())
        }
    }
}
