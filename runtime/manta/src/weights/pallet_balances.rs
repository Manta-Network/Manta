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

//! Autogenerated weights for pallet_balances
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("manta-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=manta-dev
// --steps=50
// --repeat=20
// --pallet=pallet_balances
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_balances.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_balances.
pub trait WeightInfo {
    fn transfer() -> Weight;
    fn transfer_keep_alive() -> Weight;
    fn set_balance_creating() -> Weight;
    fn set_balance_killing() -> Weight;
    fn force_transfer() -> Weight;
    fn transfer_all() -> Weight;
    fn force_unreserve() -> Weight;
}

/// Weights for pallet_balances using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_balances::WeightInfo for SubstrateWeight<T> {
    // Storage: System Account (r:1 w:1)
    fn transfer() -> Weight {
        Weight::from_ref_time(42_800_000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn transfer_keep_alive() -> Weight {
        Weight::from_ref_time(34_569_000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn set_balance_creating() -> Weight {
        Weight::from_ref_time(22_836_000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn set_balance_killing() -> Weight {
        Weight::from_ref_time(26_166_000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:2 w:2)
    fn force_transfer() -> Weight {
        Weight::from_ref_time(42_363_000)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn transfer_all() -> Weight {
        Weight::from_ref_time(40_855_000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn force_unreserve() -> Weight {
        Weight::from_ref_time(20_239_000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: System Account (r:1 w:1)
    fn transfer() -> Weight {
        Weight::from_ref_time(42_800_000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn transfer_keep_alive() -> Weight {
        Weight::from_ref_time(34_569_000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn set_balance_creating() -> Weight {
        Weight::from_ref_time(22_836_000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn set_balance_killing() -> Weight {
        Weight::from_ref_time(26_166_000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:2 w:2)
    fn force_transfer() -> Weight {
        Weight::from_ref_time(42_363_000)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn transfer_all() -> Weight {
        Weight::from_ref_time(40_855_000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    // Storage: System Account (r:1 w:1)
    fn force_unreserve() -> Weight {
        Weight::from_ref_time(20_239_000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}
