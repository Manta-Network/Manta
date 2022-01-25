//! The XCM primitive trait implementations

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	traits::{tokens::fungibles::Mutate, Get, OriginTrait},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use sp_runtime::traits::Zero;
use sp_std::borrow::Borrow;
use sp_std::{convert::TryInto, marker::PhantomData};
use xcm::latest::{
	AssetId as xcmAssetId, Error as XcmError, Fungibility,
	Junction::{AccountId32, Parachain},
	Junctions::*,
	MultiAsset, MultiLocation, NetworkId,
};
use xcm_builder::TakeRevenue;
use xcm_executor::traits::{FilterAssetLocation, MatchesFungibles, WeightTrader};

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID
/// (must be `TryFrom/TryInto<u128>`) into a MultiLocation Value and Viceversa through
/// an intermediate generic type AssetType.
/// The trait bounds enforce is that the AssetTypeGetter trait is also implemented for
/// AssetIdInfoGetter
pub struct AsAssetType<AssetId, AssetType, AssetIdInfoGetter>(
	PhantomData<(AssetId, AssetType, AssetIdInfoGetter)>,
);
impl<AssetId, AssetType, AssetIdInfoGetter> xcm_executor::traits::Convert<MultiLocation, AssetId>
	for AsAssetType<AssetId, AssetType, AssetIdInfoGetter>
where
	AssetId: From<AssetType> + Clone,
	AssetType: From<MultiLocation> + Into<Option<MultiLocation>> + Clone,
	AssetIdInfoGetter: AssetTypeGetter<AssetId, AssetType>,
{
	fn convert_ref(id: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
		let asset_type: AssetType = id.borrow().clone().into();
		Ok(AssetId::from(asset_type))
	}
	fn reverse_ref(what: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
		if let Some(asset_type) = AssetIdInfoGetter::get_asset_type(what.borrow().clone()) {
			log::info!(
				"\n Calling `reverse_ref()` 1 asset_type is: {:?} \n",
				asset_type.clone().into()
			);
			if let Some(location) = asset_type.into() {
				log::info!(
					"\n Calling `reverse_ref()` 2 location is: {:?} \n",
					location.clone()
				);
				Ok(location)
			} else {
				log::info!("\n Calling `reverse_ref()` 3 \n");
				Err(())
			}
		} else {
			Err(())
		}
	}
}

pub struct AccountIdToMultiLocation<AccountId>(sp_std::marker::PhantomData<AccountId>);
impl<AccountId> sp_runtime::traits::Convert<AccountId, MultiLocation>
	for AccountIdToMultiLocation<AccountId>
where
	AccountId: Into<[u8; 32]>,
{
	fn convert(account: AccountId) -> MultiLocation {
		let res = MultiLocation {
			parents: 0,
			interior: X1(AccountId32 {
				network: NetworkId::Any,
				id: account.into(),
			}),
		};
		log::info!(
			"\n Calling `impl<AccountId> sp_runtime::traits::Convert<AccountId, MultiLocation> for AccountIdToMultiLocation<AccountId> convert()` MultiLocation res is: {:?} \n",
			res
		);
		res
	}
}

// Convert a local Origin (i.e., a signed 20 byte account Origin)  to a Multilocation
pub struct SignedToAccountId32<Origin, AccountId, Network>(
	sp_std::marker::PhantomData<(Origin, AccountId, Network)>,
);
impl<Origin: OriginTrait + Clone, AccountId: Into<[u8; 32]>, Network: Get<NetworkId>>
	xcm_executor::traits::Convert<Origin, MultiLocation>
	for SignedToAccountId32<Origin, AccountId, Network>
where
	Origin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
		+ TryInto<frame_system::RawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
	fn convert(o: Origin) -> Result<MultiLocation, Origin> {
		o.try_with_caller(|caller| match caller.try_into() {
			Ok(frame_system::RawOrigin::Signed(who)) => Ok(AccountId32 {
				network: Network::get(),
				id: who.into(),
			}
			.into()),
			Ok(other) => Err(other.into()),
			Err(other) => Err(other),
		})
	}
}

// We need to know how to charge for incoming assets
// This takes the first fungible asset, and takes whatever UnitPerSecondGetter establishes
// UnitsToWeightRatio trait, which needs to be implemented by AssetIdInfoGetter
pub struct FirstAssetTrader<
	AssetId: From<AssetType> + Clone,
	AssetType: From<MultiLocation> + Clone,
	AssetIdInfoGetter: UnitsToWeightRatio<AssetId>,
	R: TakeRevenue,
>(
	Weight,
	Option<(MultiLocation, u128, u128)>,
	PhantomData<(AssetId, AssetType, AssetIdInfoGetter, R)>,
);
impl<
		AssetId: From<AssetType> + Clone,
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetId>,
		R: TakeRevenue,
	> WeightTrader for FirstAssetTrader<AssetId, AssetType, AssetIdInfoGetter, R>
{
	fn new() -> Self {
		FirstAssetTrader(0, None, PhantomData)
	}
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::Assets,
	) -> Result<xcm_executor::Assets, XcmError> {
		log::info!("\n Calling `FirstAssetTrader buy_weight()` \n");
		let first_asset = payment
			.clone()
			.fungible_assets_iter()
			.next()
			.ok_or(XcmError::TooExpensive)?;
		log::info!(
			"\n Calling `FirstAssetTrader buy_weight()` payment is : {:?} \n",
			payment.clone()
		);
		log::info!(
			"\n Calling `FirstAssetTrader buy_weight()` first_asset is : {:?} \n",
			first_asset.clone()
		);

		// We are only going to check first asset for now. This should be sufficient for simple token
		// transfers. We will see later if we change this.
		match (first_asset.id, first_asset.fun) {
			(xcmAssetId::Concrete(id), Fungibility::Fungible(_)) => {
				let asset_type: AssetType = id.clone().into();
				log::info!(
					"\n Calling `FirstAssetTrader buy_weight()` first match arm: `id` is: {:?} \n",
					id.clone()
				);
				let asset_id: AssetId = AssetId::from(asset_type);

				if let Some(units_per_second) = AssetIdInfoGetter::get_units_per_second(asset_id) {
					let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
					let required = MultiAsset {
						fun: Fungibility::Fungible(amount),
						id: xcmAssetId::Concrete(id.clone()),
					};
					log::info!(
						"\n Calling `FirstAssetTrader buy_weight()` amount is : {:?} and required is : {:?} \n",
						amount.clone(), required.clone()
					);
					let unused = payment
						.checked_sub(required)
						.map_err(|_| XcmError::TooExpensive)?;
					self.0 = self.0.saturating_add(weight);
					log::info!(
						"\n Calling `FirstAssetTrader buy_weight()` unused is : {:?} \n",
						unused.clone()
					);
					// In case the asset matches the one the trader already stored before, add
					// to later refund

					// Else we are always going to substract the weight if we can, but we latter do
					// not refund it

					// In short, we only refund on the asset the trader first succesfully was able
					// to pay for an execution
					let new_asset = match self.1.clone() {
						Some((prev_id, prev_amount, units_per_second)) => {
							if prev_id == id.clone() {
								log::info!(
									"\n Calling `FirstAssetTrader buy_weight()` prev_id is {:?} and id is {:?} equal \n",
									prev_id.clone(), id.clone()
								);
								Some((id, prev_amount.saturating_add(amount), units_per_second))
							} else {
								log::info!(
									"\n Calling `FirstAssetTrader buy_weight()` prev_id is {:?} and id is {:?} not equal \n",
									prev_id.clone(), id.clone()
								);
								None
							}
						}
						None => {
							log::info!(
								"\n Calling `FirstAssetTrader buy_weight()` id is {:?} and amount is {:?} and units_per_second is {:?} \n",
								id.clone(), amount.clone(), units_per_second.clone()
							);
							Some((id, amount, units_per_second))
						}
					};

					// Due to the trait bound, we can only refund one asset.
					if let Some(new_asset) = new_asset {
						log::info!(
							"\n Calling `FirstAssetTrader buy_weight()` Due to the trait bound, we can only refund one asset.` \n",
						);
						self.0 = self.0.saturating_add(weight);
						self.1 = Some(new_asset);
					};
					return Ok(unused);
				} else {
					log::info!("\n Calling `FirstAssetTrader buy_weight()` TooExpensive 1 \n",);
					return Err(XcmError::TooExpensive);
				};
			}
			_ => {
				log::info!("\n Calling `FirstAssetTrader buy_weight()` TooExpensive 2 \n",);
				return Err(XcmError::TooExpensive);
			}
		}
	}

	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		log::info!("\n Calling `FirstAssetTrader refund_weight()` \n");
		if let Some((id, prev_amount, units_per_second)) = self.1.clone() {
			let weight = weight.min(self.0);
			self.0 -= weight;
			let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
			self.1 = Some((
				id.clone(),
				prev_amount.saturating_sub(amount),
				units_per_second,
			));
			Some(MultiAsset {
				fun: Fungibility::Fungible(amount),
				id: xcmAssetId::Concrete(id.clone()),
			})
		} else {
			None
		}
	}
}

/// Deal with spent fees, deposit them as dictated by R
impl<
		AssetId: From<AssetType> + Clone,
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetId>,
		R: TakeRevenue,
	> Drop for FirstAssetTrader<AssetId, AssetType, AssetIdInfoGetter, R>
{
	fn drop(&mut self) {
		log::info!("\n Calling `FirstAssetTrader Drop()` \n");
		if let Some((id, amount, _)) = self.1.clone() {
			R::take_revenue((id, amount).into());
		}
	}
}

pub trait Reserve {
	/// Returns assets reserve location.
	fn reserve(&self) -> Option<MultiLocation>;
}

// Takes the chain part of a MultiAsset
impl Reserve for MultiAsset {
	fn reserve(&self) -> Option<MultiLocation> {
		log::info!("\n Calling `impl Reserve for MultiAsset reserve()` \n");
		if let xcmAssetId::Concrete(location) = self.id.clone() {
			let first_interior = location.first_interior();
			let parents = location.parent_count();
			match (parents, first_interior.clone()) {
				(0, Some(Parachain(id))) => Some(MultiLocation::new(0, X1(Parachain(id.clone())))),
				(1, Some(Parachain(id))) => Some(MultiLocation::new(1, X1(Parachain(id.clone())))),
				(1, _) => Some(MultiLocation::parent()),
				_ => None,
			}
		} else {
			None
		}
	}
}

/// A `FilterAssetLocation` implementation. Filters multi native assets whose
/// reserve is same with `origin`.
pub struct MultiNativeAsset;
impl FilterAssetLocation for MultiNativeAsset {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::info!(
			"\n Calling `impl FilterAssetLocation for MultiNativeAsset filter_asset_location()` \n"
		);
		if let Some(ref reserve) = asset.reserve() {
			if reserve == origin {
				return true;
			}
		}
		false
	}
}

// Defines the trait to obtain a generic AssetType from a generic AssetId
pub trait AssetTypeGetter<AssetId, AssetType> {
	// Get units per second from asset type
	fn get_asset_type(asset_id: AssetId) -> Option<AssetType>;
}

// Defines the trait to obtain the units per second of a give assetId for local execution
// This parameter will be used to charge for fees upon assetId deposit
pub trait UnitsToWeightRatio<AssetId> {
	// Get units per second from asset type
	fn get_units_per_second(asset_id: AssetId) -> Option<u128>;
}

/// XCM fee depositor to which we implement the TakeRevenue trait
/// It receives a fungibles::Mutate implemented argument, a matcher to convert MultiAsset into
/// AssetId and amount, and the fee receiver account
pub struct XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>(
	PhantomData<(Assets, Matcher, AccountId, ReceiverAccount)>,
);
impl<
		Assets: Mutate<AccountId>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		AccountId: Clone,
		ReceiverAccount: Get<AccountId>,
	> TakeRevenue for XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>
{
	fn take_revenue(revenue: MultiAsset) {
		log::info!("\n Calling `impl TakeRevenue for XcmFeesToAccount take_revenue()` \n");
		match Matcher::matches_fungibles(&revenue) {
			Ok((asset_id, amount)) => {
				if !amount.is_zero() {
					let ok = Assets::mint_into(asset_id, &ReceiverAccount::get(), amount).is_ok();
					debug_assert!(ok, "`mint_into` cannot generally fail; qed");
				}
			}
			Err(_) => log::debug!(
				target: "xcm",
				"take revenue failed matching fungible"
			),
		}
	}
}
