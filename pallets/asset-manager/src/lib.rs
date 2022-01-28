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

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use codec::{Codec, HasCompact};
	use frame_support::{pallet_prelude::*, transactional, PalletId};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{AccountIdConversion, AtLeast32BitUnsigned, Bounded, CheckedAdd, One},
		ArithmeticError,
	};
	use manta_primitives::AssetIdLocationGetter;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// The AssetManagers's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");

	/// The registrar trait: defines the interface of creating an asset in the asset implementation layer.
	/// We may revisit this interface design (e.g. add change asset interface). However, change StorageMetadata
	/// should be rare.
	pub trait AssetRegistrar<T: Config> {
		///
		/// * `asset_id`: the asset id to be created
		/// * `min_balance`: the minimum balance to hold this asset
		/// * `metadata`: the metadata that the implementation layer stores
		/// * `is_sufficient`: Whether this asset needs users to have an existential deposit to hold
		///  this asset.
		fn create_asset(
			asset_id: T::AssetId,
			min_balance: T::Balance,
			metadata: T::StorageMetadata,
			is_sufficient: bool,
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
	impl<T: Config> AssetIdLocationGetter<T::AssetId, T::AssetLocation> for Pallet<T>{
		fn get_asset_id(loc: T::AssetLocation) -> Option<T::AssetId>{
			LocationAssetId::<T>::get(loc)
		}

		fn get_asset_location(id: T::AssetId) -> Option<T::AssetLocation>{
			AssetIdLocation::<T>::get(id)
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
			+ HasCompact
			+ CheckedAdd
			+ Bounded
			+ One
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
		type AssetLocation: Member + Parameter + Default;

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
		/// An asset has been updated.
		AssetUpdated {
			asset_id: T::AssetId,
			asset_address: T::AssetLocation,
			metadata: T::AssetRegistrarMetadata,
		},
		/// Asset frozen.
		AssetFrozen { asset_id: T::AssetId },
		/// Asset unfrozen.
		AssetUnfrozen { asset_id: T::AssetId },
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
	pub type AssetTransferCost<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, T::Balance>;

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
		pub fn register_asset(
			origin: OriginFor<T>,
			location: T::AssetLocation,
			metadata: T::AssetRegistrarMetadata,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				LocationAssetId::<T>::get(&location).is_none(),
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
			AssetIdMetadata::<T>::insert(&asset_id, &metadata.clone());
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
		/// * `metadata_option`: `Some(meta)` to update the metadata to `meta`, `None` means no update on metadata.
		/// * `location_option`: `Some(location)` to update the asset location, `None` means no update on location.
		///
		/// # <weight>
		/// TODO: get actual weight
		/// # </weight>
		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn update_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			location_option: Option<T::AssetLocation>,
			metadata_option: Option<T::AssetRegistrarMetadata>,
		) -> DispatchResult {
			// check validity.
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				AssetIdLocation::<T>::contains_key(&asset_id),
				Error::<T>::UpdateNonExistAsset
			);
			if let Some(location) = location_option.clone() {
				ensure!(
					!LocationAssetId::<T>::contains_key(&location),
					Error::<T>::LocationAlreadyExists
				)
			}
			// write to the ledger state.
			if let Some(location) = location_option {
				let old_location =
					AssetIdLocation::<T>::get(&asset_id).ok_or(Error::<T>::UpdateNonExistAsset)?;
				LocationAssetId::<T>::remove(&old_location);
				LocationAssetId::<T>::insert(&location, &asset_id);
				AssetIdLocation::<T>::insert(&asset_id, &location);
			}
			if let Some(metadata) = metadata_option {
				AssetIdMetadata::<T>::insert(&asset_id, &metadata)
			}
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
