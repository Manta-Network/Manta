#[allow(non_upper_case_globals)]

// Money matters.
pub mod currency {
	pub type Balance = u128;
	pub const MA: Balance = 1_000_000_000_000; // 12 decimal
	pub const cMA: Balance = MA / 100; // 10 decimal, cent-MA
	pub const mMA: Balance = MA / 1_000; // 9 decimal, milli-MA
	pub const uMA: Balance = MA / 1_000_000; // 6 decimal, micro-MA

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * mMA + (bytes as Balance) * 6 * mMA // TODO: revisit the storage cost here
	}
}

pub mod time {
	use crate::{BlockNumber, Moment};

	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 6000;
	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
	pub const EPOCH_DURATION_IN_SLOTS: u32 = {
		const SLOT_FILL_RATE: f32 = MILLISECS_PER_BLOCK as f32 / SLOT_DURATION as f32;
		(EPOCH_DURATION_IN_BLOCKS as f32 * SLOT_FILL_RATE) as u32
	};

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

#[cfg(test)]
mod tests {
	use super::currency::{cMA, mMA, MA};
	use frame_support::weights::WeightToFeePolynomial;
	use manta_runtime::{ExtrinsicBaseWeight, IdentityFee, MAXIMUM_BLOCK_WEIGHT};

	#[test]
	// This function tests that the fee for `MAXIMUM_BLOCK_WEIGHT` of weight is correct
	fn full_block_fee_is_correct() {
		// A full block should cost 200 CENTS
		println!("Base: {}", ExtrinsicBaseWeight::get());
		// The polynormial is: f(x) = (coeff_integer + coeff_frac) * (negative * 1) * x^degree
		// But the polynormial for calculating is: f(x) = x.
		// checkout the code: https://github.com/paritytech/substrate/blob/v3.0.0/frame/support/src/weights.rs#L725
		// Follow this method to calculate fee: https://github.com/paritytech/substrate/blob/v3.0.0/frame/support/src/weights.rs#L695
		// so x = MAXIMUM_BLOCK_WEIGHT.
		let x: u128 = IdentityFee::calc(&MAXIMUM_BLOCK_WEIGHT);
		assert_eq!(x, 2 * MA);
	}

	#[test]
	// This function tests that the fee for `ExtrinsicBaseWeight` of weight is correct
	fn extrinsic_base_fee_is_correct() {
		// `ExtrinsicBaseWeight` should cost 1/10 of a CENT
		println!("Base: {}", ExtrinsicBaseWeight::get());
		let x: u128 = IdentityFee::calc(&ExtrinsicBaseWeight::get());
		let y = cMA / 10;
		assert!(x.max(y) - x.min(y) < mMA);
	}
}
