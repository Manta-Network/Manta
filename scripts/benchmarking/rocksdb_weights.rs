// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-17 (Y/M/D)
//! HOSTNAME: `georgi-desktop`, CPU: `AMD Ryzen 9 5900X 12-Core Processor`
//!
//! DATABASE: `RocksDb`, RUNTIME: `Dolphin Parachain Development`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `10`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: `0`
//! WEIGHT-PATH: `./scripts/benchmarking/rocksdb_weights.rs`
//! METRIC: `Average`, WEIGHT-MUL: `1`, WEIGHT-ADD: `0`

// Executed Command:
//   ./target/debug/manta
//   benchmark
//   storage
//   --chain=dolphin-dev
//   --state-version=1
//   --warmups=10
//   --base-path=
//   --weight-path=./scripts/benchmarking/rocksdb_weights.rs

/// Storage DB weights for the `Dolphin Parachain Development` runtime and `RocksDb`.
pub mod constants {
	use frame_support::{
		parameter_types,
		weights::{constants, RuntimeDbWeight},
	};

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			/// Time to read one storage item.
			/// Calculated by multiplying the *Average* of all values with `1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 75_973, 198_142
			///   Average:  86_652
			///   Median:   81_594
			///   Std-Dev:  16001.59
			///
			/// Percentiles nanoseconds:
			///   99th: 198_142
			///   95th: 107_222
			///   75th: 84_990
			read: 86_652 * constants::WEIGHT_PER_NANOS,

			/// Time to write one storage item.
			/// Calculated by multiplying the *Average* of all values with `1` and adding `0`.
			///
			/// Stats nanoseconds:
			///   Min, Max: 92_624, 37_827_640
			///   Average:  744_429
			///   Median:   211_007
			///   Std-Dev:  4401482.08
			///
			/// Percentiles nanoseconds:
			///   99th: 37_827_640
			///   95th: 296_167
			///   75th: 235_453
			write: 744_429 * constants::WEIGHT_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use frame_support::weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1) >= constants::WEIGHT_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1) >= constants::WEIGHT_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1) <= constants::WEIGHT_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1) <= constants::WEIGHT_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}
