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

#![allow(non_upper_case_globals)]
use manta_primitives::Balance;

pub const MANTA: Balance = 1_000_000_000_000_000_000; // 18 decimal
pub const cMANTA: Balance = MANTA / 100; // 16 decimal, cent-MA
pub const mMANTA: Balance = MANTA / 1_000; // 15 decimal, milli-MA
pub const uMANTA: Balance = MANTA / 1_000_000; // 12 decimal, micro-MA

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 15 * mMANTA + (bytes as Balance) * 6 * mMANTA // TODO: revisit the storage cost here
}
