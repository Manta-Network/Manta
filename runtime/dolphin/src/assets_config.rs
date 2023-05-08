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
    NativeTokenExistentialDeposit, Runtime, RuntimeEvent, RuntimeOrigin, TechnicalCollective,
    Timestamp, DOL,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetRegistry, AssetRegistryMetadata,
        AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::{
        ASSET_MANAGER_PALLET_ID, DOLPHIN_DECIMAL, MANTA_PAY_PALLET_ID, MANTA_SBT_PALLET_ID,
        WEIGHT_PER_MILLIS,
    },
    types::{AccountId, Balance, DolphinAssetId},
};

use frame_support::{
    pallet_prelude::DispatchResult,
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, EitherOfDiverse},
    weights::Weight,
    PalletId,
};
use frame_system::EnsureRoot;
use xcm::VersionedMultiLocation;

parameter_types! {
    pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
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
    type RemoveItemsLimit = ConstU32<1000>;
    type AssetIdParameter = DolphinAssetId;
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
    type AssetId = DolphinAssetId;
}
impl AssetRegistry for MantaAssetRegistry {
    type Metadata = AssetStorageMetadata;
    type Error = sp_runtime::DispatchError;

    fn create_asset(
        asset_id: DolphinAssetId,
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
        )?;

        Assets::force_asset_status(
            RuntimeOrigin::root(),
            asset_id,
            AssetManager::account_id().into(),
            AssetManager::account_id().into(),
            AssetManager::account_id().into(),
            AssetManager::account_id().into(),
            min_balance,
            is_sufficient,
            metadata.is_frozen,
        )
    }

    fn update_asset_metadata(
        asset_id: &DolphinAssetId,
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
    pub const StartNonNativeAssetId: DolphinAssetId = 8;
    pub const NativeAssetId: DolphinAssetId = 1;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(SelfReserve::get()));
    pub NativeAssetMetadata: AssetRegistryMetadata<Balance> = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"Dolphin".to_vec(),
            symbol: b"DOL".to_vec(),
            decimals: DOLPHIN_DECIMAL,
            is_frozen: false,
        },
        min_balance: NativeTokenExistentialDeposit::get(),
        is_sufficient: true,
    };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

pub type DolphinConcreteFungibleLedger =
    NativeAndNonNative<Runtime, DolphinAssetConfig, Balances, Assets>;

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
    type AssetRegistryMetadata = AssetRegistryMetadata<Balance>;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata = AssetStorageMetadata;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = DolphinConcreteFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
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
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_manta_pay::SubstrateWeight<Runtime>;
    type AssetConfig = DolphinAssetConfig;
    type PalletId = MantaPayPalletId;
}

parameter_types! {
    pub const MantaSbtPalletId: PalletId = MANTA_SBT_PALLET_ID;
    pub const MinimumWeightRemainInBlock: Weight = Weight::from_ref_time(25 * WEIGHT_PER_MILLIS);
}

impl pallet_manta_sbt::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = MantaSbtPalletId;
    type Currency = Balances;
    type MintsPerReserve = ConstU16<5>;
    type ReservePrice = ConstU128<DOL>;
    type SbtMetadataBound = ConstU32<300>;
    type AdminOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureMembers<AccountId, TechnicalCollective, 2>,
    >;
    type Now = Timestamp;
    type MinimumWeightRemainInBlock = MinimumWeightRemainInBlock;
    type RegistryBound = ConstU32<300>;
    type WeightInfo = ();
}
