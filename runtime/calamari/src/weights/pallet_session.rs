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

//! Autogenerated weights for pallet_session
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
// --pallet=pallet_session
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=/home/jamie/my-repo/Manta/runtime/calamari/src/weights/pallet_session.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_session.
pub trait WeightInfo {
	fn set_keys() -> Weight;
	fn purge_keys() -> Weight;
}

/// Weights for pallet_session using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::KeyOwner` (r:3 w:3)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `307`
		//  Estimated: `8722`
		// Minimum execution time: 26_951_000 picoseconds.
		Weight::from_parts(28_033_000, 8722)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::KeyOwner` (r:0 w:3)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn purge_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `308`
		//  Estimated: `3773`
		// Minimum execution time: 16_942_000 picoseconds.
		Weight::from_parts(17_774_000, 3773)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::KeyOwner` (r:3 w:3)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `307`
		//  Estimated: `8722`
		// Minimum execution time: 26_951_000 picoseconds.
		Weight::from_parts(28_033_000, 8722)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `Session::NextKeys` (r:1 w:1)
	/// Proof: `Session::NextKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Session::KeyOwner` (r:0 w:3)
	/// Proof: `Session::KeyOwner` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn purge_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `308`
		//  Estimated: `3773`
		// Minimum execution time: 16_942_000 picoseconds.
		Weight::from_parts(17_774_000, 3773)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
}
