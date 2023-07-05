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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use manta_primitives::types::Balance;
use sp_api::decl_runtime_apis;
use sp_std::vec::Vec;

decl_runtime_apis! {
    pub trait FarmingRuntimeApi<AccountId, CurrencyId, PoolId> where
        AccountId: Codec,
        PoolId: Codec,
        CurrencyId: Codec,
    {
        fn get_farming_rewards(
            who: AccountId,
            pid: PoolId,
        ) -> Vec<(CurrencyId, Balance)>;

        fn get_gauge_rewards(
            who: AccountId,
            pid: PoolId,
        ) -> Vec<(CurrencyId, Balance)>;
    }
}
