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

//! XCM Primitives

use crate::{
    assets::{AssetConfig, AssetIdLocationGetter, FungibleLedger, UnitsToWeightRatio},
    types::{AssetId, Balance},
};
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::Get,
    traits::{fungibles::Mutate, tokens::ExistenceRequirement},
    weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use frame_system::Config;
use sp_runtime::traits::{CheckedConversion, Convert, Zero};
use xcm::{
    latest::{prelude::Concrete, Error},
    v1::{
        AssetId as XcmAssetId, Fungibility,
        Junction::{AccountId32, Parachain},
        Junctions::X1,
        MultiAsset, MultiLocation, NetworkId,
    },
};
use xcm_builder::TakeRevenue;
use xcm_executor::traits::{
    Convert as XcmConvert, FilterAssetLocation, MatchesFungible, MatchesFungibles, TransactAsset,
    WeightTrader,
};

/// XCM Result
pub type Result<T = (), E = Error> = core::result::Result<T, E>;

///
pub trait Reserve {
    /// Returns assets reserve location.
    fn reserve(&self) -> Option<MultiLocation>;
}

impl Reserve for MultiAsset {
    /// Takes the chain part of a MultiAsset.
    #[inline]
    fn reserve(&self) -> Option<MultiLocation> {
        // NOTE: We only care about concrete location now.
        if let XcmAssetId::Concrete(location) = &self.id {
            match (location.parent_count(), location.first_interior()) {
                (0, Some(Parachain(id))) => Some(MultiLocation::new(0, X1(Parachain(*id)))),
                (1, Some(Parachain(id))) => Some(MultiLocation::new(1, X1(Parachain(*id)))),
                (1, _) => Some(MultiLocation::parent()),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Multi-Native Asset Filter
///
/// Filters multi-native assets whose reserve is same as the `origin`.
pub struct MultiNativeAsset;

impl FilterAssetLocation for MultiNativeAsset {
    #[inline]
    fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
        asset.reserve().map(|r| r == *origin).unwrap_or(false)
    }
}

/// AccountId to [`MultiLocation`] Converter
pub struct AccountIdToMultiLocation;

impl<A> Convert<A, MultiLocation> for AccountIdToMultiLocation
where
    A: Into<[u8; 32]>,
{
    #[inline]
    fn convert(account: A) -> MultiLocation {
        MultiLocation {
            parents: 0,
            interior: X1(AccountId32 {
                network: NetworkId::Any,
                id: account.into(),
            }),
        }
    }
}

///
/// This trader defines how to charge a XCM call.
/// This takes the first fungible asset, and takes UnitPerSecondGetter that implements
/// UnitToWeightRatio trait.
pub struct FirstAssetTrader<AssetId, AssetLocation, AssetIdInfoGetter, R>
where
    R: TakeRevenue,
{
    ///
    weight: Weight,

    ///
    refund_cache: Option<(MultiLocation, u128, u128)>,

    ///
    __: PhantomData<(AssetId, AssetLocation, AssetIdInfoGetter, R)>,
}

impl<AssetId, AssetLocation, AssetIdInfoGetter, R> WeightTrader
    for FirstAssetTrader<AssetId, AssetLocation, AssetIdInfoGetter, R>
where
    AssetLocation: From<MultiLocation>,
    AssetIdInfoGetter: AssetIdLocationGetter<AssetLocation> + UnitsToWeightRatio,
    R: TakeRevenue,
{
    /// Builds a new default [`FirstAssetTrader`].
    #[inline]
    fn new() -> Self {
        Self {
            weight: Zero::zero(),
            refund_cache: None,
            __: PhantomData,
        }
    }

    /// Buys weight for XCM execution. We always return the [`TooExpensive`](Error::TooExpensive)
    /// error if this fails.
    #[inline]
    fn buy_weight(
        &mut self,
        weight: Weight,
        payment: xcm_executor::Assets,
    ) -> Result<xcm_executor::Assets> {
        log::debug!(
            target: "FirstAssetTrader::buy_weight",
            "weight: {:?}, payment: {:?}",
            weight,
            payment
        );

        let first_asset = payment.fungible_assets_iter().next().ok_or({
            log::debug!(
                target: "FirstAssetTrader::buy_weight",
                "no assets in payment: {:?}",
                payment,
            );
            Error::TooExpensive
        })?;

        // Check the first asset
        match (first_asset.id, first_asset.fun) {
            (XcmAssetId::Concrete(id), Fungibility::Fungible(_)) => {
                let asset_id = AssetIdInfoGetter::get_asset_id(&id.clone().into()).ok_or({
                    log::debug!(
                        target: "FirstAssetTrader::buy_weight",
                        "asset_id missing for asset location with id: {:?}",
                        id,
                    );
                    Error::TooExpensive
                })?;
                let units_per_second =
                    AssetIdInfoGetter::get_units_per_second(asset_id).ok_or({
                        log::debug!(
                            target: "FirstAssetTrader::buy_weight",
                            "units_per_second missing for asset with id: {:?}",
                            id,
                        );
                        Error::TooExpensive
                    })?;

                let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
                // we don't need to proceed if amount is zero.
                // This is very useful in tests.
                if amount.is_zero() {
                    return Ok(payment);
                }
                let required = MultiAsset {
                    fun: Fungibility::Fungible(amount),
                    id: XcmAssetId::Concrete(id.clone()),
                };

                log::debug!(
                    target: "FirstAssetTrader::buy_weight",
                    "payment: {:?}, required: {:?}",
                    payment,
                    required,
                );
                let unused = payment.checked_sub(required).map_err(|_| {
                    log::debug!(
                        target: "FirstAssetTrader::buy_weight",
                        "not enough required assets in payment",
                    );
                    Error::TooExpensive
                })?;
                self.weight = self.weight.saturating_add(weight);

                // In case the asset matches the one the trader already stored before, add
                // to later refund

                // Else we are always going to subtract the weight if we can, but we latter do
                // not refund it

                // In short, we only refund on the asset the trader first successfully was able
                // to pay for an execution
                let new_asset = match self.refund_cache.clone() {
                    Some((prev_id, prev_amount, units_per_second)) => {
                        if prev_id == id {
                            Some((id, prev_amount.saturating_add(amount), units_per_second))
                        } else {
                            None
                        }
                    }
                    None => Some((id, amount, units_per_second)),
                };

                // Due to the trait bound, we can only refund one asset.
                if let Some(new_asset) = new_asset {
                    self.weight = self.weight.saturating_add(weight);
                    self.refund_cache = Some(new_asset);
                };
                Ok(unused)
            }
            _ => {
                log::debug!(
                    target: "FirstAssetTrader::buy_weight",
                    "no matching XcmAssetId for first_asset in payment: {:?}",
                    payment,
                );
                Err(Error::TooExpensive)
            }
        }
    }

    ///
    #[inline]
    fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
        if let Some((id, prev_amount, units_per_second)) = self.refund_cache.clone() {
            let weight = weight.min(self.weight);
            self.weight -= weight;
            let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
            self.refund_cache = Some((
                id.clone(),
                prev_amount.saturating_sub(amount),
                units_per_second,
            ));
            Some(MultiAsset {
                fun: Fungibility::Fungible(amount),
                id: XcmAssetId::Concrete(id),
            })
        } else {
            None
        }
    }
}

impl<AssetId, AssetLocation, AssetIdInfoGetter, R> Drop
    for FirstAssetTrader<AssetId, AssetLocation, AssetIdInfoGetter, R>
where
    R: TakeRevenue,
{
    /// Handles spent fees, depositing them as defined by `R`.
    #[inline]
    fn drop(&mut self) {
        if let Some((id, amount, _)) = self.refund_cache.clone() {
            R::take_revenue((id, amount).into());
        }
    }
}

///
/// XCM fee depositor to which we implement the TakeRevenue trait
/// It receives a fungibles::Mutate implemented argument, a matcher to convert MultiAsset into
/// AssetId and amount, and the fee receiver account
pub struct XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>(
    PhantomData<(Assets, Matcher, AccountId, ReceiverAccount)>,
);

impl<Assets, Matcher, AccountId, ReceiverAccount> TakeRevenue
    for XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>
where
    Assets: Mutate<AccountId>,
    Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
    AccountId: Clone,
    ReceiverAccount: Get<AccountId>,
{
    #[inline]
    fn take_revenue(revenue: MultiAsset) {
        match Matcher::matches_fungibles(&revenue) {
            Ok((asset_id, amount)) => {
                if !amount.is_zero() {
                    if let Err(err) = Assets::mint_into(asset_id, &ReceiverAccount::get(), amount) {
                        log::debug!(target: "manta-xcm", "mint_into failed with {:?}", err);
                    }
                }
            }
            _ => log::debug!(target: "manta-xcm", "take revenue failed matching fungible"),
        }
    }
}

/// Manta's `MatchFungible` implementation.
/// It resolves the reanchoring logic as well, i.e. it recognize `here()` as
/// `../parachain(id)`.
/// `T` should specify a `SelfLocation` in the form of absolute path to the
/// relaychain.
pub struct IsNativeConcrete<T>(PhantomData<T>);

impl<T, Balance> MatchesFungible<Balance> for IsNativeConcrete<T>
where
    T: Get<MultiLocation>,
    Balance: TryFrom<u128>,
{
    #[inline]
    fn matches_fungible(asset: &MultiAsset) -> Option<Balance> {
        if let (Fungibility::Fungible(amount), Concrete(location)) = (&asset.fun, &asset.id) {
            if location == &T::get() || MultiLocation::is_here(location) {
                return CheckedConversion::checked_from(*amount);
            }
        }
        None
    }
}

///
pub struct MultiAssetAdapter<
    T,
    AccountIdConverter,
    NativeMatcher,
    NonNativeMatcher,
    MultiAdapterAssetConfig,
>(
    PhantomData<(
        T,
        NativeMatcher,
        AccountIdConverter,
        NonNativeMatcher,
        MultiAdapterAssetConfig,
    )>,
);

impl<T, AccountIdConverter, NativeMatcher, NonNativeMatcher, MultiAdapterAssetConfig> TransactAsset
    for MultiAssetAdapter<
        T,
        AccountIdConverter,
        NativeMatcher,
        NonNativeMatcher,
        MultiAdapterAssetConfig,
    >
where
    T: Config,
    AccountIdConverter: XcmConvert<MultiLocation, T::AccountId>,
    NativeMatcher: MatchesFungible<Balance>,
    NonNativeMatcher: MatchesFungibles<AssetId, Balance>,
    MultiAdapterAssetConfig: AssetConfig<T, AssetId = AssetId, Balance = Balance>,
{
    #[inline]
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
        log::debug!(
            target: "xcm::multi_asset_adapter",
            "deposit_asset asset: {:?}, location: {:?}",
            asset, location,
        );
        let (asset_id, amount, who) = Self::match_asset_and_location(asset, location)?;
        // NOTE: If it's non-native asset we want to check with increase in total supply. Otherwise
        //       it will just use false, as it is assumed the native asset supply cannot be changed.
        <MultiAdapterAssetConfig as AssetConfig<T>>::FungibleLedger::can_deposit(
            &asset_id, &who, &amount, true,
        )
        .map_err(|_| {
            Error::FailedToTransactAsset("Failed MultiAdapterFungibleLedger::can_deposit")
        })?;
        <MultiAdapterAssetConfig as AssetConfig<T>>::FungibleLedger::deposit_can_mint(
            &asset_id, &who, &amount,
        )
        .map_err(|_| {
            Error::FailedToTransactAsset("Failed MultiAdapterFungibleLedger::deposit_can_mint")
        })?;
        Ok(())
    }

    #[inline]
    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> Result<xcm_executor::Assets> {
        log::debug!(
            target: "xcm::multi_asset_adapter",
            "withdraw_asset asset: {:?}, location: {:?}",
            asset, location,
        );
        let (asset_id, amount, who) = Self::match_asset_and_location(asset, location)?;
        <MultiAdapterAssetConfig as AssetConfig<T>>::FungibleLedger::withdraw_can_burn(
            &asset_id,
            &who,
            &amount,
            ExistenceRequirement::AllowDeath,
        )
        .map_err(|_| Error::FailedToTransactAsset("Failed Burn"))?;
        Ok(asset.clone().into())
    }
}

impl<
        T: Config,
        AccountIdConverter: XcmConvert<MultiLocation, T::AccountId>,
        NativeMatcher: MatchesFungible<Balance>,
        NonNativeMatcher: MatchesFungibles<AssetId, Balance>,
        MultiAdapterAssetConfig: AssetConfig<T, AssetId = AssetId, Balance = Balance>,
    >
    MultiAssetAdapter<
        T,
        AccountIdConverter,
        NativeMatcher,
        NonNativeMatcher,
        MultiAdapterAssetConfig,
    >
{
    /// Matches the incoming `asset` to an `asset_id` and `amount` on this chain.
    /// Matches the incoming `location` to a `receiver` account on this chain.
    /// Uses the matcher implementation of both native and non-native assets.
    /// Returns the `asset_id`, `amount` and `receiver` if all three were matched.
    #[inline]
    fn match_asset_and_location(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> Result<(AssetId, Balance, T::AccountId)> {
        let receiver = AccountIdConverter::convert_ref(location)
            .map_err(|_| Error::FailedToTransactAsset("Failed Location to AccountId Conversion"))?;
        let (asset_id, amount) = match (
            NativeMatcher::matches_fungible(asset),
            NonNativeMatcher::matches_fungibles(asset),
        ) {
            // native asset
            (Some(amount), _) => (MultiAdapterAssetConfig::NativeAssetId::get(), amount),
            // assets asset
            (_, Ok((asset_id, amount))) => (asset_id, amount),
            // unknown asset
            _ => return Err(Error::FailedToTransactAsset("Unknown Asset")),
        };
        Ok((asset_id, amount, receiver))
    }
}
