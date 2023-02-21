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

//! Mock for MantaSbt

use frame_support::{
    parameter_types,
    traits::{ConstU128, ConstU16, ConstU32, Everything, GenesisBuild, IsInVec},
    PalletId,
};
use frame_system::{EnsureRoot, RawOrigin};
use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetMetadata, AssetRegistry, BalanceType,
        FungibleAssetRegistryMetadata, FungibleAssetStorageMetadata, LocationType,
        NativeAndNonNative,
    },
    constants::{ASSET_MANAGER_PALLET_ID, MANTA_PAY_PALLET_ID, MANTA_SBT_PALLET_ID},
    types::{Balance, BlockNumber, Header},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32, DispatchResult,
};
use xcm::{
    prelude::{Parachain, X1},
    v1::MultiLocation,
    VersionedMultiLocation,
};

use crate::StandardAssetId;

pub const ALICE: AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        MantaSBTPallet: crate::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Assets: pallet_assets::{Pallet, Storage, Event<T>},
        AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
        TransactionPause: pallet_tx_pause::{Pallet, Storage, Call, Event<T>},
        MantaPay: pallet_manta_pay::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const SS58Prefix: u8 = manta_primitives::constants::CALAMARI_SS58PREFIX;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const CollectionDeposit: Balance = 100;
    pub const ItemDeposit: Balance = 1;
    pub const KeyLimit: u32 = 32;
    pub const ValueLimit: u32 = 256;
    pub const MetadataDepositBase: Balance = 1_000;
    pub const MetadataDepositPerByte: Balance = 10;
}

parameter_types! {
    pub const MantaSBTPalletId: PalletId = MANTA_SBT_PALLET_ID;
    pub const CustodialAccount: AccountId32 = ALICE;
}

impl crate::Config for Test {
    type Event = Event;
    type WeightInfo = crate::weights::SubstrateWeight<Test>;
    type Balance = Balance;
    type PalletId = MantaSBTPalletId;
    type Currency = Balances;
    type MintsPerReserve = ConstU16<5>;
    type ReservePrice = ConstU128<1000>;
    type UpdateMetadata = AssetManager;
}

parameter_types! {
    // Does not really matter as this will be only called by root
    pub const AssetDeposit: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 50;
}

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type AssetId = StandardAssetId;
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
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Test>;
}

pub struct MantaAssetRegistry;
impl BalanceType for MantaAssetRegistry {
    type Balance = Balance;
}
impl AssetIdType for MantaAssetRegistry {
    type AssetId = StandardAssetId;
}
impl AssetRegistry for MantaAssetRegistry {
    type Metadata = AssetMetadata<Balance>;
    type Error = sp_runtime::DispatchError;

    fn create_asset(asset_id: StandardAssetId, metadata: AssetMetadata<Balance>) -> DispatchResult {
        match metadata {
            AssetMetadata::FT(meta) => {
                Assets::force_create(
                    Origin::root(),
                    asset_id,
                    AssetManager::account_id(),
                    meta.is_sufficient,
                    meta.min_balance,
                )?;

                Assets::force_set_metadata(
                    Origin::root(),
                    asset_id,
                    meta.metadata.name,
                    meta.metadata.symbol,
                    meta.metadata.decimals,
                    meta.metadata.is_frozen,
                )
            }
            AssetMetadata::SBT(_) => Ok(()),
        }
    }

    fn update_asset_metadata(
        asset_id: &StandardAssetId,
        metadata: AssetMetadata<Balance>,
    ) -> DispatchResult {
        match metadata {
            AssetMetadata::FT(meta) => Assets::force_set_metadata(
                Origin::root(),
                *asset_id,
                meta.metadata.name,
                meta.metadata.symbol,
                meta.metadata.decimals,
                meta.metadata.is_frozen,
            ),
            AssetMetadata::SBT(_) => Ok(()),
        }
    }
}

parameter_types! {
    pub const DummyAssetId: StandardAssetId = 0;
    pub const NativeAssetId: StandardAssetId = 1;
    pub const StartNonNativeAssetId: StandardAssetId = 8;
    pub NativeAssetLocation: AssetLocation = AssetLocation(
        VersionedMultiLocation::V1(MultiLocation::new(1, X1(Parachain(1024)))));
    pub NativeAssetMetadata: FungibleAssetRegistryMetadata<Balance> = FungibleAssetRegistryMetadata {
        metadata: FungibleAssetStorageMetadata {
            name: b"Dolphin".to_vec(),
            symbol: b"DOL".to_vec(),
            decimals: 18,
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
    type AssetId = StandardAssetId;
}
impl BalanceType for MantaAssetConfig {
    type Balance = Balance;
}
impl AssetConfig<Test> for MantaAssetConfig {
    type NativeAssetId = NativeAssetId;
    type StartNonNativeAssetId = StartNonNativeAssetId;
    type NativeAssetLocation = NativeAssetLocation;
    type NativeAssetMetadata = NativeAssetMetadata;
    type AssetRegistry = MantaAssetRegistry;
    type FungibleLedger = NativeAndNonNative<Test, MantaAssetConfig, Balances, Assets>;
}

impl pallet_asset_manager::Config for Test {
    type Event = Event;
    type AssetId = StandardAssetId;
    type Balance = Balance;
    type Location = AssetLocation;
    type AssetConfig = MantaAssetConfig;
    type ModifierOrigin = EnsureRoot<AccountId32>;
    type PalletId = AssetManagerPalletId;
    type WeightInfo = ();
}

pub struct MantaPaySuspensionManager;
impl pallet_manta_pay::SuspendMantaPay for MantaPaySuspensionManager {
    fn suspend_manta_pay_execution() {
        let _ = TransactionPause::pause_transactions(
            RawOrigin::Root.into(),
            vec![(
                b"MantaPay".to_vec(),
                vec![
                    b"to_private".to_vec(),
                    b"private_transfer".to_vec(),
                    b"to_public".to_vec(),
                ],
            )],
        );
    }
}

parameter_types! {
    pub const MantaPayPalletId: PalletId = MANTA_PAY_PALLET_ID;
}

impl pallet_manta_pay::Config for Test {
    type Event = Event;
    type WeightInfo = pallet_manta_pay::weights::SubstrateWeight<Test>;
    type PalletId = MantaPayPalletId;
    type AssetConfig = MantaAssetConfig;
    type Suspender = MantaPaySuspensionManager;
}

parameter_types! {
    pub NonPausablePallets: Vec<Vec<u8>> = vec![b"Democracy".to_vec(), b"Balances".to_vec(), b"Council".to_vec(), b"CouncilCollective".to_vec(), b"TechnicalCommittee".to_vec(), b"TechnicalCollective".to_vec()];
}

impl pallet_tx_pause::Config for Test {
    type Event = Event;
    type Call = Call;
    type MaxCallNames = ConstU32<25>;
    type PauseOrigin = EnsureRoot<AccountId32>;
    type UnpauseOrigin = EnsureRoot<AccountId32>;
    type NonPausablePallets = IsInVec<NonPausablePallets>;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_asset_manager::GenesisConfig::<Test>::default()
        .assimilate_storage(&mut t)
        .unwrap();

    sp_io::TestExternalities::new(t)
}
