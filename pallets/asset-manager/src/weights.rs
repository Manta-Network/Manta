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

//! Autogenerated weights for pallet_asset_manager
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/manta
// benchmark
// pallet
// --chain=manta-dev
// --pallet=pallet_asset_manager
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --steps=50
// --repeat=20
// --heap-pages=4096
// --output=./pallets/asset-manager/src/weights.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_asset_manager.
pub trait WeightInfo {
    fn register_asset() -> Weight;
    fn set_units_per_second() -> Weight;
    fn update_asset_location() -> Weight;
    fn update_asset_metadata() -> Weight;
    fn mint_asset() -> Weight;
    fn set_min_xcm_fee() -> Weight;
    fn update_outgoing_filtered_assets() -> Weight;
    fn register_lp_asset() -> Weight;
    fn permissionless_register_asset() -> Weight;
}

/// Weights for pallet_asset_manager using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: AssetManager LocationAssetId (r:1 w:1)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	// Storage: AssetManager AssetIdLocation (r:0 w:1)
	fn register_asset() -> Weight {
		// Minimum execution time: 48_672 nanoseconds.
		Weight::from_ref_time(49_599_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:0)
	// Storage: AssetManager UnitsPerSecond (r:0 w:1)
	fn set_units_per_second() -> Weight {
		// Minimum execution time: 73_894 nanoseconds.
		Weight::from_ref_time(77_860_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:1)
	// Storage: AssetManager LocationAssetId (r:1 w:2)
	// Storage: AssetManager AllowedDestParaIds (r:2 w:2)
	fn update_asset_location() -> Weight {
		// Minimum execution time: 87_842 nanoseconds.
		Weight::from_ref_time(102_703_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:0)
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	fn update_asset_metadata() -> Weight {
		// Minimum execution time: 86_649 nanoseconds.
		Weight::from_ref_time(93_473_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn mint_asset() -> Weight {
		// Minimum execution time: 100_617 nanoseconds.
		Weight::from_ref_time(106_195_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: AssetManager MinXcmFee (r:0 w:1)
	fn set_min_xcm_fee() -> Weight {
		// Minimum execution time: 60_673 nanoseconds.
		Weight::from_ref_time(61_758_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AssetManager FilteredOutgoingAssetLocations (r:0 w:1)
	fn update_outgoing_filtered_assets() -> Weight {
		// Minimum execution time: 52_269 nanoseconds.
		Weight::from_ref_time(55_555_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: AssetManager AssetIdLocation (r:2 w:0)
	// Storage: AssetManager AssetIdPairToLp (r:1 w:1)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	// Storage: AssetManager LpToAssetIdPair (r:0 w:1)
	fn register_lp_asset() -> Weight {
		// Minimum execution time: 54_905 nanoseconds.
		Weight::from_ref_time(56_256_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: System Account (r:1 w:1)
	// Storage: AssetManager NextPermissionlessAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	fn permissionless_register_asset() -> Weight {
		// Minimum execution time: 81_474 nanoseconds.
		Weight::from_ref_time(83_160_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: AssetManager LocationAssetId (r:1 w:1)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	// Storage: AssetManager AssetIdLocation (r:0 w:1)
	fn register_asset() -> Weight {
		// Minimum execution time: 48_672 nanoseconds.
		Weight::from_ref_time(49_599_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:0)
	// Storage: AssetManager UnitsPerSecond (r:0 w:1)
	fn set_units_per_second() -> Weight {
		// Minimum execution time: 73_894 nanoseconds.
		Weight::from_ref_time(77_860_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:1)
	// Storage: AssetManager LocationAssetId (r:1 w:2)
	// Storage: AssetManager AllowedDestParaIds (r:2 w:2)
	fn update_asset_location() -> Weight {
		// Minimum execution time: 87_842 nanoseconds.
		Weight::from_ref_time(102_703_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:0)
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	fn update_asset_metadata() -> Weight {
		// Minimum execution time: 86_649 nanoseconds.
		Weight::from_ref_time(93_473_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: AssetManager AssetIdLocation (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn mint_asset() -> Weight {
		// Minimum execution time: 100_617 nanoseconds.
		Weight::from_ref_time(106_195_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: AssetManager MinXcmFee (r:0 w:1)
	fn set_min_xcm_fee() -> Weight {
		// Minimum execution time: 60_673 nanoseconds.
		Weight::from_ref_time(61_758_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AssetManager FilteredOutgoingAssetLocations (r:0 w:1)
	fn update_outgoing_filtered_assets() -> Weight {
		// Minimum execution time: 52_269 nanoseconds.
		Weight::from_ref_time(55_555_000)
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: AssetManager AssetIdLocation (r:2 w:0)
	// Storage: AssetManager AssetIdPairToLp (r:1 w:1)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	// Storage: AssetManager LpToAssetIdPair (r:0 w:1)
	fn register_lp_asset() -> Weight {
		// Minimum execution time: 54_905 nanoseconds.
		Weight::from_ref_time(56_256_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: System Account (r:1 w:1)
	// Storage: AssetManager NextPermissionlessAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:1)
	fn permissionless_register_asset() -> Weight {
		// Minimum execution time: 81_474 nanoseconds.
		Weight::from_ref_time(83_160_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
}
