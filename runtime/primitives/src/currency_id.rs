use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	// Native token
	MA = 5,
	KMA = 1,
	// Acala tokens
	ACA = 0,
	KAR = 128,
	// Shiden
	SDN = 50,
}

impl Default for TokenSymbol {
	fn default() -> Self {
		Self::MA
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
}

impl Default for CurrencyId {
	fn default() -> Self {
		Self::Token(TokenSymbol::MA)
	}
}
