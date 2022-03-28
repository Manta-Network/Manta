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

///! Manta/Calamari/Dolphin Asset
use crate::{
	constants::DEFAULT_ASSET_ED,
	types::{AssetId, Balance},
};
use codec::{Codec, Decode, Encode};
use frame_support::{
	traits::tokens::{DepositConsequence, WithdrawConsequence},
	Parameter,
};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{traits::Member, DispatchResult};
use sp_std::{borrow::Borrow, marker::PhantomData, prelude::Vec};

use xcm::{
	v1::{Junctions, MultiLocation},
	VersionedMultiLocation,
};

/// The minimal interface of asset metadata
pub trait AssetMetadata {
	/// Returns the minimum balance to hold this asset
	fn min_balance(&self) -> Balance;

	/// Returns a boolean value indicating whether this asset needs an existential deposit
	fn is_sufficient(&self) -> bool;
}

/// The registrar trait: defines the interface of creating an asset in the asset implementation layer.
/// We may revisit this interface design (e.g. add change asset interface). However, change StorageMetadata
/// should be rare.
pub trait AssetRegistrar<T: AssetConfig> {
	/// Create an new asset.
	///
	/// * `asset_id`: the asset id to be created
	/// * `min_balance`: the minimum balance to hold this asset
	/// * `metadata`: the metadata that the implementation layer stores
	/// * `is_sufficient`: whether this asset can be used as reserve asset,
	/// 	to the first approximation. More specifically, Whether a non-zero balance of this asset is deposit of sufficient
	/// 	value to account for the state bloat associated with its balance storage. If set to
	/// 	`true`, then non-zero balances may be stored without a `consumer` reference (and thus
	/// 	an ED in the Balances pallet or whatever else is used to control user-account state
	/// 	growth).
	fn create_asset(
		asset_id: AssetId,
		min_balance: Balance,
		metadata: T::StorageMetadata,
		is_sufficient: bool,
	) -> DispatchResult;

	/// Update asset metadata by `AssetId`.
	///
	/// * `asset_id`: the asset id to be created.
	/// * `metadata`: the metadata that the implementation layer stores.
	fn update_asset_metadata(asset_id: AssetId, metadata: T::StorageMetadata) -> DispatchResult;
}

pub trait AssetConfig: 'static + Eq + Clone {
	/// The trait we use to register Assets
	type AssetRegistrar: AssetRegistrar<Self>;

	/// Metadata type that required in token storage: e.g. AssetMetadata in Pallet-Assets.
	type StorageMetadata: Member + Parameter + Default + From<Self::AssetRegistrarMetadata>;

	/// The Asset Metadata type stored in this pallet.
	type AssetRegistrarMetadata: Member + Parameter + Codec + Default + AssetMetadata;

	/// The AssetLocation type: could be just a thin wrapper of MultiLocation
	type AssetLocation: Member + Parameter + Default + TypeInfo;
}

/// The metadata of a Manta Asset
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetRegistrarMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub evm_address: Option<H160>,
	pub is_frozen: bool,
	pub min_balance: Balance,
	/// `is_sufficient`: Whether a non-zero balance of this asset is deposit of sufficient
	/// value to account for the state bloat associated with its balance storage. If set to
	/// `true`, then non-zero balances may be stored without a `consumer` reference (and thus
	/// an ED in the Balances pallet or whatever else is used to control user-account state
	/// growth).
	/// For example, if is_sufficient set to `false`, a fresh account cannot receive XCM tokens.
	pub is_sufficient: bool,
}

impl Default for AssetRegistrarMetadata {
	fn default() -> Self {
		AssetRegistrarMetadata {
			name: b"Dolphin".to_vec(),
			symbol: b"DOL".to_vec(),
			decimals: 12,
			evm_address: None,
			is_frozen: false,
			min_balance: DEFAULT_ASSET_ED,
			is_sufficient: true,
		}
	}
}

impl AssetMetadata for AssetRegistrarMetadata {
	fn min_balance(&self) -> Balance {
		self.min_balance
	}

	fn is_sufficient(&self) -> bool {
		self.is_sufficient
	}
}

/// Asset storage metadata
/// Currently, `AssetStorageMetadata` is stored at `pallet-asset`.
#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetStorageMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}

impl From<AssetRegistrarMetadata> for AssetStorageMetadata {
	fn from(source: AssetRegistrarMetadata) -> Self {
		AssetStorageMetadata {
			name: source.name,
			symbol: source.symbol,
			decimals: source.decimals,
			is_frozen: source.is_frozen,
		}
	}
}

#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct AssetLocation(pub VersionedMultiLocation);

/// Default AssetLocation
impl Default for AssetLocation {
	fn default() -> Self {
		AssetLocation(VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: Junctions::Here,
		}))
	}
}

/// Convert a `MultiLocaiton` to an `AssetLocation`
/// Note: This does not guaranttee the `AssetLocation` is registered (i.e. have an AssetId)
impl From<MultiLocation> for AssetLocation {
	fn from(location: MultiLocation) -> Self {
		AssetLocation(VersionedMultiLocation::V1(location))
	}
}

/// Convert an `AssetLocation` to a MultiLocation
/// If Native, return none.
impl From<AssetLocation> for Option<MultiLocation> {
	fn from(location: AssetLocation) -> Self {
		match location {
			AssetLocation(VersionedMultiLocation::V1(location)) => Some(location),
			_ => None,
		}
	}
}

/// Defines the trait to obtain a generic AssetId
pub trait AssetIdLocationGetter<AssetLocation> {
	// get AssetLocation from AssetId
	fn get_asset_location(asset_id: AssetId) -> Option<AssetLocation>;

	// get AssetId from AssetLocation
	fn get_asset_id(loc: &AssetLocation) -> Option<AssetId>;
}

/// Defines the units per second charged given an `AssetId`.
pub trait UnitsToWeightRatio {
	/// Get units per second from asset id
	fn get_units_per_second(asset_id: AssetId) -> Option<u128>;
}

/// Converter struct implementing `Convert`.
/// This enforce the `AssetInfoGetter` implements `AssetIdLocationGetter`
pub struct AssetIdLocationConvert<AssetLocation, AssetInfoGetter>(
	PhantomData<(AssetLocation, AssetInfoGetter)>,
);
impl<AssetLocation, AssetInfoGetter> xcm_executor::traits::Convert<MultiLocation, AssetId>
	for AssetIdLocationConvert<AssetLocation, AssetInfoGetter>
where
	AssetId: Clone,
	AssetLocation: From<MultiLocation> + Into<Option<MultiLocation>> + Clone,
	AssetInfoGetter: AssetIdLocationGetter<AssetLocation>,
{
	fn convert_ref(loc: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
		AssetInfoGetter::get_asset_id(&loc.borrow().clone().into()).ok_or(())
	}

	fn reverse_ref(id: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
		AssetInfoGetter::get_asset_location(id.borrow().clone())
			.and_then(Into::into)
			.ok_or(())
	}
}

#[derive(Debug)]
pub enum FungibleLedgerConsequence {
	/// Deposit couldn't happen due to the amount being too low. This is usually because the
	/// account doesn't yet exist and the deposit wouldn't bring it to at least the minimum needed
	/// for existance.
	BelowMinimum,
	/// Deposit cannot happen since the account cannot be created (usually because it's a consumer
	/// and there exists no provider reference).
	CannotCreate,
	/// The asset is unknown. Usually because an `AssetId` has been presented which doesn't exist
	/// on the system.
	UnknownAsset,
	/// An overflow would occur. This is practically unexpected, but could happen in test systems
	/// with extremely small balance types or balances that approach the max value of the balance
	/// type.
	Overflow,
	/// There has been an underflow in the system. This is indicative of a corrupt state and
	/// likely unrecoverable.
	Underflow,
	/// Account continued in existence.
	/// Not enough of the funds in the account are unavailable for withdrawal.
	Frozen,
	/// Account balance would reduce to zero, potentially destroying it. The parameter is the
	/// amount of balance which is destroyed.
	ReducedToZero(Balance),
	/// Withdraw could not happen since the amount to be withdrawn is less than the total funds in
	/// the account.
	NoFunds,
	/// The withdraw would mean the account dying when it needs to exist (usually because it is a
	/// provider and there are consumer references on it).
	WouldDie,
	/// Internal error.
	InternalError,
	/// Success
	Success,
}

impl From<DepositConsequence> for FungibleLedgerConsequence {
	fn from(dc: DepositConsequence) -> Self {
		match dc {
			DepositConsequence::BelowMinimum => FungibleLedgerConsequence::BelowMinimum,
			DepositConsequence::CannotCreate => FungibleLedgerConsequence::CannotCreate,
			DepositConsequence::Overflow => FungibleLedgerConsequence::Overflow,
			DepositConsequence::Success => FungibleLedgerConsequence::Success,
			DepositConsequence::UnknownAsset => FungibleLedgerConsequence::UnknownAsset,
		}
	}
}

impl From<WithdrawConsequence<Balance>> for FungibleLedgerConsequence {
	fn from(wc: WithdrawConsequence<Balance>) -> Self {
		match wc {
			WithdrawConsequence::Frozen => FungibleLedgerConsequence::Frozen,
			WithdrawConsequence::NoFunds => FungibleLedgerConsequence::NoFunds,
			WithdrawConsequence::Overflow => FungibleLedgerConsequence::Overflow,
			WithdrawConsequence::Underflow => FungibleLedgerConsequence::Underflow,
			WithdrawConsequence::ReducedToZero(balance) => {
				FungibleLedgerConsequence::ReducedToZero(balance)
			}
			WithdrawConsequence::Success => FungibleLedgerConsequence::Success,
			WithdrawConsequence::UnknownAsset => FungibleLedgerConsequence::UnknownAsset,
			WithdrawConsequence::WouldDie => FungibleLedgerConsequence::WouldDie,
		}
	}
}

/// Unified interface for fungible ledger
/// It unifies `fungible` and `fungibles`
pub trait FungibleLedger<C>
where
	C: frame_system::Config,
{
	/// check whether `asset_id`, `account` can increase certain balance
	fn can_deposit(
		asset_id: AssetId,
		account: &C::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence>;

	/// check whether `asset_id`, `account` can decrease certain balance
	fn can_withdraw(
		asset_id: AssetId,
		account: &C::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence>;

	/// transfer asset
	fn transfer(
		asset_id: AssetId,
		source: &C::AccountId,
		dest: &C::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence>;

	/// mint asset to a beneficiary
	fn mint(
		asset_id: AssetId,
		beneficiary: &C::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence>;
}
