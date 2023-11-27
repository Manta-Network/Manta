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

//! Autogenerated weights for pallet_timestamp
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-09-01, STEPS: `50`, REPEAT: `40`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `pop-os`, CPU: `AMD Ryzen 9 5950X 16-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("calamari-dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/manta
// benchmark
// pallet
// --chain=calamari-dev
// --steps=50
// --repeat=40
// --pallet=pallet_timestamp
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=/home/jamie/my-repo/Manta/runtime/calamari/src/weights/pallet_timestamp.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_timestamp.
pub trait WeightInfo {
	fn set() -> Weight;
	fn on_finalize() -> Weight;
}

/// Weights for pallet_timestamp using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Timestamp::Now` (r:1 w:1)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	fn set() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `1493`
		// Minimum execution time: 7_604_000 picoseconds.
		Weight::from_parts(8_005_000, 1493)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn on_finalize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `128`
		//  Estimated: `0`
		// Minimum execution time: 4_198_000 picoseconds.
		Weight::from_parts(4_408_000, 0)
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Timestamp::Now` (r:1 w:1)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	fn set() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `1493`
		// Minimum execution time: 7_604_000 picoseconds.
		Weight::from_parts(8_005_000, 1493)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn on_finalize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `128`
		//  Estimated: `0`
		// Minimum execution time: 4_198_000 picoseconds.
		Weight::from_parts(4_408_000, 0)
	}
}
