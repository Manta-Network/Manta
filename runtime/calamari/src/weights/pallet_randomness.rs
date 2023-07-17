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

//! Autogenerated weights for pallet_randomness
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-07, STEPS: `50`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("/home/runner/runners/2.280.1/_work/Manta/Manta/tests/data/fork.json"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=/home/runner/runners/2.280.1/_work/Manta/Manta/tests/data/fork.json
// --steps=50
// --repeat=40
// --pallet=pallet_randomness
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_randomness.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_randomness.
pub trait WeightInfo {
    fn set_babe_randomness_results() -> Weight;
}

/// Weights for pallet_randomness using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_randomness::WeightInfo for SubstrateWeight<T> {
	// Storage: Randomness RelayEpoch (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: ParachainSystem RelayStateProof (r:1 w:0)
	// Storage: Randomness RandomnessResults (r:0 w:1)
	// Storage: Randomness InherentIncluded (r:0 w:1)
	fn set_babe_randomness_results() -> Weight {
		// Minimum execution time: 14_403 nanoseconds.
		Weight::from_ref_time(14_651_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Randomness RelayEpoch (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: ParachainSystem RelayStateProof (r:1 w:0)
	// Storage: Randomness RandomnessResults (r:0 w:1)
	// Storage: Randomness InherentIncluded (r:0 w:1)
	fn set_babe_randomness_results() -> Weight {
		// Minimum execution time: 14_403 nanoseconds.
		Weight::from_ref_time(14_651_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}
