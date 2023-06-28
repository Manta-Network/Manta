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

//! Autogenerated weights for frame_system
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-26, STEPS: `50`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=40
// --pallet=frame_system
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/frame_system.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for frame_system.
pub trait WeightInfo {
    fn remark(b: u32, ) -> Weight;
    fn remark_with_event(b: u32, ) -> Weight;
    fn set_heap_pages() -> Weight;
    fn set_storage(i: u32, ) -> Weight;
    fn kill_storage(i: u32, ) -> Weight;
    fn kill_prefix(p: u32, ) -> Weight;
}

/// Weights for frame_system using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_system::WeightInfo for SubstrateWeight<T> {
	/// The range of component `b` is `[0, 3670016]`.
	fn remark(b: u32, ) -> Weight {
		// Minimum execution time: 3_528 nanoseconds.
		Weight::from_ref_time(11_101_821)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(460).saturating_mul(b.into()))
	}
	/// The range of component `b` is `[0, 3670016]`.
	fn remark_with_event(b: u32, ) -> Weight {
		// Minimum execution time: 13_068 nanoseconds.
		Weight::from_ref_time(13_364_000)
			// Standard Error: 1
			.saturating_add(Weight::from_ref_time(2_035).saturating_mul(b.into()))
	}
	// Storage: System Digest (r:1 w:1)
	// Storage: unknown [0x3a686561707061676573] (r:0 w:1)
	fn set_heap_pages() -> Weight {
		// Minimum execution time: 18_083 nanoseconds.
		Weight::from_ref_time(18_888_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn set_storage(i: u32, ) -> Weight {
		// Minimum execution time: 5_939 nanoseconds.
		Weight::from_ref_time(6_132_000)
			// Standard Error: 1_567
			.saturating_add(Weight::from_ref_time(718_087).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn kill_storage(i: u32, ) -> Weight {
		// Minimum execution time: 3_670 nanoseconds.
		Weight::from_ref_time(3_766_000)
			// Standard Error: 574
			.saturating_add(Weight::from_ref_time(517_573).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `p` is `[0, 1000]`.
	fn kill_prefix(p: u32, ) -> Weight {
		// Minimum execution time: 5_576 nanoseconds.
		Weight::from_ref_time(5_678_000)
			// Standard Error: 772
			.saturating_add(Weight::from_ref_time(1_107_514).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// The range of component `b` is `[0, 3670016]`.
	fn remark(b: u32, ) -> Weight {
		// Minimum execution time: 3_528 nanoseconds.
		Weight::from_ref_time(11_101_821)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(460).saturating_mul(b.into()))
	}
	/// The range of component `b` is `[0, 3670016]`.
	fn remark_with_event(b: u32, ) -> Weight {
		// Minimum execution time: 13_068 nanoseconds.
		Weight::from_ref_time(13_364_000)
			// Standard Error: 1
			.saturating_add(Weight::from_ref_time(2_035).saturating_mul(b.into()))
	}
	// Storage: System Digest (r:1 w:1)
	// Storage: unknown [0x3a686561707061676573] (r:0 w:1)
	fn set_heap_pages() -> Weight {
		// Minimum execution time: 18_083 nanoseconds.
		Weight::from_ref_time(18_888_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn set_storage(i: u32, ) -> Weight {
		// Minimum execution time: 5_939 nanoseconds.
		Weight::from_ref_time(6_132_000)
			// Standard Error: 1_567
			.saturating_add(Weight::from_ref_time(718_087).saturating_mul(i.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn kill_storage(i: u32, ) -> Weight {
		// Minimum execution time: 3_670 nanoseconds.
		Weight::from_ref_time(3_766_000)
			// Standard Error: 574
			.saturating_add(Weight::from_ref_time(517_573).saturating_mul(i.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `p` is `[0, 1000]`.
	fn kill_prefix(p: u32, ) -> Weight {
		// Minimum execution time: 5_576 nanoseconds.
		Weight::from_ref_time(5_678_000)
			// Standard Error: 772
			.saturating_add(Weight::from_ref_time(1_107_514).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
}
