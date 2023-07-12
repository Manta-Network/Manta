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

//! Autogenerated weights for pallet_utility
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
// --pallet=pallet_utility
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_utility.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_utility.
pub trait WeightInfo {
    fn batch(c: u32, ) -> Weight;
    fn as_derivative() -> Weight;
    fn batch_all(c: u32, ) -> Weight;
    fn dispatch_as() -> Weight;
    fn force_batch(c: u32, ) -> Weight;
}

/// Weights for pallet_utility using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for SubstrateWeight<T> {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		// Minimum execution time: 12_686 nanoseconds.
		Weight::from_ref_time(34_899_520)
			// Standard Error: 2_710
			.saturating_add(Weight::from_ref_time(5_164_434).saturating_mul(c.into()))
	}
	fn as_derivative() -> Weight {
		// Minimum execution time: 6_801 nanoseconds.
		Weight::from_ref_time(7_050_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		// Minimum execution time: 25_428 nanoseconds.
		Weight::from_ref_time(40_525_776)
			// Standard Error: 2_777
			.saturating_add(Weight::from_ref_time(5_399_565).saturating_mul(c.into()))
	}
	fn dispatch_as() -> Weight {
		// Minimum execution time: 15_015 nanoseconds.
		Weight::from_ref_time(15_418_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		// Minimum execution time: 12_702 nanoseconds.
		Weight::from_ref_time(30_521_082)
			// Standard Error: 2_032
			.saturating_add(Weight::from_ref_time(5_171_879).saturating_mul(c.into()))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		// Minimum execution time: 12_686 nanoseconds.
		Weight::from_ref_time(34_899_520)
			// Standard Error: 2_710
			.saturating_add(Weight::from_ref_time(5_164_434).saturating_mul(c.into()))
	}
	fn as_derivative() -> Weight {
		// Minimum execution time: 6_801 nanoseconds.
		Weight::from_ref_time(7_050_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		// Minimum execution time: 25_428 nanoseconds.
		Weight::from_ref_time(40_525_776)
			// Standard Error: 2_777
			.saturating_add(Weight::from_ref_time(5_399_565).saturating_mul(c.into()))
	}
	fn dispatch_as() -> Weight {
		// Minimum execution time: 15_015 nanoseconds.
		Weight::from_ref_time(15_418_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		// Minimum execution time: 12_702 nanoseconds.
		Weight::from_ref_time(30_521_082)
			// Standard Error: 2_032
			.saturating_add(Weight::from_ref_time(5_171_879).saturating_mul(c.into()))
	}
}
