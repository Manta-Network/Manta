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

//! Autogenerated weights for manta_farming
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dolphin-dev"), DB CACHE: 1024

// Executed Command:
// target/release/manta
// benchmark
// pallet
// --chain=dolphin-dev
// --pallet=manta-farming
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --steps=50
// --repeat=20
// --heap-pages=4096
// --output=./pallets/farming/src/weights.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for manta_farming.
pub trait WeightInfo {
    fn on_initialize() -> Weight;
    fn create_farming_pool() -> Weight;
    fn charge() -> Weight;
    fn deposit() -> Weight;
    fn withdraw() -> Weight;
    fn claim() -> Weight;
    fn gauge_withdraw() -> Weight;
}

/// Weights for manta_farming using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Farming PoolInfos (r:1 w:0)
	// Storage: Farming GaugePoolInfos (r:1 w:0)
	fn on_initialize() -> Weight {
		// Minimum execution time: 5_170 nanoseconds.
		Weight::from_ref_time(5_433_000)
			.saturating_add(T::DbWeight::get().reads(2))
	}
	// Storage: Farming PoolNextId (r:1 w:1)
	// Storage: Farming GaugePoolNextId (r:1 w:1)
	// Storage: Farming GaugePoolInfos (r:0 w:1)
	// Storage: Farming PoolInfos (r:0 w:1)
	fn create_farming_pool() -> Weight {
		// Minimum execution time: 27_007 nanoseconds.
		Weight::from_ref_time(27_448_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn charge() -> Weight {
		// Minimum execution time: 897_000 nanoseconds.
		Weight::from_ref_time(902_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	fn deposit() -> Weight {
		// Minimum execution time: 64_240 nanoseconds.
		Weight::from_ref_time(65_244_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	fn withdraw() -> Weight {
		// Minimum execution time: 36_542 nanoseconds.
		Weight::from_ref_time(37_369_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: Farming GaugeInfos (r:1 w:0)
	fn claim() -> Weight {
		// Minimum execution time: 35_822 nanoseconds.
		Weight::from_ref_time(37_226_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming GaugePoolInfos (r:1 w:1)
	// Storage: Farming GaugeInfos (r:1 w:1)
	// Storage: Farming PoolInfos (r:1 w:0)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:0)
	fn gauge_withdraw() -> Weight {
		// Minimum execution time: 37_084 nanoseconds.
		Weight::from_ref_time(37_875_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Farming PoolInfos (r:1 w:0)
	// Storage: Farming GaugePoolInfos (r:1 w:0)
	fn on_initialize() -> Weight {
		// Minimum execution time: 5_170 nanoseconds.
		Weight::from_ref_time(5_433_000)
			.saturating_add(RocksDbWeight::get().reads(2))
	}
	// Storage: Farming PoolNextId (r:1 w:1)
	// Storage: Farming GaugePoolNextId (r:1 w:1)
	// Storage: Farming GaugePoolInfos (r:0 w:1)
	// Storage: Farming PoolInfos (r:0 w:1)
	fn create_farming_pool() -> Weight {
		// Minimum execution time: 27_007 nanoseconds.
		Weight::from_ref_time(27_448_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn charge() -> Weight {
		// Minimum execution time: 897_000 nanoseconds.
		Weight::from_ref_time(902_000_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	fn deposit() -> Weight {
		// Minimum execution time: 64_240 nanoseconds.
		Weight::from_ref_time(65_244_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	fn withdraw() -> Weight {
		// Minimum execution time: 36_542 nanoseconds.
		Weight::from_ref_time(37_369_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Farming PoolInfos (r:1 w:1)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: Farming GaugeInfos (r:1 w:0)
	fn claim() -> Weight {
		// Minimum execution time: 35_822 nanoseconds.
		Weight::from_ref_time(37_226_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Farming GaugePoolInfos (r:1 w:1)
	// Storage: Farming GaugeInfos (r:1 w:1)
	// Storage: Farming PoolInfos (r:1 w:0)
	// Storage: Farming SharesAndWithdrawnRewards (r:1 w:0)
	fn gauge_withdraw() -> Weight {
		// Minimum execution time: 37_084 nanoseconds.
		Weight::from_ref_time(37_875_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
