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
use frame_support::migration::{get_storage_value, put_storage_value, storage_key_iter};
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

pub struct AssetIdMigration<T>(PhantomData<T>);
impl<T: pallet_asset_manager::Config + pallet_assets::Config> OnRuntimeUpgrade
    for AssetIdMigration<T>
where
    u128: From<<T as pallet_asset_manager::Config>::AssetId>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut num_reads = 0;
        let mut num_writes = 0;

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let stored_data: Vec<_> = storage_key_iter::<u32, AssetLocation, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .drain()
        .collect();
        for (key, value) in stored_data {
            let new: u128 = key as u128;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let stored_data: Vec<_> = storage_key_iter::<AssetLocation, u32, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .drain()
        .collect();
        for (key, value) in stored_data {
            let new: u128 = value as u128;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&key.encode()),
                new,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        let stored_data: Vec<_> = storage_key_iter::<
            u32,
            AssetRegistryMetadata<Balance>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (key, value) in stored_data {
            let new: u128 = key as u128;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        let stored_data: Vec<_> =
            storage_key_iter::<u32, u128, Blake2_128Concat>(pallet_prefix, storage_item_prefix)
                .drain()
                .collect();
        for (key, value) in stored_data {
            let new: u128 = key as u128;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        let value: u32 = get_storage_value::<u32>(pallet_prefix, storage_item_prefix, &[]).unwrap();
        let new: u128 = value as u128;
        put_storage_value(pallet_prefix, storage_item_prefix, &[], new);
        num_reads += 1;
        num_writes += 1;

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        let stored_data: Vec<_> = storage_key_iter::<
            u32,
            // <pallet_assets::Pallet<T> as pallet_assets::Config>::AssetDetails,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                pallet_asset_manager::DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (key, value) in stored_data {
            let new: u128 = key as u128;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new.encode()),
                value,
            );
            num_reads += 1;
            num_writes += 1;
        }

        // Account

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Account";
        let stored_data: Vec<_> = storage_key_iter::<
            (u32, <T as frame_system::Config>::AccountId),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for ((asset_id_key, account_id_key), value) in stored_data {
            let new: u128 = asset_id_key as u128;
            let c: Vec<u8> = Blake2_128Concat::hash(&new.encode())
                .into_iter()
                .chain(Blake2_128Concat::hash(&account_id_key.encode()).into_iter())
                .collect();
            put_storage_value(pallet_prefix, storage_item_prefix, &c, value);
            num_reads += 1;
            num_writes += 1;
        }

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        let stored_data: Vec<_> = storage_key_iter::<
            u32,
            pallet_assets::AssetMetadata<
                pallet_asset_manager::DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (asset_id_key, value) in stored_data {
            let new: u128 = asset_id_key as u128;
            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new.encode()),
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
                u32,
                <T as frame_system::Config>::AccountId,
                <T as frame_system::Config>::AccountId,
            ),
            pallet_asset_manager::AssetAccountOf<T, ()>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for ((asset_id_key, owner_account_id_key, delegator_account_id_key), value) in stored_data {
            let new: u128 = asset_id_key as u128;
            let c: Vec<u8> = Blake2_128Concat::hash(&new.encode())
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
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        Ok(())
    }
}
