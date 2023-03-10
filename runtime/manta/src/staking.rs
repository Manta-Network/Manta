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

use crate::{currency::MANTA, Balance};
use pallet_parachain_staking::{BalanceOf, InflationInfo};

pub const NORMAL_COLLATOR_MINIMUM_STAKE: Balance = 50 * MANTA;
pub const EARLY_COLLATOR_MINIMUM_STAKE: Balance = 50 * MANTA;
pub const MIN_BOND_TO_BE_CONSIDERED_COLLATOR: Balance = NORMAL_COLLATOR_MINIMUM_STAKE;

pub fn inflation_config<T: frame_system::Config + pallet_parachain_staking::Config>(
) -> InflationInfo<BalanceOf<T>> {
    use pallet_parachain_staking::inflation::Range;
    use sp_arithmetic::Perbill;
    use sp_runtime::traits::UniqueSaturatedInto;

    fn to_round_inflation(annual: Range<Perbill>) -> Range<Perbill> {
        use pallet_parachain_staking::inflation::{
            perbill_annual_to_perbill_round, BLOCKS_PER_YEAR,
        };
        perbill_annual_to_perbill_round(
            annual,
            // rounds per year
            BLOCKS_PER_YEAR / crate::DefaultBlocksPerRound::get(),
        )
    }
    let annual = Range {
        min: Perbill::zero(),
        ideal: Perbill::zero(),
        max: Perbill::zero(),
    };
    InflationInfo::<BalanceOf<T>> {
        // staking expectations **per round**
        expect: Range {
            // TODO: Correct this for manta numbers
            min: 0u128.unique_saturated_into(),
            ideal: 0u128.unique_saturated_into(), // annual inflation / number of rounds
            max: 0u128.unique_saturated_into(),
        },
        // annual inflation
        annual,
        round: to_round_inflation(annual),
    }
}
