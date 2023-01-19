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

//! Autogenerated weights for pallet_democracy
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-18, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("calamari-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=calamari-dev
// --steps=50
// --repeat=20
// --pallet=pallet_democracy
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_democracy.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_democracy.
pub trait WeightInfo {
    fn propose() -> Weight;
    fn second(s: u32, ) -> Weight;
    fn vote_new(r: u32, ) -> Weight;
    fn vote_existing(r: u32, ) -> Weight;
    fn emergency_cancel() -> Weight;
    fn blacklist(p: u32, ) -> Weight;
    fn external_propose(v: u32, ) -> Weight;
    fn external_propose_majority() -> Weight;
    fn external_propose_default() -> Weight;
    fn fast_track() -> Weight;
    fn veto_external(v: u32, ) -> Weight;
    fn cancel_proposal(p: u32, ) -> Weight;
    fn cancel_referendum() -> Weight;
    fn cancel_queued(r: u32, ) -> Weight;
    fn on_initialize_base(r: u32, ) -> Weight;
    fn on_initialize_base_with_launch_period(r: u32, ) -> Weight;
    fn delegate(r: u32, ) -> Weight;
    fn undelegate(r: u32, ) -> Weight;
    fn clear_public_proposals() -> Weight;
    fn note_preimage(b: u32, ) -> Weight;
    fn note_imminent_preimage(b: u32, ) -> Weight;
    fn reap_preimage(b: u32, ) -> Weight;
    fn unlock_remove(r: u32, ) -> Weight;
    fn unlock_set(r: u32, ) -> Weight;
    fn remove_vote(r: u32, ) -> Weight;
    fn remove_other_vote(r: u32, ) -> Weight;
}

/// Weights for pallet_democracy using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_democracy::WeightInfo for SubstrateWeight<T> {
    // Storage: Democracy PublicPropCount (r:1 w:1)
    // Storage: Democracy PublicProps (r:1 w:1)
    // Storage: Democracy Blacklist (r:1 w:0)
    // Storage: Democracy DepositOf (r:0 w:1)
    fn propose() -> Weight {
        (54_429_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy DepositOf (r:1 w:1)
    fn second(s: u32, ) -> Weight {
        (32_033_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((120_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vote_new(r: u32, ) -> Weight {
        (41_042_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((185_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vote_existing(r: u32, ) -> Weight {
        (41_542_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((157_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy Cancellations (r:1 w:1)
    fn emergency_cancel() -> Weight {
        (19_310_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy PublicProps (r:1 w:1)
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy Blacklist (r:0 w:1)
    // Storage: Democracy DepositOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn blacklist(p: u32, ) -> Weight {
        (52_456_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((208_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy Blacklist (r:1 w:0)
    fn external_propose(v: u32, ) -> Weight {
        (11_556_000 as Weight)
            // Standard Error: 0
            .saturating_add((21_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy NextExternal (r:0 w:1)
    fn external_propose_majority() -> Weight {
        (4_210_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy NextExternal (r:0 w:1)
    fn external_propose_default() -> Weight {
        (4_282_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy ReferendumCount (r:1 w:1)
    // Storage: Democracy ReferendumInfoOf (r:0 w:1)
    fn fast_track() -> Weight {
        (20_605_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy Blacklist (r:1 w:1)
    fn veto_external(v: u32, ) -> Weight {
        (21_791_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((36_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy PublicProps (r:1 w:1)
    // Storage: Democracy DepositOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn cancel_proposal(p: u32, ) -> Weight {
        (40_122_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((187_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:0 w:1)
    fn cancel_referendum() -> Weight {
        (12_936_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn cancel_queued(r: u32, ) -> Weight {
        (24_409_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((1_006_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy LowestUnbaked (r:1 w:1)
    // Storage: Democracy ReferendumCount (r:1 w:0)
    // Storage: Democracy ReferendumInfoOf (r:1 w:0)
    fn on_initialize_base(r: u32, ) -> Weight {
        (4_395_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((3_400_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy LowestUnbaked (r:1 w:1)
    // Storage: Democracy ReferendumCount (r:1 w:0)
    // Storage: Democracy LastTabledWasExternal (r:1 w:0)
    // Storage: Democracy NextExternal (r:1 w:0)
    // Storage: Democracy PublicProps (r:1 w:0)
    // Storage: Democracy ReferendumInfoOf (r:1 w:0)
    fn on_initialize_base_with_launch_period(r: u32, ) -> Weight {
        (10_396_000 as Weight)
            // Standard Error: 7_000
            .saturating_add((3_386_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy VotingOf (r:3 w:3)
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn delegate(r: u32, ) -> Weight {
        (42_327_000 as Weight)
            // Standard Error: 9_000
            .saturating_add((4_564_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    // Storage: Democracy VotingOf (r:2 w:2)
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    fn undelegate(r: u32, ) -> Weight {
        (22_879_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((4_495_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    // Storage: Democracy PublicProps (r:0 w:1)
    fn clear_public_proposals() -> Weight {
        (4_726_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy Preimages (r:1 w:1)
    fn note_preimage(b: u32, ) -> Weight {
        (30_106_000 as Weight)
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy Preimages (r:1 w:1)
    fn note_imminent_preimage(b: u32, ) -> Weight {
        (20_649_000 as Weight)
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy Preimages (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    fn reap_preimage(b: u32, ) -> Weight {
        (29_294_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unlock_remove(r: u32, ) -> Weight {
        (27_659_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((58_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unlock_set(r: u32, ) -> Weight {
        (26_421_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((122_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    fn remove_vote(r: u32, ) -> Weight {
        (16_682_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((129_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    fn remove_other_vote(r: u32, ) -> Weight {
        (15_982_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((149_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: Democracy PublicPropCount (r:1 w:1)
    // Storage: Democracy PublicProps (r:1 w:1)
    // Storage: Democracy Blacklist (r:1 w:0)
    // Storage: Democracy DepositOf (r:0 w:1)
    fn propose() -> Weight {
        (54_429_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy DepositOf (r:1 w:1)
    fn second(s: u32, ) -> Weight {
        (32_033_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((120_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vote_new(r: u32, ) -> Weight {
        (41_042_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((185_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vote_existing(r: u32, ) -> Weight {
        (41_542_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((157_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy Cancellations (r:1 w:1)
    fn emergency_cancel() -> Weight {
        (19_310_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy PublicProps (r:1 w:1)
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy Blacklist (r:0 w:1)
    // Storage: Democracy DepositOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn blacklist(p: u32, ) -> Weight {
        (52_456_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((208_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy Blacklist (r:1 w:0)
    fn external_propose(v: u32, ) -> Weight {
        (11_556_000 as Weight)
            // Standard Error: 0
            .saturating_add((21_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy NextExternal (r:0 w:1)
    fn external_propose_majority() -> Weight {
        (4_210_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy NextExternal (r:0 w:1)
    fn external_propose_default() -> Weight {
        (4_282_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy ReferendumCount (r:1 w:1)
    // Storage: Democracy ReferendumInfoOf (r:0 w:1)
    fn fast_track() -> Weight {
        (20_605_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy NextExternal (r:1 w:1)
    // Storage: Democracy Blacklist (r:1 w:1)
    fn veto_external(v: u32, ) -> Weight {
        (21_791_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((36_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy PublicProps (r:1 w:1)
    // Storage: Democracy DepositOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn cancel_proposal(p: u32, ) -> Weight {
        (40_122_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((187_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:0 w:1)
    fn cancel_referendum() -> Weight {
        (12_936_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn cancel_queued(r: u32, ) -> Weight {
        (24_409_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((1_006_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy LowestUnbaked (r:1 w:1)
    // Storage: Democracy ReferendumCount (r:1 w:0)
    // Storage: Democracy ReferendumInfoOf (r:1 w:0)
    fn on_initialize_base(r: u32, ) -> Weight {
        (4_395_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((3_400_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy LowestUnbaked (r:1 w:1)
    // Storage: Democracy ReferendumCount (r:1 w:0)
    // Storage: Democracy LastTabledWasExternal (r:1 w:0)
    // Storage: Democracy NextExternal (r:1 w:0)
    // Storage: Democracy PublicProps (r:1 w:0)
    // Storage: Democracy ReferendumInfoOf (r:1 w:0)
    fn on_initialize_base_with_launch_period(r: u32, ) -> Weight {
        (10_396_000 as Weight)
            // Standard Error: 7_000
            .saturating_add((3_386_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy VotingOf (r:3 w:3)
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn delegate(r: u32, ) -> Weight {
        (42_327_000 as Weight)
            // Standard Error: 9_000
            .saturating_add((4_564_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    // Storage: Democracy VotingOf (r:2 w:2)
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    fn undelegate(r: u32, ) -> Weight {
        (22_879_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((4_495_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
    }
    // Storage: Democracy PublicProps (r:0 w:1)
    fn clear_public_proposals() -> Weight {
        (4_726_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy Preimages (r:1 w:1)
    fn note_preimage(b: u32, ) -> Weight {
        (30_106_000 as Weight)
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy Preimages (r:1 w:1)
    fn note_imminent_preimage(b: u32, ) -> Weight {
        (20_649_000 as Weight)
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy Preimages (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    fn reap_preimage(b: u32, ) -> Weight {
        (29_294_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unlock_remove(r: u32, ) -> Weight {
        (27_659_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((58_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy VotingOf (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unlock_set(r: u32, ) -> Weight {
        (26_421_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((122_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    fn remove_vote(r: u32, ) -> Weight {
        (16_682_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((129_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Democracy ReferendumInfoOf (r:1 w:1)
    // Storage: Democracy VotingOf (r:1 w:1)
    fn remove_other_vote(r: u32, ) -> Weight {
        (15_982_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((149_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
}
