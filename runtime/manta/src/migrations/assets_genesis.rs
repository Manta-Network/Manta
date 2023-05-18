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

use crate::{
    assets_config::MantaAssetConfig,
    sp_api_hidden_includes_construct_runtime::hidden_include::StorageHasher,
};
use codec::{Decode, Encode};
use core::marker::PhantomData;
#[allow(deprecated)]
use frame_support::migration::{
    clear_storage_prefix, get_storage_value, put_storage_value, storage_key_iter,
    take_storage_value,
};
use frame_support::{
    dispatch::GetStorageVersion,
    pallet_prelude::Weight,
    storage_alias,
    traits::{Currency, Get, OnRuntimeUpgrade, StorageVersion},
    Blake2_128Concat,
};
use manta_primitives::{
    assets::{AssetConfig, AssetLocation, AssetRegistryMetadata, AssetStorageMetadata},
    types::Balance,
};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;

pub const INITIAL_PALLET_ASSETS_MANAGER_VERSION: u16 = 1;
pub const INITIAL_PALLET_ASSETS_VERSION: u16 = 0;

pub struct AssetsGenesis<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for AssetsGenesis<T>
where
    T: pallet_asset_manager::Config + pallet_assets::Config,
{
    fn on_runtime_upgrade() -> Weight {
        let pallet_prefix: &[u8] = b"AssetManager";
        clear_storage_prefix(pallet_prefix, b"AssetIdLocation", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"LocationAssetId", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"AssetIdMetadata", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"UnitsPerSecond", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"MinXcmFee", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"AllowedDestParaIds", b"", None, None);

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

        let pallet_prefix: &[u8] = b"Assets";
        clear_storage_prefix(pallet_prefix, b"Asset", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"Metadata", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"Account", b"", None, None);
        clear_storage_prefix(pallet_prefix, b"Approval", b"", None, None);

        Weight::from_ref_time(0)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        Ok(().encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        Ok(())
    }
}
