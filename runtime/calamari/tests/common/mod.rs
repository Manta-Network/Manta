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

pub mod mock;

pub use calamari_runtime::{currency::KMA, Event, Origin, Runtime, System};

use frame_support::weights::{DispatchInfo, Weight};
use manta_primitives::types::Balance;

pub const BOND_AMOUNT: Balance = 1_000 * KMA;
pub const INITIAL_BALANCE: Balance = 1_000_000_000_000 * KMA;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo {
		weight: w,
		..Default::default()
	}
}

pub fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

pub fn root_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::root()
}
