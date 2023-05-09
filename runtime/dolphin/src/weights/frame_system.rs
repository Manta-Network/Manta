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
//! DATE: 2023-04-20, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dolphin-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=dolphin-dev
// --steps=50
// --repeat=20
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
		// Minimum execution time: 3_752 nanoseconds.
		Weight::from_ref_time(42_655_264)
			// Standard Error: 2
			.saturating_add(Weight::from_ref_time(452).saturating_mul(b.into()))
	}
	/// The range of component `b` is `[0, 3670016]`.
	fn remark_with_event(b: u32, ) -> Weight {
		// Minimum execution time: 13_986 nanoseconds.
		Weight::from_ref_time(12_731_045)
			// Standard Error: 2
			.saturating_add(Weight::from_ref_time(2_035).saturating_mul(b.into()))
	}
	// Storage: System Digest (r:1 w:1)
	// Storage: unknown [0x3a686561707061676573] (r:0 w:1)
	fn set_heap_pages() -> Weight {
		// Minimum execution time: 8_931 nanoseconds.
		Weight::from_ref_time(9_224_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn set_storage(i: u32, ) -> Weight {
		// Minimum execution time: 3_997 nanoseconds.
		Weight::from_ref_time(4_084_000)
			// Standard Error: 2_161
			.saturating_add(Weight::from_ref_time(715_426).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn kill_storage(i: u32, ) -> Weight {
		// Minimum execution time: 4_018 nanoseconds.
		Weight::from_ref_time(4_143_000)
			// Standard Error: 831
			.saturating_add(Weight::from_ref_time(519_031).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `p` is `[0, 1000]`.
	fn kill_prefix(p: u32, ) -> Weight {
		// Minimum execution time: 17_447 nanoseconds.
		Weight::from_ref_time(17_836_000)
			// Standard Error: 1_367
			.saturating_add(Weight::from_ref_time(1_125_803).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// The range of component `b` is `[0, 3670016]`.
	fn remark(b: u32, ) -> Weight {
		// Minimum execution time: 3_752 nanoseconds.
		Weight::from_ref_time(42_655_264)
			// Standard Error: 2
			.saturating_add(Weight::from_ref_time(452).saturating_mul(b.into()))
	}
	/// The range of component `b` is `[0, 3670016]`.
	fn remark_with_event(b: u32, ) -> Weight {
		// Minimum execution time: 13_986 nanoseconds.
		Weight::from_ref_time(12_731_045)
			// Standard Error: 2
			.saturating_add(Weight::from_ref_time(2_035).saturating_mul(b.into()))
	}
	// Storage: System Digest (r:1 w:1)
	// Storage: unknown [0x3a686561707061676573] (r:0 w:1)
	fn set_heap_pages() -> Weight {
		// Minimum execution time: 8_931 nanoseconds.
		Weight::from_ref_time(9_224_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn set_storage(i: u32, ) -> Weight {
		// Minimum execution time: 3_997 nanoseconds.
		Weight::from_ref_time(4_084_000)
			// Standard Error: 2_161
			.saturating_add(Weight::from_ref_time(715_426).saturating_mul(i.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `i` is `[0, 1000]`.
	fn kill_storage(i: u32, ) -> Weight {
		// Minimum execution time: 4_018 nanoseconds.
		Weight::from_ref_time(4_143_000)
			// Standard Error: 831
			.saturating_add(Weight::from_ref_time(519_031).saturating_mul(i.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Skipped Metadata (r:0 w:0)
	/// The range of component `p` is `[0, 1000]`.
	fn kill_prefix(p: u32, ) -> Weight {
		// Minimum execution time: 17_447 nanoseconds.
		Weight::from_ref_time(17_836_000)
			// Standard Error: 1_367
			.saturating_add(Weight::from_ref_time(1_125_803).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
}
