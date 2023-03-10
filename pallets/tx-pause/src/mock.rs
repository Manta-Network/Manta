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
//
// The pallet-tx-pause pallet is forked from Acala's transaction-pause module https://github.com/AcalaNetwork/Acala/tree/master/modules/transaction-pause
// The original license is the following - SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

//! Mocks for the tx pause pallet.

#![cfg(test)]

use super::*;
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{ConstU32, IsInVec},
};
use frame_system::EnsureRoot;
use manta_primitives::types::{Balance, BlockNumber, Header};

use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

pub type AccountId = u128;

mod tx_pause {
    pub use super::super::*;
}

pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
    fn contains(call: &Call) -> bool {
        // filter paused calls
        !tx_pause::PausedTransactionFilter::<Runtime>::contains(call)
    }
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU32<250>;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = BaseFilter;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const NativeTokenExistentialDeposit: Balance = 10;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = NativeTokenExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = ();
    type WeightInfo = ();
}

ord_parameter_types! {
    pub const One: AccountId = 1;
}

parameter_types! {
    pub NonPausablePallets: Vec<Vec<u8>> = vec![b"Democracy".to_vec(), b"Balances".to_vec(), b"Council".to_vec(), b"CouncilCollective".to_vec(), b"TechnicalCommittee".to_vec(), b"TechnicalCollective".to_vec()];
}

impl Config for Runtime {
    type Event = Event;
    type Call = Call;
    type MaxCallNames = ConstU32<10>;
    type PauseOrigin = EnsureRoot<AccountId>;
    type UnpauseOrigin = EnsureRoot<AccountId>;
    type NonPausablePallets = IsInVec<NonPausablePallets>;
    type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TransactionPause: tx_pause::{Pallet, Storage, Call, Event<T>},
        Balances: pallet_balances::{Pallet, Storage, Call, Event<T>},
    }
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        t.into()
    }
}
