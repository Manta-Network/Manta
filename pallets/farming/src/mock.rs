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

// Ensure we're `no_std` when compiling for Wasm.

#![cfg(test)]
#![allow(non_upper_case_globals)]

use frame_support::{
    dispatch::DispatchResult,
    ord_parameter_types, parameter_types,
    traits::{AsEnsureOriginWithArg, EitherOfDiverse, GenesisBuild},
    PalletId,
};
use frame_system::{EnsureNever, EnsureRoot, EnsureSignedBy};
use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetRegistry, AssetRegistryMetadata,
        AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::ASSET_MANAGER_PALLET_ID,
    currencies::Currencies,
    types::CalamariAssetId,
};
use sp_core::{ConstU32, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};
use xcm::{
    prelude::{Parachain, X1},
    v2::MultiLocation,
    VersionedMultiLocation,
};

use crate as pallet_farming;

pub type AccountId = AccountId32;
pub type Balance = u128;

pub const KSM: CalamariAssetId = 8;
pub const KMA: CalamariAssetId = 1;

pub const ALICE: AccountId = AccountId32::new([0u8; 32]);
pub const BOB: AccountId = AccountId32::new([1u8; 32]);
pub const CHARLIE: AccountId = AccountId32::new([3u8; 32]);
pub const TREASURY_ACCOUNT: AccountId = AccountId32::new([9u8; 32]);

frame_support::construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Assets: pallet_assets::{Pallet, Storage, Config<T>, Event<T>},
        AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
        Farming: pallet_farming::{Pallet, Call, Storage, Event<T>}
    }
);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}
impl pallet_balances::Config for Runtime {
    type AccountStore = frame_system::Pallet<Runtime>;
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    // Does not really matter as this will be only called by root
    pub const AssetDeposit: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = CalamariAssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId32>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
    type RemoveItemsLimit = ConstU32<1000>;
    type AssetIdParameter = CalamariAssetId;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId32>>;
    type CallbackHandle = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub struct MantaAssetRegistry;
impl BalanceType for MantaAssetRegistry {
    type Balance = Balance;
}
impl AssetIdType for MantaAssetRegistry {
    type AssetId = CalamariAssetId;
}
impl AssetRegistry for MantaAssetRegistry {
    type Metadata = AssetStorageMetadata;
    type Error = sp_runtime::DispatchError;

    fn create_asset(
        asset_id: CalamariAssetId,
        metadata: AssetStorageMetadata,
        min_balance: Balance,
        is_sufficient: bool,
    ) -> DispatchResult {
        Assets::force_create(
            RuntimeOrigin::root(),
            asset_id,
            AssetManager::account_id(),
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
            AssetManager::account_id(),
            AssetManager::account_id(),
            AssetManager::account_id(),
            AssetManager::account_id(),
            min_balance,
            is_sufficient,
            metadata.is_frozen,
        )
    }

    fn update_asset_metadata(
        asset_id: &CalamariAssetId,
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
    pub const DummyAssetId: CalamariAssetId = 0;
    pub const NativeAssetId: CalamariAssetId = 1;
    pub const StartNonNativeAssetId: CalamariAssetId = 8;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(MultiLocation::new(1, X1(Parachain(1024)))));
    pub NativeAssetMetadata: AssetRegistryMetadata<Balance> = AssetRegistryMetadata {
        metadata: AssetStorageMetadata {
            name: b"Calamari".to_vec(),
            symbol: b"KMA".to_vec(),
            decimals: 12,
            is_frozen: false,
        },
        min_balance: 1u128,
        is_sufficient: true,
    };
    pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

/// AssetConfig implementations for this runtime
#[derive(Clone, Eq, PartialEq)]
pub struct MantaAssetConfig;
impl LocationType for MantaAssetConfig {
    type Location = AssetLocation;
}
impl AssetIdType for MantaAssetConfig {
    type AssetId = CalamariAssetId;
}
impl BalanceType for MantaAssetConfig {
    type Balance = Balance;
}
impl AssetConfig<Runtime> for MantaAssetConfig {
    type NativeAssetId = NativeAssetId;
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type AssetRegistryMetadata = AssetRegistryMetadata<Balance>;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type StorageMetadata = AssetStorageMetadata;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = NativeAndNonNative<Runtime, MantaAssetConfig, Balances, Assets>;
}

impl pallet_asset_manager::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = CalamariAssetId;
    type Balance = Balance;
    type Location = AssetLocation;
    type AssetConfig = MantaAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId32>;
    type SuspenderOrigin = EnsureRoot<AccountId32>;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = ();
}

parameter_types! {
    pub const FarmingKeeperPalletId: PalletId = PalletId(*b"bf/fmkpr");
    pub const FarmingRewardIssuerPalletId: PalletId = PalletId(*b"bf/fmrir");
    pub const TreasuryAccount: AccountId32 = TREASURY_ACCOUNT;
}

ord_parameter_types! {
    pub const One: AccountId = ALICE;
}

type MantaCurrencies = Currencies<Runtime, MantaAssetConfig, Balances, Assets>;

impl pallet_farming::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CurrencyId = CalamariAssetId;
    type MultiCurrency = MantaCurrencies;
    type ControlOrigin = EitherOfDiverse<EnsureRoot<AccountId>, EnsureSignedBy<One, AccountId>>;
    type TreasuryAccount = TreasuryAccount;
    type Keeper = FarmingKeeperPalletId;
    type RewardIssuer = FarmingRewardIssuerPalletId;
    type WeightInfo = ();
}

#[derive(Default)]
pub struct ExtBuilder {
    endowed_accounts: Vec<(AccountId, CalamariAssetId, Balance)>,
}

impl ExtBuilder {
    pub fn balances(
        mut self,
        endowed_accounts: Vec<(AccountId, CalamariAssetId, Balance)>,
    ) -> Self {
        self.endowed_accounts = endowed_accounts;
        self
    }

    pub fn one_hundred_for_alice_n_bob(self) -> Self {
        self.balances(vec![
            (ALICE, 1, 3000),
            (BOB, 1, 400_000),
            (CHARLIE, 1, 1),
            (ALICE, KSM, 3000),
            (BOB, KSM, 10_000_000),
        ])
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        let initial_asset_accounts = self
            .endowed_accounts
            .clone()
            .into_iter()
            .filter(|(_, asset_id, _)| *asset_id != 1)
            .map(|(account_id, asset_id, initial_balance)| (asset_id, account_id, initial_balance))
            .collect::<Vec<_>>();

        let config: pallet_assets::GenesisConfig<Runtime> = pallet_assets::GenesisConfig {
            assets: vec![
                // id, owner, is_sufficient, min_balance
                (KSM, ALICE, true, 1),
            ],
            metadata: vec![
                // id, name, symbol, decimals
                (KSM, "KSM".into(), "Kusama".into(), 12),
            ],
            accounts: initial_asset_accounts,
        };
        config.assimilate_storage(&mut t).unwrap();

        pallet_asset_manager::GenesisConfig::<Runtime> {
            start_id: <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get() + 1,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self
                .endowed_accounts
                .into_iter()
                .filter(|(_, asset_id, _)| *asset_id == 1)
                .map(|(account_id, _, initial_balance)| (account_id, initial_balance))
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        sp_io::TestExternalities::new(t)
    }
}
