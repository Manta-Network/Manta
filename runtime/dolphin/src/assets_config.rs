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

use super::{
    cDOL, weights, xcm_config::SelfReserve, AssetManager, Assets, Balances, Event,
    NativeTokenExistentialDeposit, Origin, Runtime, Uniques, DOL,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetIdMapping, AssetIdType, AssetLocation, AssetRegistry,
        AssetRegistryMetadata, AssetStorageMetadata,
        AssetStorageMetadata::{Fungible, NonFungible, SBT},
        BalanceType, FungibleAssetStorageMetadata, LocationType, NativeAndNonNative,
    },
    constants::{ASSET_MANAGER_PALLET_ID, DOLPHIN_DECIMAL, MANTA_PAY_PALLET_ID},
    nft::NonFungibleAsset,
    types::{AccountId, Balance, DolphinAssetId},
};

use frame_support::{
    pallet_prelude::DispatchResult, parameter_types, traits::AsEnsureOriginWithArg, PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_runtime::traits::ConstU32;
use xcm::VersionedMultiLocation;

parameter_types! {
    pub const AssetDeposit: Balance = 1000 * DOL;
    pub const AssetAccountDeposit: Balance = NativeTokenExistentialDeposit::get();
    pub const ApprovalDeposit: Balance = 10 * cDOL;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = DOL;
    pub const MetadataDepositPerByte: Balance = cDOL;
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = DolphinAssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = weights::pallet_assets::SubstrateWeight<Runtime>;
}

pub struct MantaAssetRegistry;
impl BalanceType for MantaAssetRegistry {
    type Balance = Balance;
}
impl AssetIdType for MantaAssetRegistry {
    type AssetId = DolphinAssetId;
}

impl AssetRegistry<Runtime> for MantaAssetRegistry {
    type Metadata =
        AssetStorageMetadata<Balance, DolphinAssetId, DolphinAssetId, DolphinAssetId, AccountId>;
    type Error = sp_runtime::DispatchError;

    fn create_asset(
        who: Origin,
        asset_id: DolphinAssetId,
        admin: AccountId,
        metadata: Self::Metadata,
    ) -> DispatchResult {
        match metadata {
            Fungible(meta_registry) => {
                let meta = meta_registry.metadata;
                Assets::create(
                    who.clone(),
                    asset_id,
                    sp_runtime::MultiAddress::Id(admin),
                    meta_registry.min_balance,
                )?;

                Assets::set_metadata(who, asset_id, meta.name, meta.symbol, meta.decimals)
            }
            NonFungible(_meta) => Ok(()),
            SBT(_meta) => Ok(()),
        }
    }

    fn force_create_asset(asset_id: DolphinAssetId, metadata: Self::Metadata) -> DispatchResult {
        match metadata {
            Fungible(meta_registry) => {
                let meta = meta_registry.metadata;
                Assets::force_create(
                    Origin::root(),
                    asset_id,
                    sp_runtime::MultiAddress::Id(AssetManager::account_id()),
                    meta_registry.is_sufficient,
                    meta_registry.min_balance,
                )?;

                Assets::force_set_metadata(
                    Origin::root(),
                    asset_id,
                    meta.name,
                    meta.symbol,
                    meta.decimals,
                    meta.is_frozen,
                )
            }
            NonFungible(_meta) => Ok(()),
            SBT(_meta) => Ok(()),
        }
    }

    fn force_update_metadata(
        asset_id: &DolphinAssetId,
        metadata: Self::Metadata,
    ) -> DispatchResult {
        match metadata {
            Fungible(meta_registry) => {
                let meta = meta_registry.metadata;
                Assets::force_set_metadata(
                    Origin::root(),
                    *asset_id,
                    meta.name,
                    meta.symbol,
                    meta.decimals,
                    meta.is_frozen,
                )
            }
            NonFungible(_meta) => Ok(()),
            SBT(_meta) => Ok(()),
        }
    }

    fn get_metadata(
        asset_id: &DolphinAssetId,
    ) -> Option<
        AssetStorageMetadata<Balance, DolphinAssetId, DolphinAssetId, DolphinAssetId, AccountId>,
    > {
        AssetManager::get_metadata(asset_id)
    }
}

parameter_types! {
    pub const StartNonNativeAssetId: DolphinAssetId = 8;
    pub const NativeAssetId: DolphinAssetId = 1;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(SelfReserve::get()));
    pub NativeAssetMetadata: AssetRegistryMetadata<Balance, DolphinAssetId> =
        AssetRegistryMetadata {
            metadata: FungibleAssetStorageMetadata {
                name: b"Dolphin".to_vec(),
                symbol: b"DOL".to_vec(),
                decimals: DOLPHIN_DECIMAL,
                is_frozen: false,
            },
            asset_id: 1,
            min_balance: NativeTokenExistentialDeposit::get(),
            is_sufficient: true,
        };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

pub type DolphinConcreteFungibleLedger =
    NativeAndNonNative<Runtime, DolphinAssetConfig, Balances, Assets>;

pub type DolphinNonFungibleLedger = NonFungibleAsset<Runtime, DolphinAssetId, Balance, Uniques>;

/// AssetConfig implementations for this runtime
#[derive(Clone, Eq, PartialEq)]
pub struct DolphinAssetConfig;
impl LocationType for DolphinAssetConfig {
    type Location = AssetLocation;
}
impl BalanceType for DolphinAssetConfig {
    type Balance = Balance;
}
impl AssetIdType for DolphinAssetConfig {
    type AssetId = DolphinAssetId;
}
impl AssetConfig<Runtime> for DolphinAssetConfig {
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type NativeAssetId = NativeAssetId;
    type AssetRegistryMetadata =
        AssetStorageMetadata<Balance, DolphinAssetId, DolphinAssetId, DolphinAssetId, AccountId>;
    type IdentifierMapping =
        AssetIdMapping<DolphinAssetId, DolphinAssetId, DolphinAssetId, AccountId>;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata =
        AssetStorageMetadata<Balance, DolphinAssetId, DolphinAssetId, DolphinAssetId, AccountId>;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = DolphinConcreteFungibleLedger;
    type NonFungibleLedger = DolphinNonFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
    type Event = Event;
    type AssetId = DolphinAssetId;
    type Balance = Balance;
    type Location = AssetLocation;
    type AssetConfig = DolphinAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId>;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = weights::pallet_asset_manager::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const MantaPayPalletId: PalletId = MANTA_PAY_PALLET_ID;
}

impl pallet_manta_pay::Config for Runtime {
    type Event = Event;
    type WeightInfo = weights::pallet_manta_pay::SubstrateWeight<Runtime>;
    type AssetConfig = DolphinAssetConfig;
    type PalletId = MantaPayPalletId;
}

parameter_types! {
    pub const CollectionDeposit: Balance = 100;
    pub const ItemDeposit: Balance = 1;
    pub const KeyLimit: u32 = 32;
    pub const ValueLimit: u32 = 256;
}

impl pallet_uniques::Config for Runtime {
    type Event = Event;
    type CollectionId = DolphinAssetId;
    type ItemId = DolphinAssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type CollectionDeposit = CollectionDeposit;
    type ItemDeposit = ItemDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type AttributeDepositBase = MetadataDepositBase;
    type DepositPerByte = MetadataDepositPerByte;
    type StringLimit = ConstU32<1000>;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = pallet_uniques::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type Locker = ();
}
