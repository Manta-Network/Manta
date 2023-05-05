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
    weights, xcm_config::SelfReserve, AssetManager, Assets, Balances, Event,
    NativeTokenExistentialDeposit, Origin, Runtime, TechnicalCollective, Timestamp, KMA,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetRegistry, AssetRegistryMetadata,
        AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::{
        ASSET_MANAGER_PALLET_ID, CALAMARI_DECIMAL, MANTA_PAY_PALLET_ID, MANTA_SBT_PALLET_ID,
        WEIGHT_PER_MILLIS,
    },
    types::{AccountId, Balance, CalamariAssetId},
};

use frame_support::{
    pallet_prelude::DispatchResult,
    parameter_types,
    traits::{ConstU128, ConstU16, ConstU32, ConstU64, EitherOfDiverse},
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
    type Event = Event;
    type Balance = Balance;
    type AssetId = CalamariAssetId;
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

pub struct CalamariAssetRegistry;
impl BalanceType for CalamariAssetRegistry {
    type Balance = Balance;
}
impl AssetIdType for CalamariAssetRegistry {
    type AssetId = CalamariAssetId;
}
impl AssetRegistry for CalamariAssetRegistry {
    type Metadata = AssetStorageMetadata;
    type Error = sp_runtime::DispatchError;

    fn create_asset(
        asset_id: CalamariAssetId,
        metadata: AssetStorageMetadata,
        min_balance: Balance,
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

    fn update_asset_metadata(
        asset_id: &CalamariAssetId,
        metadata: AssetStorageMetadata,
    ) -> DispatchResult {
        Assets::force_set_metadata(
            Origin::root(),
            *asset_id,
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            metadata.is_frozen,
        )
    }
}

parameter_types! {
    pub const StartNonNativeAssetId: CalamariAssetId = 8;
    pub const NativeAssetId: CalamariAssetId = 1;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(SelfReserve::get()));
    pub NativeAssetMetadata: AssetRegistryMetadata<Balance> = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"Calamari".to_vec(),
            symbol: b"KMA".to_vec(),
            decimals: CALAMARI_DECIMAL,
            is_frozen: false,
        },
        min_balance: NativeTokenExistentialDeposit::get(),
        is_sufficient: true,
    };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

pub type CalamariConcreteFungibleLedger =
    NativeAndNonNative<Runtime, CalamariAssetConfig, Balances, Assets>;

/// AssetConfig implementations for this runtime
#[derive(Clone, Eq, PartialEq)]
pub struct CalamariAssetConfig;
impl LocationType for CalamariAssetConfig {
    type Location = AssetLocation;
}
impl BalanceType for CalamariAssetConfig {
    type Balance = Balance;
}
impl AssetIdType for CalamariAssetConfig {
    type AssetId = CalamariAssetId;
}
impl AssetConfig<Runtime> for CalamariAssetConfig {
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type NativeAssetId = NativeAssetId;
    type AssetRegistryMetadata = AssetRegistryMetadata<Balance>;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata = AssetStorageMetadata;
    type AssetRegistry = CalamariAssetRegistry;
    type FungibleLedger = CalamariConcreteFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
    type Event = Event;
    type AssetId = CalamariAssetId;
    type Balance = Balance;
    type Location = AssetLocation;
    type AssetConfig = CalamariAssetConfig;
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
    type AssetConfig = CalamariAssetConfig;
    type PalletId = MantaPayPalletId;
}

parameter_types! {
    pub const MantaSbtPalletId: PalletId = MANTA_SBT_PALLET_ID;
}

impl pallet_manta_sbt::Config for Runtime {
    type Event = Event;
    type PalletId = MantaSbtPalletId;
    type Currency = Balances;
    type MintsPerReserve = ConstU16<5>;
    type ReservePrice = ConstU128<{ 100_000 * KMA }>;
    type SbtMetadataBound = ConstU32<300>;
    type RegistryBound = ConstU32<300>;
    type AdminOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>,
    >;
    type MinimumWeightRemainInBlock = ConstU64<{ 25 * WEIGHT_PER_MILLIS }>;
    type Now = Timestamp;
    type WeightInfo = weights::pallet_manta_sbt::SubstrateWeight<Runtime>;
}
