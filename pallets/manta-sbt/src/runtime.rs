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

//! MantaPay Runtime APIs

use manta_support::manta_pay::{PullResponse, RawCheckpoint};

sp_api::decl_runtime_apis! {
    pub trait SBTPullLedgerDiffApi {
        fn sbt_pull_ledger_diff(checkpoint: RawCheckpoint, max_receivers: u64, max_senders: u64) -> PullResponse;
        fn sbt_pull_ledger_total_count() -> [u8; 16];
    }
}
