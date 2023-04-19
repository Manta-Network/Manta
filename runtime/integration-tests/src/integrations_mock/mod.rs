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

#![cfg(test)]

pub mod calamari;
pub mod common;
pub mod manta;
pub mod mock;

pub use mock::run_to_block;

cfg_if::cfg_if! {
    if #[cfg(feature = "calamari")] {
        pub use calamari_runtime::{
            currency::KMA,
            fee::{FEES_PERCENTAGE_TO_AUTHOR, FEES_PERCENTAGE_TO_TREASURY},
            opaque::SessionKeys,
            staking::{self, EARLY_COLLATOR_MINIMUM_STAKE, MIN_BOND_TO_BE_CONSIDERED_COLLATOR},
            xcm_config::{XcmExecutorConfig, XcmFeesAccount},
            AssetManager, Assets, AuthorInherent, Authorship, Balances, CalamariVesting, Call,
            CollatorSelection, Council, DefaultBlocksPerRound, Democracy, EnactmentPeriod, Event,
            InflationInfo, LaunchPeriod, LeaveDelayRounds, NativeTokenExistentialDeposit, Origin,
            ParachainStaking, Period, PolkadotXcm, Range, Runtime, Scheduler, Session, System,
            TechnicalCommittee, Timestamp, TransactionPause, TransactionPayment, Treasury, Utility,
            VotingPeriod,
        };
        type RuntimeAssetConfig = calamari_runtime::assets_config::CalamariAssetConfig;
        type RuntimeConcreteFungibleLedger =
            calamari_runtime::assets_config::CalamariConcreteFungibleLedger;
    } else {
        pub use manta_runtime::{
            assets_config::MantaConcreteFungibleLedger,
            currency::MANTA as KMA,
            opaque::SessionKeys,
            staking::{self, EARLY_COLLATOR_MINIMUM_STAKE, MIN_BOND_TO_BE_CONSIDERED_COLLATOR},
            xcm_config::{XcmExecutorConfig, XcmFeesAccount},
            AssetManager, Assets, AuthorInherent, Authorship, Balances, Call, CollatorSelection,
            DefaultBlocksPerRound, Event, InflationInfo, LeaveDelayRounds, NativeTokenExistentialDeposit,
            Origin, ParachainStaking, Period, PolkadotXcm, Range, Runtime, Session, System, Timestamp,
            TransactionPayment, Treasury, Utility,
        };
        type RuntimeAssetConfig = manta_runtime::assets_config::MantaAssetConfig;
        type RuntimeConcreteFungibleLedger = manta_runtime::assets_config::MantaConcreteFungibleLedger;
    }
}

use codec::Encode;
use frame_support::{
    assert_ok,
    weights::{DispatchInfo, Weight},
};
use lazy_static::lazy_static;
use manta_primitives::types::{AccountId, Balance, Header};
use nimbus_primitives::NIMBUS_ENGINE_ID;
use session_key_primitives::util::unchecked_account_id;
use sp_core::sr25519::Public;
use sp_runtime::{traits::Header as HeaderT, DigestItem};

pub const INITIAL_BALANCE: Balance = 1_000_000_000_000 * KMA;

lazy_static! {
    pub(crate) static ref ALICE: AccountId = unchecked_account_id::<Public>("Alice");
    pub(crate) static ref BOB: AccountId = unchecked_account_id::<Public>("Bob");
    pub(crate) static ref CHARLIE: AccountId = unchecked_account_id::<Public>("Charlie");
    pub(crate) static ref DAVE: AccountId = unchecked_account_id::<Public>("Dave");
    pub(crate) static ref EVE: AccountId = unchecked_account_id::<Public>("Eve");
    pub(crate) static ref FERDIE: AccountId = unchecked_account_id::<Public>("Ferdie");
    pub(crate) static ref USER: AccountId = unchecked_account_id::<Public>("User");
    pub(crate) static ref ALICE_SESSION_KEYS: SessionKeys =
        SessionKeys::from_seed_unchecked("Alice");
    pub(crate) static ref BOB_SESSION_KEYS: SessionKeys = SessionKeys::from_seed_unchecked("Bob");
    pub(crate) static ref CHARLIE_SESSION_KEYS: SessionKeys =
        SessionKeys::from_seed_unchecked("Charlie");
    pub(crate) static ref DAVE_SESSION_KEYS: SessionKeys = SessionKeys::from_seed_unchecked("Dave");
    pub(crate) static ref EVE_SESSION_KEYS: SessionKeys = SessionKeys::from_seed_unchecked("Eve");
    pub(crate) static ref FERDIE_SESSION_KEYS: SessionKeys =
        SessionKeys::from_seed_unchecked("Ferdie");
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

pub fn initialize_collators_through_whitelist(collators: Vec<AccountId>) {
    // Add collators through the whitelist
    let candidate_count = collators.len() as u32;
    assert_ok!(CollatorSelection::set_desired_candidates(
        root_origin(),
        candidate_count
    ));
    for aid in collators.clone() {
        assert_ok!(CollatorSelection::register_candidate(root_origin(), aid));
    }
    assert_eq!(
        CollatorSelection::candidates().len(),
        candidate_count as usize
    );
    // Migrate to staking - reserves (lower) whitelist bond
    assert_ok!(ParachainStaking::initialize_pallet(
        1,
        collators,
        staking::inflation_config::<Runtime>()
    ));
    assert_eq!(
        ParachainStaking::candidate_pool().len(),
        candidate_count as usize
    );
    assert_ok!(ParachainStaking::set_total_selected(
        root_origin(),
        candidate_count
    ));
}

#[allow(dead_code)]
fn seal_header(mut header: Header, author: AccountId) -> Header {
    {
        let digest = header.digest_mut();
        digest
            .logs
            .push(DigestItem::PreRuntime(NIMBUS_ENGINE_ID, author.encode()));
        digest
            .logs
            .push(DigestItem::Seal(NIMBUS_ENGINE_ID, author.encode()));
    }

    header
}
