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

//! A list of the different weight modules for our runtime.

pub mod cumulus_pallet_xcmp_queue;
pub mod frame_system;
pub mod manta_collator_selection;
pub mod pallet_asset_manager;
pub mod pallet_assets;
pub mod pallet_author_inherent;
pub mod pallet_balances;
pub mod pallet_collective;
pub mod pallet_democracy;
pub mod pallet_farming;
pub mod pallet_lottery;
pub mod pallet_manta_pay;
pub mod pallet_manta_sbt;
pub mod pallet_membership;
pub mod pallet_multisig;
pub mod pallet_name_service;
pub mod pallet_parachain_staking;
pub mod pallet_preimage;
pub mod pallet_randomness;
pub mod pallet_scheduler;
pub mod pallet_session;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_tx_pause;
pub mod pallet_utility;
pub mod xcm;
pub mod zenlink_protocol;
