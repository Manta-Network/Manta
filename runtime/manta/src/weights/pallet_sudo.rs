#![cfg_attr(rustfmt, rustfmt_skip)]
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

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_sudo`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_sudo::WeightInfo for SubstrateWeight<T> {
	/// Storage: Sudo Key (r:1 w:1)
	/// Proof: Sudo Key (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn set_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 12_332_000 picoseconds.
		Weight::from_parts(12_554_000, 0)
			.saturating_add(Weight::from_parts(0, 1517))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Sudo Key (r:1 w:0)
	/// Proof: Sudo Key (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn sudo() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 12_031_000 picoseconds.
		Weight::from_parts(12_395_000, 0)
			.saturating_add(Weight::from_parts(0, 1517))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: Sudo Key (r:1 w:0)
	/// Proof: Sudo Key (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn sudo_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `132`
		//  Estimated: `1517`
		// Minimum execution time: 12_036_000 picoseconds.
		Weight::from_parts(12_433_000, 0)
			.saturating_add(Weight::from_parts(0, 1517))
			.saturating_add(T::DbWeight::get().reads(1))
	}
}
