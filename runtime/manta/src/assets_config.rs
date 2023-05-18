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

use super::{
    weights, xcm_config::SelfReserve, AssetManager, Assets, Balances,
    NativeTokenExistentialDeposit, Runtime, RuntimeEvent, RuntimeOrigin,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetRegistry, AssetRegistryMetadata,
        AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::{ASSET_MANAGER_PALLET_ID, MANTA_DECIMAL},
    types::{AccountId, Balance, MantaAssetId},
};

use frame_support::{
    pallet_prelude::DispatchResult,
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU32},
    PalletId,
};
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
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = MantaAssetId;
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
    type RemoveItemsLimit = ConstU32<1000>;
    type AssetIdParameter = MantaAssetId;
    #[cfg(feature = "runtime-benchmarks")]
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureNever<AccountId>>;
    type CallbackHandle = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub struct MantaAssetRegistry;
impl BalanceType for MantaAssetRegistry {
    type Balance = Balance;
}
impl AssetIdType for MantaAssetRegistry {
    type AssetId = MantaAssetId;
}
impl AssetRegistry for MantaAssetRegistry {
    type Metadata = AssetStorageMetadata;
    type Error = sp_runtime::DispatchError;

    fn create_asset(
        asset_id: MantaAssetId,
        metadata: AssetStorageMetadata,
        min_balance: Balance,
        is_sufficient: bool,
    ) -> DispatchResult {
        Assets::force_create(
            RuntimeOrigin::root(),
            asset_id,
            sp_runtime::MultiAddress::Id(AssetManager::account_id()),
            is_sufficient,
            min_balance,
        )?;

        Assets::force_set_metadata(
            RuntimeOrigin::root(),
            asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }

    fn update_asset_metadata(
        asset_id: &MantaAssetId,
        metadata: AssetStorageMetadata,
    ) -> DispatchResult {
        Assets::force_set_metadata(
            RuntimeOrigin::root(),
            *asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }
}

parameter_types! {
    pub const StartNonNativeAssetId: MantaAssetId = 8;
    pub const NativeAssetId: MantaAssetId = 1;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(SelfReserve::get()));
    pub NativeAssetMetadata: AssetRegistryMetadata<Balance> = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"Manta".to_vec(),
            symbol: b"MANTA".to_vec(),
            decimals: MANTA_DECIMAL,
            is_frozen: false,
        },
        min_balance: NativeTokenExistentialDeposit::get(),
        is_sufficient: true,
    };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

pub type MantaConcreteFungibleLedger =
    NativeAndNonNative<Runtime, MantaAssetConfig, Balances, Assets>;

/// AssetConfig implementations for this runtime
#[derive(Clone, Eq, PartialEq)]
pub struct MantaAssetConfig;
impl LocationType for MantaAssetConfig {
    type Location = AssetLocation;
}
impl BalanceType for MantaAssetConfig {
    type Balance = Balance;
}
impl AssetIdType for MantaAssetConfig {
    type AssetId = MantaAssetId;
}
impl AssetConfig<Runtime> for MantaAssetConfig {
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type NativeAssetId = NativeAssetId;
    type AssetRegistryMetadata = AssetRegistryMetadata<Balance>;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata = AssetStorageMetadata;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = MantaConcreteFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = MantaAssetId;
    type Balance = Balance;
    type Location = AssetLocation;
    type AssetConfig = MantaAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId>;
    type SuspenderOrigin = EnsureRoot<AccountId>;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = weights::pallet_asset_manager::SubstrateWeight<Runtime>;
}
