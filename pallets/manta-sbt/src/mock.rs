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

use frame_support::{
    pallet_prelude::DispatchResult,
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, Everything},
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use manta_primitives::{
    assets::{
        AssetConfig, AssetIdType, AssetLocation, AssetRegistry, AssetRegistryMetadata,
        AssetStorageMetadata, BalanceType, LocationType, NativeAndNonNative,
    },
    constants::MANTA_SBT_PALLET_ID,
    types::{Balance, BlockNumber, Header},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, Get, IdentityLookup},
    AccountId32,
};
use xcm::{
    prelude::{Parachain, X1},
    v1::MultiLocation,
    VersionedMultiLocation,
};

use crate::StandardAssetId;

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
        Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
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

impl pallet_uniques::Config for Test {
    type Event = Event;
    type CollectionId = StandardAssetId;
    type ItemId = StandardAssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId32>;
    type CollectionDeposit = CollectionDeposit;
    type ItemDeposit = ItemDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type AttributeDepositBase = MetadataDepositBase;
    type DepositPerByte = MetadataDepositPerByte;
    type StringLimit = ConstU32<1000>;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId32>>;
    type Locker = ();
}

parameter_types! {
    pub const MantaSBTPalletId: PalletId = MANTA_SBT_PALLET_ID;
}

impl crate::Config for Test {
    type Event = Event;
    type WeightInfo = crate::weights::SubstrateWeight<Test>;
    type CollectionIdentifier = ConstU128<0>;
    type PalletId = MantaSBTPalletId;
    type NFT = Uniques;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
