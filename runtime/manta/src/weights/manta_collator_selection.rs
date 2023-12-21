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
//! DATE: 2023-12-21, STEPS: `50`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("/home/aye/actions-runner/_worker/Manta/Manta/tests/data/fork.json"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=/home/aye/actions-runner/_worker/Manta/Manta/tests/data/fork.json
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
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

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
	/// Storage: CollatorSelection Invulnerables (r:0 w:1)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 5]`.
	fn set_invulnerables(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_662_000 picoseconds.
		Weight::from_parts(8_106_039, 0)
			// Standard Error: 2_237
			.saturating_add(Weight::from_parts(33_156, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection DesiredCandidates (r:0 w:1)
	/// Proof Skipped: CollatorSelection DesiredCandidates (max_values: Some(1), max_size: None, mode: Measured)
	fn set_desired_candidates() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_472_000 picoseconds.
		Weight::from_parts(9_990_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection CandidacyBond (r:0 w:1)
	/// Proof Skipped: CollatorSelection CandidacyBond (max_values: Some(1), max_size: None, mode: Measured)
	fn set_candidacy_bond() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_540_000 picoseconds.
		Weight::from_parts(7_828_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection EvictionBaseline (r:0 w:1)
	/// Proof Skipped: CollatorSelection EvictionBaseline (max_values: Some(1), max_size: None, mode: Measured)
	fn set_eviction_baseline() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_213_000 picoseconds.
		Weight::from_parts(7_491_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection EvictionTolerance (r:0 w:1)
	/// Proof Skipped: CollatorSelection EvictionTolerance (max_values: Some(1), max_size: None, mode: Measured)
	fn set_eviction_tolerance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_329_000 picoseconds.
		Weight::from_parts(7_539_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	/// Proof Skipped: CollatorSelection DesiredCandidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Session NextKeys (r:1 w:0)
	/// Proof Skipped: Session NextKeys (max_values: None, max_size: None, mode: Measured)
	/// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// Proof Skipped: CollatorSelection CandidacyBond (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn register_as_candidate(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `475 + c * (60 ±0)`
		//  Estimated: `3952 + c * (60 ±0)`
		// Minimum execution time: 41_855_000 picoseconds.
		Weight::from_parts(44_800_344, 3952)
			// Standard Error: 3_752
			.saturating_add(Weight::from_parts(277_942, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 60).saturating_mul(c.into()))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn leave_intent(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `259 + c * (48 ±0)`
		//  Estimated: `1738 + c * (49 ±0)`
		// Minimum execution time: 29_548_000 picoseconds.
		Weight::from_parts(32_463_838, 1738)
			// Standard Error: 1_713
			.saturating_add(Weight::from_parts(144_978, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(c.into()))
	}
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn remove_collator(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `283 + c * (48 ±0)`
		//  Estimated: `1762 + c * (49 ±0)`
		// Minimum execution time: 31_220_000 picoseconds.
		Weight::from_parts(33_891_925, 1762)
			// Standard Error: 1_729
			.saturating_add(Weight::from_parts(145_454, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(c.into()))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	/// Proof Skipped: CollatorSelection DesiredCandidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Session NextKeys (r:1 w:0)
	/// Proof Skipped: Session NextKeys (max_values: None, max_size: None, mode: Measured)
	/// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// Proof Skipped: CollatorSelection CandidacyBond (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn register_candidate(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `475 + c * (60 ±0)`
		//  Estimated: `3952 + c * (60 ±0)`
		// Minimum execution time: 41_828_000 picoseconds.
		Weight::from_parts(45_085_720, 3952)
			// Standard Error: 3_983
			.saturating_add(Weight::from_parts(250_809, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 60).saturating_mul(c.into()))
	}
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: CollatorSelection BlocksPerCollatorThisSession (r:1 w:1)
	/// Proof Skipped: CollatorSelection BlocksPerCollatorThisSession (max_values: None, max_size: None, mode: Measured)
	/// Storage: System BlockWeight (r:1 w:1)
	/// Proof: System BlockWeight (max_values: Some(1), max_size: Some(48), added: 543, mode: MaxEncodedLen)
	fn note_author() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `146`
		//  Estimated: `6196`
		// Minimum execution time: 48_752_000 picoseconds.
		Weight::from_parts(49_238_000, 6196)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection EvictionBaseline (r:1 w:0)
	/// Proof Skipped: CollatorSelection EvictionBaseline (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection EvictionTolerance (r:1 w:0)
	/// Proof Skipped: CollatorSelection EvictionTolerance (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection BlocksPerCollatorThisSession (r:51 w:2)
	/// Proof Skipped: CollatorSelection BlocksPerCollatorThisSession (max_values: None, max_size: None, mode: Measured)
	/// Storage: System Account (r:49 w:49)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System BlockWeight (r:1 w:1)
	/// Proof: System BlockWeight (max_values: Some(1), max_size: Some(48), added: 543, mode: MaxEncodedLen)
	/// Storage: Session Validators (r:1 w:0)
	/// Proof Skipped: Session Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn new_session(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `62 + c * (230 ±0)`
		//  Estimated: `3593 + c * (2705 ±0)`
		// Minimum execution time: 34_509_000 picoseconds.
		Weight::from_parts(23_774_429, 3593)
			// Standard Error: 12_976
			.saturating_add(Weight::from_parts(26_749_722, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 2705).saturating_mul(c.into()))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: CollatorSelection Invulnerables (r:0 w:1)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 5]`.
	fn set_invulnerables(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_662_000 picoseconds.
		Weight::from_parts(8_106_039, 0)
			// Standard Error: 2_237
			.saturating_add(Weight::from_parts(33_156, 0).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection DesiredCandidates (r:0 w:1)
	/// Proof Skipped: CollatorSelection DesiredCandidates (max_values: Some(1), max_size: None, mode: Measured)
	fn set_desired_candidates() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_472_000 picoseconds.
		Weight::from_parts(9_990_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection CandidacyBond (r:0 w:1)
	/// Proof Skipped: CollatorSelection CandidacyBond (max_values: Some(1), max_size: None, mode: Measured)
	fn set_candidacy_bond() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_540_000 picoseconds.
		Weight::from_parts(7_828_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection EvictionBaseline (r:0 w:1)
	/// Proof Skipped: CollatorSelection EvictionBaseline (max_values: Some(1), max_size: None, mode: Measured)
	fn set_eviction_baseline() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_213_000 picoseconds.
		Weight::from_parts(7_491_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection EvictionTolerance (r:0 w:1)
	/// Proof Skipped: CollatorSelection EvictionTolerance (max_values: Some(1), max_size: None, mode: Measured)
	fn set_eviction_tolerance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_329_000 picoseconds.
		Weight::from_parts(7_539_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	/// Proof Skipped: CollatorSelection DesiredCandidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Session NextKeys (r:1 w:0)
	/// Proof Skipped: Session NextKeys (max_values: None, max_size: None, mode: Measured)
	/// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// Proof Skipped: CollatorSelection CandidacyBond (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn register_as_candidate(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `475 + c * (60 ±0)`
		//  Estimated: `3952 + c * (60 ±0)`
		// Minimum execution time: 41_855_000 picoseconds.
		Weight::from_parts(44_800_344, 3952)
			// Standard Error: 3_752
			.saturating_add(Weight::from_parts(277_942, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 60).saturating_mul(c.into()))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn leave_intent(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `259 + c * (48 ±0)`
		//  Estimated: `1738 + c * (49 ±0)`
		// Minimum execution time: 29_548_000 picoseconds.
		Weight::from_parts(32_463_838, 1738)
			// Standard Error: 1_713
			.saturating_add(Weight::from_parts(144_978, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(c.into()))
	}
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn remove_collator(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `283 + c * (48 ±0)`
		//  Estimated: `1762 + c * (49 ±0)`
		// Minimum execution time: 31_220_000 picoseconds.
		Weight::from_parts(33_891_925, 1762)
			// Standard Error: 1_729
			.saturating_add(Weight::from_parts(145_454, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(c.into()))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	/// Proof Skipped: CollatorSelection DesiredCandidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Session NextKeys (r:1 w:0)
	/// Proof Skipped: Session NextKeys (max_values: None, max_size: None, mode: Measured)
	/// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	/// Proof Skipped: CollatorSelection CandidacyBond (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn register_candidate(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `475 + c * (60 ±0)`
		//  Estimated: `3952 + c * (60 ±0)`
		// Minimum execution time: 41_828_000 picoseconds.
		Weight::from_parts(45_085_720, 3952)
			// Standard Error: 3_983
			.saturating_add(Weight::from_parts(250_809, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 60).saturating_mul(c.into()))
	}
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: CollatorSelection BlocksPerCollatorThisSession (r:1 w:1)
	/// Proof Skipped: CollatorSelection BlocksPerCollatorThisSession (max_values: None, max_size: None, mode: Measured)
	/// Storage: System BlockWeight (r:1 w:1)
	/// Proof: System BlockWeight (max_values: Some(1), max_size: Some(48), added: 543, mode: MaxEncodedLen)
	fn note_author() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `146`
		//  Estimated: `6196`
		// Minimum execution time: 48_752_000 picoseconds.
		Weight::from_parts(49_238_000, 6196)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: CollatorSelection Candidates (r:1 w:1)
	/// Proof Skipped: CollatorSelection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection EvictionBaseline (r:1 w:0)
	/// Proof Skipped: CollatorSelection EvictionBaseline (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection EvictionTolerance (r:1 w:0)
	/// Proof Skipped: CollatorSelection EvictionTolerance (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CollatorSelection BlocksPerCollatorThisSession (r:51 w:2)
	/// Proof Skipped: CollatorSelection BlocksPerCollatorThisSession (max_values: None, max_size: None, mode: Measured)
	/// Storage: System Account (r:49 w:49)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: CollatorSelection Invulnerables (r:1 w:0)
	/// Proof Skipped: CollatorSelection Invulnerables (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System BlockWeight (r:1 w:1)
	/// Proof: System BlockWeight (max_values: Some(1), max_size: Some(48), added: 543, mode: MaxEncodedLen)
	/// Storage: Session Validators (r:1 w:0)
	/// Proof Skipped: Session Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 50]`.
	fn new_session(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `62 + c * (230 ±0)`
		//  Estimated: `3593 + c * (2705 ±0)`
		// Minimum execution time: 34_509_000 picoseconds.
		Weight::from_parts(23_774_429, 3593)
			// Standard Error: 12_976
			.saturating_add(Weight::from_parts(26_749_722, 0).saturating_mul(c.into()))
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 2705).saturating_mul(c.into()))
	}
}
