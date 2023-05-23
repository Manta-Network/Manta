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

//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("calamari-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=calamari-dev
// --steps=50
// --repeat=20
// --pallet=pallet_multisig
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_multisig.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_multisig.
pub trait WeightInfo {
    fn as_multi_threshold_1(z: u32, ) -> Weight;
    fn as_multi_create(s: u32, z: u32, ) -> Weight;
    fn as_multi_approve(s: u32, z: u32, ) -> Weight;
    fn as_multi_complete(s: u32, z: u32, ) -> Weight;
    fn approve_as_multi_create(s: u32, ) -> Weight;
    fn approve_as_multi_approve(s: u32, ) -> Weight;
    fn cancel_as_multi(s: u32, ) -> Weight;
}

/// Weights for pallet_multisig using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for SubstrateWeight<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		// Minimum execution time: 18_392 nanoseconds.
		Weight::from_ref_time(19_423_290)
			// Standard Error: 11
			.saturating_add(Weight::from_ref_time(685).saturating_mul(z.into()))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 52_125 nanoseconds.
		Weight::from_ref_time(40_353_378)
			// Standard Error: 2_451
			.saturating_add(Weight::from_ref_time(139_619).saturating_mul(s.into()))
			// Standard Error: 24
			.saturating_add(Weight::from_ref_time(1_944).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 39_774 nanoseconds.
		Weight::from_ref_time(29_838_089)
			// Standard Error: 1_984
			.saturating_add(Weight::from_ref_time(123_019).saturating_mul(s.into()))
			// Standard Error: 19
			.saturating_add(Weight::from_ref_time(1_821).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 56_602 nanoseconds.
		Weight::from_ref_time(43_950_416)
			// Standard Error: 1_694
			.saturating_add(Weight::from_ref_time(143_065).saturating_mul(s.into()))
			// Standard Error: 16
			.saturating_add(Weight::from_ref_time(1_763).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Minimum execution time: 35_630 nanoseconds.
		Weight::from_ref_time(39_010_694)
			// Standard Error: 2_201
			.saturating_add(Weight::from_ref_time(135_798).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Minimum execution time: 25_282 nanoseconds.
		Weight::from_ref_time(26_497_652)
			// Standard Error: 1_599
			.saturating_add(Weight::from_ref_time(133_021).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Minimum execution time: 35_710 nanoseconds.
		Weight::from_ref_time(37_867_378)
			// Standard Error: 2_243
			.saturating_add(Weight::from_ref_time(136_198).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		// Minimum execution time: 18_392 nanoseconds.
		Weight::from_ref_time(19_423_290)
			// Standard Error: 11
			.saturating_add(Weight::from_ref_time(685).saturating_mul(z.into()))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 52_125 nanoseconds.
		Weight::from_ref_time(40_353_378)
			// Standard Error: 2_451
			.saturating_add(Weight::from_ref_time(139_619).saturating_mul(s.into()))
			// Standard Error: 24
			.saturating_add(Weight::from_ref_time(1_944).saturating_mul(z.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 39_774 nanoseconds.
		Weight::from_ref_time(29_838_089)
			// Standard Error: 1_984
			.saturating_add(Weight::from_ref_time(123_019).saturating_mul(s.into()))
			// Standard Error: 19
			.saturating_add(Weight::from_ref_time(1_821).saturating_mul(z.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 56_602 nanoseconds.
		Weight::from_ref_time(43_950_416)
			// Standard Error: 1_694
			.saturating_add(Weight::from_ref_time(143_065).saturating_mul(s.into()))
			// Standard Error: 16
			.saturating_add(Weight::from_ref_time(1_763).saturating_mul(z.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Minimum execution time: 35_630 nanoseconds.
		Weight::from_ref_time(39_010_694)
			// Standard Error: 2_201
			.saturating_add(Weight::from_ref_time(135_798).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Minimum execution time: 25_282 nanoseconds.
		Weight::from_ref_time(26_497_652)
			// Standard Error: 1_599
			.saturating_add(Weight::from_ref_time(133_021).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Minimum execution time: 35_710 nanoseconds.
		Weight::from_ref_time(37_867_378)
			// Standard Error: 2_243
			.saturating_add(Weight::from_ref_time(136_198).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
