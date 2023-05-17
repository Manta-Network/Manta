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
use sp_runtime::PerThing;

pub const NORMAL_COLLATOR_MINIMUM_STAKE: Balance = 400_000 * MANTA;
pub const EARLY_COLLATOR_MINIMUM_STAKE: Balance = 40_000 * MANTA;
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
        min: Perbill::from_rational_with_rounding(5u32, 200u32, sp_arithmetic::Rounding::Down)
            .expect("constant denom is not 0. qed"), // = 2.5%
        ideal: Perbill::from_percent(3),
        max: Perbill::from_percent(3),
    };
    InflationInfo::<BalanceOf<T>> {
        // staking expectations **per round**
        // TOTAL_ISSUANCE: 1_000_000_000 * MANTA
        // ideal = TOTAL_ISSUANCE * annual.ideal (3%) / rounds_per_year (2629800 / (6 * 300))
        //       = 30_000_000 * MANTA / 1461 ~ 20_534 * MANTA
        expect: Range {
            min: (17_110 * MANTA).unique_saturated_into(),
            ideal: (20_534 * MANTA).unique_saturated_into(), // annual inflation / number of rounds
            max: (21_000 * MANTA).unique_saturated_into(),
        },
        // annual inflation
        annual,
        round: to_round_inflation(annual),
    }
}
