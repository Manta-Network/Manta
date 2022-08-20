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
    weights, xcm_config::SelfReserve, AssetManager, Assets, Balances, Event,
    NativeTokenExistentialDeposit, Origin, Runtime,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetLocation, AssetRegistrar, AssetRegistrarMetadata, AssetStorageMetadata,
        ConcreteFungibleLedger,
    },
    constants::{ASSET_MANAGER_PALLET_ID, CALAMARI_DECIMAL},
    types::{AccountId, AssetId, Balance},
};

use frame_support::{pallet_prelude::DispatchResult, parameter_types, traits::ConstU32, PalletId};

use frame_system::EnsureRoot;

use xcm::VersionedMultiLocation;

parameter_types! {
    // Does not really matter as this will be only called by root
    pub const AssetDeposit: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = AssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = weights::pallet_assets::SubstrateWeight<Runtime>;
}

pub struct CalamariAssetRegistrar;
impl AssetRegistrar<Runtime, CalamariAssetConfig> for CalamariAssetRegistrar {
    fn create_asset(
        asset_id: AssetId,
        min_balance: Balance,
        metadata: AssetStorageMetadata,
        is_sufficient: bool,
    ) -> DispatchResult {
        Assets::force_create(
            Origin::root(),
            asset_id,
            sp_runtime::MultiAddress::Id(AssetManager::account_id()),
            is_sufficient,
            min_balance,
        )?;

        Assets::force_set_metadata(
            Origin::root(),
            asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }

    fn update_asset_metadata(asset_id: AssetId, metadata: AssetStorageMetadata) -> DispatchResult {
        Assets::force_set_metadata(
            Origin::root(),
            asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }
}

parameter_types! {
    pub const StartNonNativeAssetId: AssetId = 8;
    pub const NativeAssetId: AssetId = 1;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(SelfReserve::get()));
    pub NativeAssetMetadata: AssetRegistrarMetadata = AssetRegistrarMetadata {
        name: b"Calamari".to_vec(),
        symbol: b"KMA".to_vec(),
        decimals: CALAMARI_DECIMAL,
        min_balance: NativeTokenExistentialDeposit::get(),
        evm_address: None,
        is_frozen: false,
        is_sufficient: true,
    };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

pub type CalamariConcreteFungibleLedger =
    ConcreteFungibleLedger<Runtime, CalamariAssetConfig, Balances, Assets>;

#[derive(Clone, Eq, PartialEq)]
pub struct CalamariAssetConfig;

impl AssetConfig<Runtime> for CalamariAssetConfig {
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type NativeAssetId = NativeAssetId;
    type AssetRegistrarMetadata = AssetRegistrarMetadata;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata = AssetStorageMetadata;
    type AssetLocation = AssetLocation;
    type AssetRegistrar = CalamariAssetRegistrar;
    type FungibleLedger = CalamariConcreteFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
    type Event = Event;
    type AssetConfig = CalamariAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId>;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = weights::pallet_asset_manager::SubstrateWeight<Runtime>;
}
