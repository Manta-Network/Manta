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

//! Dolphin Parachain runtime.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Perbill, Permill,
};

use sp_core::u32_trait::{_1, _2, _3, _4, _5};
use sp_std::{cmp::Ordering, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
	construct_runtime, match_type, parameter_types,
	traits::{
		fungible::Inspect,
		fungibles::{Inspect as AssetInspect, Transfer as AssetTransfer},
		tokens::{DepositConsequence, ExistenceRequirement, WithdrawConsequence},
		ConstU16, ConstU32, ConstU8, Contains, Currency, EnsureOneOf, Everything, Nothing,
		PrivilegeCmp,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
		DispatchClass, Weight,
	},
	PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};
use manta_primitives::{
	assets::{
		AssetConfig, AssetIdLocationConvert, AssetLocation, AssetRegistrar, AssetRegistrarMetadata,
		AssetStorageMetadata, FungibleLedger, FungibleLedgerConsequence,
	},
	constants::{
		time::*, ASSET_MANAGER_PALLET_ID, MANTA_PAY_PALLET_ID, STAKING_PALLET_ID,
		TREASURY_PALLET_ID,
	},
	types::{AccountId, AssetId, AuraId, Balance, BlockNumber, Hash, Header, Index, Signature},
	xcm::{AccountIdToMultiLocation, FirstAssetTrader, IsNativeConcrete, MultiNativeAsset},
};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::{BlockHashCount, RocksDbWeight, SlowAdjustingFeeUpdate};
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	FungiblesAdapter, LocationInverter, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

pub mod currency;
pub mod fee;
pub mod impls;

use currency::*;
use fee::WeightToFee;
use impls::DealWithFees;

pub type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
		}
	}
}

// Weights used in the runtime.
mod weights;

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("dolphin"),
	impl_name: create_runtime_str!("dolphin"),
	authoring_version: 1,
	spec_version: 3141,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 70%, the rest can be used by
/// Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(70);

/// We allow for 0.5 seconds of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = manta_primitives::constants::CALAMARI_SS58PREFIX;
}

impl pallet_tx_pause::Config for Runtime {
	type Event = Event;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = weights::pallet_tx_pause::SubstrateWeight<Runtime>;
}

// Don't allow permission-less asset creation.
pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
	fn contains(call: &Call) -> bool {
		if matches!(
			call,
			Call::Timestamp(_) | Call::ParachainSystem(_) | Call::System(_)
		) {
			// always allow core call
			// pallet-timestamp and parachainSystem could not be filtered because they are used in commuication between releychain and parachain.
			return true;
		}

		if pallet_tx_pause::PausedTransactionFilter::<Runtime>::contains(call) {
			// no paused call
			return false;
		}

		match call {
			| Call::Authorship(_)
			// Sudo also cannot be filtered because it is used in runtime upgrade.
			| Call::Sudo(_)
			| Call::Multisig(_)
			// For now disallow public proposal workflows, treasury workflows,
			// as well as external_propose and external_propose_majority.
			// The following are filtered out:
			// pallet_democracy::Call::propose(_)
			// pallet_democracy::Call::second(_, _)
			// pallet_democracy::Call::cancel_proposal(_)
			// pallet_democracy::Call::clear_public_proposals()
			// pallet_democracy::Call::external_propose(_)
			// pallet_democracy::Call::external_propose_majority(_)
			| Call::Democracy(pallet_democracy::Call::vote {..}
								| pallet_democracy::Call::emergency_cancel {..}
								| pallet_democracy::Call::external_propose_default {..}
								| pallet_democracy::Call::fast_track  {..}
								| pallet_democracy::Call::veto_external {..}
								| pallet_democracy::Call::cancel_referendum {..}
								| pallet_democracy::Call::cancel_queued {..}
								| pallet_democracy::Call::delegate {..}
								| pallet_democracy::Call::undelegate {..}
								| pallet_democracy::Call::note_preimage {..}
								| pallet_democracy::Call::note_preimage_operational {..}
								| pallet_democracy::Call::note_imminent_preimage {..}
								| pallet_democracy::Call::note_imminent_preimage_operational {..}
								| pallet_democracy::Call::reap_preimage {..}
								| pallet_democracy::Call::unlock {..}
								| pallet_democracy::Call::remove_vote {..}
								| pallet_democracy::Call::remove_other_vote {..}
								| pallet_democracy::Call::enact_proposal {..}
								| pallet_democracy::Call::blacklist {..})
			| Call::Council(_)
			| Call::TechnicalCommittee(_)
			| Call::CouncilMembership(_)
			| Call::TechnicalMembership(_)
			// Treasury calls are filtered while it is accumulating funds.
			//| Call::Treasury(_)
			| Call::Scheduler(_)
			| Call::Session(_) // User must be able to set their session key when applying for a collator
			| Call::CollatorSelection(
				manta_collator_selection::Call::set_invulnerables{..}
				| manta_collator_selection::Call::set_desired_candidates{..}
				| manta_collator_selection::Call::set_candidacy_bond{..}
				| manta_collator_selection::Call::set_eviction_baseline{..}
				| manta_collator_selection::Call::set_eviction_tolerance{..}
				| manta_collator_selection::Call::register_candidate{..}
				// Currently, we filter `register_as_candidate` as this call is not yet ready for community.
				| manta_collator_selection::Call::remove_collator{..}
				| manta_collator_selection::Call::leave_intent{..})
			| Call::Balances(_)
			| Call::Preimage(_)
			| Call::Utility(_)
			| Call::MantaPay(_)
			| Call::XTokens(_) => true,
			// Filter XCM pallets, we only allow transfer with XTokens.
			// Filter Assets. Assets should only be accessed by AssetManager.
			// AssetManager is also filtered because all of its extrinsics are callable only by Root,
			// and Root calls skip this whole filter.
			_ => false,
		}
	}
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
	type BaseCallFilter = BaseFilter; // Let filter activate.
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = weights::frame_system::SubstrateWeight<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::SubstrateWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = ConstU32<0>;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const NativeTokenExistentialDeposit: u128 = 10 * cDOL; // 0.1 DOL
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type WeightInfo = weights::pallet_balances::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = mDOL / 100;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU16<100>;
	type WeightInfo = weights::pallet_multisig::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 5 * MINUTES;
	pub const VotingPeriod: BlockNumber = 5 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 5 * MINUTES;
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 20 * DOL;
	pub const EnactmentPeriod: BlockNumber = 5 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 5 * MINUTES;
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type VoteLockingPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = ();
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = ConstU32<100>;
	type WeightInfo = weights::pallet_democracy::SubstrateWeight<Runtime>;
	type MaxProposals = ConstU32<100>;
}

parameter_types! {
	/// The maximum amount of time (in blocks) for council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::SubstrateWeight<Runtime>;
}

pub type EnsureRootOrThreeFourthsCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
>;

type CouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<CouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrThreeFourthsCouncil;
	type RemoveOrigin = EnsureRootOrThreeFourthsCouncil;
	type SwapOrigin = EnsureRootOrThreeFourthsCouncil;
	type ResetOrigin = EnsureRootOrThreeFourthsCouncil;
	type PrimeOrigin = EnsureRootOrThreeFourthsCouncil;
	type MembershipInitialized = Council;
	type MembershipChanged = Council;
	type MaxMembers = ConstU32<100>;
	type WeightInfo = weights::pallet_membership::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::SubstrateWeight<Runtime>;
}

type TechnicalMembershipInstance = pallet_membership::Instance2;
impl pallet_membership::Config<TechnicalMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrThreeFourthsCouncil;
	type RemoveOrigin = EnsureRootOrThreeFourthsCouncil;
	type SwapOrigin = EnsureRootOrThreeFourthsCouncil;
	type ResetOrigin = EnsureRootOrThreeFourthsCouncil;
	type PrimeOrigin = EnsureRootOrThreeFourthsCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = ConstU32<100>;
	type WeightInfo = weights::pallet_membership::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(1);
	pub const ProposalBondMinimum: Balance = 500 * DOL;
	pub const ProposalBondMaximum: Balance = 10_000 * DOL;
	pub const SpendPeriod: BlockNumber = 10 * MINUTES;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;
}

type EnsureRootOrThreeFifthsCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
>;

type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrThreeFifthsCouncil;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type Event = Event;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type MaxApprovals = ConstU32<100>;
	type WeightInfo = weights::pallet_treasury::SubstrateWeight<Runtime>;
	type SpendFunds = ();
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

type ScheduleOrigin = EnsureRoot<AccountId>;
/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;
impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
	fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
		if left == right {
			return Some(Ordering::Equal);
		}

		match (left, right) {
			// Root is greater than anything.
			(OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
			// Check which one has more yes votes.
			(
				OriginCaller::Council(pallet_collective::RawOrigin::Members(l_yes_votes, l_count)),
				OriginCaller::Council(pallet_collective::RawOrigin::Members(r_yes_votes, r_count)),
			) => Some((l_yes_votes * r_count).cmp(&(r_yes_votes * l_count))),
			// For every other origin we don't care, as they are not used for `ScheduleOrigin`.
			_ => None,
		}
	}
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = ScheduleOrigin;
	type MaxScheduledPerBlock = ConstU32<50>; // 50 scheduled calls at most in the queue for a single block.
	type WeightInfo = weights::pallet_scheduler::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = OriginPrivilegeCmp;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
	// Our NORMAL_DISPATCH_RATIO is 70% of the 5MB limit
	// So anything more than 3.5MB doesn't make sense here
	pub const PreimageMaxSize: u32 = 3584 * 1024;
	pub const PreimageBaseDeposit: Balance = 1 * DOL;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = ();
	type Event = Event;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type MaxSize = PreimageMaxSize;
	// The sum of the below 2 amounts will get reserved every time someone submits a preimage.
	// Their sum will be unreserved when the preimage is requested, i.e. when it is going to be used.
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type OnSystemEvent = ();
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const KsmLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	pub SelfReserve: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Transactor for native currency, i.e. implements `fungible` trait
pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Transacting native currency, i.e. MANTA, KMA, DOL
	Balances,
	// Used when the incoming asset is a fungible concrete asset matching the given location or name:
	IsNativeConcrete<SelfReserve>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`.
/// It uses some Rust magic macro to do the pattern matching sequentially.
/// There is an `OriginKind` which can biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// If the incoming XCM origin is of type `AccountId32` and the Network is Network::Any
	// or `RelayNetwork`, convert it to a Native 32 byte account.
	SignedAccountId32AsNative<RelayNetwork, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 1_000_000_000;
	// Used in native traders
	// This might be able to skipped.
	// We have to use `here()` because of reanchoring logic
	pub ParaTokenPerSecond: (xcm::v2::AssetId, u128) = (Concrete(MultiLocation::here()), 1_000_000_000);
	pub const MaxInstructions: u32 = 100;
}

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

match_type! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}
match_type! {
	pub type ParentOrSiblings: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(_) }
	};
}

pub type Barrier = (
	// Allows local origin messages which call weight_credit >= weight_limit.
	TakeWeightCredit,
	// Allows non-local origin messages, for example from from the xcmp queue,
	// which have the ability to deposit assets and pay for their own execution.
	AllowTopLevelPaidExecutionFrom<Everything>,
	// Parent and its exec plurality get free execution
	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
	// Expected responses are OK.
	// Allows `Pending` or `VersionNotifier` query responses.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	// Allows execution of `SubscribeVersion` or `UnsubscribeVersion` instruction,
	// from parent or sibling chains.
	AllowSubscriptionsFrom<ParentOrSiblings>,
);

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

/// No one is allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = ();

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

	type Origin = Origin;
	type Call = Call;
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	/// This means that no location will pass XcmExecuteFilter, so a dispatched `execute` message will be filtered.
	/// This shouldn't be reachable since `LocalOriginToLocation = ();`, but let's be on the safe side.
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
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
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<AssetIdLocationConvert<AssetLocation, AssetManager>>;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	type SelfLocation = SelfReserve;
	// Take note that this pallet does not have the typical configurable WeightInfo.
	// It uses the Weigher configuration to calculate weights for the user callable extrinsics on this chain,
	// as well as weights for execution on the destination chain. Both based on the composed xcm messages.
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
}

parameter_types! {
	// Rotate collator's spot each 6 hours.
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = manta_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler =
		<opaque::SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = weights::pallet_session::SubstrateWeight<Runtime>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
}

parameter_types! {
	// Pallet account for record rewards and give rewards to collator.
	pub const PotId: PalletId = STAKING_PALLET_ID;
}

parameter_types! {
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

/// We allow root and the Relay Chain council to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>,
>;

impl manta_collator_selection::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = ConstU32<50>; // 50 candidates at most
	type MaxInvulnerables = ConstU32<5>; // 5 invulnerables at most
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = manta_collator_selection::IdentityCollator;
	type AccountIdOf = manta_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = weights::manta_collator_selection::SubstrateWeight<Runtime>;
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

parameter_types! {
	// Pallet account for record rewards and give rewards to collator.
	pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
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
		)?;

		Assets::force_asset_status(
			Origin::root(),
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

impl pallet_asset_manager::Config for Runtime {
	type Event = Event;
	type AssetConfig = MantaAssetConfig;
	type ModifierOrigin = EnsureRoot<AccountId>;
	type PalletId = AssetManagerPalletId;
}

pub struct MantaFungibleLedger;
impl FungibleLedger<Runtime> for MantaFungibleLedger {
	fn can_deposit(
		asset_id: AssetId,
		account: &<Runtime as frame_system::Config>::AccountId,
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
		account: &<Runtime as frame_system::Config>::AccountId,
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
		source: &<Runtime as frame_system::Config>::AccountId,
		dest: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		if asset_id == 0 {
			<Balances as Currency<<Runtime as frame_system::Config>::AccountId>>::transfer(
				source,
				dest,
				amount,
				ExistenceRequirement::KeepAlive,
			)
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		} else {
			<Assets as AssetTransfer<<Runtime as frame_system::Config>::AccountId>>::transfer(
				asset_id, source, dest, amount, true,
			)
			.and_then(|_| Ok(()))
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		}
	}

	fn mint(
		asset_id: AssetId,
		beneficiary: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		Self::can_deposit(asset_id, beneficiary, amount)?;
		if asset_id == 0 {
			let _ = <Balances as Currency<<Runtime as frame_system::Config>::AccountId>>::deposit_creating(beneficiary, amount);
			Ok(())
		} else {
			Assets::mint(
				Origin::signed(AssetManager::account_id()),
				asset_id,
				beneficiary.clone().into(),
				amount,
			)
			.and_then(|_| Ok(()))
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		}
	}
}

parameter_types! {
	pub const MantaPayPalletId: PalletId = MANTA_PAY_PALLET_ID;
}

impl pallet_manta_pay::Config for Runtime {
	type Event = Event;
	type WeightInfo = weights::pallet_manta_pay::SubstrateWeight<Runtime>;
	type AssetConfig = MantaAssetConfig;
	type FungibleLedger = MantaFungibleLedger;
	type PalletId = MantaPayPalletId;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 3,
		TransactionPause: pallet_tx_pause::{Pallet, Call, Storage, Event<T>} = 9,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 11,

		// Governance stuff.
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 14,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 15,
		CouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 16,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 17,
		TechnicalMembership: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 18,

		// Collator support. the order of these 5 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		CollatorSelection: manta_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,

		// Treasury
		Treasury: pallet_treasury::{Pallet, Call, Storage, Event<T>} = 26,

		// Preimage registrar.
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 28,
		// System scheduler.
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 29,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 31,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,
		XTokens: orml_xtokens::{Pallet, Call, Event<T>, Storage} = 34,

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event} = 40,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 41,
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 42,

		// Asset and Private Payment
		Assets: pallet_assets::{Pallet, Storage, Event<T>} = 45,
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 46,
		MantaPay: pallet_manta_pay::{Pallet, Call, Storage, Event<T>} = 47,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsReversedWithSystemFirst,
>;

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_multisig, Multisig);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_utility, Utility);
			list_benchmark!(list, extra, manta_collator_selection, CollatorSelection);
			list_benchmark!(list, extra, pallet_democracy, Democracy);
			list_benchmark!(list, extra, pallet_collective, Council);
			list_benchmark!(list, extra, pallet_membership, CouncilMembership);
			list_benchmark!(list, extra, pallet_treasury, Treasury);
			list_benchmark!(list, extra, pallet_preimage, Preimage);
			list_benchmark!(list, extra, pallet_scheduler, Scheduler);
			list_benchmark!(list, extra, pallet_session, SessionBench::<Runtime>);
			list_benchmark!(list, extra, pallet_tx_pause, TransactionPause);
			list_benchmark!(list, extra, pallet_assets, Assets);
			list_benchmark!(list, extra, pallet_manta_pay, MantaPay);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_multisig, Multisig);
			add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, pallet_utility, Utility);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, manta_collator_selection, CollatorSelection);
			add_benchmark!(params, batches, pallet_democracy, Democracy);
			add_benchmark!(params, batches, pallet_collective, Council);
			add_benchmark!(params, batches, pallet_membership, CouncilMembership);
			add_benchmark!(params, batches, pallet_scheduler, Scheduler);
			add_benchmark!(params, batches, pallet_preimage, Preimage);
			add_benchmark!(params, batches, pallet_treasury, Treasury);
			add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, pallet_tx_pause, TransactionPause);
			add_benchmark!(params, batches, pallet_assets, Assets);
			add_benchmark!(params, batches, pallet_manta_pay, MantaPay);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;
impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
