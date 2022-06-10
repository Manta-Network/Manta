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

use super::*;
use crate as collator_selection;
use frame_support::{
    ord_parameter_types, parameter_types,
    traits::{
        ConstU16, ConstU32, ConstU64, FindAuthor, GenesisBuild, ValidatorRegistration, ValidatorSet,
    },
    PalletId,
};

use frame_system::EnsureSignedBy;
use sp_arithmetic::Percent;
use sp_core::H256;
use sp_runtime::{
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
    RuntimeAppPublic,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        CollatorSelection: collator_selection::{Pallet, Call, Storage, Event<T>},
        Aura: pallet_aura::{Pallet, Storage, Config<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<78>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<5>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
}

pub struct Author4;
impl FindAuthor<u64> for Author4 {
    fn find_author<'a, I>(_digests: I) -> Option<u64>
    where
        I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
    {
        Some(4)
    }
}

impl pallet_authorship::Config for Test {
    type FindAuthor = Author4;
    type UncleGenerations = ();
    type FilterUncle = ();
    type EventHandler = CollatorSelection;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

impl pallet_aura::Config for Test {
    type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
    type MaxAuthorities = ConstU32<100_000>;
    type DisabledValidators = ();
}

sp_runtime::impl_opaque_keys! {
    pub struct MockSessionKeys {
        // a key for aura authoring
        pub aura: UintAuthorityId,
    }
}

impl From<UintAuthorityId> for MockSessionKeys {
    fn from(aura: sp_runtime::testing::UintAuthorityId) -> Self {
        Self { aura }
    }
}

parameter_types! {
    pub static SessionHandlerCollators: Vec<u64> = Vec::new();
    pub static SessionChangeBlock: u64 = 0;
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<u64> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
    fn on_genesis_session<Ks: OpaqueKeys>(keys: &[(u64, Ks)]) {
        SessionHandlerCollators::set(keys.iter().map(|(a, _)| *a).collect::<Vec<_>>())
    }
    fn on_new_session<Ks: OpaqueKeys>(_: bool, keys: &[(u64, Ks)], _: &[(u64, Ks)]) {
        SessionChangeBlock::set(System::block_number());
        SessionHandlerCollators::set(keys.iter().map(|(a, _)| *a).collect::<Vec<_>>())
    }
    fn on_before_session_ending() {}
    fn on_disabled(_: u32) {}
}

parameter_types! {
    pub const Offset: u64 = 0;
    pub const Period: u64 = 10;
}

impl pallet_session::Config for Test {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    type SessionHandler = TestSessionHandler;
    type Keys = MockSessionKeys;
    type WeightInfo = ();
}

ord_parameter_types! {
    pub const RootAccount: u64 = 777;
}

parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
}

pub struct IsRegistered;
impl ValidatorRegistration<u64> for IsRegistered {
    fn is_registered(id: &u64) -> bool {
        *id != 7u64
    }
}

impl ValidatorSet<u64> for IsRegistered {
    type ValidatorId = u64;
    type ValidatorIdOf = IdentityCollator;
    fn session_index() -> sp_staking::SessionIndex {
        Session::current_index()
    }
    fn validators() -> Vec<Self::ValidatorId> {
        Session::validators()
    }
}

impl Config for Test {
    type Event = Event;
    type Currency = Balances;
    type UpdateOrigin = EnsureSignedBy<RootAccount, u64>;
    type PotId = PotId;
    type MaxCandidates = ConstU32<20>;
    type MaxInvulnerables = ConstU32<20>;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = IdentityCollator;
    type AccountIdOf = IdentityCollator;
    type ValidatorRegistration = IsRegistered;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let invulnerables = vec![1, 2];
    let keys = invulnerables
        .iter()
        .map(|i| {
            (
                *i,
                *i,
                MockSessionKeys {
                    aura: UintAuthorityId(*i),
                },
            )
        })
        .collect::<Vec<_>>();

    let balances = pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)],
    };
    let collator_selection = collator_selection::GenesisConfig::<Test> {
        desired_candidates: 2,
        candidacy_bond: 10,
        eviction_baseline: Percent::from_percent(80),
        eviction_tolerance: Percent::from_percent(10),
        invulnerables,
    };
    let session = pallet_session::GenesisConfig::<Test> { keys };
    balances.assimilate_storage(&mut t).unwrap();
    // collator selection must be initialized before session.
    collator_selection.assimilate_storage(&mut t).unwrap();
    session.assimilate_storage(&mut t).unwrap();

    t.into()
}

pub fn initialize_to_block(n: u64) {
    for i in System::block_number() + 1..=n {
        System::set_block_number(i);
        <AllPalletsReversedWithSystemFirst as frame_support::traits::OnInitialize<u64>>::on_initialize(i);
    }
}
