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

pub use frame_support::traits::Get;
use manta_collator_selection::IdentityCollator;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perbill, Percent,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU128, ConstU16, ConstU32, ConstU8, Contains, Currency},
    weights::{ConstantMultiplier, DispatchClass, Weight},
    PalletId,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use manta_primitives::{
    constants::{time::*, RocksDbWeight, STAKING_PALLET_ID, WEIGHT_PER_SECOND},
    types::{AccountId, Balance, BlockNumber, Hash, Header, Index, Signature},
};
pub use pallet_parachain_staking::{InflationInfo, Range};
use pallet_session::ShouldEndSession;
use runtime_common::{
    prod_or_fast, BlockExecutionWeight, BlockHashCount, ExtrinsicBaseWeight, SlowAdjustingFeeUpdate,
};
use session_key_primitives::{AuraId, NimbusId, VrfId};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub mod currency;
pub mod fee;
pub mod impls;
mod nimbus_session_adapter;
pub mod staking;
pub mod xcm_config;

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
    spec_version: 4001,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
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
    pub const SS58Prefix: u8 = manta_primitives::constants::MANTA_SS58PREFIX;
}

// Don't allow permission-less asset creation.
pub struct MantaFilter;
impl Contains<Call> for MantaFilter {
    fn contains(call: &Call) -> bool {
        if matches!(
            call,
            Call::Timestamp(_) | Call::ParachainSystem(_) | Call::System(_)
        ) {
            // always allow core call
            // pallet-timestamp and parachainSystem could not be filtered because they are used in communication between releychain and parachain.
            return true;
        }

        #[allow(clippy::match_like_matches_macro)]
        // keep CallFilter with explicit true/false for documentation
        match call {
            // Explicitly DISALLOWED calls ( Pallet user extrinsics we don't want used WITH REASONING )
            // Explicitly ALLOWED calls
            | Call::Authorship(_)
            // Sudo also cannot be filtered because it is used in runtime upgrade.
            | Call::Sudo(_)
            | Call::Multisig(_)
            | Call::AuthorInherent(pallet_author_inherent::Call::kick_off_authorship_validation {..}) // executes unsigned on every block
            | Call::Balances(_)
            | Call::Preimage(_)
            | Call::Utility(_) => true,

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
    type FindAuthor = AuthorInherent;
    type UncleGenerations = ConstU32<0>;
    type FilterUncle = ();
    type EventHandler = (CollatorSelection,);
}

parameter_types! {
    pub const NativeTokenExistentialDeposit: u128 = MANTA;
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
    /// Relay Chain `TransactionLengthToFeeCoeff` / 10
    pub const TransactionLengthToFeeCoeff: Balance = mMANTA / 10;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type WeightToFee = WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionLengthToFeeCoeff>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type Event = Event;
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
    pub DefaultBlocksPerRound: BlockNumber = prod_or_fast!(6 * HOURS,15,"MANTA_DEFAULTBLOCKSPERROUND");
    pub LeaveDelayRounds: BlockNumber = prod_or_fast!(28,1,"MANTA_LEAVEDELAYROUNDS"); // == 7 * DAYS / 6 * HOURS
}
impl pallet_parachain_staking::Config for Runtime {
    type Event = Event;
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
    type MinDelegation = ConstU128<{ 50 * MANTA }>;
    /// Minimum stake required to be reserved to be a delegator
    type MinDelegatorStk = ConstU128<{ 50 * MANTA }>;
    type OnCollatorPayout = ();
    type OnNewRound = ();
    type WeightInfo = weights::pallet_parachain_staking::SubstrateWeight<Runtime>;
}

impl pallet_author_inherent::Config for Runtime {
    // We start a new slot each time we see a new relay block.
    type SlotBeacon = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
    type AccountLookup = CollatorSelection;
    type WeightInfo = weights::pallet_author_inherent::SubstrateWeight<Runtime>;
    /// Nimbus filter pipeline step 1:
    /// Filters out NimbusIds not registered as SessionKeys of some AccountId
    type CanAuthor = AuraAuthorFilter;
}

parameter_types! {
    // Our NORMAL_DISPATCH_RATIO is 70% of the 5MB limit
    // So anything more than 3.5MB doesn't make sense here
    pub const PreimageMaxSize: u32 = 3584 * 1024;
    pub const PreimageBaseDeposit: Balance = 1 * MANTA;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 1 * cMANTA;
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = weights::pallet_preimage::SubstrateWeight<Runtime>;
    type Event = Event;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type DmpMessageHandler = ();
    type ReservedDmpWeight = ();
    type OutboundXcmpMessageSource = ();
    type XcmpMessageHandler = ();
    type ReservedXcmpWeight = ();
    type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

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
    type Event = Event;
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
    type Event = Event;
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

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,

        ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 48,
        // Collator support.
        AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 60,
        AuraAuthorFilter: pallet_aura_style_filter::{Pallet, Storage} = 63,
        // The order of the next 4 is important and shall not change.
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
        CollatorSelection: manta_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
        Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,

        // Preimage registry.
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 28,

        // XCM helpers.
        PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 31,
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

        // Handy utilities.
        Utility: pallet_utility::{Pallet, Call, Event} = 40,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 41,
        // Temporary
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 42,
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

/// Types for runtime upgrading.
/// Each type should implement trait `OnRuntimeUpgrade`.
pub type OnRuntimeUpgradeHooks = ();
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsReversedWithSystemFirst,
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
        [pallet_multisig, Multisig]
        [frame_system, SystemBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_utility, Utility]
        [pallet_preimage, Preimage]
        [pallet_session, SessionBench::<Runtime>]
        // Manta pallets
        [manta_collator_selection, CollatorSelection]
        [pallet_parachain_staking, ParachainStaking]
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

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, Call>
        for Runtime
    {
        fn query_call_info(
            call: Call,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: Call,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
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
                let active: Vec<AccountId> = pallet_parachain_staking::Pallet::<Self>::compute_top_candidates();
                account == active[truncated_half_slot % active.len()]
            } else {
                // We're not changing rounds, `PotentialAuthors` is not changing, just use can_author
                <AuthorInherent as nimbus_primitives::CanAuthor<_>>::can_author(&author, &relay_parent)
            }
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
            use frame_benchmarking::{Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsReversedWithSystemFirst::storage_info();
            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};

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
                // ParachainStaking Round
                hex_literal::hex!("a686a3043d0adcf2fa655e57bc595a7813792e785168f725b60e2969c7fc2552").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

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
    BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}
