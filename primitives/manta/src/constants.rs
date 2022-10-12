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

use crate::types::Balance;
use frame_support::PalletId;

// Calamari constants
pub const CALAMARI_SS58PREFIX: u8 = 78;
pub const CALAMARI_DECIMAL: u8 = 12;
pub const CALAMARI_TOKEN_SYMBOL: &str = "KMA";

// Manta constants
pub const MANTA_SS58PREFIX: u8 = 77;
pub const MANTA_DECIMAL: u8 = 18;
pub const MANTA_TOKEN_SYMBOL: &str = "MANTA";

// Dolphin constants
pub const DOLPHIN_DECIMAL: u8 = 18;
pub const DOLPHIN_TOKEN_SYMBOL: &str = "DOL";

/// Manta parachain time-related
pub mod time {
    use crate::types::{BlockNumber, Moment};
    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
    /// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
    /// slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const SECONDS_PER_BLOCK: Moment = 12;
    pub const MILLISECS_PER_BLOCK: Moment = SECONDS_PER_BLOCK * 1000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

pub const ASSET_STRING_LIMIT: u32 = 50;

// Identifiers of pallets
pub const STAKING_PALLET_ID: PalletId = PalletId(*b"PotStake");
pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");
pub const ASSET_MANAGER_PALLET_ID: PalletId = PalletId(*b"asstmngr");
pub const MANTA_PAY_PALLET_ID: PalletId = PalletId(*b"mantapay");

/// Default Asset Existential Deposit: Should only be used in TEST
pub const DEFAULT_ASSET_ED: Balance = 1;
