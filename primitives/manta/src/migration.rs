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

//! Migration creates some helper function to make storage migration more convenient.

use frame_support::{
    dispatch::Weight,
    migrations::migrate_from_pallet_version_to_storage_version,
    traits::{GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess},
    weights::constants::RocksDbWeight,
};
use sp_std::marker::PhantomData;

/// MigratePalletPv2Sv means a wrapped handler to automatically upgrade our pallet
/// from PalletVersion(Pv) to StorageVersion(Sv).
///
/// It's actually a simple rewriting about storage flag: delete [pallet_name] + '__STORAGE_VERSION__' key
/// and reset [pallet_name] + '__PALLET_VERSION__' key.
/// So It's a one-time job, and should be removed soon to minimize runtime size.
pub struct MigratePalletPv2Sv<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for MigratePalletPv2Sv<T>
where
    T: GetStorageVersion + PalletInfoAccess,
{
    fn on_runtime_upgrade() -> Weight {
        // let db_weight = <T::Runtime as Config>::DbWeight::get();
        let db_weight = RocksDbWeight::get();
        migrate_from_pallet_version_to_storage_version::<T>(&db_weight)
    }
}
