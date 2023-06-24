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

//! Autogenerated weights for zenlink_protocol
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-16, STEPS: `50`, REPEAT: 40, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("calamari-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=calamari-dev
// --steps=50
// --repeat=40
// --pallet=zenlink_protocol
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/zenlink_protocol.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for zenlink_protocol.
pub trait WeightInfo {
    fn set_fee_receiver() -> Weight;
    fn set_fee_point() -> Weight;
    fn create_pair() -> Weight;
    fn bootstrap_create() -> Weight;
    fn bootstrap_contribute() -> Weight;
    fn bootstrap_claim() -> Weight;
    fn bootstrap_end() -> Weight;
    fn bootstrap_update() -> Weight;
    fn bootstrap_refund() -> Weight;
    fn add_liquidity() -> Weight;
    fn remove_liquidity() -> Weight;
    fn swap_exact_assets_for_assets() -> Weight;
    fn swap_assets_for_exact_assets() -> Weight;
}

/// Weights for zenlink_protocol using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> zenlink_protocol::WeightInfo for SubstrateWeight<T> {
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_receiver() -> Weight {
		// Minimum execution time: 18_780 nanoseconds.
		Weight::from_ref_time(19_454_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_point() -> Weight {
		// Minimum execution time: 12_070 nanoseconds.
		Weight::from_ref_time(12_449_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignList (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	fn create_pair() -> Weight {
		// Minimum execution time: 28_323 nanoseconds.
		Weight::from_ref_time(31_133_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapLimits (r:0 w:1)
	// Storage: ZenlinkProtocol BootstrapRewards (r:0 w:1)
	fn bootstrap_create() -> Weight {
		// Minimum execution time: 23_627 nanoseconds.
		Weight::from_ref_time(24_503_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: ZenlinkProtocol BootstrapLimits (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	fn bootstrap_contribute() -> Weight {
		// Minimum execution time: 66_108 nanoseconds.
		Weight::from_ref_time(68_303_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:0)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:1 w:0)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: AssetManager LocationAssetId (r:3 w:0)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:5 w:5)
	// Storage: Assets Metadata (r:4 w:4)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapRewards (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:2 w:2)
	// Storage: AssetManager AssetIdMetadata (r:0 w:4)
	fn bootstrap_claim() -> Weight {
		// Minimum execution time: 240_595 nanoseconds.
		Weight::from_ref_time(245_086_000)
			.saturating_add(T::DbWeight::get().reads(24))
			.saturating_add(T::DbWeight::get().writes(20))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: AssetManager LocationAssetId (r:3 w:3)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:5 w:5)
	// Storage: Assets Metadata (r:5 w:5)
	// Storage: Assets Account (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:5)
	// Storage: AssetManager AssetIdLocation (r:0 w:3)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:0 w:1)
	fn bootstrap_end() -> Weight {
		// Minimum execution time: 207_899 nanoseconds.
		Weight::from_ref_time(221_469_000)
			.saturating_add(T::DbWeight::get().reads(22))
			.saturating_add(T::DbWeight::get().writes(31))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapRewards (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapLimits (r:0 w:1)
	fn bootstrap_update() -> Weight {
		// Minimum execution time: 30_974 nanoseconds.
		Weight::from_ref_time(31_494_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	fn bootstrap_refund() -> Weight {
		// Minimum execution time: 63_658 nanoseconds.
		Weight::from_ref_time(69_312_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:0)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: AssetManager LocationAssetId (r:3 w:3)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:5 w:5)
	// Storage: Assets Metadata (r:5 w:5)
	// Storage: Assets Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:5)
	// Storage: AssetManager AssetIdLocation (r:0 w:3)
	fn add_liquidity() -> Weight {
		// Minimum execution time: 215_726 nanoseconds.
		Weight::from_ref_time(217_839_000)
			.saturating_add(T::DbWeight::get().reads(24))
			.saturating_add(T::DbWeight::get().writes(28))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:0)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: AssetManager LocationAssetId (r:3 w:0)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Metadata (r:2 w:2)
	// Storage: Assets Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:2)
	fn remove_liquidity() -> Weight {
		// Minimum execution time: 163_415 nanoseconds.
		Weight::from_ref_time(165_016_000)
			.saturating_add(T::DbWeight::get().reads(19))
			.saturating_add(T::DbWeight::get().writes(14))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:6 w:6)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	fn swap_exact_assets_for_assets() -> Weight {
		// Minimum execution time: 98_109 nanoseconds.
		Weight::from_ref_time(103_746_000)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:6 w:6)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	fn swap_assets_for_exact_assets() -> Weight {
		// Minimum execution time: 98_449 nanoseconds.
		Weight::from_ref_time(99_498_000)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(6))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_receiver() -> Weight {
		// Minimum execution time: 18_780 nanoseconds.
		Weight::from_ref_time(19_454_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_point() -> Weight {
		// Minimum execution time: 12_070 nanoseconds.
		Weight::from_ref_time(12_449_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignList (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	fn create_pair() -> Weight {
		// Minimum execution time: 28_323 nanoseconds.
		Weight::from_ref_time(31_133_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapLimits (r:0 w:1)
	// Storage: ZenlinkProtocol BootstrapRewards (r:0 w:1)
	fn bootstrap_create() -> Weight {
		// Minimum execution time: 23_627 nanoseconds.
		Weight::from_ref_time(24_503_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: ZenlinkProtocol BootstrapLimits (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	fn bootstrap_contribute() -> Weight {
		// Minimum execution time: 66_108 nanoseconds.
		Weight::from_ref_time(68_303_000)
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:0)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:1 w:0)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: AssetManager LocationAssetId (r:3 w:0)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:5 w:5)
	// Storage: Assets Metadata (r:4 w:4)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapRewards (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:2 w:2)
	// Storage: AssetManager AssetIdMetadata (r:0 w:4)
	fn bootstrap_claim() -> Weight {
		// Minimum execution time: 240_595 nanoseconds.
		Weight::from_ref_time(245_086_000)
			.saturating_add(RocksDbWeight::get().reads(24))
			.saturating_add(RocksDbWeight::get().writes(20))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: AssetManager LocationAssetId (r:3 w:3)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:5 w:5)
	// Storage: Assets Metadata (r:5 w:5)
	// Storage: Assets Account (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:5)
	// Storage: AssetManager AssetIdLocation (r:0 w:3)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:0 w:1)
	fn bootstrap_end() -> Weight {
		// Minimum execution time: 207_899 nanoseconds.
		Weight::from_ref_time(221_469_000)
			.saturating_add(RocksDbWeight::get().reads(22))
			.saturating_add(RocksDbWeight::get().writes(31))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapRewards (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapLimits (r:0 w:1)
	fn bootstrap_update() -> Weight {
		// Minimum execution time: 30_974 nanoseconds.
		Weight::from_ref_time(31_494_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	fn bootstrap_refund() -> Weight {
		// Minimum execution time: 63_658 nanoseconds.
		Weight::from_ref_time(69_312_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:0)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: AssetManager LocationAssetId (r:3 w:3)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:5 w:5)
	// Storage: Assets Metadata (r:5 w:5)
	// Storage: Assets Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:5)
	// Storage: AssetManager AssetIdLocation (r:0 w:3)
	fn add_liquidity() -> Weight {
		// Minimum execution time: 215_726 nanoseconds.
		Weight::from_ref_time(217_839_000)
			.saturating_add(RocksDbWeight::get().reads(24))
			.saturating_add(RocksDbWeight::get().writes(28))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:4 w:4)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:0)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: AssetManager LocationAssetId (r:3 w:0)
	// Storage: AssetManager NextAssetId (r:1 w:1)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Metadata (r:2 w:2)
	// Storage: Assets Account (r:1 w:1)
	// Storage: AssetManager AssetIdMetadata (r:0 w:2)
	fn remove_liquidity() -> Weight {
		// Minimum execution time: 163_415 nanoseconds.
		Weight::from_ref_time(165_016_000)
			.saturating_add(RocksDbWeight::get().reads(19))
			.saturating_add(RocksDbWeight::get().writes(14))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:6 w:6)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	fn swap_exact_assets_for_assets() -> Weight {
		// Minimum execution time: 98_109 nanoseconds.
		Weight::from_ref_time(103_746_000)
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol ForeignLedger (r:6 w:6)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	fn swap_assets_for_exact_assets() -> Weight {
		// Minimum execution time: 98_449 nanoseconds.
		Weight::from_ref_time(99_498_000)
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
}
