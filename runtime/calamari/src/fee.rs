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

use frame_support::weights::{
	WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use manta_primitives::Balance;
use smallvec::smallvec;
pub use sp_runtime::Perbill;

/// The block saturation level. Fees will be updates based on this value.
pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// Consider the daily cost to fully congest our network to be defined as:
		// daily_cost_to_fully_congest = inclusion_fee * txs_per_block * blocks_per_day * kma_price
		// The weight fee is defined as:
		// weight_fee = coeff_integer * (weight ^ degree) + coeff_friction * (weight ^ degree)
		// The inclusion fee is defined as:
		// inclusion_fee = base_fee + length_fee + [targeted_fee_adjustment * weight_fee]
		// As of the day of writing this code a single `balances.transfer` is benchmarked at 156626000 weight.
		// Let's assume worst case scenarios where the `length_fee` of a transfer is negligible,
		// and that `targeted_fee_adjustment` is simply 1, as if the network is not congested.
		// Furthermore we know the `base_fee` is 0.000125KMA defined in our runtime. So:
		// inclusion_fee = 0.000125 * coeff + 0.000156626 * coeff = 0.000281626 * coeff
		// We have profiled `txs_per_block` to be around 1134 and `blocks_per_day` is known to be 7200.
		// KMA price in dollars can be checked daily but at the time of writing the code it was $0.02. So:
		// daily_cost_to_fully_congest = 0.000281626 * coeff * 1134 * 7200 * 0.02 = 45.988399296 * coeff
		// Assuming we want the daily cost to be around $250000 we get:
		// 250000 = 45.988399296 * coeff
		// coeff = ~5436

		// Keep in mind this is a rough worst-case scenario calculation.
		// The `length_fee` could not be negligible, and the `targeted_fee_adjustment` will hike the fees
		// as the network gets more and more congested, which will further increase the costs.
		smallvec![WeightToFeeCoefficient {
			coeff_integer: 5000u32.into(),
			coeff_frac: Perbill::zero(),
			negative: false,
			degree: 1,
		}]
	}
}
