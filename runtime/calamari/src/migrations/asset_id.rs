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

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::StorageHasher;
use codec::Encode;
use core::marker::PhantomData;
#[allow(deprecated)]
use frame_support::migration::{
    get_storage_value, put_storage_value, storage_key_iter, take_storage_value,
};
use frame_support::{
    pallet_prelude::Weight,
    traits::{Get, OnRuntimeUpgrade},
    Blake2_128Concat,
};
use manta_primitives::{
    assets::{AssetLocation, AssetRegistryMetadata},
    types::Balance,
};
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;

type OldAssetId = u32;
type NewAssetId = u128;

pub struct AssetIdMigration<T>(PhantomData<T>);
impl<T: pallet_asset_manager::Config + pallet_assets::Config> OnRuntimeUpgrade
    for AssetIdMigration<T>
where
    NewAssetId: From<<T as pallet_asset_manager::Config>::AssetId>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut num_reads = 0;
        let mut num_writes = 0;

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let stored_data: Vec<_> = storage_key_iter::<OldAssetId, AssetLocation, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .drain()
        .collect();
        for (old_key, value) in stored_data {
            let new_key: NewAssetId = old_key as NewAssetId;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new_key.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let stored_data: Vec<_> = storage_key_iter::<AssetLocation, OldAssetId, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .drain()
        .collect();
        for (old_key, value) in stored_data {
            let new_key: NewAssetId = value as NewAssetId;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&old_key.encode()),
                new_key,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        let stored_data: Vec<_> = storage_key_iter::<
            OldAssetId,
            AssetRegistryMetadata<Balance>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (old_key, value) in stored_data {
            let new_key: NewAssetId = old_key as NewAssetId;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new_key.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        let stored_data: Vec<_> = storage_key_iter::<OldAssetId, u128, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .drain()
        .collect();
        for (old_key, value) in stored_data {
            let new_key: NewAssetId = old_key as NewAssetId;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new_key.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        num_reads += 1;
        num_writes += 1;
        let value = match take_storage_value::<OldAssetId>(pallet_prefix, storage_item_prefix, &[])
        {
            Some(value) => value,
            None => {
                return T::DbWeight::get()
                    .reads(num_reads as Weight)
                    .saturating_add(T::DbWeight::get().writes(num_writes as Weight));
            }
        };
        let new_value: NewAssetId = value as NewAssetId;
        put_storage_value(pallet_prefix, storage_item_prefix, &[], new_value);

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        let stored_data: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                pallet_asset_manager::DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (old_key, value) in stored_data {
            let new_key: NewAssetId = old_key as NewAssetId;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new_key.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // Account

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Account";
        let stored_data: Vec<_> = storage_key_iter::<
            (OldAssetId, <T as frame_system::Config>::AccountId),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for ((asset_id_key, account_id_key), value) in stored_data {
            let new_key: NewAssetId = asset_id_key as NewAssetId;
            let key1: Vec<u8> = Blake2_128Concat::hash(&new_key.encode());
            let key1_plus_key2: Vec<u8> = key1
                .into_iter()
                .chain(Blake2_128Concat::hash(&account_id_key.encode()).into_iter())
                .collect();
            put_storage_value(pallet_prefix, storage_item_prefix, &key1_plus_key2, value);
            num_reads += 1;
            num_writes += 1;
        }

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        let stored_data: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetMetadata<
                pallet_asset_manager::DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (asset_id_key, value) in stored_data {
            let new_key: NewAssetId = asset_id_key as NewAssetId;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new_key.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // Approvals

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Approvals";
        let stored_data: Vec<_> = storage_key_iter::<
            (
                OldAssetId,
                <T as frame_system::Config>::AccountId,
                <T as frame_system::Config>::AccountId,
            ),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for ((asset_id_key, owner_account_id_key, delegator_account_id_key), value) in stored_data {
            let new_key: NewAssetId = asset_id_key as NewAssetId;
            let c: Vec<u8> = Blake2_128Concat::hash(&new_key.encode())
                .into_iter()
                .chain(Blake2_128Concat::hash(&owner_account_id_key.encode()).into_iter())
                .collect();
            let c: Vec<u8> = c
                .into_iter()
                .chain(Blake2_128Concat::hash(&delegator_account_id_key.encode()).into_iter())
                .collect();
            put_storage_value(pallet_prefix, storage_item_prefix, &c, value);
            num_reads += 1;
            num_writes += 1;
        }

        T::DbWeight::get()
            .reads(num_reads as Weight)
            .saturating_add(T::DbWeight::get().writes(num_writes as Weight))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;

        // We want to test that:
        // There are no entries in the new storage beforehand
        // The same number of mappings exist before and after
        // There are no entries in the old storage afterward

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let stored_data_new: Vec<_> =
            storage_key_iter::<NewAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> =
            storage_key_iter::<OldAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        let asset_id_location_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "asset_id_location_map_count: {:?} ", asset_id_location_map_count);
        Self::set_temp_storage(asset_id_location_map_count, "asset_id_location_map_count");

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let stored_data_new: Vec<_> =
            storage_key_iter::<AssetLocation, NewAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> =
            storage_key_iter::<AssetLocation, OldAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        let location_asset_id_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "location_asset_id_map_count: {:?} ", location_asset_id_map_count);
        Self::set_temp_storage(location_asset_id_map_count, "location_asset_id_map_count");

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            AssetRegistryMetadata<Balance>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            AssetRegistryMetadata<Balance>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let asset_id_metadata_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "asset_id_metadata_map_count: {:?} ", asset_id_metadata_map_count);
        Self::set_temp_storage(asset_id_metadata_map_count, "asset_id_metadata_map_count");

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        let stored_data_new: Vec<_> = storage_key_iter::<NewAssetId, u128, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> = storage_key_iter::<OldAssetId, u128, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .collect();
        let units_per_sec_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "units_per_sec_map_count: {:?} ", units_per_sec_map_count);
        Self::set_temp_storage(units_per_sec_map_count, "units_per_sec_map_count");

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        assert!(get_storage_value::<NewAssetId>(pallet_prefix, storage_item_prefix, &[]).is_none());
        let next_asset_id: OldAssetId =
            get_storage_value::<OldAssetId>(pallet_prefix, storage_item_prefix, &[]).unwrap();
        log::info!(target: "OnRuntimeUpgrade", "next_asset_id: {:?} ", next_asset_id);
        Self::set_temp_storage(next_asset_id, "next_asset_id");

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                pallet_asset_manager::DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                pallet_asset_manager::DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let asset_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "asset_map_count: {:?} ", asset_map_count);
        Self::set_temp_storage(asset_map_count, "asset_map_count");

        // Account

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Account";
        // // TODO:
        // let stored_data_new: Vec<_> = storage_key_iter::<
        //     (NewAssetId, <T as frame_system::Config>::AccountId),
        //     pallet_asset_manager::AssetAccountOf<T, ()>,
        //     Blake2_128Concat,
        // >(pallet_prefix, storage_item_prefix)
        // .collect();
        // assert_eq!(stored_data_new.len() as u32, 0u32);
        let stored_data_old: Vec<_> = storage_key_iter::<
            (OldAssetId, <T as frame_system::Config>::AccountId),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let account_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "account_map_count: {:?} ", account_map_count);
        Self::set_temp_storage(account_map_count, "account_map_count");

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            pallet_assets::AssetMetadata<
                pallet_asset_manager::DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetMetadata<
                pallet_asset_manager::DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let metadata_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "metadata_map_count: {:?} ", metadata_map_count);
        Self::set_temp_storage(metadata_map_count, "metadata_map_count");

        // Approvals

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Approvals";
        let stored_data_new: Vec<_> = storage_key_iter::<
            (
                NewAssetId,
                <T as frame_system::Config>::AccountId,
                <T as frame_system::Config>::AccountId,
            ),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        assert!(stored_data_new.len() == 0);
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetMetadata<
                pallet_asset_manager::DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let approvals_map_count = stored_data_old.len() as u32;
        log::info!(target: "OnRuntimeUpgrade", "approvals_map_count: {:?} ", approvals_map_count);
        Self::set_temp_storage(approvals_map_count, "approvals_map_count");

        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;

        // We want to test that:
        // There are no entries in the new storage beforehand
        // The same number of mappings exist before and after
        // There are no entries in the old storage afterward

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        // // TODO:
        // let stored_data_old: Vec<_> =
        //     storage_key_iter::<OldAssetId, AssetLocation, Blake2_128Concat>(
        //         pallet_prefix,
        //         storage_item_prefix,
        //     )
        //     .collect();
        // assert!(stored_data_old.len() as u32 == 0u32);
        let stored_data_new: Vec<_> =
            storage_key_iter::<NewAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        assert_eq!(
            Self::get_temp_storage("asset_id_location_map_count"),
            Some(stored_data_new.len() as u32)
        );

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        // // TODO:
        // let stored_data_old: Vec<_> =
        //     storage_key_iter::<AssetLocation, OldAssetId, Blake2_128Concat>(
        //         pallet_prefix,
        //         storage_item_prefix,
        //     )
        //     .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> =
            storage_key_iter::<AssetLocation, NewAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        assert_eq!(
            Self::get_temp_storage("location_asset_id_map_count",),
            Some(stored_data_new.len() as u32)
        );

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        // // TODO:
        // let stored_data_old: Vec<_> = storage_key_iter::<
        //     OldAssetId,
        //     AssetRegistryMetadata<Balance>,
        //     Blake2_128Concat,
        // >(pallet_prefix, storage_item_prefix)
        // .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            AssetRegistryMetadata<Balance>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        assert_eq!(
            Self::get_temp_storage("asset_id_metadata_map_count",),
            Some(stored_data_new.len() as u32)
        );

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        // // TODO:
        // let stored_data_old: Vec<_> = storage_key_iter::<OldAssetId, u128, Blake2_128Concat>(
        //     pallet_prefix,
        //     storage_item_prefix,
        // )
        // .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> = storage_key_iter::<NewAssetId, u128, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .collect();
        assert_eq!(
            Self::get_temp_storage("units_per_sec_map_count"),
            Some(stored_data_new.len() as u32)
        );

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        // TODO:
        // assert!(get_storage_value::<OldAssetId>(pallet_prefix, storage_item_prefix, &[]).is_none());
        let next_asset_id: NewAssetId =
            get_storage_value::<NewAssetId>(pallet_prefix, storage_item_prefix, &[]).unwrap();
        let temp: u32 = Self::get_temp_storage("next_asset_id").unwrap();
        assert_eq!(temp as u128, next_asset_id);

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        // // TODO:
        // let stored_data_old: Vec<_> = storage_key_iter::<
        //     OldAssetId,
        //     pallet_assets::AssetDetails<
        //         <T as pallet_assets::Config>::Balance,
        //         <T as frame_system::Config>::AccountId,
        //         pallet_asset_manager::DepositBalanceOf<T>,
        //     >,
        //     Blake2_128Concat,
        // >(pallet_prefix, storage_item_prefix)
        // .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                pallet_asset_manager::DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let asset_map_count = stored_data_new.len() as u32;
        assert_eq!(
            Self::get_temp_storage("asset_map_count"),
            Some(asset_map_count)
        );

        // Account

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Account";
        // // TODO:
        // let stored_data_old: Vec<_> = storage_key_iter::<
        //     (OldAssetId, <T as frame_system::Config>::AccountId),
        //     pallet_asset_manager::AssetAccountOf<T, ()>,
        //     Blake2_128Concat,
        // >(pallet_prefix, storage_item_prefix)
        // .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> = storage_key_iter::<
            (NewAssetId, <T as frame_system::Config>::AccountId),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let account_map_count = stored_data_new.len() as u32;
        assert_eq!(
            Self::get_temp_storage("account_map_count"),
            Some(account_map_count)
        );

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        // // TODO:
        // let stored_data_old: Vec<_> = storage_key_iter::<
        //     OldAssetId,
        //     pallet_assets::AssetMetadata<
        //         pallet_asset_manager::DepositBalanceOf<T>,
        //         BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
        //     >,
        //     Blake2_128Concat,
        // >(pallet_prefix, storage_item_prefix)
        // .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            pallet_assets::AssetMetadata<
                pallet_asset_manager::DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let metadata_map_count = stored_data_new.len() as u32;
        assert_eq!(
            Self::get_temp_storage("metadata_map_count"),
            Some(metadata_map_count)
        );

        // Approvals

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Approvals";
        // // TODO:
        // let stored_data_old: Vec<_> = storage_key_iter::<
        //     (
        //         OldAssetId,
        //         <T as frame_system::Config>::AccountId,
        //         <T as frame_system::Config>::AccountId,
        //     ),
        //     pallet_asset_manager::AssetAccountOf<T, ()>,
        //     Blake2_128Concat,
        // >(pallet_prefix, storage_item_prefix)
        // .collect();
        // assert!(stored_data_old.len() == 0);
        let stored_data_new: Vec<_> = storage_key_iter::<
            (
                NewAssetId,
                <T as frame_system::Config>::AccountId,
                <T as frame_system::Config>::AccountId,
            ),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let approvals_map_count = stored_data_new.len() as u32;
        assert_eq!(
            Self::get_temp_storage("approvals_map_count"),
            Some(approvals_map_count)
        );

        Ok(())
    }
}
