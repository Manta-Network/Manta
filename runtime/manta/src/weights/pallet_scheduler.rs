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

//! Autogenerated weights for pallet_scheduler
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=20
// --pallet=pallet_scheduler
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_scheduler.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_scheduler.
pub trait WeightInfo {
    fn service_agendas_base() -> Weight;
    fn service_agenda_base(s: u32, ) -> Weight;
    fn service_task_base() -> Weight;
    fn service_task_fetched(s: u32, ) -> Weight;
    fn service_task_named() -> Weight;
    fn service_task_periodic() -> Weight;
    fn execute_dispatch_signed() -> Weight;
    fn execute_dispatch_unsigned() -> Weight;
    fn schedule(s: u32, ) -> Weight;
    fn cancel(s: u32, ) -> Weight;
    fn schedule_named(s: u32, ) -> Weight;
    fn cancel_named(s: u32, ) -> Weight;
}

/// Weights for pallet_scheduler using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_scheduler::WeightInfo for SubstrateWeight<T> {
	// Storage: Scheduler IncompleteSince (r:1 w:1)
	fn service_agendas_base() -> Weight {
		// Minimum execution time: 4_626 nanoseconds.
		Weight::from_ref_time(4_746_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[0, 50]`.
	fn service_agenda_base(s: u32, ) -> Weight {
		// Minimum execution time: 7_982 nanoseconds.
		Weight::from_ref_time(8_979_566)
			// Standard Error: 8_783
			.saturating_add(Weight::from_ref_time(639_249).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn service_task_base() -> Weight {
		// Minimum execution time: 10_640 nanoseconds.
		Weight::from_ref_time(10_831_000)
	}
	// Storage: Preimage PreimageFor (r:1 w:1)
	// Storage: Preimage StatusFor (r:1 w:1)
	/// The range of component `s` is `[128, 4194304]`.
	fn service_task_fetched(s: u32, ) -> Weight {
		// Minimum execution time: 69_680 nanoseconds.
		Weight::from_ref_time(70_776_000)
			// Standard Error: 2
			.saturating_add(Weight::from_ref_time(1_739).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Scheduler Lookup (r:0 w:1)
	fn service_task_named() -> Weight {
		// Minimum execution time: 12_243 nanoseconds.
		Weight::from_ref_time(12_487_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn service_task_periodic() -> Weight {
		// Minimum execution time: 10_618 nanoseconds.
		Weight::from_ref_time(10_860_000)
	}
	fn execute_dispatch_signed() -> Weight {
		// Minimum execution time: 4_408 nanoseconds.
		Weight::from_ref_time(4_494_000)
	}
	fn execute_dispatch_unsigned() -> Weight {
		// Minimum execution time: 13_070 nanoseconds.
		Weight::from_ref_time(13_624_000)
	}
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[0, 49]`.
	fn schedule(s: u32, ) -> Weight {
		// Minimum execution time: 23_461 nanoseconds.
		Weight::from_ref_time(25_359_624)
			// Standard Error: 7_330
			.saturating_add(Weight::from_ref_time(629_077).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Scheduler Agenda (r:1 w:1)
	// Storage: Scheduler Lookup (r:0 w:1)
	/// The range of component `s` is `[1, 50]`.
	fn cancel(s: u32, ) -> Weight {
		// Minimum execution time: 23_602 nanoseconds.
		Weight::from_ref_time(27_541_394)
			// Standard Error: 12_571
			.saturating_add(Weight::from_ref_time(1_086_436).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Scheduler Lookup (r:1 w:1)
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[0, 49]`.
	fn schedule_named(s: u32, ) -> Weight {
		// Minimum execution time: 21_418 nanoseconds.
		Weight::from_ref_time(34_372_626)
			// Standard Error: 28_265
			.saturating_add(Weight::from_ref_time(531_180).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Scheduler Lookup (r:1 w:1)
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[1, 50]`.
	fn cancel_named(s: u32, ) -> Weight {
		// Minimum execution time: 25_315 nanoseconds.
		Weight::from_ref_time(26_725_614)
			// Standard Error: 6_378
			.saturating_add(Weight::from_ref_time(1_194_759).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Scheduler IncompleteSince (r:1 w:1)
	fn service_agendas_base() -> Weight {
		// Minimum execution time: 4_626 nanoseconds.
		Weight::from_ref_time(4_746_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[0, 50]`.
	fn service_agenda_base(s: u32, ) -> Weight {
		// Minimum execution time: 7_982 nanoseconds.
		Weight::from_ref_time(8_979_566)
			// Standard Error: 8_783
			.saturating_add(Weight::from_ref_time(639_249).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn service_task_base() -> Weight {
		// Minimum execution time: 10_640 nanoseconds.
		Weight::from_ref_time(10_831_000)
	}
	// Storage: Preimage PreimageFor (r:1 w:1)
	// Storage: Preimage StatusFor (r:1 w:1)
	/// The range of component `s` is `[128, 4194304]`.
	fn service_task_fetched(s: u32, ) -> Weight {
		// Minimum execution time: 69_680 nanoseconds.
		Weight::from_ref_time(70_776_000)
			// Standard Error: 2
			.saturating_add(Weight::from_ref_time(1_739).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Scheduler Lookup (r:0 w:1)
	fn service_task_named() -> Weight {
		// Minimum execution time: 12_243 nanoseconds.
		Weight::from_ref_time(12_487_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn service_task_periodic() -> Weight {
		// Minimum execution time: 10_618 nanoseconds.
		Weight::from_ref_time(10_860_000)
	}
	fn execute_dispatch_signed() -> Weight {
		// Minimum execution time: 4_408 nanoseconds.
		Weight::from_ref_time(4_494_000)
	}
	fn execute_dispatch_unsigned() -> Weight {
		// Minimum execution time: 13_070 nanoseconds.
		Weight::from_ref_time(13_624_000)
	}
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[0, 49]`.
	fn schedule(s: u32, ) -> Weight {
		// Minimum execution time: 23_461 nanoseconds.
		Weight::from_ref_time(25_359_624)
			// Standard Error: 7_330
			.saturating_add(Weight::from_ref_time(629_077).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Scheduler Agenda (r:1 w:1)
	// Storage: Scheduler Lookup (r:0 w:1)
	/// The range of component `s` is `[1, 50]`.
	fn cancel(s: u32, ) -> Weight {
		// Minimum execution time: 23_602 nanoseconds.
		Weight::from_ref_time(27_541_394)
			// Standard Error: 12_571
			.saturating_add(Weight::from_ref_time(1_086_436).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Scheduler Lookup (r:1 w:1)
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[0, 49]`.
	fn schedule_named(s: u32, ) -> Weight {
		// Minimum execution time: 21_418 nanoseconds.
		Weight::from_ref_time(34_372_626)
			// Standard Error: 28_265
			.saturating_add(Weight::from_ref_time(531_180).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: Scheduler Lookup (r:1 w:1)
	// Storage: Scheduler Agenda (r:1 w:1)
	/// The range of component `s` is `[1, 50]`.
	fn cancel_named(s: u32, ) -> Weight {
		// Minimum execution time: 25_315 nanoseconds.
		Weight::from_ref_time(26_725_614)
			// Standard Error: 6_378
			.saturating_add(Weight::from_ref_time(1_194_759).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
