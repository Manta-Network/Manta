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

//! A list of the different weight modules for our runtime.

pub mod calamari_vesting;
pub mod frame_system;
pub mod pallet_balances;
pub mod pallet_collective;
pub mod pallet_democracy;
pub mod pallet_membership;
pub mod pallet_multisig;
pub mod pallet_scheduler;
pub mod pallet_session;
pub mod pallet_timestamp;
pub mod pallet_tx_pause;
pub mod pallet_utility;
