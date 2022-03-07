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

//! # Asset Manager Pallet
//!
//! A simple asset manager for native and cross-chain tokens
//!
//! ## Overview
//!
//! The Asset manager module provides functionality for registering cross chain assets
//!
//! TODO: detailed doc-string comment

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use codec::Codec;
	use frame_support::{pallet_prelude::*, transactional, PalletId};
	use frame_system::pallet_prelude::*;
	use manta_primitives::assets::{AssetIdLocationGetter, UnitsToWeightRatio};
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One},
		ArithmeticError,
	};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The AssetManagers's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");

	/// The registrar trait: defines the interface of creating an asset in the asset implementation layer.
	/// We may revisit this interface design (e.g. add change asset interface). However, change StorageMetadata
	/// should be rare.
	pub trait AssetRegistrar<T: Config> {
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
			asset_id: T::AssetId,
			min_balance: T::Balance,
			metadata: T::StorageMetadata,
			is_sufficient: bool,
		) -> DispatchResult;

		/// Update asset metadata by `AssetId`.
		///
		/// * `asset_id`: the asset id to be created.
		/// * `metadata`: the metadata that the implementation layer stores.
		fn update_asset_metadata(
			asset_id: T::AssetId,
			metadata: T::StorageMetadata,
		) -> DispatchResult;
	}

	/// The AssetMetadata trait:
	pub trait AssetMetadata<T: Config> {
		/// Returns the minimum balance to hold this asset
		fn min_balance(&self) -> T::Balance;

		/// Returns a boolean value indicating whether this asset needs an existential deposit
		fn is_sufficient(&self) -> bool;
	}

	/// Convert AssetId and AssetLocation
	impl<T: Config> AssetIdLocationGetter<T::AssetId, T::AssetLocation> for Pallet<T> {
		fn get_asset_id(loc: &T::AssetLocation) -> Option<T::AssetId> {
			LocationAssetId::<T>::get(loc)
		}

		fn get_asset_location(id: T::AssetId) -> Option<T::AssetLocation> {
			AssetIdLocation::<T>::get(id)
		}
	}

	/// Get unit per second from `AssetId`
	impl<T: Config> UnitsToWeightRatio<T::AssetId> for Pallet<T> {
		fn get_units_per_second(id: T::AssetId) -> Option<u128> {
			UnitsPerSecond::<T>::get(id)
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The asset id type, this have to be consistent with pallet-manta-pay
		type AssetId: Member
			+ Parameter
			+ Default
			+ Copy
			+ AtLeast32BitUnsigned
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// The trait we use to register Assets
		type AssetRegistrar: AssetRegistrar<Self>;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// Metadata type that required in token storage: e.g. AssetMetadata in Pallet-Assets.
		type StorageMetadata: Member + Parameter + Default;

		/// The Asset Metadata type stored in this pallet.
		type AssetRegistrarMetadata: Member
			+ Parameter
			+ Codec
			+ Default
			+ Into<Self::StorageMetadata>
			+ AssetMetadata<Self>;

		/// The AssetLocation type: could be just a thin wrapper of MultiLocation
		type AssetLocation: Member + Parameter + Default + TypeInfo;

		/// The origin which may forcibly create or destroy an asset or otherwise alter privileged
		/// attributes.
		type ModifierOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new asset registered.
		AssetRegistered {
			asset_id: T::AssetId,
			asset_address: T::AssetLocation,
			metadata: T::AssetRegistrarMetadata,
		},
		/// An asset's location has been updated.
		AssetLocationUpdated {
			asset_id: T::AssetId,
			location: T::AssetLocation,
		},
		/// An asset;s metadata has been updated.
		AssetMetadataUpdated {
			asset_id: T::AssetId,
			metadata: T::AssetRegistrarMetadata,
		},
		/// Update units per second of an asset
		UnitsPerSecondUpdated {
			asset_id: T::AssetId,
			units_per_second: u128,
		},
	}

	/// Error.
	#[pallet::error]
	pub enum Error<T> {
		/// Location already exists.
		LocationAlreadyExists,
		/// Error creating asset, e.g. error returned from the implementation layer.
		ErrorCreatingAsset,
		/// Update a non-exist asset
		UpdateNonExistAsset,
		/// Asset already registered.
		AssetAlreadyRegistered,
	}

	/// AssetId to MultiLocation Map.
	/// This is mostly useful when sending an asset to a foreign location.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_location)]
	pub(super) type AssetIdLocation<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetLocation>;

	/// MultiLocation to AssetId Map.
	/// This is mostly useful when receiving an asset from a foreign location.
	#[pallet::storage]
	#[pallet::getter(fn location_asset_id)]
	pub(super) type LocationAssetId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetLocation, T::AssetId>;

	/// AssetId to AssetRegistrar Map.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_metadata)]
	pub(super) type AssetIdMetadata<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetRegistrarMetadata>;

	/// Get the next available AssetId.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	/// XCM transfer cost for different asset.
	#[pallet::storage]
	pub type UnitsPerSecond<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, u128>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new asset in the asset manager.
		///
		/// * `origin`: Caller of this extrinsic, the acess control is specfied by `ForceOrigin`.
		/// * `location`: Location of the asset.
		/// * `metadata`: Asset metadata.
		/// * `min_balance`: Minimum balance to keep an account alive, used in conjunction with `is_sufficient`.
		/// * `is_sufficient`: Whether this asset needs users to have an existential deposit to hold
		///  this asset.
		///
		/// # <weight>
		/// TODO: get actual weight
		/// # </weight>
		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn register_asset(
			origin: OriginFor<T>,
			location: T::AssetLocation,
			metadata: T::AssetRegistrarMetadata,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				!LocationAssetId::<T>::contains_key(&location),
				Error::<T>::LocationAlreadyExists
			);
			let asset_id = Self::get_next_asset_id()?;
			T::AssetRegistrar::create_asset(
				asset_id,
				metadata.min_balance(),
				metadata.clone().into(),
				metadata.is_sufficient(),
			)
			.map_err(|_| Error::<T>::ErrorCreatingAsset)?;
			AssetIdLocation::<T>::insert(&asset_id, &location);
			AssetIdMetadata::<T>::insert(&asset_id, &metadata);
			LocationAssetId::<T>::insert(&location, &asset_id);
			Self::deposit_event(Event::<T>::AssetRegistered {
				asset_id,
				asset_address: location,
				metadata,
			});
			Ok(())
		}

		/// Update an asset by its asset id in the asset manager.
		///
		/// * `origin`: Caller of this extrinsic, the acess control is specfied by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `location`: `location` to update the asset location.
		/// # <weight>
		/// TODO: get actual weight
		/// # </weight>
		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn update_asset_location(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			location: T::AssetLocation,
		) -> DispatchResult {
			// checks validity
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				AssetIdLocation::<T>::contains_key(&asset_id),
				Error::<T>::UpdateNonExistAsset
			);
			ensure!(
				!LocationAssetId::<T>::contains_key(&location),
				Error::<T>::LocationAlreadyExists
			);
			// change the ledger state.
			let old_location =
				AssetIdLocation::<T>::get(&asset_id).ok_or(Error::<T>::UpdateNonExistAsset)?;
			LocationAssetId::<T>::remove(&old_location);
			LocationAssetId::<T>::insert(&location, &asset_id);
			AssetIdLocation::<T>::insert(&asset_id, &location);
			// deposit event.
			Self::deposit_event(Event::<T>::AssetLocationUpdated { asset_id, location });
			Ok(())
		}

		/// Update an asset's metadata by its `asset_id`
		///
		/// * `origin`: Caller of this extrinsic, the acess control is specfied by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `metadata`: new `metadata` to be associated with `asset_id`.
		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn update_asset_metadata(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			metadata: T::AssetRegistrarMetadata,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				AssetIdLocation::<T>::contains_key(&asset_id),
				Error::<T>::UpdateNonExistAsset
			);
			AssetIdMetadata::<T>::insert(&asset_id, &metadata);
			Self::deposit_event(Event::<T>::AssetMetadataUpdated { asset_id, metadata });
			Ok(())
		}

		/// Update an asset by its asset id in the asset manager.
		///
		/// * `origin`: Caller of this extrinsic, the acess control is specfied by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `units_per_second`: units per second for `asset_id`
		/// # <weight>
		/// TODO: get actual weight
		/// # </weight>
		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn set_units_per_second(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			#[pallet::compact] units_per_second: u128,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				AssetIdLocation::<T>::contains_key(&asset_id),
				Error::<T>::UpdateNonExistAsset
			);
			UnitsPerSecond::<T>::insert(&asset_id, &units_per_second);
			Self::deposit_event(Event::<T>::UnitsPerSecondUpdated {
				asset_id,
				units_per_second,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get and increment the `NextAssetID` by one.
		fn get_next_asset_id() -> Result<T::AssetId, DispatchError> {
			NextAssetId::<T>::try_mutate(|current| -> Result<T::AssetId, DispatchError> {
				let id = *current;
				*current = current
					.checked_add(&One::one())
					.ok_or(ArithmeticError::Overflow)?;
				Ok(id)
			})
		}

		/// The account ID of AssetManager
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account()
		}
	}
}
