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

//! Autogenerated weights for pallet_collective
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("calamari-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/manta
// benchmark
// pallet
// --chain=calamari-dev
// --steps=50
// --repeat=20
// --pallet=pallet_collective
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./scripts/benchmarking/frame-weights-output/pallet_collective.rs
// --template=.github/resources/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use manta_primitives::constants::RocksDbWeight;

/// Weight functions needed for pallet_collective.
pub trait WeightInfo {
    fn set_members(m: u32, n: u32, p: u32, ) -> Weight;
    fn execute(b: u32, m: u32, ) -> Weight;
    fn propose_execute(b: u32, m: u32, ) -> Weight;
    fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight;
    fn vote(m: u32, ) -> Weight;
    fn close_early_disapproved(m: u32, p: u32, ) -> Weight;
    fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight;
    fn close_disapproved(m: u32, p: u32, ) -> Weight;
    fn close_approved(b: u32, m: u32, p: u32, ) -> Weight;
    fn disapprove_proposal(p: u32, ) -> Weight;
}

/// Weights for pallet_collective using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for SubstrateWeight<T> {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Prime (r:0 w:1)
	// Storage: Council Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 20_353 nanoseconds.
		Weight::from_ref_time(20_614_000)
			// Standard Error: 72_777
			.saturating_add(Weight::from_ref_time(5_902_155).saturating_mul(m.into()))
			// Standard Error: 72_777
			.saturating_add(Weight::from_ref_time(8_439_785).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	// Storage: Council Members (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, _m: u32, ) -> Weight {
		// Minimum execution time: 21_756 nanoseconds.
		Weight::from_ref_time(31_491_309)
			// Standard Error: 1_339
			.saturating_add(Weight::from_ref_time(1_763).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, _m: u32, ) -> Weight {
		// Minimum execution time: 24_847 nanoseconds.
		Weight::from_ref_time(42_501_098)
			// Standard Error: 1_620
			.saturating_add(Weight::from_ref_time(5_038).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(2))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 29_965 nanoseconds.
		Weight::from_ref_time(31_566_838)
			// Standard Error: 179
			.saturating_add(Weight::from_ref_time(4_756).saturating_mul(b.into()))
			// Standard Error: 1_869
			.saturating_add(Weight::from_ref_time(29_805).saturating_mul(m.into()))
			// Standard Error: 1_845
			.saturating_add(Weight::from_ref_time(167_272).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Minimum execution time: 34_069 nanoseconds.
		Weight::from_ref_time(35_129_986)
			// Standard Error: 923
			.saturating_add(Weight::from_ref_time(50_042).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 33_060 nanoseconds.
		Weight::from_ref_time(36_428_486)
			// Standard Error: 1_645
			.saturating_add(Weight::from_ref_time(30_858).saturating_mul(m.into()))
			// Standard Error: 1_604
			.saturating_add(Weight::from_ref_time(150_682).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 45_852 nanoseconds.
		Weight::from_ref_time(46_720_007)
			// Standard Error: 175
			.saturating_add(Weight::from_ref_time(2_969).saturating_mul(b.into()))
			// Standard Error: 1_850
			.saturating_add(Weight::from_ref_time(29_750).saturating_mul(m.into()))
			// Standard Error: 1_803
			.saturating_add(Weight::from_ref_time(172_035).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 35_758 nanoseconds.
		Weight::from_ref_time(39_202_474)
			// Standard Error: 1_652
			.saturating_add(Weight::from_ref_time(32_337).saturating_mul(m.into()))
			// Standard Error: 1_610
			.saturating_add(Weight::from_ref_time(150_974).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 47_717 nanoseconds.
		Weight::from_ref_time(50_277_089)
			// Standard Error: 190
			.saturating_add(Weight::from_ref_time(3_077).saturating_mul(b.into()))
			// Standard Error: 2_017
			.saturating_add(Weight::from_ref_time(19_066).saturating_mul(m.into()))
			// Standard Error: 1_966
			.saturating_add(Weight::from_ref_time(174_244).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 20_438 nanoseconds.
		Weight::from_ref_time(25_116_529)
			// Standard Error: 2_027
			.saturating_add(Weight::from_ref_time(153_622).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Prime (r:0 w:1)
	// Storage: Council Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 20_353 nanoseconds.
		Weight::from_ref_time(20_614_000)
			// Standard Error: 72_777
			.saturating_add(Weight::from_ref_time(5_902_155).saturating_mul(m.into()))
			// Standard Error: 72_777
			.saturating_add(Weight::from_ref_time(8_439_785).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(RocksDbWeight::get().writes(2))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	// Storage: Council Members (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, _m: u32, ) -> Weight {
		// Minimum execution time: 21_756 nanoseconds.
		Weight::from_ref_time(31_491_309)
			// Standard Error: 1_339
			.saturating_add(Weight::from_ref_time(1_763).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, _m: u32, ) -> Weight {
		// Minimum execution time: 24_847 nanoseconds.
		Weight::from_ref_time(42_501_098)
			// Standard Error: 1_620
			.saturating_add(Weight::from_ref_time(5_038).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 29_965 nanoseconds.
		Weight::from_ref_time(31_566_838)
			// Standard Error: 179
			.saturating_add(Weight::from_ref_time(4_756).saturating_mul(b.into()))
			// Standard Error: 1_869
			.saturating_add(Weight::from_ref_time(29_805).saturating_mul(m.into()))
			// Standard Error: 1_845
			.saturating_add(Weight::from_ref_time(167_272).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Minimum execution time: 34_069 nanoseconds.
		Weight::from_ref_time(35_129_986)
			// Standard Error: 923
			.saturating_add(Weight::from_ref_time(50_042).saturating_mul(m.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 33_060 nanoseconds.
		Weight::from_ref_time(36_428_486)
			// Standard Error: 1_645
			.saturating_add(Weight::from_ref_time(30_858).saturating_mul(m.into()))
			// Standard Error: 1_604
			.saturating_add(Weight::from_ref_time(150_682).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 45_852 nanoseconds.
		Weight::from_ref_time(46_720_007)
			// Standard Error: 175
			.saturating_add(Weight::from_ref_time(2_969).saturating_mul(b.into()))
			// Standard Error: 1_850
			.saturating_add(Weight::from_ref_time(29_750).saturating_mul(m.into()))
			// Standard Error: 1_803
			.saturating_add(Weight::from_ref_time(172_035).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 35_758 nanoseconds.
		Weight::from_ref_time(39_202_474)
			// Standard Error: 1_652
			.saturating_add(Weight::from_ref_time(32_337).saturating_mul(m.into()))
			// Standard Error: 1_610
			.saturating_add(Weight::from_ref_time(150_974).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 47_717 nanoseconds.
		Weight::from_ref_time(50_277_089)
			// Standard Error: 190
			.saturating_add(Weight::from_ref_time(3_077).saturating_mul(b.into()))
			// Standard Error: 2_017
			.saturating_add(Weight::from_ref_time(19_066).saturating_mul(m.into()))
			// Standard Error: 1_966
			.saturating_add(Weight::from_ref_time(174_244).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 20_438 nanoseconds.
		Weight::from_ref_time(25_116_529)
			// Standard Error: 2_027
			.saturating_add(Weight::from_ref_time(153_622).saturating_mul(p.into()))
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}
