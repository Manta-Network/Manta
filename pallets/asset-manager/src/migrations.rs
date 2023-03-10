// Copyright 2020-2023 Manta Network.
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

//! Do storage migration for AllowedDestParaIds which records the count of associated assets for
//! each para id.

use super::*;
use core::marker::PhantomData;
use frame_support::{
    dispatch::GetStorageVersion,
    pallet_prelude::Weight,
    traits::{Get, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
};

/// Storage migration to populate the existing assets'
/// entries in the new AllowedDestParaIds storage item
pub struct AllowedDestParaIdsMigration<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for AllowedDestParaIdsMigration<T>
where
    T: GetStorageVersion + Config + PalletInfoAccess,
{
    fn on_runtime_upgrade() -> Weight {
        // currently, it's 0 on calamari.
        let storage_version = <T as GetStorageVersion>::on_chain_storage_version();
        if storage_version < 1 {
            log::info!(target: "asset-manager", "Start to execute storage migration for asset-manager.");
            let mut reads: Weight = 0;
            let mut writes: Weight = 0;
            LocationAssetId::<T>::iter().for_each(|(location, _asset_id)| {
                reads += 1;
                if let Some(para_id) =
                    Pallet::<T>::para_id_from_multilocation(location.into().as_ref())
                {
                    if *para_id != 2084 {
                        let _ = Pallet::<T>::increase_count_of_associated_assets(*para_id);
                        reads += 1; // There's one read in method increase_count_of_associated_assets.
                        writes += 1; // There's one write in method increase_count_of_associated_assets.
                    }
                }
            });
            // Update storage version.
            StorageVersion::new(1u16).put::<T>();
            writes += 1;
            T::DbWeight::get()
                .reads(reads)
                .saturating_add(T::DbWeight::get().writes(writes))
        } else {
            log::info!("✅ no migration for asset-manager.");
            // only 1 read
            T::DbWeight::get().reads(1)
        }
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        let storage_version = <T as GetStorageVersion>::on_chain_storage_version();
        if storage_version >= 1 {
            return Err("Storage version is >= 1, the migration won't be executed.");
        }
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let storage_version = <T as GetStorageVersion>::on_chain_storage_version();
        if storage_version < 1 {
            return Err("Storage version is >= 1, the migration won't be executed.");
        }
        let acala = (2000, 3); // karura has 3 asset locations on calamari.
        let moonbeam = (2023, 1); // moonbean has 1 asset location on calamari.
        let calamari = 2084; // our own asset location won't be counted.
        if AllowedDestParaIds::<T>::get(acala.0) == Some(acala.1)
            && AllowedDestParaIds::<T>::get(moonbeam.0) == Some(moonbeam.1)
            && AllowedDestParaIds::<T>::get(calamari).is_none()
        {
            log::info!("✅ Storage migration for asset-manager has been executed successfully.");
            Ok(())
        } else {
            Err("Failed to executed storage migration for asset-manager.")
        }
    }
}
