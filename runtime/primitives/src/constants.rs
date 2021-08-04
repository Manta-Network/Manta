pub const CALAMARI_SS58PREFIX: u8 = 78;
pub const MANTAPC_SS58PREFIX: u8 = 77;
pub const MANTA_DECIMAL: u8 = 12;
pub const CALAMARI_TOKEN_SYMBOL: &str = "KMA";
pub const MANTA_TOKEN_SYMBOL: &str = "MA";

// Money matters.
pub mod currency {
	use crate::Balance;

	pub const MA: Balance = 1_000_000_000_000; // 12 decimal
	pub const cMA: Balance = MA / 100; // 10 decimal, cent-MA
	pub const mMA: Balance = MA / 1_000; // 9 decimal, milli-MA
	pub const uMA: Balance = MA / 1_000_000; // 6 decimal, micro-MA

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * mMA + (bytes as Balance) * 6 * mMA // TODO: revisit the storage cost here
	}
}

/// Manta parachain time-related
pub mod time {
	use crate::{BlockNumber, Moment};
	/// This determines the average expected block time that we are targeting. Blocks will be
	/// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
	/// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
	/// slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: Moment = 12_000; // 12s
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

/// Fee-related.
pub mod fee {
	use crate::Balance;
	use frame_support::weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	};
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
			let p = super::currency::cMA;
			let q = 100 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}
