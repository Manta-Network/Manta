#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]
use codec::{Decode, Encode};
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	MultiSignature, RuntimeDebug,
};

use sp_std::convert::Into; // TODO: add TryFrom and TryInto

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub mod constants;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

// Moment
pub type Moment = u64;

// now we only have MA
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	MA = 0,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn token_symbol_should_work() {
		let native_token = TokenSymbol::MA;

		assert_eq!(native_token as u8, 0);
	}
}
