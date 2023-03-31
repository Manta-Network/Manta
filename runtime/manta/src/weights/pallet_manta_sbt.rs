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

//! Autogenerated weights for pallet_manta_sbt
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-31, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=20
// --pallet=pallet_manta_sbt
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_manta_sbt.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_manta_sbt.
pub trait WeightInfo {
    fn to_private() -> Weight;
    fn reserve_sbt() -> Weight;
}

/// Weights for pallet_manta_sbt using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_manta_sbt::WeightInfo for SubstrateWeight<T> {
    // Storage: MantaSbt ReservedIds (r:1 w:1)
    // Storage: MantaSbt UtxoSet (r:1 w:1)
    // Storage: MantaSbt ShardTrees (r:1 w:1)
    // Storage: MantaSbt UtxoAccumulatorOutputs (r:0 w:1)
    // Storage: MantaSbt SbtMetadata (r:0 w:1)
    // Storage: MantaSbt Shards (r:0 w:1)
    fn to_private() -> Weight {
        (39_776_056_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: MantaSbt NextSbtId (r:1 w:1)
    // Storage: MantaSbt ReservedIds (r:0 w:1)
    fn reserve_sbt() -> Weight {
        (53_484_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: MantaSbt ReservedIds (r:1 w:1)
    // Storage: MantaSbt UtxoSet (r:1 w:1)
    // Storage: MantaSbt ShardTrees (r:1 w:1)
    // Storage: MantaSbt UtxoAccumulatorOutputs (r:0 w:1)
    // Storage: MantaSbt SbtMetadata (r:0 w:1)
    // Storage: MantaSbt Shards (r:0 w:1)
    fn to_private() -> Weight {
        (39_776_056_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: MantaSbt NextSbtId (r:1 w:1)
    // Storage: MantaSbt ReservedIds (r:0 w:1)
    fn reserve_sbt() -> Weight {
        (53_484_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
}
