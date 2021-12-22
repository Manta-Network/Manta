use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
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

	// Relaychain Token
	KSM,
	DOT,
}

impl Default for TokenSymbol {
	fn default() -> Self {
		Self::MA
	}
}

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
}

impl Default for CurrencyId {
	fn default() -> Self {
		Self::Token(TokenSymbol::MA)
	}
}

#[allow(dead_code)]
impl CurrencyId {
	fn is_native(&self) -> bool {
		matches!(
			*self,
			Self::Token(TokenSymbol::KMA) | Self::Token(TokenSymbol::MA)
		)
	}

	fn is_parachain(&self) -> bool {
		matches!(
			*self,
			Self::Token(TokenSymbol::ACA)
				| Self::Token(TokenSymbol::KAR)
				| Self::Token(TokenSymbol::SDN)
		)
	}

	fn is_relaychain(&self) -> bool {
		matches!(
			*self,
			Self::Token(TokenSymbol::KSM) | Self::Token(TokenSymbol::DOT)
		)
	}
}
