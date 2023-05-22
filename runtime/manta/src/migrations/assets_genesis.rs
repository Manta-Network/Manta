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

#![allow(clippy::unnecessary_cast)]

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::StorageHasher;
use codec::Encode;
use core::marker::PhantomData;
use cumulus_primitives_core::ParaId;
#[allow(deprecated)]
use frame_support::migration::{
    clear_storage_prefix, get_storage_value, put_storage_value, storage_key_iter,
};
use frame_support::{
    dispatch::GetStorageVersion,
    pallet_prelude::Weight,
    traits::{Currency, Get, OnRuntimeUpgrade, StorageVersion},
    Blake2_128Concat,
};
use manta_primitives::{
    assets::{AssetConfig, AssetLocation, AssetRegistryMetadata},
    types::{Balance, MantaAssetId},
};
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;

pub type DepositBalanceOf<T, I = ()> = <<T as pallet_assets::Config<I>>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

pub const INITIAL_PALLET_ASSETS_MANAGER_VERSION: u16 = 0;
pub const INITIAL_PALLET_ASSETS_VERSION: u16 = 0;

pub struct AssetsGenesis<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for AssetsGenesis<T>
where
    T: pallet_asset_manager::Config + pallet_assets::Config,
    u128: From<<T as pallet_asset_manager::Config>::AssetId>,
{
    fn on_runtime_upgrade() -> Weight {
        let asset_manager_storage_version =
            <pallet_asset_manager::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        let assets_storage_version =
            <pallet_assets::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if asset_manager_storage_version != INITIAL_PALLET_ASSETS_MANAGER_VERSION
            || assets_storage_version != INITIAL_PALLET_ASSETS_VERSION
        {
            log::info!("Aborting migration due to unexpected on-chain storage versions for pallet-assets-manager: {:?} and pallet-assets: {:?}. Expectation was: {:?} and {:?}.", asset_manager_storage_version, assets_storage_version, INITIAL_PALLET_ASSETS_MANAGER_VERSION, INITIAL_PALLET_ASSETS_VERSION );
            return T::DbWeight::get().reads(2);
        }

        log::info!(target: "asset-manager", "Starting migration for AssetManager...");

        let pallet_prefix: &[u8] = b"AssetManager";
        let _ = clear_storage_prefix(pallet_prefix, b"AssetIdLocation", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"LocationAssetId", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"AssetIdMetadata", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"UnitsPerSecond", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"MinXcmFee", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"AllowedDestParaIds", b"", None, None);

        put_storage_value(
            pallet_prefix,
            b"NextAssetId",
            b"",
            <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(),
        );
        let asset_id = <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get();
        let metadata = <T::AssetConfig as AssetConfig<T>>::NativeAssetMetadata::get();
        let location = <T::AssetConfig as AssetConfig<T>>::NativeAssetLocation::get();
        put_storage_value(
            pallet_prefix,
            b"AssetIdLocation",
            &Blake2_128Concat::hash(&asset_id.encode()),
            location.clone(),
        );
        put_storage_value(
            pallet_prefix,
            b"LocationAssetId",
            &Blake2_128Concat::hash(&location.encode()),
            asset_id,
        );
        put_storage_value(
            pallet_prefix,
            b"AssetIdMetadata",
            &Blake2_128Concat::hash(&asset_id.encode()),
            metadata,
        );

        StorageVersion::new(INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1)
            .put::<pallet_asset_manager::Pallet<T>>();

        log::info!(target: "asset-manager", "✅ Storage migration for AssetManager has been executed successfully and storage version has been update to: {:?}.", INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1);

        log::info!(target: "assets", "Starting migration for pallet-assets...");

        let pallet_prefix: &[u8] = b"Assets";
        let _ = clear_storage_prefix(pallet_prefix, b"Asset", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"Metadata", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"Account", b"", None, None);
        let _ = clear_storage_prefix(pallet_prefix, b"Approval", b"", None, None);

        log::info!(target: "assets", "✅ Storage migration for Assets has been executed successfully and storage version has been update to: {:?}.", INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1);

        StorageVersion::new(INITIAL_PALLET_ASSETS_VERSION + 1).put::<pallet_assets::Pallet<T>>();

        T::BlockWeights::get().max_block // simply use the whole block
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let asset_manager_storage_version =
            <pallet_asset_manager::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if asset_manager_storage_version != INITIAL_PALLET_ASSETS_MANAGER_VERSION {
            return Err("AssetManager storage version is not 0, the migration won't be executed.");
        }

        let assets_storage_version =
            <pallet_assets::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if assets_storage_version != INITIAL_PALLET_ASSETS_VERSION {
            return Err("Assets storage version is not 0, the migration won't be executed.");
        }

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        assert_eq!(
            storage_key_iter::<MantaAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            2
        );

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        assert_eq!(
            storage_key_iter::<AssetLocation, MantaAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            2
        );

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        assert_eq!(
            storage_key_iter::<MantaAssetId, AssetRegistryMetadata<Balance>, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix
            )
            .count(),
            2
        );

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        assert_eq!(
            storage_key_iter::<MantaAssetId, u128, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );

        // MinXcmFee

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"MinXcmFee";
        assert_eq!(
            storage_key_iter::<AssetLocation, u128, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );

        // AllowedDestParaIds

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AllowedDestParaIds";
        assert_eq!(
            storage_key_iter::<ParaId, u32, Blake2_128Concat>(pallet_prefix, storage_item_prefix,)
                .count(),
            1
        );

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        assert_eq!(
            get_storage_value::<MantaAssetId>(pallet_prefix, storage_item_prefix, &[]).unwrap(),
            2
        );

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        assert_eq!(
            storage_key_iter::<
                MantaAssetId,
                pallet_assets::AssetDetails<
                    <T as pallet_assets::Config>::Balance,
                    <T as frame_system::Config>::AccountId,
                    DepositBalanceOf<T>,
                >,
                Blake2_128Concat,
            >(pallet_prefix, storage_item_prefix)
            .count(),
            2
        );

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        assert_eq!(
            storage_key_iter::<
                MantaAssetId,
                pallet_assets::AssetMetadata<
                    DepositBalanceOf<T>,
                    BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
                >,
                Blake2_128Concat,
            >(pallet_prefix, storage_item_prefix)
            .count(),
            2
        );

        log::info!(target: "assets", "✅ Migrations pre checks finished!");

        Ok(().encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        let asset_manager_storage_version =
            <pallet_asset_manager::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if asset_manager_storage_version != INITIAL_PALLET_ASSETS_MANAGER_VERSION + 1 {
            return Err("AssetManager storage version is not 1, the migration was not executed.");
        }

        let assets_storage_version =
            <pallet_assets::Pallet<T> as GetStorageVersion>::on_chain_storage_version();
        if assets_storage_version != INITIAL_PALLET_ASSETS_VERSION + 1 {
            return Err("Assets storage version is not 1, the migration was not executed.");
        }

        // AssetIdLocation

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        assert_eq!(
            storage_key_iter::<MantaAssetId, AssetLocation, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            1
        );

        // LocationAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        assert_eq!(
            storage_key_iter::<AssetLocation, MantaAssetId, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            1
        );

        // AssetIdMetadata

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdMetadata";
        assert_eq!(
            storage_key_iter::<MantaAssetId, AssetRegistryMetadata<Balance>, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix
            )
            .count(),
            1
        );

        // UnitsPerSecond

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"UnitsPerSecond";
        assert_eq!(
            storage_key_iter::<MantaAssetId, u128, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );

        // MinXcmFee

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"MinXcmFee";
        assert_eq!(
            storage_key_iter::<AssetLocation, u128, Blake2_128Concat>(
                pallet_prefix,
                storage_item_prefix,
            )
            .count(),
            0
        );

        // AllowedDestParaIds

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AllowedDestParaIds";
        assert_eq!(
            storage_key_iter::<ParaId, u32, Blake2_128Concat>(pallet_prefix, storage_item_prefix,)
                .count(),
            0
        );

        // NextAssetId

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"NextAssetId";
        assert_eq!(
            get_storage_value::<MantaAssetId>(pallet_prefix, storage_item_prefix, &[]).unwrap(),
            u128::from(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get())
        );

        // Asset

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Asset";
        assert_eq!(
            storage_key_iter::<
                MantaAssetId,
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

        // Metadata

        let pallet_prefix: &[u8] = b"Assets";
        let storage_item_prefix: &[u8] = b"Metadata";
        assert_eq!(
            storage_key_iter::<
                MantaAssetId,
                pallet_assets::AssetMetadata<
                    DepositBalanceOf<T>,
                    BoundedVec<u8, <T as pallet_assets::Config>::StringLimit>,
                >,
                Blake2_128Concat,
            >(pallet_prefix, storage_item_prefix)
            .count(),
            0
        );

        log::info!(target: "assets", "✅ Migrations post checks finished!");

        Ok(())
    }
}
