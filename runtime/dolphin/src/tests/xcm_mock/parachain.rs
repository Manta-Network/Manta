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

//! Parachain runtime mock.

use codec::{Decode, Encode};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, Everything, Nothing},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
	PalletId,
};
use frame_system::EnsureRoot;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{Hash, IdentityLookup},
	AccountId32,
};
use sp_std::{convert::TryFrom, prelude::*};

use manta_primitives::{
	assets::{AssetIdLocationConvert, AssetLocation, ConcreteFungibleLedger},
	constants::*,
	xcm::{FirstAssetTrader, IsNativeConcrete, MultiNativeAsset},
};
use pallet_xcm::XcmPassthrough;
use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use polkadot_parachain::primitives::{
	DmpMessageHandler, Id as ParaId, Sibling, XcmpMessageFormat, XcmpMessageHandler,
};
use xcm::{latest::prelude::*, Version as XcmVersion, VersionedMultiLocation, VersionedXcm};
use xcm_builder::{
	AccountId32Aliases, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	FungiblesAdapter, LocationInverter, ParentIsDefault, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};
use xcm_simulator::Get;

pub use manta_primitives::{
	assets::{AssetConfig, AssetRegistrar, AssetRegistrarMetadata, AssetStorageMetadata},
	types::AssetId,
};
pub type AccountId = AccountId32;
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub ExistentialDeposit: Balance = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
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
	pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
	pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

parameter_types! {
	pub const KsmLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub Ancestry: MultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
	pub SelfReserve: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
}

parameter_types! {
	pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
	pub const AssetAccountDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
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
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`.
/// It uses some Rust magic macro to do the pattern matching sequentially.
/// There is an `OriginKind` which can biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// If the incoming XCM origin is of type `AccountId32` and the Network is Network::Any
	// or `RelayNetwork`, convert it to a Native 32 byte account.
	SignedAccountId32AsNative<RelayNetwork, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

parameter_types! {
	pub const UnitWeightCost: Weight = 1_000_000_000;
	// Used in native traders
	// This might be able to skipped.
	// We have to use `here()` because of reanchoring logic
	pub ParaTokenPerSecond: (xcm::v2::AssetId, u128) = (Concrete(MultiLocation::here()), 1_000_000_000);
	pub const MaxInstructions: u32 = 100;
}

/// Transactor for native currency, i.e. implements `fungible` trait
pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Transacting native currency, i.e. MANTA, KMA, DOL
	Balances,
	IsNativeConcrete<SelfReserve>,
	LocationToAccountId,
	AccountId,
	(),
>;

/// Transactor for currency in pallet-assets, i.e. implements `fungibles` trait
pub type FungiblesTransactor = FungiblesAdapter<
	Assets,
	ConvertedConcreteAssetId<
		AssetId,
		Balance,
		AssetIdLocationConvert<AssetLocation, AssetManager>,
		JustTry,
	>,
	// "default" implementation of converting a `MultiLocation` to an `AccountId`
	LocationToAccountId,
	AccountId,
	// No teleport support.
	Nothing,
	// No teleport tracking.
	(),
>;

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

parameter_types! {
	/// Xcm fees will go to the asset manager (we don't implement treasury yet)
	pub XcmFeesAccount: AccountId = AssetManager::account_id();
}

pub type XcmFeesToAccount = manta_primitives::xcm::XcmFeesToAccount<
	Assets,
	ConvertedConcreteAssetId<
		AssetId,
		Balance,
		AssetIdLocationConvert<AssetLocation, AssetManager>,
		JustTry,
	>,
	AccountId,
	XcmFeesAccount,
>;

pub struct XcmExecutorConfig;
impl Config for XcmExecutorConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// Defines how to Withdraw and Deposit instruction work
	// Under the hood, substrate framework will do pattern matching in macro,
	// as a result, the order of the following tuple matters.
	type AssetTransactor = (LocalAssetTransactor, FungiblesTransactor);
	type OriginConverter = XcmOriginToCallOrigin;
	// Combinations of (Location, Asset) pairs which we trust as reserves.
	type IsReserve = MultiNativeAsset;
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	// Trader is the means to purchasing weight credit for XCM execution.
	// We define two traders:
	// The first one will charge parachain's native currency, who's `MultiLocation`
	// is defined in `SelfReserve`.
	// The second one will charge the first asset in the MultiAssets with pre-defined rate
	// i.e. units_per_second in `AssetManager`
	type Trader = (
		FixedRateOfFungible<ParaTokenPerSecond, ()>,
		FirstAssetTrader<AssetId, AssetLocation, AssetManager, XcmFeesToAccount>,
	);
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	// This is needed for the version change notifier work
	type SubscriptionService = PolkadotXcm;
}

// Pallet to provide the version, used to test runtime upgrade version changes
#[frame_support::pallet]
pub mod mock_version_changer {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn current_version)]
	pub(super) type CurrentVersion<T: Config> = StorageValue<_, XcmVersion, ValueQuery>;

	impl<T: Config> Get<XcmVersion> for Pallet<T> {
		fn get() -> XcmVersion {
			Self::current_version()
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// XCMP
		/// Some XCM was executed OK.
		VersionChanged(XcmVersion),
	}

	impl<T: Config> Pallet<T> {
		pub fn set_version(version: XcmVersion) {
			CurrentVersion::<T>::put(version);
			Self::deposit_event(Event::VersionChanged(version));
		}
	}
}

#[frame_support::pallet]
pub mod mock_msg_queue {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type XcmExecutor: ExecuteXcm<Self::Call>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	// without storage info is a work around
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn parachain_id)]
	pub(super) type ParachainId<T: Config> = StorageValue<_, ParaId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn received_dmp)]
	/// A queue of received DMP messages
	pub(super) type ReceivedDmp<T: Config> = StorageValue<_, Vec<Xcm<T::Call>>, ValueQuery>;

	impl<T: Config> Get<ParaId> for Pallet<T> {
		fn get() -> ParaId {
			Self::parachain_id()
		}
	}

	pub type MessageId = [u8; 32];

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// XCMP
		/// Some XCM was executed OK.
		Success(Option<T::Hash>),
		/// Some XCM failed.
		Fail(Option<T::Hash>, XcmError),
		/// Bad XCM version used.
		BadVersion(Option<T::Hash>),
		/// Bad XCM format used.
		BadFormat(Option<T::Hash>),

		// DMP
		/// Downward message is invalid XCM.
		InvalidFormat(MessageId),
		/// Downward message is unsupported version of XCM.
		UnsupportedVersion(MessageId),
		/// Downward message executed with the given outcome.
		ExecutedDownward(MessageId, Outcome),
	}

	impl<T: Config> Pallet<T> {
		pub fn set_para_id(para_id: ParaId) {
			ParachainId::<T>::put(para_id);
		}

		fn handle_xcmp_message(
			sender: ParaId,
			_sent_at: RelayBlockNumber,
			xcm: VersionedXcm<T::Call>,
			max_weight: Weight,
		) -> Result<Weight, XcmError> {
			let hash = Encode::using_encoded(&xcm, T::Hashing::hash);
			let (result, event) = match Xcm::<T::Call>::try_from(xcm) {
				Ok(xcm) => {
					let location = (1, Parachain(sender.into()));
					match T::XcmExecutor::execute_xcm(location, xcm, max_weight) {
						Outcome::Error(e) => (Err(e.clone()), Event::Fail(Some(hash), e)),
						Outcome::Complete(w) => (Ok(w), Event::Success(Some(hash))),
						// As far as the caller is concerned, this was dispatched without error, so
						// we just report the weight used.
						Outcome::Incomplete(w, e) => (Ok(w), Event::Fail(Some(hash), e)),
					}
				}
				Err(()) => (
					Err(XcmError::UnhandledXcmVersion),
					Event::BadVersion(Some(hash)),
				),
			};
			Self::deposit_event(event);
			result
		}
	}

	impl<T: Config> XcmpMessageHandler for Pallet<T> {
		fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
			iter: I,
			max_weight: Weight,
		) -> Weight {
			for (sender, sent_at, data) in iter {
				let mut data_ref = data;
				let _ = XcmpMessageFormat::decode(&mut data_ref)
					.expect("Simulator encodes with versioned xcm format; qed");

				let mut remaining_fragments = &data_ref[..];
				while !remaining_fragments.is_empty() {
					if let Ok(xcm) = VersionedXcm::<T::Call>::decode(&mut remaining_fragments) {
						let _ = Self::handle_xcmp_message(sender, sent_at, xcm, max_weight);
					} else {
						debug_assert!(false, "Invalid incoming XCMP message data");
					}
				}
			}
			max_weight
		}
	}

	impl<T: Config> DmpMessageHandler for Pallet<T> {
		fn handle_dmp_messages(
			iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
			limit: Weight,
		) -> Weight {
			for (_i, (_sent_at, data)) in iter.enumerate() {
				let id = sp_io::hashing::blake2_256(&data[..]);
				let maybe_msg =
					VersionedXcm::<T::Call>::decode(&mut &data[..]).map(Xcm::<T::Call>::try_from);
				match maybe_msg {
					Err(_) => {
						Self::deposit_event(Event::InvalidFormat(id));
					}
					Ok(Err(())) => {
						Self::deposit_event(Event::UnsupportedVersion(id));
					}
					Ok(Ok(x)) => {
						let outcome = T::XcmExecutor::execute_xcm(Parent, x.clone(), limit);
						<ReceivedDmp<T>>::append(x);
						Self::deposit_event(Event::ExecutedDownward(id, outcome));
					}
				}
			}
			limit
		}
	}
}

impl mock_msg_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

impl mock_version_changer::Config for Runtime {
	type Event = Event;
}

pub type LocalOriginToLocation = ();

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	// Do not allow teleports
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = XcmVersioner;
}

pub struct MantaAssetRegistrar;
use frame_support::pallet_prelude::DispatchResult;
impl AssetRegistrar<Runtime, MantaAssetConfig> for MantaAssetRegistrar {
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

parameter_types! {
	pub const DummyAssetId: AssetId = 0;
	pub const NativeAssetId: AssetId = 1;
	pub const StartNonNativeAssetId: AssetId = 8;
	pub NativeAssetLocation: AssetLocation = AssetLocation(
		VersionedMultiLocation::V1(MultiLocation::new(1, X1(Parachain(1024)))));
	pub NativeAssetMetadata: AssetRegistrarMetadata = AssetRegistrarMetadata {
		name: b"Dolphin".to_vec(),
		symbol: b"DOL".to_vec(),
		decimals: 18,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

#[derive(Clone, Eq, PartialEq)]
pub struct MantaAssetConfig;

impl AssetConfig<Runtime> for MantaAssetConfig {
	type AssetRegistrarMetadata = AssetRegistrarMetadata;
	type StorageMetadata = AssetStorageMetadata;
	type AssetLocation = AssetLocation;
	type AssetRegistrar = MantaAssetRegistrar;
	type DummyAssetId = DummyAssetId;
	type NativeAssetId = NativeAssetId;
	type StartNonNativeAssetId = StartNonNativeAssetId;
	type NativeAssetLocation = NativeAssetLocation;
	type NativeAssetMetadata = NativeAssetMetadata;
	type FungibleLedger = ConcreteFungibleLedger<Runtime, MantaAssetConfig, Balances, Assets>;
}

impl pallet_asset_manager::Config for Runtime {
	type Event = Event;
	type AssetConfig = MantaAssetConfig;
	type ModifierOrigin = EnsureRoot<AccountId>;
	type PalletId = AssetManagerPalletId;
	type WeightInfo = ();
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

// We wrap AssetId for XToken
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	MantaCurrency(AssetId),
}

pub struct CurrencyIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
	for CurrencyIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: xcm_executor::traits::Convert<MultiLocation, AssetId>,
{
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::MantaCurrency(asset_id) => match AssetXConverter::reverse_ref(&asset_id) {
				Ok(location) => Some(location),
				Err(_) => None,
			},
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = 100_000_000;
	pub const MaxAssetsForTransfer: usize = 1;
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = manta_primitives::xcm::AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<AssetIdLocationConvert<AssetLocation, AssetManager>>;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	type SelfLocation = SelfReserve;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
}

impl parachain_info::Config for Runtime {}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub const PALLET_ASSET_INDEX: u8 = 1;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		Assets: pallet_assets::{Pallet, Storage, Event<T>} = 1,
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 2,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
		MsgQueue: mock_msg_queue::{Pallet, Storage, Event<T>} = 4,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 5,
		XTokens: orml_xtokens::{Pallet, Call, Event<T>, Storage} = 6,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 7,
		XcmVersioner: mock_version_changer::{Pallet, Storage, Event<T>} = 8,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 9,
	}
);

pub(crate) fn para_events() -> Vec<Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| Some(e))
		.collect::<Vec<_>>()
}

use frame_support::traits::{OnFinalize, OnInitialize, OnRuntimeUpgrade};
pub(crate) fn on_runtime_upgrade() {
	PolkadotXcm::on_runtime_upgrade();
}

pub(crate) fn para_roll_to(n: u64) {
	while System::block_number() < n {
		PolkadotXcm::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		PolkadotXcm::on_initialize(System::block_number());
	}
}
