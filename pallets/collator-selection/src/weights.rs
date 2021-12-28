// Copyright 2020-2021 Manta Network.
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

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

// The weight info trait for `pallet_collator_selection`.
pub trait WeightInfo {
	fn set_invulnerables(_b: u32) -> Weight;
	fn set_desired_candidates() -> Weight;
	fn set_candidacy_bond() -> Weight;
	fn register_as_candidate(_c: u32) -> Weight;
	fn leave_intent(_c: u32) -> Weight;
	fn note_author() -> Weight;
	fn new_session(_c: u32, _r: u32) -> Weight;
}

/// Weights for pallet_collator_selection using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn set_invulnerables(b: u32) -> Weight {
		(18_563_000 as Weight)
			// Standard Error: 0
			.saturating_add((68_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_desired_candidates() -> Weight {
		(16_363_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_candidacy_bond() -> Weight {
		(16_840_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn register_as_candidate(c: u32) -> Weight {
		(71_196_000 as Weight)
			// Standard Error: 0
			.saturating_add((198_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn leave_intent(c: u32) -> Weight {
		(55_336_000 as Weight)
			// Standard Error: 0
			.saturating_add((151_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn note_author() -> Weight {
		(71_461_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn new_session(r: u32, c: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 1_010_000
			.saturating_add((109_961_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 1_010_000
			.saturating_add((151_952_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(r as Weight)))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(c as Weight)))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn set_invulnerables(b: u32) -> Weight {
		(18_563_000 as Weight)
			// Standard Error: 0
			.saturating_add((68_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_desired_candidates() -> Weight {
		(16_363_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_candidacy_bond() -> Weight {
		(16_840_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn register_as_candidate(c: u32) -> Weight {
		(71_196_000 as Weight)
			// Standard Error: 0
			.saturating_add((198_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn leave_intent(c: u32) -> Weight {
		(55_336_000 as Weight)
			// Standard Error: 0
			.saturating_add((151_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn note_author() -> Weight {
		(71_461_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
	fn new_session(r: u32, c: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 1_010_000
			.saturating_add((109_961_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 1_010_000
			.saturating_add((151_952_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
			.saturating_add(RocksDbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
			.saturating_add(RocksDbWeight::get().writes((2 as Weight).saturating_mul(r as Weight)))
			.saturating_add(RocksDbWeight::get().writes((2 as Weight).saturating_mul(c as Weight)))
	}
}
