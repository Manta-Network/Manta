// Copyright 2020-2024 Manta Network.
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
use frame_support::migration::clear_storage_prefix;
use frame_support::{
    migration::have_storage_value,
    pallet_prelude::Weight,
    traits::{Get, OnRuntimeUpgrade},
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub struct RemoveMantaPay<T>(PhantomData<T>);
impl<T: frame_system::Config> OnRuntimeUpgrade for RemoveMantaPay<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut reads = 0;
        let mut writes = 0;
        log::info!(target: "MantaPay", "✅ {:?} has been removed.", have_storage_value(b"MantaPay", b"", b""));
        // NullifierCommitmentSet
        if have_storage_value(b"MantaPay", b"NullifierCommitmentSet", b"") {
            clear_storage_prefix(b"MantaPay", b"Key", b"", None, None);
            clear_storage_prefix(b"MantaPay", b":__STORAGE_VERSION__:", b"", None, None);
            log::info!(target: "MantaPay", "✅ NullifierCommitmentSet has been removed.");
            log::info!(target: "MantaPay", "✅ The pallet version has been removed.");
            reads += 1;
            writes += 2;
        }
        // NullifierSetInsertionOrder
        if have_storage_value(b"MantaPay", b"NullifierSetInsertionOrder", b"") {
            clear_storage_prefix(b"MantaPay", b"NullifierSetInsertionOrder", b"", None, None);
            log::info!(target: "MantaPay", "✅ NullifierSetInsertionOrder has been removed.");
            reads += 1;
            writes += 1;
        }
        // NullifierSetSize
        if have_storage_value(b"MantaPay", b"NullifierSetSize", b"") {
            clear_storage_prefix(b"MantaPay", b"NullifierSetSize", b"", None, None);
            log::info!(target: "MantaPay", "✅ NullifierSetSize has been removed.");
            reads += 1;
            writes += 1;
        }
        // ShardTrees
        if have_storage_value(b"MantaPay", b"ShardTrees", b"") {
            clear_storage_prefix(b"MantaPay", b"ShardTrees", b"", None, None);
            log::info!(target: "MantaPay", "✅ ShardTrees has been removed.");
            reads += 1;
            writes += 1;
        }
        // Shards
        if have_storage_value(b"MantaPay", b"Shards", b"") {
            clear_storage_prefix(b"MantaPay", b"Shards", b"", None, None);
            log::info!(target: "MantaPay", "✅ Shards has been removed.");
            reads += 1;
            writes += 1;
        }
        // UtxoAccumulatorOutputs
        if have_storage_value(b"MantaPay", b"UtxoAccumulatorOutputs", b"") {
            clear_storage_prefix(b"MantaPay", b"UtxoAccumulatorOutputs", b"", None, None);
            log::info!(target: "MantaPay", "✅ UtxoAccumulatorOutputs has been removed.");
            reads += 1;
            writes += 1;
        }
        // UtxoSet
        if have_storage_value(b"MantaPay", b"UtxoSet", b"") {
            clear_storage_prefix(b"MantaPay", b"UtxoSet", b"", None, None);
            log::info!(target: "MantaPay", "✅ UtxoSet has been removed.");
            reads += 1;
            writes += 1;
        }
        log::info!(target: "MantaPay", "✅ {:?} has been removed.", have_storage_value(b"MantaPay", b"", b""));
        T::DbWeight::get()
            .reads(reads)
            .saturating_add(T::DbWeight::get().writes(writes))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        if have_storage_value(b"MantaPay", b"", b"") {
            log::info!(target: "MantaPay will be removed soon.");
            Ok(Vec::new())
        } else {
            Err(DispatchError::Other("Sudo doesn't exist."))
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), DispatchError> {
        if have_storage_value(b"MantaPay", b"", b"") {
            Err(DispatchError::Other("Failed to remove MantaPay module."))
        } else {
            Ok(())
        }
    }
}
