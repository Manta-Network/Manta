// Copyright 2019-2022 Manta Network.
// This file is part of pallet-manta-pay.
//
// pallet-manta-pay is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pallet-manta-pay is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pallet-manta-pay.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::{
	parameter_types,
	traits::{
		fungible::Inspect,
		fungibles::{Inspect as AssetInspect, Transfer as AssetTransfer},
		tokens::{DepositConsequence, ExistenceRequirement, WithdrawConsequence},
		ConstU32, Currency, Everything,
	},
	PalletId,
};
use frame_system::EnsureRoot;
use manta_primitives::{
	assets::{
		AssetConfig, AssetLocation, AssetRegistrar, AssetRegistrarMetadata, AssetStorageMetadata,
		FungibleLedger, FungibleLedgerConsequence,
	},
	constants::{ASSET_MANAGER_PALLET_ID, MANTA_PAY_PALLET_ID},
	types::{AssetId, Balance},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		MantaPayPallet: crate::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Storage, Event<T>},
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
	}
);

type BlockNumber = u64;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const SS58Prefix: u8 = 42;
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
	pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
	pub const AssetAccountDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
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

pub struct MantaFungibleLedger;
impl FungibleLedger<Test> for MantaFungibleLedger {
	fn can_deposit(
		asset_id: AssetId,
		account: &<Test as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		if asset_id == 0 {
			// we assume native asset with id 0
			match Balances::can_deposit(account, amount) {
				DepositConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		} else {
			match Assets::can_deposit(asset_id, account, amount) {
				DepositConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		}
	}

	fn can_withdraw(
		asset_id: AssetId,
		account: &<Test as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		if asset_id == 0 {
			// we assume native asset with id 0
			match Balances::can_withdraw(account, amount) {
				WithdrawConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		} else {
			match Assets::can_withdraw(asset_id, account, amount) {
				WithdrawConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		}
	}

	fn transfer(
		asset_id: AssetId,
		source: &<Test as frame_system::Config>::AccountId,
		dest: &<Test as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		if asset_id == 0 {
			<Balances as Currency<<Test as frame_system::Config>::AccountId>>::transfer(
				source,
				dest,
				amount,
				ExistenceRequirement::KeepAlive,
			)
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		} else {
			<Assets as AssetTransfer<<Test as frame_system::Config>::AccountId>>::transfer(
				asset_id, source, dest, amount, true,
			)
			.map(|_| ())
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		}
	}

	fn mint(
		asset_id: AssetId,
		beneficiary: &<Test as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		Self::can_deposit(asset_id, beneficiary, amount)?;
		if asset_id == 0 {
			let _ =
				<Balances as Currency<<Test as frame_system::Config>::AccountId>>::deposit_creating(
					beneficiary,
					amount,
				);
			Ok(())
		} else {
			Assets::mint(
				Origin::signed(AssetManager::account_id()),
				asset_id,
				beneficiary.clone(),
				amount,
			)
			.map(|_| ())
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		}
	}
}

pub struct MantaAssetRegistrar;
use frame_support::pallet_prelude::DispatchResult;
impl AssetRegistrar<MantaAssetConfig> for MantaAssetRegistrar {
	fn create_asset(
		asset_id: AssetId,
		min_balance: Balance,
		metadata: AssetStorageMetadata,
		is_sufficient: bool,
	) -> DispatchResult {
		Assets::force_create(
			Origin::root(),
			asset_id,
			AssetManager::account_id(),
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
		)?;

		Assets::force_asset_status(
			Origin::root(),
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

#[derive(Clone, Eq, PartialEq)]
pub struct MantaAssetConfig;
impl AssetConfig for MantaAssetConfig {
	type AssetRegistrarMetadata = AssetRegistrarMetadata;
	type StorageMetadata = AssetStorageMetadata;
	type AssetLocation = AssetLocation;
	type AssetRegistrar = MantaAssetRegistrar;
}

parameter_types! {
	pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

impl pallet_asset_manager::Config for Test {
	type Event = Event;
	type AssetConfig = MantaAssetConfig;
	type ModifierOrigin = EnsureRoot<AccountId32>;
	type PalletId = AssetManagerPalletId;
}

parameter_types! {
	pub const MantaPayPalletId: PalletId = MANTA_PAY_PALLET_ID;
}

impl crate::Config for Test {
	type Event = Event;
	type WeightInfo = crate::weights::SubstrateWeight<Self>;
	type FungibleLedger = MantaFungibleLedger;
	type PalletId = MantaPayPalletId;
	type AssetConfig = MantaAssetConfig;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
