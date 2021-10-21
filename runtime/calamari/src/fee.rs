use crate::currency;
use frame_support::weights::{
	constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
	WeightToFeePolynomial,
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
		// in Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
		// in Manta Parachain, we map to 1/10 of that, or 1/100 CENT
		// TODO, revisit here to figure out why use this polynomial
		let p = currency::cKMA;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}
