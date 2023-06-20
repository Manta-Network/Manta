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
//! DATE: 2023-06-19, STEPS: `50`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=40
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
	/// The range of component `b` is `[1, 5]`.
	fn set_invulnerables(b: u32, ) -> Weight {
		// Minimum execution time: 12_684 nanoseconds.
		Weight::from_ref_time(14_220_831)
			// Standard Error: 4_120
			.saturating_add(Weight::from_ref_time(65_137).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection DesiredCandidates (r:0 w:1)
	fn set_desired_candidates() -> Weight {
		// Minimum execution time: 14_607 nanoseconds.
		Weight::from_ref_time(15_463_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection CandidacyBond (r:0 w:1)
	fn set_candidacy_bond() -> Weight {
		// Minimum execution time: 13_782 nanoseconds.
		Weight::from_ref_time(13_987_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection EvictionBaseline (r:0 w:1)
	fn set_eviction_baseline() -> Weight {
		// Minimum execution time: 12_666 nanoseconds.
		Weight::from_ref_time(13_262_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection EvictionTolerance (r:0 w:1)
	fn set_eviction_tolerance() -> Weight {
		// Minimum execution time: 12_550 nanoseconds.
		Weight::from_ref_time(13_252_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: Session NextKeys (r:1 w:0)
	// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// The range of component `c` is `[1, 50]`.
	fn register_as_candidate(c: u32, ) -> Weight {
		// Minimum execution time: 44_946 nanoseconds.
		Weight::from_ref_time(49_565_137)
			// Standard Error: 2_601
			.saturating_add(Weight::from_ref_time(206_420).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	/// The range of component `c` is `[1, 50]`.
	fn leave_intent(c: u32, ) -> Weight {
		// Minimum execution time: 30_388 nanoseconds.
		Weight::from_ref_time(33_678_682)
			// Standard Error: 1_984
			.saturating_add(Weight::from_ref_time(188_185).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: CollatorSelection Candidates (r:1 w:1)
	/// The range of component `c` is `[1, 50]`.
	fn remove_collator(c: u32, ) -> Weight {
		// Minimum execution time: 32_589 nanoseconds.
		Weight::from_ref_time(36_196_530)
			// Standard Error: 2_058
			.saturating_add(Weight::from_ref_time(184_290).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: Session NextKeys (r:1 w:0)
	// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// The range of component `c` is `[1, 50]`.
	fn register_candidate(c: u32, ) -> Weight {
		// Minimum execution time: 45_531 nanoseconds.
		Weight::from_ref_time(49_604_856)
			// Standard Error: 2_489
			.saturating_add(Weight::from_ref_time(209_983).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: System Account (r:2 w:2)
	// Storage: CollatorSelection BlocksPerCollatorThisSession (r:1 w:1)
	// Storage: System BlockWeight (r:1 w:1)
	fn note_author() -> Weight {
		// Minimum execution time: 33_881 nanoseconds.
		Weight::from_ref_time(36_710_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: CollatorSelection Candidates (r:1 w:0)
	// Storage: CollatorSelection EvictionBaseline (r:1 w:0)
	// Storage: CollatorSelection EvictionTolerance (r:1 w:0)
	// Storage: CollatorSelection BlocksPerCollatorThisSession (r:2 w:2)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: System BlockWeight (r:1 w:1)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	/// The range of component `c` is `[1, 50]`.
	fn new_session(c: u32, ) -> Weight {
		// Minimum execution time: 34_647 nanoseconds.
		Weight::from_ref_time(23_961_919)
			// Standard Error: 51_499
			.saturating_add(Weight::from_ref_time(22_133_990).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(c.into())))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: CollatorSelection Invulnerables (r:0 w:1)
	/// The range of component `b` is `[1, 5]`.
	fn set_invulnerables(b: u32, ) -> Weight {
		// Minimum execution time: 12_684 nanoseconds.
		Weight::from_ref_time(14_220_831)
			// Standard Error: 4_120
			.saturating_add(Weight::from_ref_time(65_137).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection DesiredCandidates (r:0 w:1)
	fn set_desired_candidates() -> Weight {
		// Minimum execution time: 14_607 nanoseconds.
		Weight::from_ref_time(15_463_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection CandidacyBond (r:0 w:1)
	fn set_candidacy_bond() -> Weight {
		// Minimum execution time: 13_782 nanoseconds.
		Weight::from_ref_time(13_987_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection EvictionBaseline (r:0 w:1)
	fn set_eviction_baseline() -> Weight {
		// Minimum execution time: 12_666 nanoseconds.
		Weight::from_ref_time(13_262_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection EvictionTolerance (r:0 w:1)
	fn set_eviction_tolerance() -> Weight {
		// Minimum execution time: 12_550 nanoseconds.
		Weight::from_ref_time(13_252_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: Session NextKeys (r:1 w:0)
	// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// The range of component `c` is `[1, 50]`.
	fn register_as_candidate(c: u32, ) -> Weight {
		// Minimum execution time: 44_946 nanoseconds.
		Weight::from_ref_time(49_565_137)
			// Standard Error: 2_601
			.saturating_add(Weight::from_ref_time(206_420).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	/// The range of component `c` is `[1, 50]`.
	fn leave_intent(c: u32, ) -> Weight {
		// Minimum execution time: 30_388 nanoseconds.
		Weight::from_ref_time(33_678_682)
			// Standard Error: 1_984
			.saturating_add(Weight::from_ref_time(188_185).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: CollatorSelection Candidates (r:1 w:1)
	/// The range of component `c` is `[1, 50]`.
	fn remove_collator(c: u32, ) -> Weight {
		// Minimum execution time: 32_589 nanoseconds.
		Weight::from_ref_time(36_196_530)
			// Standard Error: 2_058
			.saturating_add(Weight::from_ref_time(184_290).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: Session NextKeys (r:1 w:0)
	// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// The range of component `c` is `[1, 50]`.
	fn register_candidate(c: u32, ) -> Weight {
		// Minimum execution time: 45_531 nanoseconds.
		Weight::from_ref_time(49_604_856)
			// Standard Error: 2_489
			.saturating_add(Weight::from_ref_time(209_983).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: System Account (r:2 w:2)
	// Storage: CollatorSelection BlocksPerCollatorThisSession (r:1 w:1)
	// Storage: System BlockWeight (r:1 w:1)
	fn note_author() -> Weight {
		// Minimum execution time: 33_881 nanoseconds.
		Weight::from_ref_time(36_710_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: CollatorSelection Candidates (r:1 w:0)
	// Storage: CollatorSelection EvictionBaseline (r:1 w:0)
	// Storage: CollatorSelection EvictionTolerance (r:1 w:0)
	// Storage: CollatorSelection BlocksPerCollatorThisSession (r:2 w:2)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: System BlockWeight (r:1 w:1)
	// Storage: Session Validators (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	/// The range of component `c` is `[1, 50]`.
	fn new_session(c: u32, ) -> Weight {
		// Minimum execution time: 34_647 nanoseconds.
		Weight::from_ref_time(23_961_919)
			// Standard Error: 51_499
			.saturating_add(Weight::from_ref_time(22_133_990).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(RocksDbWeight::get().writes(3))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(c.into())))
	}
}
