pub const CALAMARI_SS58PREFIX: u8 = 78;
pub const MANTAPC_SS58PREFIX: u8 = 77;
pub const MANTA_DECIMAL: u8 = 12;
pub const CALAMARI_TOKEN_SYMBOL: &str = "KMA";
pub const MANTA_TOKEN_SYMBOL: &str = "MA";

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
