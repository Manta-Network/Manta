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

#![allow(clippy::unnecessary_cast)]

use core::marker::PhantomData;
use frame_support::{
    migration::{get_storage_value, have_storage_value, put_storage_value},
    pallet_prelude::Weight,
    traits::{Get, OnRuntimeUpgrade},
};
pub struct ResetHighestSeen<T>(PhantomData<T>);
impl<T: frame_system::Config> OnRuntimeUpgrade for ResetHighestSeen<T> {
    fn on_runtime_upgrade() -> Weight {
        log::info!(target: "OnRuntimeUpgrade", "✅ is it even called.");
        if have_storage_value(b"AuthorInherent", b"HighestSlotSeen", b"") {
            let value: u32 = get_storage_value(b"AuthorInherent", b"HighestSlotSeen", b"").unwrap();
            log::info!(target: "OnRuntimeUpgrade", "\n✅ Initial value: {:?} \n", value);
            put_storage_value(b"AuthorInherent", b"HighestSlotSeen:", b"", b"0x00000000");
            log::info!(target: "OnRuntimeUpgrade", "✅ HighestSlotSeen was set to 0.");
            T::DbWeight::get()
                .reads(1 as Weight)
                .saturating_add(T::DbWeight::get().writes(1 as Weight))
        } else {
            T::DbWeight::get().reads(1 as Weight)
        }
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        if have_storage_value(b"AuthorInherent", b"HighestSlotSeen", b"") {
            log::info!(target: "OnRuntimeUpgrade", "✅ HighestSlotSeen will be reset to 0 soon.");
            Ok(())
        } else {
            Err("")
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        if have_storage_value(b"AuthorInherent", b"HighestSlotSeen", b"") {
            let value: u32 = get_storage_value(b"AuthorInherent", b"HighestSlotSeen", b"").unwrap();
            log::info!(target: "OnRuntimeUpgrade", "\n ✅ HighestSlotSeen was reset successfully. {:?} \n", value);
            Ok(())
        } else {
            Err("HighestSlotSeen value is not set!")
        }
    }
}
