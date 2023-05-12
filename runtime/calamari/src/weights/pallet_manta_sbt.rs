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
//! DATE: 2023-05-03, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("calamari-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=calamari-dev
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
    fn change_allowlist_account() -> Weight;
    fn allowlist_evm_account() -> Weight;
    fn new_mint_info() -> Weight;
    fn update_mint_info() -> Weight;
    fn mint_sbt_eth() -> Weight;
}

/// Weights for pallet_manta_sbt using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_manta_sbt::WeightInfo for SubstrateWeight<T> {
    // Storage: MantaSbt ReservedIds (r:1 w:1)
    // Storage: MantaSbt UtxoSet (r:1 w:1)
    // Storage: MantaSbt ShardTrees (r:1 w:1)
    // Storage: MantaSbt UtxoAccumulatorOutputs (r:0 w:1)
    // Storage: MantaSbt Shards (r:0 w:1)
    // Storage: MantaSbt SbtMetadataV2 (r:0 w:1)
    fn to_private() -> Weight {
        Weight::from_ref_time(30_118_417_000)
            .saturating_add(T::DbWeight::get().reads(3u64))
            .saturating_add(T::DbWeight::get().writes(6u64))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: MantaSbt NextSbtId (r:1 w:1)
    // Storage: MantaSbt ReservedIds (r:0 w:1)
    fn reserve_sbt() -> Weight {
        Weight::from_ref_time(48_730_000)
            .saturating_add(T::DbWeight::get().reads(2u64))
            .saturating_add(T::DbWeight::get().writes(3u64))
    }
   

    fn change_allowlist_account() -> Weight {
        Weight::from_ref_time(13_553_000)
            .saturating_add(T::DbWeight::get().writes(1u64))
    }
    // Storage: MantaSbt MintIdRegistry (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MantaSbt AllowlistAccount (r:1 w:0)
    // Storage: MantaSbt EvmAccountAllowlist (r:1 w:1)
    // Storage: MantaSbt NextSbtId (r:1 w:1)
    fn allowlist_evm_account() -> Weight {
        Weight::from_ref_time(28_472_000)
            .saturating_add(T::DbWeight::get().reads(5u64))
            .saturating_add(T::DbWeight::get().writes(2u64))
    }
    // Storage: MantaSbt NextMintId (r:1 w:1)
    // Storage: MantaSbt MintIdRegistry (r:0 w:1)
    fn new_mint_info() -> Weight {
        Weight::from_ref_time(15_471_000)
            .saturating_add(T::DbWeight::get().reads(1u64))
            .saturating_add(T::DbWeight::get().writes(2u64))
    }
    // Storage: MantaSbt MintIdRegistry (r:1 w:1)
    fn update_mint_info() -> Weight {
        Weight::from_ref_time(16_570_000)
            .saturating_add(T::DbWeight::get().reads(1u64))
            .saturating_add(T::DbWeight::get().writes(1u64))
    }
    // Storage: MantaSbt MintIdRegistry (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: System BlockHash (r:1 w:0)
    // Storage: MantaSbt EvmAccountAllowlist (r:1 w:1)
    // Storage: MantaSbt UtxoSet (r:1 w:1)
    // Storage: MantaSbt ShardTrees (r:1 w:1)
    // Storage: MantaSbt UtxoAccumulatorOutputs (r:0 w:1)
    // Storage: MantaSbt Shards (r:0 w:1)
    // Storage: MantaSbt SbtMetadataV2 (r:0 w:1)
    fn mint_sbt_eth() -> Weight {
        Weight::from_ref_time(30_190_491_000)
            .saturating_add(T::DbWeight::get().reads(6u64))
            .saturating_add(T::DbWeight::get().writes(6u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: MantaSbt ReservedIds (r:1 w:1)
    // Storage: MantaSbt UtxoSet (r:1 w:1)
    // Storage: MantaSbt ShardTrees (r:1 w:1)
    // Storage: MantaSbt UtxoAccumulatorOutputs (r:0 w:1)
    // Storage: MantaSbt Shards (r:0 w:1)
    // Storage: MantaSbt SbtMetadataV2 (r:0 w:1)
    fn to_private() -> Weight {
        Weight::from_ref_time(30_118_417_000)
            .saturating_add(RocksDbWeight::get().reads(3u64))
            .saturating_add(RocksDbWeight::get().writes(6u64))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: MantaSbt NextSbtId (r:1 w:1)
    // Storage: MantaSbt ReservedIds (r:0 w:1)
    fn reserve_sbt() -> Weight {
        Weight::from_ref_time(48_730_000)
            .saturating_add(RocksDbWeight::get().reads(2u64))
            .saturating_add(RocksDbWeight::get().writes(3u64))
    }
   

    fn change_allowlist_account() -> Weight {
        Weight::from_ref_time(13_553_000)
            .saturating_add(RocksDbWeight::get().writes(1u64))
    }
    // Storage: MantaSbt MintIdRegistry (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MantaSbt AllowlistAccount (r:1 w:0)
    // Storage: MantaSbt EvmAccountAllowlist (r:1 w:1)
    // Storage: MantaSbt NextSbtId (r:1 w:1)
    fn allowlist_evm_account() -> Weight {
        Weight::from_ref_time(28_472_000u64)
            .saturating_add(RocksDbWeight::get().reads(5u64))
            .saturating_add(RocksDbWeight::get().writes(2u64))
    }
    // Storage: MantaSbt NextMintId (r:1 w:1)
    // Storage: MantaSbt MintIdRegistry (r:0 w:1)
    fn new_mint_info() -> Weight {
        Weight::from_ref_time(15_471_000u64)
            .saturating_add(RocksDbWeight::get().reads(1u64))
            .saturating_add(RocksDbWeight::get().writes(2u64))
    }
    // Storage: MantaSbt MintIdRegistry (r:1 w:1)
    fn update_mint_info() -> Weight {
        Weight::from_ref_time(16_570_000u64)
            .saturating_add(RocksDbWeight::get().reads(1u64))
            .saturating_add(RocksDbWeight::get().writes(1u64))
    }
    // Storage: MantaSbt MintIdRegistry (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: System BlockHash (r:1 w:0)
    // Storage: MantaSbt EvmAccountAllowlist (r:1 w:1)
    // Storage: MantaSbt UtxoSet (r:1 w:1)
    // Storage: MantaSbt ShardTrees (r:1 w:1)
    // Storage: MantaSbt UtxoAccumulatorOutputs (r:0 w:1)
    // Storage: MantaSbt Shards (r:0 w:1)
    // Storage: MantaSbt SbtMetadataV2 (r:0 w:1)
    fn mint_sbt_eth() -> Weight {
        Weight::from_ref_time(30_190_491_000u64)
            .saturating_add(RocksDbWeight::get().reads(6u64))
            .saturating_add(RocksDbWeight::get().writes(6u64))
    }
}
