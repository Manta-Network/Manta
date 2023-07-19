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

//! Manta Parachain runtime.

#![allow(clippy::identity_op)] // keep e.g. 1 * DAYS for legibility
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub use frame_support::traits::{Get, IsInVec};
use manta_collator_selection::IdentityCollator;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perbill, Percent, Permill,
};

use sp_std::{cmp::Ordering, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use cumulus_pallet_parachain_system::{
    register_validate_block, CheckInherents, ParachainSetCode, RelayChainStateProof,
    RelaychainBlockNumberProvider,
};
use frame_support::{
    construct_runtime,
    dispatch::DispatchClass,
    parameter_types,
    traits::{
        ConstU128, ConstU32, ConstU8, Contains, Currency, EitherOfDiverse, NeverEnsureOrigin,
        PrivilegeCmp,
    },
    weights::{ConstantMultiplier, Weight},
    PalletId,
};

use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use manta_primitives::{
    constants::{
        time::*, RocksDbWeight, LOTTERY_PALLET_ID, NAME_SERVICE_PALLET_ID, STAKING_PALLET_ID,
        TREASURY_PALLET_ID, WEIGHT_PER_SECOND,
    },
    types::{AccountId, Balance, BlockNumber, Hash, Header, Index, PoolId, Signature},
};
use manta_support::manta_pay::{InitialSyncResponse, PullResponse, RawCheckpoint};
pub use pallet_parachain_staking::{InflationInfo, Range};
use pallet_session::ShouldEndSession;
use runtime_common::{
    prod_or_fast, BlockExecutionWeight, BlockHashCount, ExtrinsicBaseWeight,
    MantaSlowAdjustingFeeUpdate,
};
use session_key_primitives::{AuraId, NimbusId, VrfId};
use zenlink_protocol::{AssetBalance, AssetId as ZenlinkAssetId, MultiAssetsHandler, PairInfo};

#[cfg(feature = "runtime-benchmarks")]
use xcm::latest::prelude::*;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub mod assets_config;
pub mod currency;
pub mod fee;
pub mod impls;
pub mod migrations;
mod nimbus_session_adapter;
pub mod staking;
pub mod xcm_config;
pub mod zenlink;

use currency::*;
use impls::DealWithFees;
use manta_primitives::{currencies::Currencies, types::MantaAssetId};

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
    use nimbus_session_adapter::{AuthorInherentWithNoOpSession, VrfWithNoOpSession};
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub nimbus: AuthorInherentWithNoOpSession<Runtime>,
            pub vrf: VrfWithNoOpSession,
        }
    }
    impl SessionKeys {
        pub fn new(tuple: (NimbusId, VrfId)) -> SessionKeys {
            let (nimbus, vrf) = tuple;
            SessionKeys { nimbus, vrf }
        }
        /// Derives all collator keys from `seed` without checking that the `seed` is valid.
        #[cfg(feature = "std")]
        pub fn from_seed_unchecked(seed: &str) -> SessionKeys {
            Self::new((
                session_key_primitives::util::unchecked_public_key::<NimbusId>(seed),
                session_key_primitives::util::unchecked_public_key::<VrfId>(seed),
            ))
        }
    }
}

// Weights used in the runtime.
pub mod weights;

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("manta"),
    impl_name: create_runtime_str!("manta"),
    authoring_version: 1,
    spec_version: 4311,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 3,
    state_version: 1,
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
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_ref_time(WEIGHT_PER_SECOND)
    .saturating_div(2)
    .set_proof_size(cumulus_primitives_core::relay_chain::v2::MAX_POV_SIZE as u64);

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
    pub const SS58Prefix: u8 = manta_primitives::constants::MANTA_SS58PREFIX;
}

// Don't allow permission-less asset creation.
pub struct MantaFilter;
impl Contains<RuntimeCall> for MantaFilter {
    fn contains(call: &RuntimeCall) -> bool {
        if matches!(
            call,
            RuntimeCall::Timestamp(_) | RuntimeCall::ParachainSystem(_) | RuntimeCall::System(_)
        ) {
            // always allow core call
            // pallet-timestamp and parachainSystem could not be filtered because they are used in communication between releychain and parachain.
            return true;
        }

        if pallet_tx_pause::PausedTransactionFilter::<Runtime>::contains(call) {
            // no paused call
            return false;
        }

        #[allow(clippy::match_like_matches_macro)]
        // keep CallFilter with explicit true/false for documentation
        match call {
            // Explicitly DISALLOWED calls ( Pallet user extrinsics we don't want used WITH REASONING )
            // Explicitly ALLOWED calls
            | RuntimeCall::Authorship(_)
            | RuntimeCall::Democracy(pallet_democracy::Call::vote {..}
                | pallet_democracy::Call::emergency_cancel {..}
                | pallet_democracy::Call::external_propose_default {..}
                | pallet_democracy::Call::external_propose_majority {..}
                | pallet_democracy::Call::external_propose {..}
                | pallet_democracy::Call::fast_track  {..}
                | pallet_democracy::Call::veto_external {..}
                | pallet_democracy::Call::cancel_referendum {..}
                | pallet_democracy::Call::delegate {..}
                | pallet_democracy::Call::undelegate {..}
                | pallet_democracy::Call::unlock {..}
                | pallet_democracy::Call::remove_vote {..}
                | pallet_democracy::Call::remove_other_vote {..}
                | pallet_democracy::Call::blacklist {..})
            | RuntimeCall::Council(_)
            | RuntimeCall::TechnicalCommittee(_)
            | RuntimeCall::CouncilMembership(_)
            | RuntimeCall::TechnicalMembership(_)
            | RuntimeCall::Lottery(_)
            | RuntimeCall::Randomness(pallet_randomness::Call::set_babe_randomness_results{..})
            | RuntimeCall::Scheduler(_)
            // Sudo also cannot be filtered because it is used in runtime upgrade.
            | RuntimeCall::Sudo(_)
            | RuntimeCall::Multisig(_)
            | RuntimeCall::AuthorInherent(pallet_author_inherent::Call::kick_off_authorship_validation {..}) // executes unsigned on every block
            | RuntimeCall::Session(_) // User must be able to set their session key when applying for a collator
            | RuntimeCall::ParachainStaking(
                // Collator extrinsics
                pallet_parachain_staking::Call::join_candidates{..}
                | pallet_parachain_staking::Call::schedule_leave_candidates{..}
                | pallet_parachain_staking::Call::execute_leave_candidates{..}
                | pallet_parachain_staking::Call::cancel_leave_candidates{..}
                | pallet_parachain_staking::Call::go_offline{..}
                | pallet_parachain_staking::Call::go_online{..}
                | pallet_parachain_staking::Call::candidate_bond_more{..}
                | pallet_parachain_staking::Call::schedule_candidate_bond_less{..}
                | pallet_parachain_staking::Call::execute_candidate_bond_less{..}
                | pallet_parachain_staking::Call::cancel_candidate_bond_less{..}
                // Delegator extrinsics
                | pallet_parachain_staking::Call::delegate{..}
                | pallet_parachain_staking::Call::schedule_leave_delegators{..}
                | pallet_parachain_staking::Call::execute_leave_delegators{..}
                | pallet_parachain_staking::Call::cancel_leave_delegators{..}
                | pallet_parachain_staking::Call::schedule_revoke_delegation{..}
                | pallet_parachain_staking::Call::delegator_bond_more{..}
                | pallet_parachain_staking::Call::schedule_delegator_bond_less{..}
                | pallet_parachain_staking::Call::execute_delegation_request{..}
                | pallet_parachain_staking::Call::cancel_delegation_request{..})
            | RuntimeCall::XTokens(orml_xtokens::Call::transfer {..} | orml_xtokens::Call::transfer_multiassets {..})
            | RuntimeCall::Balances(_)
            | RuntimeCall::Preimage(_)
            | RuntimeCall::MantaPay(_)
            | RuntimeCall::MantaSbt(_)
            | RuntimeCall::NameService(_)
            | RuntimeCall::TransactionPause(_)
            | RuntimeCall::ZenlinkProtocol(_)
            | RuntimeCall::Farming(_)
            | RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {..})
            | RuntimeCall::AssetManager(pallet_asset_manager::Call::update_outgoing_filtered_assets {..})
            | RuntimeCall::Utility(_) => true,

            // DISALLOW anything else
            | _ => false
        }
    }
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
    type BaseCallFilter = MantaFilter; // Customized Filter for Manta
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type AccountId = AccountId;
    type RuntimeCall = RuntimeCall;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type SystemWeightInfo = weights::frame_system::SubstrateWeight<Runtime>;
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ParachainSetCode<Self>;
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub NonPausablePallets: Vec<Vec<u8>> = vec![b"Democracy".to_vec(), b"Balances".to_vec(), b"Council".to_vec(), b"CouncilMembership".to_vec(), b"TechnicalCommittee".to_vec(), b"TechnicalMembership".to_vec()];
}

impl pallet_tx_pause::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type MaxCallNames = ConstU32<25>;
    type PauseOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureMembers<AccountId, TechnicalCollective, 2>,
    >;
    type UnpauseOrigin = EnsureRoot<AccountId>;
    type NonPausablePallets = IsInVec<NonPausablePallets>;
    type WeightInfo = weights::pallet_tx_pause::SubstrateWeight<Runtime>;
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

/// Only callable after `set_validation_data` is called which forms this proof the same way
fn relay_chain_state_proof() -> cumulus_pallet_parachain_system::RelayChainStateProof {
    let relay_storage_root = ParachainSystem::validation_data()
        .expect("set in `set_validation_data`")
        .relay_parent_storage_root;
    let relay_chain_state =
        ParachainSystem::relay_state_proof().expect("set in `set_validation_data`");
    cumulus_pallet_parachain_system::RelayChainStateProof::new(
        ParachainInfo::get(),
        relay_storage_root,
        relay_chain_state,
    )
    .expect("Invalid relay chain state proof, already constructed in `set_validation_data`")
}
pub struct BabeDataGetter;
impl pallet_randomness::GetBabeData<u64, Option<Hash>> for BabeDataGetter {
    // Tolerate panic here because only ever called in inherent (so can be omitted)
    fn get_epoch_index() -> u64 {
        if cfg!(feature = "runtime-benchmarks") {
            // storage reads as per actual reads
            let _relay_storage_root = ParachainSystem::validation_data();
            let _relay_chain_state = ParachainSystem::relay_state_proof();
            const BENCHMARKING_NEW_EPOCH: u64 = 10u64;
            return BENCHMARKING_NEW_EPOCH;
        }
        relay_chain_state_proof()
            .read_optional_entry(cumulus_primitives_core::relay_chain::well_known_keys::EPOCH_INDEX)
            .ok()
            .flatten()
            .expect("expected to be able to read epoch index from relay chain state proof")
    }
    fn get_epoch_randomness() -> Option<Hash> {
        if cfg!(feature = "runtime-benchmarks") {
            // storage reads as per actual reads
            let _relay_storage_root = ParachainSystem::validation_data();
            let _relay_chain_state = ParachainSystem::relay_state_proof();
            let benchmarking_babe_output = Hash::default();
            return Some(benchmarking_babe_output);
        }
        relay_chain_state_proof()
            .read_optional_entry(
                // We use randomness from one relaychain epoch ago (4hrs Polkadot, 1hr Kusama) in combination with a (longer) `DrawingFreezeout` in pallet lottery
                // to prevent manipulation of the winning set by participants of the lottery
                // NOTE: We operate under the following assumption https://github.com/Manta-Network/Manta/blob/garandor/pt/pallets/randomness/README.md#risks,
                // i.e. polkadot validators are not incentivized enough to skip their block rewards to influence the drawing.
                cumulus_primitives_core::relay_chain::well_known_keys::ONE_EPOCH_AGO_RANDOMNESS,
            )
            .ok()
            .flatten()
    }
}
impl pallet_randomness::Config for Runtime {
    type BabeDataGetter = BabeDataGetter;
    type WeightInfo = weights::pallet_randomness::SubstrateWeight<Runtime>;
}
parameter_types! {
    pub const LotteryPotId: PalletId = LOTTERY_PALLET_ID;
    /// Time in blocks between lottery drawings
    pub DrawingInterval: BlockNumber = prod_or_fast!(7 * DAYS, 3 * MINUTES);
    /// Time in blocks *before* a drawing in which modifications of the win-eligble pool are prevented
    pub DrawingFreezeout: BlockNumber = prod_or_fast!(1 * DAYS, 1 * MINUTES);
    /// Time in blocks until a collator is done unstaking
    pub UnstakeLockTime: BlockNumber = LeaveDelayRounds::get() * DefaultBlocksPerRound::get();
}
impl pallet_lottery::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = Scheduler;
    type EstimateCallFee = TransactionPayment;
    type RandomnessSource = Randomness;
    type ManageOrigin = EnsureRootOrMoreThanHalfCouncil;
    type PalletsOrigin = OriginCaller;
    type LotteryPot = LotteryPotId;
    type DrawingInterval = DrawingInterval;
    type DrawingFreezeout = DrawingFreezeout;
    type UnstakeLockTime = UnstakeLockTime;
    type WeightInfo = weights::pallet_lottery::SubstrateWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = AuthorInherent;
    type UncleGenerations = ConstU32<0>;
    type FilterUncle = ();
    type EventHandler = (CollatorSelection,);
}

parameter_types! {
    pub const NativeTokenExistentialDeposit: u128 = 10 * cMANTA; // 0.1 MANTA
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = NativeTokenExistentialDeposit;
    type AccountStore = frame_system::Pallet<Runtime>;
    type WeightInfo = weights::pallet_balances::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionLengthToFeeCoeff: Balance = 10 * uMANTA;
    pub const WeightToFeeCoeff: Balance = 50_000_000;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type WeightToFee = ConstantMultiplier<Balance, WeightToFeeCoeff>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionLengthToFeeCoeff>;
    type FeeMultiplierUpdate = MantaSlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = ConstU32<100>;
    type WeightInfo = weights::pallet_multisig::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = weights::pallet_utility::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
}

impl pallet_aura_style_filter::Config for Runtime {
    /// Nimbus filter pipeline (final) step 3:
    /// Choose 1 collator from PotentialAuthors as eligible
    /// for each slot in round-robin fashion
    type PotentialAuthors = ParachainStaking;
}
parameter_types! {
    /// Fixed percentage a collator takes off the top of due rewards
    pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(10);
    /// Default percent of inflation set aside for parachain bond every round
    pub const DefaultParachainBondReservePercent: Percent = Percent::zero();
    pub DefaultBlocksPerRound: BlockNumber = prod_or_fast!(6 * HOURS,15,"MANTA_DEFAULT_BLOCKS_PER_ROUND");
    pub LeaveDelayRounds: BlockNumber = prod_or_fast!(28,1,"MANTA_LEAVE_DELAY_ROUNDS"); // == 7 * DAYS / 6 * HOURS
}
impl pallet_parachain_staking::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BlockAuthor = AuthorInherent;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
    /// Minimum round length is 2 minutes (10 * 12 second block times)
    type MinBlocksPerRound = ConstU32<10>;
    /// Blocks per round
    type DefaultBlocksPerRound = DefaultBlocksPerRound;
    /// Rounds before the collator leaving the candidates request can be executed
    type LeaveCandidatesDelay = LeaveDelayRounds;
    /// Rounds before the candidate bond increase/decrease can be executed
    type CandidateBondLessDelay = LeaveDelayRounds;
    /// Rounds before the delegator exit can be executed
    type LeaveDelegatorsDelay = LeaveDelayRounds;
    /// Rounds before the delegator revocation can be executed
    type RevokeDelegationDelay = LeaveDelayRounds;
    /// Rounds before the delegator bond increase/decrease can be executed
    type DelegationBondLessDelay = LeaveDelayRounds;
    /// Rounds before the reward is paid
    type RewardPaymentDelay = ConstU32<2>;
    /// Minimum collators selected per round, default at genesis and minimum forever after
    type MinSelectedCandidates = ConstU32<5>;
    /// Maximum top delegations per candidate
    type MaxTopDelegationsPerCandidate = ConstU32<100>;
    /// Maximum bottom delegations per candidate
    type MaxBottomDelegationsPerCandidate = ConstU32<50>;
    /// Maximum delegations per delegator
    type MaxDelegationsPerDelegator = ConstU32<25>;
    type DefaultCollatorCommission = DefaultCollatorCommission;
    type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
    /// Minimum stake on a collator to be considered for block production
    type MinCollatorStk = ConstU128<{ crate::staking::MIN_BOND_TO_BE_CONSIDERED_COLLATOR }>;
    /// Minimum stake the collator runner must bond to register as collator candidate
    type MinCandidateStk = ConstU128<{ crate::staking::NORMAL_COLLATOR_MINIMUM_STAKE }>;
    /// WHITELIST: Minimum stake required for *a whitelisted* account to be a collator candidate
    type MinWhitelistCandidateStk = ConstU128<{ crate::staking::EARLY_COLLATOR_MINIMUM_STAKE }>;
    /// Smallest amount that can be delegated
    type MinDelegation = ConstU128<{ 500 * MANTA }>;
    /// Minimum stake required to be reserved to be a delegator
    type MinDelegatorStk = ConstU128<{ 500 * MANTA }>;
    type OnCollatorPayout = ();
    type OnNewRound = ();
    type WeightInfo = weights::pallet_parachain_staking::SubstrateWeight<Runtime>;
}

impl pallet_author_inherent::Config for Runtime {
    // We start a new slot each time we see a new relay block.
    type SlotBeacon = RelaychainBlockNumberProvider<Self>;
    type AccountLookup = CollatorSelection;
    type WeightInfo = weights::pallet_author_inherent::SubstrateWeight<Runtime>;
    /// Nimbus filter pipeline step 1:
    /// Filters out NimbusIds not registered as SessionKeys of some AccountId
    type CanAuthor = AuraAuthorFilter;
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
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = ScheduleOrigin;
    type MaxScheduledPerBlock = ConstU32<50>; // 50 scheduled calls at most in the queue for a single block.
    type WeightInfo = weights::pallet_scheduler::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = OriginPrivilegeCmp;
    type Preimages = Preimage;
}

parameter_types! {
    // Our NORMAL_DISPATCH_RATIO is 70% of the 5MB limit
    // So anything more than 3.5MB doesn't make sense here
    pub const PreimageMaxSize: u32 = 3584 * 1024;
    pub const PreimageBaseDeposit: Balance = 40 * mMANTA;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 400 * uMANTA;
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = weights::pallet_preimage::SubstrateWeight<Runtime>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

// NOTE: pallet_parachain_staking rounds are now used,
// session rotation through pallet session no longer needed
// but the pallet is used for SessionKeys storage
pub struct NeverEndSession;
impl ShouldEndSession<u32> for NeverEndSession {
    fn should_end_session(_: u32) -> bool {
        false
    }
}

parameter_types! {
    // Rotate collator's spot each 6 hours.
    pub Period: u32 = prod_or_fast!(6 * HOURS, 2 * MINUTES, "MANTA_PERIOD");
    pub const Offset: u32 = 0;
}
impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = IdentityCollator;
    type ShouldEndSession = NeverEndSession;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = ();
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

/// We allow root and the Relay Chain council to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl manta_collator_selection::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type UpdateOrigin = CollatorSelectionUpdateOrigin;
    type PotId = PotId;
    type MaxCandidates = ConstU32<50>; // 50 candidates at most
    type MaxInvulnerables = ConstU32<5>; // 5 invulnerables at most
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = IdentityCollator;
    type AccountIdOf = IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = weights::manta_collator_selection::SubstrateWeight<Runtime>;
    /// Nimbus filter pipeline step 2:
    /// Filters collators not part of the current pallet_session::validators()
    type CanAuthor = AuraAuthorFilter;
}

parameter_types! {
    pub LaunchPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 5 * MINUTES, "MANTA_LAUNCH_PERIOD");
    pub VotingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 5 * MINUTES, "MANTA_VOTING_PERIOD");
    pub FastTrackVotingPeriod: BlockNumber = prod_or_fast!(3 * HOURS, 2 * MINUTES, "MANTA_FAST_TRACK_VOTING_PERIOD");
    pub const InstantAllowed: bool = true;
    pub const MinimumDeposit: Balance = 40 * MANTA;
    pub EnactmentPeriod: BlockNumber = prod_or_fast!(1 * DAYS, 2 * MINUTES, "MANTA_ENACTMENTPERIOD");
    pub CooloffPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 2 * MINUTES, "MANTA_COOLOFFPERIOD");
}

impl pallet_democracy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type VoteLockingPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
    type InstantOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
    >;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cool-off period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type Slash = ();
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = ConstU32<100>;
    type WeightInfo = weights::pallet_democracy::SubstrateWeight<Runtime>;
    type MaxProposals = ConstU32<100>;
    type Preimages = Preimage;
    type MaxDeposits = ConstU32<100>;
    type MaxBlacklisted = ConstU32<100>;
}

parameter_types! {
    /// The maximum amount of time (in blocks) for council members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = ConstU32<100>;
    type MaxMembers = ConstU32<100>;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = weights::pallet_collective::SubstrateWeight<Runtime>;
}

pub type EnsureRootOrThreeFourthsCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
>;

type CouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<CouncilMembershipInstance> for Runtime {
    type RuntimeEvent = RuntimeEvent;
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
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = ConstU32<100>;
    type MaxMembers = ConstU32<100>;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = weights::pallet_collective::SubstrateWeight<Runtime>;
}

type TechnicalMembershipInstance = pallet_membership::Instance2;
impl pallet_membership::Config<TechnicalMembershipInstance> for Runtime {
    type RuntimeEvent = RuntimeEvent;
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
    pub const ProposalBondMinimum: Balance = 30 * MANTA;
    pub const ProposalBondMaximum: Balance = 500 * MANTA;
    pub SpendPeriod: BlockNumber = prod_or_fast!(14 * DAYS, 2 * MINUTES, "MANTA_SPEND_PERIOD");
    pub const Burn: Permill = Permill::from_percent(0);
    pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;
}

type EnsureRootOrThreeFifthsCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
>;

type EnsureRootOrMoreThanHalfCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EnsureRootOrThreeFifthsCouncil;
    type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
    type RuntimeEvent = RuntimeEvent;
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
    // Expects an implementation of `EnsureOrigin` with a `Success` generic,
    // which is the the maximum amount that this origin is allowed to spend at a time.
    type SpendOrigin = NeverEnsureOrigin<Balance>;
}

parameter_types! {
    pub const FarmingKeeperPalletId: PalletId = PalletId(*b"mt/fmkpr");
    pub const FarmingRewardIssuerPalletId: PalletId = PalletId(*b"mt/fmrir");
    pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

/// Zenlink protocol Asset adaptor for orml_traits::MultiCurrency.
type MantaCurrencies = Currencies<Runtime, assets_config::MantaAssetConfig, Balances, Assets>;

impl pallet_farming::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CurrencyId = MantaAssetId;
    type MultiCurrency = MantaCurrencies;
    type ControlOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>,
    >;
    type TreasuryAccount = TreasuryAccount;
    type Keeper = FarmingKeeperPalletId;
    type RewardIssuer = FarmingRewardIssuerPalletId;
    type WeightInfo = weights::pallet_farming::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const NameServicePalletId: PalletId = NAME_SERVICE_PALLET_ID;
}

impl pallet_name_service::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type PalletId = NameServicePalletId;
    type RegisterWaitingPeriod = ConstU32<2>;
    /// Register pricing around 5$ with estimated MANTA/USD
    type RegisterPrice = ConstU128<{ 15 * MANTA }>;
    type WeightInfo = weights::pallet_name_service::SubstrateWeight<Runtime>;
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
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,

        // Governance stuff.
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 14,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 15,
        CouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 16,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 17,
        TechnicalMembership: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 18,

        ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 48,
        // Collator support.
        AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 60,
        AuraAuthorFilter: pallet_aura_style_filter::{Pallet, Storage} = 63,
        // The order of the next 4 is important and shall not change.
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
        CollatorSelection: manta_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
        Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,

        Treasury: pallet_treasury::{Pallet, Call, Storage, Event<T>} = 26,

        // Preimage registry.
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 28,
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
        // Temporary
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 42,

        // Assets management
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 45,
        AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Config<T>, Event<T>} = 46,
        MantaPay: pallet_manta_pay::{Pallet, Call, Storage, Event<T>} = 47,
        MantaSbt: pallet_manta_sbt::{Pallet, Call, Storage, Event<T>} = 49,
        NameService: pallet_name_service::{Pallet, Call, Storage, Event<T>} = 52,

        ZenlinkProtocol: zenlink_protocol::{Pallet, Call, Storage, Event<T>} = 51,
        Farming: pallet_farming::{Pallet, Call, Storage, Event<T>} = 54,

        // Lottery
        Randomness: pallet_randomness::{Pallet, Call, Storage, Inherent} = 70,
        Lottery: pallet_lottery::{Pallet, Call, Storage, Event<T>, Config<T>} = 71 // Beware: Lottery depends on Randomness inherent

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
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Types for runtime upgrading.
/// Each type should implement trait `OnRuntimeUpgrade`.
pub type OnRuntimeUpgradeHooks = ();
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    OnRuntimeUpgradeHooks,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    frame_benchmarking::define_benchmarks!(
        // Substrate pallets
        [pallet_balances, Balances]
        [pallet_democracy, Democracy]
        [pallet_collective, Council]
        [pallet_membership, CouncilMembership]
        [pallet_multisig, Multisig]
        [frame_system, SystemBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_utility, Utility]
        [pallet_preimage, Preimage]
        [pallet_treasury, Treasury]
        [pallet_assets, Assets]
        [pallet_asset_manager, AssetManager]
        [pallet_scheduler, Scheduler]
        // XCM
        [cumulus_pallet_xcmp_queue, XcmpQueue]
        [pallet_xcm_benchmarks::fungible, pallet_xcm_benchmarks::fungible::Pallet::<Runtime>]
        [pallet_xcm_benchmarks::generic, pallet_xcm_benchmarks::generic::Pallet::<Runtime>]
        [pallet_session, SessionBench::<Runtime>]
        // Manta pallets
        [pallet_tx_pause, TransactionPause]
        [manta_collator_selection, CollatorSelection]
        [pallet_parachain_staking, ParachainStaking]
        [pallet_randomness, Randomness]
        [pallet_lottery, Lottery]
        [pallet_manta_pay, MantaPay]
        [pallet_manta_sbt, MantaSbt]
        [pallet_name_service, NameService]
        [zenlink_protocol, ZenlinkProtocol]
        [pallet_farming, Farming]
        // Nimbus pallets
        [pallet_author_inherent, AuthorInherent]
    );
}

impl_runtime_apis! {
    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            // NOTE: AuraAPI must exist for node/src/aura_or_nimbus_consensus.rs
            // But is intentionally DISABLED starting with manta v3.3.0
            vec![]
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

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
    }
    impl pallet_lottery::runtime::LotteryApi<Block> for Runtime {
        fn not_in_drawing_freezeout(
        ) -> bool {
            Lottery::not_in_drawing_freezeout()
        }
        fn current_prize_pool() -> u128 {
            Lottery::current_prize_pool()
        }
        fn next_drawing_at() -> Option<u128> {
            Lottery::next_drawing_at().map(|x| x as u128)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
        }
    }

    impl pallet_manta_pay::runtime::PullLedgerDiffApi<Block> for Runtime {
        fn pull_ledger_diff(
            checkpoint: RawCheckpoint,
            max_receiver: u64,
            max_sender: u64
        ) -> PullResponse {
            MantaPay::pull_ledger_diff(checkpoint.into(), max_receiver, max_sender)
        }
        fn pull_ledger_total_count() -> [u8; 16] {
            MantaPay::pull_ledger_total_count()
        }
        fn initial_pull(checkpoint: RawCheckpoint, max_receiver: u64) -> InitialSyncResponse {
            MantaPay::initial_pull(checkpoint.into(), max_receiver)
        }
    }

    impl nimbus_primitives::NimbusApi<Block> for Runtime {
        fn can_author(author: NimbusId, relay_parent: u32, parent_header: &<Block as BlockT>::Header) -> bool {
            let next_block_number = parent_header.number + 1;
            let slot = relay_parent;
            // Because the staking solution calculates the next staking set at the beginning
            // of the first block in the new round, the only way to accurately predict the
            // authors is to compute the selection during prediction.
            // NOTE: This logic must manually be kept in sync with the nimbus filter pipeline
            if pallet_parachain_staking::Pallet::<Self>::round().should_update(next_block_number)
            {
                // lookup account from nimbusId
                // mirrors logic in `pallet_author_inherent`
                use nimbus_primitives::AccountLookup;
                let account = match manta_collator_selection::Pallet::<Self>::lookup_account(&author) {
                    Some(account) => account,
                    // Authors whose account lookups fail will not be eligible
                    None => {
                        return false;
                    }
                };
                // manually check aura eligibility (in the new round)
                // mirrors logic in `aura_style_filter`
                let truncated_half_slot = (slot >> 1) as usize;
                let mut active: Vec<AccountId> = pallet_parachain_staking::Pallet::<Self>::compute_top_candidates();
                if active.is_empty() {
                    // `SelectedCandidates` remains unchanged from last round (fallback)
                    active = pallet_parachain_staking::Pallet::<Self>::selected_candidates();
                    if active.is_empty() {
                        log::error!("NimbusApi::can_author found no valid authors");
                        return false;
                    }
                }
                account == active[truncated_half_slot % active.len()]
            } else {
                // We're not changing rounds, `PotentialAuthors` is not changing, just use can_author
                <AuthorInherent as nimbus_primitives::CanAuthor<_>>::can_author(&author, &relay_parent)
            }
        }
    }

    impl pallet_manta_sbt::runtime::SBTPullLedgerDiffApi<Block> for Runtime {
        fn sbt_pull_ledger_diff(
            checkpoint: RawCheckpoint,
            max_receiver: u64,
            max_sender: u64
        ) -> PullResponse {
            MantaSbt::pull_ledger_diff(checkpoint.into(), max_receiver, max_sender)
        }
        fn sbt_pull_ledger_total_count() -> [u8; 16] {
            MantaSbt::pull_ledger_total_count()
        }
    }

    // zenlink runtime outer apis
    impl zenlink_protocol_runtime_api::ZenlinkProtocolApi<Block, AccountId, ZenlinkAssetId> for Runtime {

        fn get_balance(
            asset_id: ZenlinkAssetId,
            owner: AccountId
        ) -> AssetBalance {
            <<Runtime as zenlink_protocol::Config>::MultiAssetsHandler as MultiAssetsHandler<AccountId, ZenlinkAssetId>>::balance_of(asset_id, &owner)
        }

        fn get_pair_by_asset_id(
            asset_0: ZenlinkAssetId,
            asset_1: ZenlinkAssetId
        ) -> Option<PairInfo<AccountId, AssetBalance, ZenlinkAssetId>> {
            ZenlinkProtocol::get_pair_by_asset_id(asset_0, asset_1)
        }

        fn get_amount_in_price(
            supply: AssetBalance,
            path: Vec<ZenlinkAssetId>
        ) -> AssetBalance {
            ZenlinkProtocol::desired_in_amount(supply, path)
        }

        fn get_amount_out_price(
            supply: AssetBalance,
            path: Vec<ZenlinkAssetId>
        ) -> AssetBalance {
            ZenlinkProtocol::supply_out_amount(supply, path)
        }

        fn get_estimate_lptoken(
            token_0: ZenlinkAssetId,
            token_1: ZenlinkAssetId,
            amount_0_desired: AssetBalance,
            amount_1_desired: AssetBalance,
            amount_0_min: AssetBalance,
            amount_1_min: AssetBalance,
        ) -> AssetBalance{
            ZenlinkProtocol::get_estimate_lptoken(
                token_0,
                token_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min
            )
        }

        fn calculate_remove_liquidity(
            asset_0: ZenlinkAssetId,
            asset_1: ZenlinkAssetId,
            amount: AssetBalance,
        ) -> Option<(AssetBalance, AssetBalance)> {
            ZenlinkProtocol::calculate_remove_liquidity(
                asset_0,
                asset_1,
                amount
            )
        }
    }

    impl pallet_farming_rpc_runtime_api::FarmingRuntimeApi<Block, AccountId, MantaAssetId, PoolId> for Runtime {
        fn get_farming_rewards(who: AccountId, pid: PoolId) -> Vec<(MantaAssetId, Balance)> {
            Farming::get_farming_rewards(&who, pid).unwrap_or(Vec::new())
        }

        fn get_gauge_rewards(who: AccountId, pid: PoolId) -> Vec<(MantaAssetId, Balance)> {
            Farming::get_gauge_rewards(&who, pid).unwrap_or(Vec::new())
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, RuntimeBlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            Executive::try_execute_block(block, state_root_check, signature_check, select).expect("try_execute_block failed")
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();
            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey, BenchmarkError};

            use frame_system_benchmarking::Pallet as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
            impl cumulus_pallet_session_benchmarking::Config for Runtime {}

            use pallet_xcm_benchmarks::asset_instance_from;
            use xcm_config::{LocationToAccountId, XcmExecutorConfig};

            parameter_types! {
                pub const TrustedTeleporter: Option<(MultiLocation, MultiAsset)> = None;
                pub const TrustedReserve: Option<(MultiLocation, MultiAsset)> = Some((
                    DotLocation::get(),
                    // Random amount for the benchmark.
                    MultiAsset { fun: Fungible(1_000_000_000_000), id: Concrete(DotLocation::get()) },
                ));
                pub const CheckedAccount: Option<AccountId> = None;
                pub const DotLocation: MultiLocation = MultiLocation::parent();
                pub MantaLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
            }

            impl pallet_xcm_benchmarks::Config for Runtime {
                type XcmConfig = XcmExecutorConfig;
                type AccountIdConverter = LocationToAccountId;

                fn valid_destination() -> Result<MultiLocation, BenchmarkError> {
                 Ok(DotLocation::get())
                }

                fn worst_case_holding() -> MultiAssets {
                    // A mix of fungible, non-fungible, and concrete assets.
                    const HOLDING_FUNGIBLES: u32 = 100;
                    const HOLDING_NON_FUNGIBLES: u32 = 100;
                    let fungibles_amount: u128 = 100;
                    let mut assets = (0..HOLDING_FUNGIBLES)
                        .map(|i| {
                            MultiAsset {
                                id: Concrete(GeneralIndex(i as u128).into()),
                                fun: Fungible(fungibles_amount * i as u128),
                            }
                        })
                        .chain(core::iter::once(MultiAsset { id: Concrete(Here.into()), fun: Fungible(u128::MAX) }))
                        .chain((0..HOLDING_NON_FUNGIBLES).map(|i| MultiAsset {
                            id: Concrete(GeneralIndex(i as u128).into()),
                            fun: NonFungible(asset_instance_from(i)),
                        }))
                        .collect::<Vec<_>>();

                        assets.push(MultiAsset{
                            id: Concrete(MantaLocation::get()),
                            fun: Fungible(1_000_000 * MANTA),
                        });
                        assets.into()
                }
            }

            impl pallet_xcm_benchmarks::fungible::Config for Runtime {
                type TransactAsset = Balances;

                type CheckedAccount = CheckedAccount;
                type TrustedTeleporter = TrustedTeleporter;
                type TrustedReserve = TrustedReserve;

                fn get_multi_asset() -> MultiAsset {
                    MultiAsset {
                        id: Concrete(MantaLocation::get()),
                        fun: Fungible(1 * MANTA),
                    }
                }
            }

            impl pallet_xcm_benchmarks::generic::Config for Runtime {
                type RuntimeCall = RuntimeCall;

                fn worst_case_response() -> (u64, Response) {
                    (0u64, Response::Version(Default::default()))
                }

                fn transact_origin() -> Result<MultiLocation, BenchmarkError> {
                    Ok(DotLocation::get())
                }

                fn subscribe_origin() -> Result<MultiLocation, BenchmarkError> {
                    Ok(DotLocation::get())
                }

                fn claimable_asset() -> Result<(MultiLocation, MultiLocation, MultiAssets), BenchmarkError> {
                    let origin = MantaLocation::get();
                    let assets: MultiAssets = (Concrete(MantaLocation::get()), 1_000 * MANTA).into();
                    let ticket = MultiLocation { parents: 0, interior: Here };
                    Ok((origin, ticket, assets))
                }
            }

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // ParachainStaking Round
                hex_literal::hex!("a686a3043d0adcf2fa655e57bc595a7813792e785168f725b60e2969c7fc2552").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
                // Treasury Account
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }
}

struct CheckInherentsStruct;
impl CheckInherents<Block> for CheckInherentsStruct {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &RelayChainStateProof,
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

register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherentsStruct,
}

impl parachain_info::Config for Runtime {}
