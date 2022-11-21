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
use codec::{Decode, Encode};
use core::marker::PhantomData;
#[allow(deprecated)]
use frame_support::migration::{
    get_storage_value, put_storage_value, storage_key_iter, take_storage_value,
};
use frame_support::{
    dispatch::GetStorageVersion,
    pallet_prelude::Weight,
    storage_alias,
    traits::{Currency, Get, OnRuntimeUpgrade, StorageVersion},
    Blake2_128Concat,
};
use manta_primitives::{
    assets::{AssetLocation, AssetRegistryMetadata, AssetStorageMetadata},
    types::Balance,
};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;

pub type DepositBalanceOf<T, I = ()> = <<T as pallet_assets::Config<I>>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
pub type AssetAccountOf<T, I> = pallet_assets::AssetAccount<
    <T as pallet_assets::Config<I>>::Balance,
    DepositBalanceOf<T, I>,
    <T as pallet_assets::Config<I>>::Extra,
>;

type OldAssetId = u32;
type NewAssetId = u128;

pub mod old {
    use crate::migrations::asset_id::OldAssetId;
    use frame_support::{storage_alias, Blake2_128Concat};

    #[storage_alias]
    pub type Account<T: frame_system::Config<I>, I: 'static = ()> = StorageDoubleMap<
        Assets,
        Blake2_128Concat,
        OldAssetId,
        Blake2_128Concat,
        <T as frame_system::Config>::AccountId,
        super::AssetAccountOf<T, I>,
    >;
}

#[storage_alias]
type Account<T: frame_system::Config<I>, I: 'static = ()> = StorageDoubleMap<
    Assets,
    Blake2_128Concat,
    NewAssetId,
    Blake2_128Concat,
    <T as frame_system::Config>::AccountId,
    AssetAccountOf<T, I>,
>;

type AssetMapKVP<T> = (
    OldAssetId,
    pallet_assets::AssetDetails<
        <T as pallet_assets::Config>::Balance,
        <T as frame_system::Config>::AccountId,
        DepositBalanceOf<T>,
    >,
);

type MetadataMapKVP<T> = (
    OldAssetId,
    pallet_assets::AssetMetadata<
        DepositBalanceOf<T>,
        BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
    >,
);

#[derive(Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct OldAssetRegistrarMetadata {
    pub name: Vec<u8>,
    pub symbol: Vec<u8>,
    pub decimals: u8,
    pub evm_address: Option<H160>,
    pub is_frozen: bool,
    pub min_balance: Balance,
    pub is_sufficient: bool,
}

pub const INITIAL_PALLET_ASSETS_MANAGER_VERSION: u16 = 1;
pub const INITIAL_PALLET_ASSETS_VERSION: u16 = 0;

pub struct AssetIdMigration<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for AssetIdMigration<T>
where
    T: pallet_asset_manager::Config + pallet_assets::Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut num_reads = 0;
        let mut num_writes = 0;

        let asset_manager_storage_version =
            <pallet_asset_manager::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        let assets_storage_version =
            <pallet_assets::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        num_reads += 2;
        if asset_manager_storage_version != INITIAL_PALLET_ASSETS_MANAGER_VERSION
            || assets_storage_version != INITIAL_PALLET_ASSETS_VERSION
        {
            log::info!("Aborting migration due to unexpected on-chain storage versions for pallet-assets-manager: {:?} and pallet-assets: {:?}. Expectation was: {:?} and {:?}.", asset_manager_storage_version, assets_storage_version, INITIAL_PALLET_ASSETS_MANAGER_VERSION, INITIAL_PALLET_ASSETS_VERSION );
            return T::DbWeight::get().reads(num_reads as Weight);
        }

        // AssetIdLocation

        log::info!(target: "asset-manager", "Starting migration for AssetManager...");

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
        log::info!(
            target: "asset-manager", "Storage migration for AssetManager's AssetIdLocation storage item has been executed."
        );

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
        log::info!(
            target: "asset-manager", "Storage migration for AssetManager's LocationAssetId storage item has been executed."
        );

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        let stored_data: Vec<_> = storage_key_iter::<
            OldAssetId,
            OldAssetRegistrarMetadata,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        for (old_key, old_value) in stored_data {
            let new_key: NewAssetId = old_key as NewAssetId;
            let new_value: AssetRegistryMetadata<Balance> = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: old_value.name,
                    symbol: old_value.symbol,
                    decimals: old_value.decimals,
                    is_frozen: old_value.is_frozen,
                },
                min_balance: old_value.min_balance,
                is_sufficient: old_value.is_sufficient,
            };

            put_storage_value(
                pallet_prefix,
                storage_item_prefix,
                &Blake2_128Concat::hash(&new_key.encode()),
                new_value,
            );
            num_reads += 1;
            num_writes += 1;
        }
        log::info!(
            target: "asset-manager", "Storage migration for AssetManager's AssetIdMetadata storage item has been executed."
        );

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
        log::info!(
            target: "asset-manager", "Storage migration for AssetManager's UnitsPerSecond storage item has been executed."
        );

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
        log::info!(
            target: "asset-manager", "Storage migration for AssetManager's NextAssetId storage item has been executed."
        );

        StorageVersion::new(INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1)
            .put::<pallet_asset_manager::Pallet<T>>();

        log::info!(target: "asset-manager", "✅ Storage migration for AssetManager has been executed successfully and storage version has been update to: {:?}.", INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1);

        log::info!(target: "assets", "Starting migration for pallet-assets...");

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        let stored_data: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                DepositBalanceOf<T>,
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
        log::info!(
            target: "assets", "Storage migration for Assets' Asset storage item has been executed."
        );

        // Account

        let mut stored_data: Vec<_> = Vec::new();
        old::Account::<T, ()>::drain().for_each(|(asset_id_key, account_id_key, value)| {
            let new_asset_id_key: NewAssetId = asset_id_key as NewAssetId;
            stored_data.push((new_asset_id_key, account_id_key, value));
        });
        stored_data
            .iter()
            .for_each(|(new_asset_id_key, account_id_key, value)| {
                Account::<T, ()>::insert(new_asset_id_key, account_id_key, value);
            });
        log::info!(
            target: "assets", "Storage migration for Assets' Account storage item has been executed."
        );

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        let stored_data: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetMetadata<
                DepositBalanceOf<T>,
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
        log::info!(
            target: "assets", "Storage migration for Assets' Metadata storage item has been executed."
        );

        StorageVersion::new(INITIAL_PALLET_ASSETS_VERSION + 1).put::<pallet_assets::Pallet<T>>();

        log::info!(target: "assets", "✅ Storage migration for Assets has been executed successfully and storage version has been update to: {:?}.", INITIAL_PALLET_ASSETS_VERSION + 1);

        T::DbWeight::get()
            .reads(num_reads as Weight)
            .saturating_add(T::DbWeight::get().writes(num_writes as Weight))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;

        let asset_manager_storage_version =
            <pallet_asset_manager::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if asset_manager_storage_version != INITIAL_PALLET_ASSETS_MANAGER_VERSION {
            return Err("AssetManager storage version is not 1, the migration won't be executed.");
        }

        let assets_storage_version =
            <pallet_assets::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if assets_storage_version != INITIAL_PALLET_ASSETS_VERSION {
            return Err("Assets storage version is not 0, the migration won't be executed.");
        }

        // We want to test that:
        // There are no entries in the new storage beforehand
        // The same number of mappings exist before and after
        //
        // We have to manually check that there are no entries
        // with the old storage keys, as the new u128 asset-id
        // would still decode into the old u32 values.

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        assert_eq!(
            storage_key_iter::<NewAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );
        let stored_data_old: Vec<_> =
            storage_key_iter::<OldAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        Self::set_temp_storage(stored_data_old, "asset_id_location_stored_data_old");

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        assert_eq!(
            storage_key_iter::<AssetLocation, NewAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );
        let stored_data_old: Vec<_> =
            storage_key_iter::<AssetLocation, OldAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        Self::set_temp_storage(stored_data_old, "location_asset_id_stored_data_old");

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        assert_eq!(
            storage_key_iter::<NewAssetId, AssetRegistryMetadata<Balance>, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix
            )
            .count(),
            0
        );
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            OldAssetRegistrarMetadata,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        Self::set_temp_storage(stored_data_old, "asset_id_metadata_stored_data_old");

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        assert_eq!(
            storage_key_iter::<NewAssetId, u128, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );
        let stored_data_old: Vec<_> = storage_key_iter::<OldAssetId, u128, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .collect();
        Self::set_temp_storage(stored_data_old, "units_per_sec_stored_data_old");

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        assert!(get_storage_value::<NewAssetId>(pallet_prefix, storage_item_prefix, &[]).is_none());
        let next_asset_id: OldAssetId =
            get_storage_value::<OldAssetId>(pallet_prefix, storage_item_prefix, &[]).unwrap();
        Self::set_temp_storage(next_asset_id, "next_asset_id");

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        assert_eq!(
            storage_key_iter::<
                NewAssetId,
                pallet_assets::AssetDetails<
                    <T as pallet_assets::Config>::Balance,
                    <T as frame_system::Config>::AccountId,
                    DepositBalanceOf<T>,
                >,
                Blake2_128Concat,
            >(pallet_prefix, storage_item_prefix)
            .count(),
            0
        );
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        Self::set_temp_storage(stored_data_old, "asset_map_stored_data_old");

        // Account

        let mut stored_data_new: Vec<_> = Vec::new();
        Account::<T, ()>::iter().for_each(|kvp| {
            stored_data_new.push(kvp);
        });
        assert_eq!(stored_data_new.len(), 0);
        let mut stored_data_old: Vec<_> = Vec::new();
        old::Account::<T, ()>::iter().for_each(|kvp| {
            stored_data_old.push(kvp);
        });
        Self::set_temp_storage(stored_data_old, "account_map_stored_data_old");

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        assert_eq!(
            storage_key_iter::<
                NewAssetId,
                pallet_assets::AssetMetadata<
                    DepositBalanceOf<T>,
                    BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
                >,
                Blake2_128Concat,
            >(pallet_prefix, storage_item_prefix)
            .count(),
            0
        );
        let stored_data_old: Vec<_> = storage_key_iter::<
            OldAssetId,
            pallet_assets::AssetMetadata<
                DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        Self::set_temp_storage(stored_data_old, "metadata_map_stored_data_old");

        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;

        let asset_manager_storage_version =
            <pallet_asset_manager::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if asset_manager_storage_version != INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1 {
            return Err("AssetManager storage version is not 2, the migration wasn't executed.");
        }

        let assets_storage_version =
            <pallet_assets::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if assets_storage_version != INITIAL_PALLET_ASSETS_VERSION + 1 {
            return Err("Assets storage version is not 1, the migration wasn't executed.");
        }

        // We want to test that:
        // There are no entries in the new storage beforehand
        // The same number of mappings exist before and after
        //
        // We have to manually check that there are no entries
        // with the old storage keys, as the new u128 asset-id
        // would still decode into the old u32 values.

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let stored_data_new: Vec<_> =
            storage_key_iter::<NewAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        let stored_data_old: Vec<(OldAssetId, AssetLocation)> =
            Self::get_temp_storage("asset_id_location_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old.iter().for_each(|(key, value)| {
            let check = (*key as NewAssetId, value.clone());
            assert!(stored_data_new.contains(&check));
        });
        log::info!("✅ Storage migration for AssetManager's AssetIdLocation storage item has been executed successfully.");

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let stored_data_new: Vec<_> =
            storage_key_iter::<AssetLocation, NewAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .collect();
        let stored_data_old: Vec<(AssetLocation, OldAssetId)> =
            Self::get_temp_storage("location_asset_id_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old.iter().for_each(|(key, value)| {
            let check = (key.clone(), *value as NewAssetId);
            assert!(stored_data_new.contains(&check));
        });
        log::info!("✅ Storage migration for AssetManager's LocationAssetId storage item has been executed successfully.");

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            AssetRegistryMetadata<Balance>,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let stored_data_old: Vec<(OldAssetId, OldAssetRegistrarMetadata)> =
            Self::get_temp_storage("asset_id_metadata_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old.iter().for_each(|(key, value)| {
            let new_storage = (
                *key as NewAssetId,
                AssetRegistryMetadata {
                    metadata: AssetStorageMetadata {
                        name: value.name.clone(),
                        symbol: value.symbol.clone(),
                        decimals: value.decimals,
                        is_frozen: value.is_frozen,
                    },
                    min_balance: value.min_balance,
                    is_sufficient: value.is_sufficient,
                },
            );
            assert!(stored_data_new.contains(&new_storage));
        });
        log::info!("✅ Storage migration for AssetManager's AssetIdMetadata storage item has been executed successfully.");

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        let stored_data_new: Vec<_> = storage_key_iter::<NewAssetId, u128, Blake2_128Concat>(
            pallet_prefix,
            storage_item_prefix,
        )
        .collect();
        let stored_data_old: Vec<(OldAssetId, u128)> =
            Self::get_temp_storage("units_per_sec_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old.iter().for_each(|(key, value)| {
            let check = (*key as NewAssetId, *value);
            assert!(stored_data_new.contains(&check));
        });
        log::info!("✅ Storage migration for AssetManager's UnitsPerSecond storage item has been executed successfully.");

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        let next_asset_id: NewAssetId =
            get_storage_value::<NewAssetId>(pallet_prefix, storage_item_prefix, &[]).unwrap();
        let old_next_asset_id: u32 = Self::get_temp_storage("next_asset_id").unwrap();
        assert_eq!(old_next_asset_id as u128, next_asset_id);
        log::info!("✅ Storage migration for AssetManager's NextAssetId storage item has been executed successfully.");

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            pallet_assets::AssetDetails<
                <T as pallet_assets::Config>::Balance,
                <T as frame_system::Config>::AccountId,
                DepositBalanceOf<T>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let stored_data_old: Vec<AssetMapKVP<T>> =
            Self::get_temp_storage("asset_map_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old.iter().for_each(|(key, value)| {
            let check = (*key as NewAssetId, value.clone());
            assert!(stored_data_new.contains(&check));
        });
        log::info!(
            "✅ Storage migration for Assets' Asset storage item has been executed successfully."
        );

        // Account

        let mut stored_data_new: Vec<_> = Vec::new();
        Account::<T, ()>::iter().for_each(|(asset_id_key, account_id_key, value)| {
            stored_data_new.push((asset_id_key, account_id_key, value));
        });
        let stored_data_old: Vec<(
            OldAssetId,
            <T as frame_system::Config>::AccountId,
            AssetAccountOf<T, ()>,
        )> = Self::get_temp_storage("account_map_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old
            .iter()
            .for_each(|(asset_id_key, account_id_key, value)| {
                let check = (
                    *asset_id_key as NewAssetId,
                    account_id_key.clone(),
                    value.clone(),
                );
                assert!(stored_data_new.contains(&check));
            });
        log::info!(
            "✅ Storage migration for Assets' Account storage item has been executed successfully."
        );

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        let stored_data_new: Vec<_> = storage_key_iter::<
            NewAssetId,
            pallet_assets::AssetMetadata<
                DepositBalanceOf<T>,
                BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
            >,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .collect();
        let stored_data_old: Vec<MetadataMapKVP<T>> =
            Self::get_temp_storage("metadata_map_stored_data_old").unwrap();
        assert_eq!(stored_data_old.len(), stored_data_new.len());
        stored_data_old.iter().for_each(|(key, value)| {
            let check = (*key as NewAssetId, value.clone());
            assert!(stored_data_new.contains(&check));
        });
        log::info!(
            "✅ Storage migration for Assets' Metadata storage item has been executed successfully."
        );

        Ok(())
    }
}
