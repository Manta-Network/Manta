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

//! Test utilities
use core::marker::PhantomData;

use crate as pallet_lottery;
use crate::{pallet, Config};
use calamari_runtime::currency::KMA;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU128, ConstU32, Everything, GenesisBuild, OnFinalize, OnInitialize},
    weights::Weight,
};
use frame_system::pallet_prelude::*;
use manta_primitives::types::{BlockNumber, Header};
use pallet_parachain_staking::{InflationInfo, Range};
use sp_core::H256;

use sp_runtime::{
    traits::{BlakeTwo256, Hash, IdentityLookup},
    Perbill, Percent,
};

pub type AccountId = u64;
pub type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Config<T>, Event<T>},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        BlockAuthor: block_author::{Pallet, Storage},
        CollatorSelection: manta_collator_selection::{Pallet, Call, Storage, Config<T>, Event<T>},
        Lottery: pallet_lottery::{Pallet, Call, Storage, Event<T>, Config<T>},
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>},
    }
);

// Randomness trait
pub struct TestRandomness<T> {
    _marker: PhantomData<T>,
}
impl<T: Config> frame_support::traits::Randomness<T::Hash, BlockNumberFor<T>>
    for TestRandomness<T>
{
    fn random(subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
        use rand::{rngs::OsRng, RngCore};
        let mut digest: Vec<_> = [0u8; 32].into();
        OsRng.fill_bytes(&mut digest);
        digest.extend_from_slice(subject);
        let randomness = T::Hashing::hash(&digest);
        // NOTE: Test randomness is always "fresh" assuming block_number is > DrawingFreezeout
        let block_number = 0u32.into();
        (randomness, block_number)
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: u64 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const SS58Prefix: u8 = manta_primitives::constants::CALAMARI_SS58PREFIX;
}
impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}
parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Test {
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 4];
    type MaxLocks = ();
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    // Our NORMAL_DISPATCH_RATIO is 70% of the 5MB limit
    // So anything more than 3.5MB doesn't make sense here
    pub const PreimageMaxSize: u32 = 3584 * 1024;
}

impl pallet_preimage::Config for Test {
    type WeightInfo = calamari_runtime::weights::pallet_preimage::SubstrateWeight<Test>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    // The sum of the below 2 amounts will get reserved every time someone submits a preimage.
    // Their sum will be unreserved when the preimage is requested, i.e. when it is going to be used.
    type BaseDeposit = ConstU128<
        {
            /* 1 */
            KMA
        },
    >;
    type ByteDeposit = ConstU128<
        {
            /* 1 */
            KMA
        },
    >;
}
use sp_std::cmp::Ordering;
pub struct OriginPrivilegeCmp;
impl frame_support::traits::PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
    fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
        if left == right {
            return Some(Ordering::Equal);
        }
        match (left, right) {
            (OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
            _ => None,
        }
    }
}
parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * calamari_runtime::MAXIMUM_BLOCK_WEIGHT;
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}
impl pallet_scheduler::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = ConstU32<50>; // 50 scheduled calls at most in the queue for a single block.
    type WeightInfo = calamari_runtime::weights::pallet_scheduler::SubstrateWeight<Test>;
    type OriginPrivilegeCmp = OriginPrivilegeCmp;
    type Preimages = Preimage;
}

pub struct IsRegistered;
impl ValidatorRegistration<u64> for IsRegistered {
    fn is_registered(id: &u64) -> bool {
        *id != 7u64
    }
}
impl ValidatorSet<u64> for IsRegistered {
    type ValidatorId = u64;
    type ValidatorIdOf = manta_collator_selection::IdentityCollator;
    fn session_index() -> sp_staking::SessionIndex {
        1
    }
    fn validators() -> Vec<Self::ValidatorId> {
        vec![]
    }
}
parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
}
impl manta_collator_selection::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type PotId = PotId;
    type MaxCandidates = ConstU32<20>;
    type MaxInvulnerables = ConstU32<20>;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = manta_collator_selection::IdentityCollator;
    type AccountIdOf = manta_collator_selection::IdentityCollator;
    type ValidatorRegistration = IsRegistered;
    type WeightInfo = ();
    type CanAuthor = ();
}

parameter_types! {
    /// Fixed percentage a collator takes off the top of due rewards
    pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(10);
    /// Default percent of inflation set aside for parachain bond every round
    pub const DefaultParachainBondReservePercent: Percent = Percent::zero();
    pub DefaultBlocksPerRound: BlockNumber = 15;
    pub LeaveDelayRounds: BlockNumber = 1; // == 7 * DAYS / 6 * HOURS = 28
}
impl pallet_parachain_staking::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BlockAuthor = BlockAuthor;
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
    type MinCollatorStk =
        ConstU128<{ calamari_runtime::staking::MIN_BOND_TO_BE_CONSIDERED_COLLATOR }>;
    /// Minimum stake the collator runner must bond to register as collator candidate
    type MinCandidateStk = ConstU128<{ calamari_runtime::staking::NORMAL_COLLATOR_MINIMUM_STAKE }>;
    /// WHITELIST: Minimum stake required for *a whitelisted* account to be a collator candidate
    type MinWhitelistCandidateStk =
        ConstU128<{ calamari_runtime::staking::EARLY_COLLATOR_MINIMUM_STAKE }>;
    /// Smallest amount that can be delegated
    type MinDelegation = ConstU128<{ 5_000 * KMA }>;
    /// Minimum stake required to be reserved to be a delegator
    type MinDelegatorStk = ConstU128<{ 5_000 * KMA }>;
    type OnCollatorPayout = ();
    type OnNewRound = ();
    type WeightInfo = calamari_runtime::weights::pallet_parachain_staking::SubstrateWeight<Test>; // XXX: Maybe use the actual calamari weights?
}

impl block_author::Config for Test {}

use frame_support::PalletId;
use frame_system::EnsureRoot;
use manta_primitives::constants::LOTTERY_PALLET_ID;
parameter_types! {
    pub const LotteryPotId: PalletId = LOTTERY_PALLET_ID; // ensure we don't deposit/withdraw in the drawing block
    /// Time in blocks between lottery drawings
    pub DrawingInterval: BlockNumber = DefaultBlocksPerRound::get();
    /// Time in blocks *before* a drawing in which modifications of the win-eligble pool are prevented
    pub DrawingFreezeout: BlockNumber = 5;
    /// Time in blocks until a collator is done unstaking
    pub UnstakeLockTime: BlockNumber = LeaveDelayRounds::get() * DefaultBlocksPerRound::get();
}

use frame_support::traits::Currency;
pub type BalanceOf<T> = <<T as pallet_parachain_staking::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
pub struct MockEstimateFee {}
impl frame_support::traits::EstimateCallFee<pallet_parachain_staking::Call<Test>, BalanceOf<Test>>
    for MockEstimateFee
{
    fn estimate_call_fee(
        _call: &pallet_parachain_staking::Call<Test>,
        _post_info: frame_support::dispatch::PostDispatchInfo,
    ) -> BalanceOf<Test> {
        7 * KMA
    }
}
impl frame_support::traits::EstimateCallFee<pallet::Call<Test>, BalanceOf<Test>>
    for MockEstimateFee
{
    fn estimate_call_fee(
        _call: &pallet::Call<Test>,
        _post_info: frame_support::dispatch::PostDispatchInfo,
    ) -> BalanceOf<Test> {
        3 * KMA
    }
}
impl Config for Test {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = Scheduler;
    type EstimateCallFee = MockEstimateFee;
    type RandomnessSource = TestRandomness<Test>;
    type ManageOrigin = frame_system::EnsureRoot<AccountId>;
    type PalletsOrigin = OriginCaller;
    type LotteryPot = LotteryPotId;
    type DrawingInterval = DrawingInterval;
    type DrawingFreezeout = DrawingFreezeout;
    type UnstakeLockTime = UnstakeLockTime;
    type WeightInfo = ();
}

use frame_support::traits::{ValidatorRegistration, ValidatorSet};

pub(crate) struct ExtBuilder {
    // endowed accounts with balances
    balances: Vec<(AccountId, Balance)>,
    // [collator, amount]
    collators: Vec<(AccountId, Balance)>,
    // [delegator, collator, delegation_amount]
    delegations: Vec<(AccountId, AccountId, Balance)>,
    // inflation config
    inflation: InflationInfo<Balance>,
}

impl Default for ExtBuilder {
    fn default() -> ExtBuilder {
        ExtBuilder {
            balances: vec![],
            delegations: vec![],
            collators: vec![],
            inflation: InflationInfo {
                expect: Range {
                    min: 700,
                    ideal: 700,
                    max: 700,
                },
                // not used
                annual: Range {
                    min: Perbill::from_percent(50),
                    ideal: Perbill::from_percent(50),
                    max: Perbill::from_percent(50),
                },
                // unrealistically high parameterization, only for testing
                round: Range {
                    min: Perbill::from_percent(5),
                    ideal: Perbill::from_percent(5),
                    max: Perbill::from_percent(5),
                },
            },
        }
    }
}

impl ExtBuilder {
    pub(crate) fn with_funded_lottery_account(mut self, balance: Balance) -> Self {
        self.balances
            .push((crate::Pallet::<Test>::account_id(), balance));
        self
    }

    pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub(crate) fn with_candidates(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
        self.collators = collators;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
        self.inflation = inflation;
        self
    }

    pub(crate) fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .expect("Frame system builds valid default genesis config");

        pallet_balances::GenesisConfig::<Test> {
            balances: self.balances,
        }
        .assimilate_storage(&mut t)
        .expect("Pallet balances storage can be assimilated");
        pallet_parachain_staking::GenesisConfig::<Test> {
            candidates: self.collators,
            delegations: self.delegations,
            inflation_config: self.inflation,
        }
        .assimilate_storage(&mut t)
        .expect("Parachain Staking's storage can be assimilated");
        pallet_lottery::GenesisConfig::<Test> {
            min_deposit: 5_000 * KMA,
            min_withdraw: 5_000 * KMA,
            gas_reserve: 10_000 * KMA,
        }
        .assimilate_storage(&mut t)
        .expect("pallet_lottery's storage can be assimilated");

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub mod from_bench {
    /// copied from frame benchmarking
    use super::*;
    use codec::{Decode, Encode};
    use frame_support::traits::Get;
    use sp_io::hashing::blake2_256;
    use sp_runtime::traits::TrailingZeroInput;
    pub fn account<AccountId: Decode>(name: &'static str, index: u32, seed: u32) -> AccountId {
        let entropy = (name, index, seed).using_encoded(blake2_256);
        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed")
    }
    pub fn create_funded_user<T: Config>(
        string: &'static str,
        n: u32,
        extra: BalanceOf<T>,
    ) -> (T::AccountId, BalanceOf<T>) {
        const SEED: u32 = 0;
        let user = account(string, n, SEED);
        let min_candidate_stk =
            <<T as pallet_parachain_staking::Config>::MinCandidateStk as Get<BalanceOf<T>>>::get();
        let total = min_candidate_stk + extra;
        <T as pallet_parachain_staking::Config>::Currency::make_free_balance_be(&user, total);
        <T as pallet_parachain_staking::Config>::Currency::issue(total);
        (user, total)
    }
}

/// Rolls forward one block. Returns the new block number.
pub(crate) fn roll_one_block() -> u32 {
    Balances::on_finalize(System::block_number());
    System::on_finalize(System::block_number());
    System::set_block_number(System::block_number() + 1);
    System::on_initialize(System::block_number());
    Balances::on_initialize(System::block_number());
    ParachainStaking::on_initialize(System::block_number());
    Scheduler::on_initialize(System::block_number());
    System::block_number()
}

/// Rolls to the desired block. Returns the number of blocks played.
pub(crate) fn roll_to(n: u32) -> u32 {
    let mut num_blocks = 0;
    let mut block = System::block_number();
    while block < n {
        block = roll_one_block();
        num_blocks += 1;
    }
    num_blocks
}

/// Rolls block-by-block to the beginning of the specified round.
/// This will complete the block in which the round change occurs.
/// Returns the number of blocks played.
pub(crate) fn roll_to_round_begin(round: u32) -> u32 {
    let block = (round - 1) * DefaultBlocksPerRound::get();
    roll_to(block)
}

/// Rolls block-by-block to the end of the specified round.
/// The block following will be the one in which the specified round change occurs.
pub(crate) fn roll_to_round_end(round: u32) -> u32 {
    let block = round * DefaultBlocksPerRound::get() - 1;
    roll_to(block)
}

pub(crate) fn last_event() -> RuntimeEvent {
    System::events().pop().expect("Event expected").event
}

/// Assert input equal to the last event emitted
#[macro_export]
macro_rules! assert_last_event {
    ($event:expr) => {
        match &$event {
            e => assert_eq!(*e, $crate::mock::last_event()),
        }
    };
}

#[frame_support::pallet]
pub mod block_author {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::Get};

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn block_author)]
    pub(super) type BlockAuthor<T> = StorageValue<_, AccountId, ValueQuery>;

    impl<T: Config> Get<AccountId> for Pallet<T> {
        fn get() -> AccountId {
            <BlockAuthor<T>>::get()
        }
    }
}

#[test]
fn roll_to_round_begin_works() {
    ExtBuilder::default().build().execute_with(|| {
        // these tests assume blocks-per-round of 15, as established by DefaultBlocksPerRound
        assert_eq!(System::block_number(), 1); // we start on block 1

        let num_blocks = roll_to_round_begin(1);
        assert_eq!(System::block_number(), 1); // no-op, we're already on this round
        assert_eq!(num_blocks, 0);

        let num_blocks = roll_to_round_begin(2);
        assert_eq!(System::block_number(), 15);
        assert_eq!(num_blocks, 14);

        let num_blocks = roll_to_round_begin(3);
        assert_eq!(System::block_number(), 30);
        assert_eq!(num_blocks, 15);
    });
}

#[test]
fn roll_to_round_end_works() {
    ExtBuilder::default().build().execute_with(|| {
        // these tests assume blocks-per-round of 15, as established by DefaultBlocksPerRound
        assert_eq!(System::block_number(), 1); // we start on block 1

        let num_blocks = roll_to_round_end(1);
        assert_eq!(System::block_number(), 14);
        assert_eq!(num_blocks, 13);

        let num_blocks = roll_to_round_end(2);
        assert_eq!(System::block_number(), 29);
        assert_eq!(num_blocks, 15);

        let num_blocks = roll_to_round_end(3);
        assert_eq!(System::block_number(), 44);
        assert_eq!(num_blocks, 15);
    });
}
