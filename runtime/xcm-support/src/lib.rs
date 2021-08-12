#![forbid(clippy::unwrap_used)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use core::{convert::TryFrom, marker::PhantomData};
use frame_support::traits::{Currency, ExistenceRequirement, Get, WithdrawReasons};
use manta_primitives::currency_id::{CurrencyId as MantaCurrencyId, TokenSymbol};
use sp_runtime::traits::{MaybeSerializeDeserialize, StaticLookup};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	vec::Vec,
};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult};
use xcm_executor::traits::{Convert, FilterAssetLocation, TransactAsset};

// A Handler for withdrawing/depositting relaychain/parachain tokens.
pub struct MantaTransactorAdaptor<
	NativeCurrency,
	XCurrency,
	AccountIdConverter,
	AccountId,
	CurrencyId,
	LocationMapCurrencyId,
>(
	PhantomData<(
		NativeCurrency,
		XCurrency,
		AccountIdConverter,
		AccountId,
		CurrencyId,
		LocationMapCurrencyId,
	)>,
);
impl<
		NativeCurrency: Currency<AccountId>,
		XCurrency: manta_primitives::traits::XCurrency<
			AccountId,
			Balance = NativeCurrency::Balance,
			CurrencyId = MantaCurrencyId,
		>,
		AccountIdConverter: Convert<MultiLocation, AccountId>,
		AccountId: sp_std::fmt::Debug + Clone,
		CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
		LocationMapCurrencyId: StaticLookup<Source = MultiLocation, Target = MantaCurrencyId>,
	> TransactAsset
	for MantaTransactorAdaptor<
		NativeCurrency,
		XCurrency,
		AccountIdConverter,
		AccountId,
		CurrencyId,
		LocationMapCurrencyId,
	>
{
	fn can_check_in(_origin: &MultiLocation, _what: &MultiAsset) -> XcmResult {
		Ok(())
	}

	fn deposit_asset(asset: &MultiAsset, who: &MultiLocation) -> XcmResult {
		log::info!(target: "manta-xassets", "deposit_asset: asset = {:?}, who = {:?}", asset, who);

		let who = AccountIdConverter::convert_ref(who).map_err(|_| {
			XcmError::FailedToTransactAsset("Failed to convert multilocation to account id")
		})?;

		match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				let currency_id = LocationMapCurrencyId::lookup(id.clone()).map_err(|_| {
					XcmError::FailedToTransactAsset("Now we didn't support this multiLocation")
				})?;
				let amount =
					NativeCurrency::Balance::try_from(*amount).map_err(|_| XcmError::Overflow)?;

				match currency_id {
					MantaCurrencyId::Token(TokenSymbol::MA)
					| MantaCurrencyId::Token(TokenSymbol::KMA) => {
						NativeCurrency::deposit_creating(&who, amount);
					}
					MantaCurrencyId::Token(TokenSymbol::ACA)
					| MantaCurrencyId::Token(TokenSymbol::KAR)
					| MantaCurrencyId::Token(TokenSymbol::SDN)
					| MantaCurrencyId::Token(TokenSymbol::KSM) => {
						XCurrency::deposit(currency_id, &who, amount)
							.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
					}
					_ => {
						log::info!(target: "manta-xassets", "Failed to deposit Unknow asset.");
					}
				}

				Ok(())
			}
			_ => Err(XcmError::FailedToTransactAsset(
				"We don't support this multi-asset now",
			)),
		}
	}

	fn withdraw_asset(
		asset: &MultiAsset,
		who: &MultiLocation,
	) -> Result<xcm_executor::Assets, XcmError> {
		log::info!(target: "manta-xassets", "withdraw_asset: asset = {:?}, who = {:?}", asset, who);

		let who = AccountIdConverter::convert_ref(who).map_err(|_| {
			XcmError::FailedToTransactAsset("Failed to convert multilocation to account id")
		})?;

		match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				let amount =
					NativeCurrency::Balance::try_from(*amount).map_err(|_| XcmError::Overflow)?;
				let currency_id = LocationMapCurrencyId::lookup(id.clone()).map_err(|_| {
					XcmError::FailedToTransactAsset("Now we didn't support this multiLocation")
				})?;

				match currency_id {
					MantaCurrencyId::Token(TokenSymbol::MA)
					| MantaCurrencyId::Token(TokenSymbol::KMA) => {
						NativeCurrency::withdraw(
							&who,
							amount,
							WithdrawReasons::TRANSFER,
							ExistenceRequirement::AllowDeath,
						)
						.map_err(|e| {
							log::info!(target: "manta-xassets", "withdraw_asset: error = {:?}", e);
							XcmError::FailedToTransactAsset(e.into())
						})?;
					}
					MantaCurrencyId::Token(TokenSymbol::ACA)
					| MantaCurrencyId::Token(TokenSymbol::KAR)
					| MantaCurrencyId::Token(TokenSymbol::SDN)
					| MantaCurrencyId::Token(TokenSymbol::KSM) => {
						XCurrency::withdraw(currency_id, &who, amount)
							.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
					}
					_ => {
						log::info!(target: "manta-xassets", "Failed to deposit Unknow asset.");
					}
				}

				Ok(asset.clone().into())
			}
			_ => Err(XcmError::NotWithdrawable),
		}
	}
}

/// Which parachain or asset we support
pub struct TrustedParachains<Chains>(PhantomData<Chains>);
impl<Chains: Get<Vec<(MultiLocation, u128)>>> FilterAssetLocation for TrustedParachains<Chains> {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::info!(target: "manta-xassets", "filter_asset_location: origin = {:?}, asset = {:?}", origin, asset);

		Chains::get()
			.iter()
			.map(|(location, _)| location)
			.any(|location| *location == *origin)
	}
}

/// Lookup table for finding currency id by multilocation.
pub struct MultiLocationToCurrencyId<T>(PhantomData<T>);
impl<MultiLocationMapCurrencyId: Get<Vec<(MultiLocation, MantaCurrencyId)>>> StaticLookup
	for MultiLocationToCurrencyId<MultiLocationMapCurrencyId>
{
	type Source = MultiLocation;
	type Target = MantaCurrencyId;

	fn lookup(s: Self::Source) -> Result<Self::Target, frame_support::error::LookupError> {
		let get_all = MultiLocationMapCurrencyId::get();
		for i in get_all.iter() {
			if s == i.0 {
				return Ok(i.1);
			}
		}

		Err(frame_support::error::LookupError)
	}

	fn unlookup(t: Self::Target) -> Self::Source {
		let get_all = MultiLocationMapCurrencyId::get();
		for i in get_all.iter() {
			if t == i.1 {
				return i.0.clone();
			}
		}

		// This means we don't find multilocation by currency id.
		MultiLocation::Null
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::parameter_types;
	use xcm::v0::Junction;

	parameter_types! {
		pub MultiLocationMapCurrencyId: Vec<(MultiLocation, MantaCurrencyId)> = vec![
			// Acala karura => KAR, native token
			(MultiLocation::X3(Junction::Parent, Junction::Parachain(2000), Junction::GeneralKey([0, 128].to_vec())), MantaCurrencyId::Token(TokenSymbol::KAR)),
			// Manta manta-pc => MA, native token, for example, acala can send it back to manta parachain.
			(MultiLocation::X3(Junction::Parent, Junction::Parachain(2000), Junction::GeneralKey([0, 5].to_vec())), MantaCurrencyId::Token(TokenSymbol::MA)),
			// Manta calamari => KMA, native token, for example, karura can send it back to manta parachain.
			(MultiLocation::X3(Junction::Parent, Junction::Parachain(2000), Junction::GeneralKey([0, 133].to_vec())), MantaCurrencyId::Token(TokenSymbol::KMA)),
		];
		pub TrustedChains: Vec<(MultiLocation, u128)> = vec![
			// Acala local and live, 0.01 ACA
			(MultiLocation::X2(Junction::Parent, Junction::Parachain(2000)), 10_000_000_000),
			(MultiLocation::X2(Junction::Parent, Junction::Parachain(2084)), 10_000_000_000),
		];
	}

	#[test]
	fn find_currency_id_by_multilocation_should_work() {
		let karura = MultiLocation::X3(
			Junction::Parent,
			Junction::Parachain(2000),
			Junction::GeneralKey([0, 128].to_vec()),
		);
		let currency_id = MultiLocationToCurrencyId::<MultiLocationMapCurrencyId>::lookup(karura);
		assert!(currency_id.is_ok());
		assert_eq!(
			currency_id.unwrap(),
			MantaCurrencyId::Token(TokenSymbol::KAR)
		);

		let unknown = MultiLocation::X3(
			Junction::Parent,
			Junction::Parachain(1999),
			Junction::GeneralKey([0, 1].to_vec()),
		);
		let currency_id = MultiLocationToCurrencyId::<MultiLocationMapCurrencyId>::lookup(unknown);
		assert!(currency_id.is_err());

		let kar = MantaCurrencyId::Token(TokenSymbol::KAR);
		let location = MultiLocationToCurrencyId::<MultiLocationMapCurrencyId>::unlookup(kar);
		assert_eq!(
			location,
			MultiLocation::X3(
				Junction::Parent,
				Junction::Parachain(2000),
				Junction::GeneralKey([0, 128].to_vec())
			)
		);

		let shiden = MantaCurrencyId::Token(TokenSymbol::SDN);
		let location = MultiLocationToCurrencyId::<MultiLocationMapCurrencyId>::unlookup(shiden);
		assert_eq!(location, MultiLocation::Null);
	}

	#[test]
	fn filter_asset_location_should_work() {
		let karura = MultiLocation::X2(Junction::Parent, Junction::Parachain(2000));

		let found =
			TrustedParachains::<TrustedChains>::filter_asset_location(&MultiAsset::None, &karura);
		assert!(found);

		let unknown = MultiLocation::X2(Junction::Parent, Junction::Parachain(1999));
		let not_found =
			TrustedParachains::<TrustedChains>::filter_asset_location(&MultiAsset::None, &unknown);
		assert!(!not_found);
	}
}
