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

#![cfg(test)]

pub mod integration_tests;
pub mod mock;

use calamari_runtime::opaque::SessionKeys;
pub use calamari_runtime::{currency::KMA, Event, Origin, Runtime, System};

use frame_support::weights::{DispatchInfo, Weight};
use lazy_static::lazy_static;
use manta_primitives::types::{AccountId, Balance};
use session_key_primitives::util::{unchecked_account_id, unchecked_collator_keys};
use sp_core::sr25519::Public;
#[cfg(feature = "std")]
pub(crate) use std::clone::Clone;

pub const COLLATOR_MIN_BOND: Balance = 4_000_000 * KMA;
pub const WHITELIST_MIN_BOND: Balance = 400_000 * KMA;
pub const INITIAL_BALANCE: Balance = 1_000_000_000_000 * KMA;

lazy_static! {
    pub(crate) static ref ALICE: AccountId = unchecked_account_id::<Public>("Alice");
    pub(crate) static ref BOB: AccountId = unchecked_account_id::<Public>("Bob");
    pub(crate) static ref CHARLIE: AccountId = unchecked_account_id::<Public>("Charlie");
    pub(crate) static ref DAVE: AccountId = unchecked_account_id::<Public>("Dave");
    pub(crate) static ref EVE: AccountId = unchecked_account_id::<Public>("Eve");
    pub(crate) static ref FERDIE: AccountId = unchecked_account_id::<Public>("Ferdie");
    pub(crate) static ref ALICE_SESSION_KEYS: SessionKeys =
        SessionKeys::new(unchecked_collator_keys("Alice"));
    pub(crate) static ref BOB_SESSION_KEYS: SessionKeys =
        SessionKeys::new(unchecked_collator_keys("Bob"));
    pub(crate) static ref CHARLIE_SESSION_KEYS: SessionKeys =
        SessionKeys::new(unchecked_collator_keys("Charlie"));
    pub(crate) static ref DAVE_SESSION_KEYS: SessionKeys =
        SessionKeys::new(unchecked_collator_keys("Dave"));
    pub(crate) static ref EVE_SESSION_KEYS: SessionKeys =
        SessionKeys::new(unchecked_collator_keys("Eve"));
    pub(crate) static ref FERDIE_SESSION_KEYS: SessionKeys =
        SessionKeys::new(unchecked_collator_keys("Ferdie"));
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
    // pays_fee: Pays::Yes -- class: DispatchClass::Normal
    DispatchInfo {
        weight: w,
        ..Default::default()
    }
}

pub fn last_event() -> Event {
    System::events().pop().expect("Event expected").event
}

pub fn root_origin() -> <Runtime as frame_system::Config>::Origin {
    <Runtime as frame_system::Config>::Origin::root()
}
