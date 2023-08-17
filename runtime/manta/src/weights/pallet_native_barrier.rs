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

//! Autogenerated weights for pallet_native_barrier
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-08-17, STEPS: `50`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=40
// --pallet=pallet_native_barrier
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_native_barrier.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_native_barrier.
pub trait WeightInfo {
    fn on_initialize() -> Weight;
    fn set_start_unix_time() -> Weight;
    fn set_daily_xcm_limit() -> Weight;
    fn add_accounts_to_native_barrier() -> Weight;
    fn remove_accounts_from_native_barrier() -> Weight;
}

/// Weights for pallet_native_barrier using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_native_barrier::WeightInfo for SubstrateWeight<T> {
	// Storage: NativeBarrier StartUnixTime (r:1 w:0)
	// Storage: NativeBarrier DailyXcmLimit (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: NativeBarrier LastDayProcessed (r:1 w:1)
	// Storage: NativeBarrier RemainingXcmLimit (r:6 w:5)
	fn on_initialize() -> Weight {
		// Minimum execution time: 45_706 nanoseconds.
		Weight::from_ref_time(46_457_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: NativeBarrier StartUnixTime (r:0 w:1)
	fn set_start_unix_time() -> Weight {
		// Minimum execution time: 14_287 nanoseconds.
		Weight::from_ref_time(14_948_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: NativeBarrier DailyXcmLimit (r:0 w:1)
	fn set_daily_xcm_limit() -> Weight {
		// Minimum execution time: 14_017 nanoseconds.
		Weight::from_ref_time(14_628_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: NativeBarrier DailyXcmLimit (r:1 w:0)
	// Storage: NativeBarrier StartUnixTime (r:1 w:0)
	// Storage: NativeBarrier RemainingXcmLimit (r:5 w:5)
	fn add_accounts_to_native_barrier() -> Weight {
		// Minimum execution time: 28_283 nanoseconds.
		Weight::from_ref_time(28_994_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: NativeBarrier RemainingXcmLimit (r:0 w:3)
	fn remove_accounts_from_native_barrier() -> Weight {
		// Minimum execution time: 16_831 nanoseconds.
		Weight::from_ref_time(17_824_000)
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: NativeBarrier StartUnixTime (r:1 w:0)
	// Storage: NativeBarrier DailyXcmLimit (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: NativeBarrier LastDayProcessed (r:1 w:1)
	// Storage: NativeBarrier RemainingXcmLimit (r:6 w:5)
	fn on_initialize() -> Weight {
		// Minimum execution time: 45_706 nanoseconds.
		Weight::from_ref_time(46_457_000)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: NativeBarrier StartUnixTime (r:0 w:1)
	fn set_start_unix_time() -> Weight {
		// Minimum execution time: 14_287 nanoseconds.
		Weight::from_ref_time(14_948_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: NativeBarrier DailyXcmLimit (r:0 w:1)
	fn set_daily_xcm_limit() -> Weight {
		// Minimum execution time: 14_017 nanoseconds.
		Weight::from_ref_time(14_628_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: NativeBarrier DailyXcmLimit (r:1 w:0)
	// Storage: NativeBarrier StartUnixTime (r:1 w:0)
	// Storage: NativeBarrier RemainingXcmLimit (r:5 w:5)
	fn add_accounts_to_native_barrier() -> Weight {
		// Minimum execution time: 28_283 nanoseconds.
		Weight::from_ref_time(28_994_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: NativeBarrier RemainingXcmLimit (r:0 w:3)
	fn remove_accounts_from_native_barrier() -> Weight {
		// Minimum execution time: 16_831 nanoseconds.
		Weight::from_ref_time(17_824_000)
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}
