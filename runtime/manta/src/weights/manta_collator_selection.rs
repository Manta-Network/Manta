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

//! Autogenerated weights for manta_collator_selection
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=20
// --pallet=manta_collator_selection
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/manta_collator_selection.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for manta_collator_selection.
pub trait WeightInfo {
    fn set_invulnerables(b: u32, ) -> Weight;
    fn set_desired_candidates() -> Weight;
    fn set_candidacy_bond() -> Weight;
    fn set_eviction_baseline() -> Weight;
    fn set_eviction_tolerance() -> Weight;
    fn register_as_candidate(c: u32, ) -> Weight;
    fn leave_intent(c: u32, ) -> Weight;
    fn remove_collator(c: u32, ) -> Weight;
    fn register_candidate(c: u32, ) -> Weight;
    fn note_author() -> Weight;
    fn new_session(c: u32, ) -> Weight;
}

/// Weights for manta_collator_selection using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> manta_collator_selection::WeightInfo for SubstrateWeight<T> {
    // Storage: CollatorSelection Invulnerables (r:0 w:1)
    fn set_invulnerables(b: u32, ) -> Weight {
        (12_705_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((56_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection DesiredCandidates (r:0 w:1)
    fn set_desired_candidates() -> Weight {
        (14_397_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection CandidacyBond (r:0 w:1)
    fn set_candidacy_bond() -> Weight {
        (12_501_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection EvictionBaseline (r:0 w:1)
    fn set_eviction_baseline() -> Weight {
        (11_730_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection EvictionTolerance (r:0 w:1)
    fn set_eviction_tolerance() -> Weight {
        (11_800_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:1)
    // Storage: CollatorSelection DesiredCandidates (r:1 w:0)
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:0)
    // Storage: CollatorSelection CandidacyBond (r:1 w:0)
    fn register_as_candidate(c: u32, ) -> Weight {
        (42_927_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((332_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:1)
    fn leave_intent(c: u32, ) -> Weight {
        (31_164_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((233_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: CollatorSelection Candidates (r:1 w:1)
    fn remove_collator(c: u32, ) -> Weight {
        (33_945_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((206_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:1)
    // Storage: CollatorSelection DesiredCandidates (r:1 w:0)
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:0)
    // Storage: CollatorSelection CandidacyBond (r:1 w:0)
    fn register_candidate(c: u32, ) -> Weight {
        (44_458_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((218_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: System Account (r:2 w:2)
    // Storage: CollatorSelection BlocksPerCollatorThisSession (r:1 w:1)
    // Storage: System BlockWeight (r:1 w:1)
    fn note_author() -> Weight {
        (38_035_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:0)
    // Storage: CollatorSelection EvictionBaseline (r:1 w:0)
    // Storage: CollatorSelection EvictionTolerance (r:1 w:0)
    // Storage: CollatorSelection BlocksPerCollatorThisSession (r:2 w:2)
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: System BlockWeight (r:1 w:1)
    // Storage: Session Validators (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn new_session(c: u32, ) -> Weight {
        (18_237_000 as Weight)
            // Standard Error: 75_000
            .saturating_add((23_862_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: CollatorSelection Invulnerables (r:0 w:1)
    fn set_invulnerables(b: u32, ) -> Weight {
        (12_705_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((56_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection DesiredCandidates (r:0 w:1)
    fn set_desired_candidates() -> Weight {
        (14_397_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection CandidacyBond (r:0 w:1)
    fn set_candidacy_bond() -> Weight {
        (12_501_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection EvictionBaseline (r:0 w:1)
    fn set_eviction_baseline() -> Weight {
        (11_730_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection EvictionTolerance (r:0 w:1)
    fn set_eviction_tolerance() -> Weight {
        (11_800_000 as Weight)
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:1)
    // Storage: CollatorSelection DesiredCandidates (r:1 w:0)
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:0)
    // Storage: CollatorSelection CandidacyBond (r:1 w:0)
    fn register_as_candidate(c: u32, ) -> Weight {
        (42_927_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((332_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:1)
    fn leave_intent(c: u32, ) -> Weight {
        (31_164_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((233_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: CollatorSelection Candidates (r:1 w:1)
    fn remove_collator(c: u32, ) -> Weight {
        (33_945_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((206_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:1)
    // Storage: CollatorSelection DesiredCandidates (r:1 w:0)
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:0)
    // Storage: CollatorSelection CandidacyBond (r:1 w:0)
    fn register_candidate(c: u32, ) -> Weight {
        (44_458_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((218_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: System Account (r:2 w:2)
    // Storage: CollatorSelection BlocksPerCollatorThisSession (r:1 w:1)
    // Storage: System BlockWeight (r:1 w:1)
    fn note_author() -> Weight {
        (38_035_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }
    // Storage: CollatorSelection Candidates (r:1 w:0)
    // Storage: CollatorSelection EvictionBaseline (r:1 w:0)
    // Storage: CollatorSelection EvictionTolerance (r:1 w:0)
    // Storage: CollatorSelection BlocksPerCollatorThisSession (r:2 w:2)
    // Storage: CollatorSelection Invulnerables (r:1 w:0)
    // Storage: System BlockWeight (r:1 w:1)
    // Storage: Session Validators (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn new_session(c: u32, ) -> Weight {
        (18_237_000 as Weight)
            // Standard Error: 75_000
            .saturating_add((23_862_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
}
