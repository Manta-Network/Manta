// Copyright 2020-2022 Manta Network.
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

//! Autogenerated weights for pallet_treasury
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-09, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("calamari-dev"), DB CACHE: 1024

// Executed Command:
// manta
// benchmark
// pallet
// --chain=calamari-dev
// --pallet=pallet_treasury
// --extrinsic=*
// --execution=Wasm
// --wasm-execution=Compiled
// --heap-pages=4096
// --repeat=20
// --steps=50
// --template=.github/resources/frame-weight-template.hbs
// --output=pallet_treasury.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_treasury.
pub trait WeightInfo {
    fn spend() -> Weight;
    fn propose_spend() -> Weight;
    fn reject_proposal() -> Weight;
    fn approve_proposal(p: u32, ) -> Weight;
    fn remove_approval() -> Weight;
    fn on_initialize_proposals(p: u32, ) -> Weight;
}

/// Weights for pallet_treasury using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_treasury::WeightInfo for SubstrateWeight<T> {
    fn spend() -> Weight {
        (167_000 as Weight)
    }
    // Storage: Treasury ProposalCount (r:1 w:1)
    // Storage: Treasury Proposals (r:0 w:1)
    fn propose_spend() -> Weight {
        (30_165_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Treasury Proposals (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn reject_proposal() -> Weight {
        (52_229_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Treasury Proposals (r:1 w:0)
    // Storage: Treasury Approvals (r:1 w:1)
    fn approve_proposal(p: u32, ) -> Weight {
        (13_256_000 as Weight)
            // Standard Error: 0
            .saturating_add((121_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Treasury Approvals (r:1 w:1)
    fn remove_approval() -> Weight {
        (8_924_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Treasury Approvals (r:1 w:1)
    // Storage: Treasury Proposals (r:2 w:2)
    // Storage: System Account (r:4 w:4)
    fn on_initialize_proposals(p: u32, ) -> Weight {
        (28_668_000 as Weight)
            // Standard Error: 33_000
            .saturating_add((33_162_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(p as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(p as Weight)))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn spend() -> Weight {
        (167_000 as Weight)
    }
    // Storage: Treasury ProposalCount (r:1 w:1)
    // Storage: Treasury Proposals (r:0 w:1)
    fn propose_spend() -> Weight {
        (30_165_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Treasury Proposals (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn reject_proposal() -> Weight {
        (52_229_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Treasury Proposals (r:1 w:0)
    // Storage: Treasury Approvals (r:1 w:1)
    fn approve_proposal(p: u32, ) -> Weight {
        (13_256_000 as Weight)
            // Standard Error: 0
            .saturating_add((121_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Treasury Approvals (r:1 w:1)
    fn remove_approval() -> Weight {
        (8_924_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Treasury Approvals (r:1 w:1)
    // Storage: Treasury Proposals (r:2 w:2)
    // Storage: System Account (r:4 w:4)
    fn on_initialize_proposals(p: u32, ) -> Weight {
        (28_668_000 as Weight)
            // Standard Error: 33_000
            .saturating_add((33_162_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().reads((3 as Weight).saturating_mul(p as Weight)))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes((3 as Weight).saturating_mul(p as Weight)))
    }
}
