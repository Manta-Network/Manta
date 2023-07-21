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
    weights, xcm_config::SelfReserve, AssetManager, Assets, Balances, CouncilCollective,
    NativeTokenExistentialDeposit, Runtime, RuntimeEvent, RuntimeOrigin, TechnicalCollective,
    Timestamp, MANTA,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetRegistry, AssetRegistryMetadata,
        AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::{ASSET_MANAGER_PALLET_ID, MANTA_DECIMAL, MANTA_PAY_PALLET_ID, MANTA_SBT_PALLET_ID},
    types::{AccountId, Balance, MantaAssetId, Signature, Signer},
};

use frame_support::{
    pallet_prelude::DispatchResult,
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, EitherOfDiverse},
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

type StringLimit = ConstU32<50>;

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
    type StringLimit = StringLimit;
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
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = MantaConcreteFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
    type PermissionlessStartId = ConstU128<1_000_000_000>;
    type TokenNameMaxLen = StringLimit;
    type TokenSymbolMaxLen = ConstU32<10>;
    type PermissionlessAssetRegistryCost = ConstU128<{ 50 * MANTA }>;
    type RuntimeEvent = RuntimeEvent;
    type AssetId = MantaAssetId;
    type Location = AssetLocation;
    type AssetConfig = MantaAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId>;
    type SuspenderOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
    >;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = weights::pallet_asset_manager::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const MantaPayPalletId: PalletId = MANTA_PAY_PALLET_ID;
    pub const MantaSbtPalletId: PalletId = MANTA_SBT_PALLET_ID;
}

impl pallet_manta_pay::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_manta_pay::SubstrateWeight<Runtime>;
    type AssetConfig = MantaAssetConfig;
    type PalletId = MantaPayPalletId;
}

impl pallet_manta_sbt::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = MantaSbtPalletId;
    type Currency = Balances;
    type MintsPerReserve = ConstU16<5>;
    type ReservePrice = ConstU128<{ 50 * MANTA }>;
    type SbtMetadataBound = ConstU32<300>;
    type RegistryBound = ConstU32<300>;
    type AdminOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>,
    >;
    type Now = Timestamp;
    type Signature = Signature;
    type PublicKey = Signer;
    type WeightInfo = weights::pallet_manta_sbt::SubstrateWeight<Runtime>;
}
