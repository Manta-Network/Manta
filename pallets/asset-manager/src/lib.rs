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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use crate::weights::WeightInfo;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use crate::weights::WeightInfo;
	use frame_support::{pallet_prelude::*, traits::Contains, transactional, PalletId};
	use frame_system::pallet_prelude::*;
	use manta_primitives::{
		assets::{
			AssetConfig, AssetIdLocationGetter, AssetMetadata, AssetRegistrar, FungibleLedger,
			UnitsToWeightRatio,
		},
		types::{AssetId, Balance},
	};
	use orml_traits::GetByKey;
	use sp_runtime::{
		traits::{AccountIdConversion, One},
		ArithmeticError,
	};
	use xcm::latest::prelude::*;

	/// Alias for the junction Parachain(#[codec(compact)] u32),
	pub(crate) type ParaId = u32;
	pub(crate) type AssetCount = u32;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Convert AssetId and AssetLocation
	impl<T: Config> AssetIdLocationGetter<<T::AssetConfig as AssetConfig<T>>::AssetLocation>
		for Pallet<T>
	{
		fn get_asset_id(
			loc: &<T::AssetConfig as AssetConfig<T>>::AssetLocation,
		) -> Option<AssetId> {
			LocationAssetId::<T>::get(loc)
		}

		fn get_asset_location(
			id: AssetId,
		) -> Option<<T::AssetConfig as AssetConfig<T>>::AssetLocation> {
			AssetIdLocation::<T>::get(id)
		}
	}

	/// Get unit per second from `AssetId`
	impl<T: Config> UnitsToWeightRatio for Pallet<T> {
		fn get_units_per_second(id: AssetId) -> Option<u128> {
			UnitsPerSecond::<T>::get(id)
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Asset configuration, e.g. AssetId, Balance, Metadata
		type AssetConfig: AssetConfig<Self>;

		/// The origin which may forcibly create or destroy an asset or otherwise alter privileged
		/// attributes.
		type ModifierOrigin: EnsureOrigin<Self::Origin>;

		/// Pallet ID
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this pallet.
		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub start_id: AssetId,
		pub _marker: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				start_id: <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(),
				_marker: PhantomData,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			NextAssetId::<T>::set(self.start_id);
			let asset_id = <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get();
			let metadata = <T::AssetConfig as AssetConfig<T>>::NativeAssetMetadata::get();
			let location = <T::AssetConfig as AssetConfig<T>>::NativeAssetLocation::get();
			AssetIdLocation::<T>::insert(&asset_id, &location);
			AssetIdMetadata::<T>::insert(&asset_id, &metadata);
			LocationAssetId::<T>::insert(&location, &asset_id);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new asset registered.
		AssetRegistered {
			asset_id: AssetId,
			asset_address: <T::AssetConfig as AssetConfig<T>>::AssetLocation,
			metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata,
		},
		/// An asset's location has been updated.
		AssetLocationUpdated {
			asset_id: AssetId,
			location: <T::AssetConfig as AssetConfig<T>>::AssetLocation,
		},
		/// An asset;s metadata has been updated.
		AssetMetadataUpdated {
			asset_id: AssetId,
			metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata,
		},
		/// Update units per second of an asset
		UnitsPerSecondUpdated {
			asset_id: AssetId,
			units_per_second: u128,
		},
		/// Asset minted.
		AssetMinted {
			asset_id: AssetId,
			beneficiary: T::AccountId,
			amount: Balance,
		},
		/// Update min xcm fee of an asset
		MinXcmFeeUpdated {
			reserve_chain: <T::AssetConfig as AssetConfig<T>>::AssetLocation,
			min_xcm_fee: u128,
		},
	}

	/// Error.
	#[pallet::error]
	pub enum Error<T> {
		/// Location already exists.
		LocationAlreadyExists,
		/// Error creating asset, e.g. error returned from the implementation layer.
		ErrorCreatingAsset,
		/// Update a non-exist asset.
		UpdateNonExistAsset,
		/// Cannot update reserved assets metadata (0 and 1)
		CannotUpdateNativeAssetMetadata,
		/// Asset already registered.
		AssetAlreadyRegistered,
		/// Error on minting asset.
		MintError,
		/// Fail to update para id.
		UpdateParaIdError,
	}

	/// AssetId to MultiLocation Map.
	/// This is mostly useful when sending an asset to a foreign location.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_location)]
	pub(super) type AssetIdLocation<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, <T::AssetConfig as AssetConfig<T>>::AssetLocation>;

	/// MultiLocation to AssetId Map.
	/// This is mostly useful when receiving an asset from a foreign location.
	#[pallet::storage]
	#[pallet::getter(fn location_asset_id)]
	pub(super) type LocationAssetId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T::AssetConfig as AssetConfig<T>>::AssetLocation, AssetId>;

	/// AssetId to AssetRegistrar Map.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_metadata)]
	pub(super) type AssetIdMetadata<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AssetId,
		<T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata,
	>;

	/// Get the next available AssetId.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T: Config> = StorageValue<_, AssetId, ValueQuery>;

	/// XCM transfer cost for different asset.
	#[pallet::storage]
	pub type UnitsPerSecond<T: Config> = StorageMap<_, Blake2_128Concat, AssetId, u128>;

	/// Minimum xcm execution fee paid on destination chain.
	#[pallet::storage]
	#[pallet::getter(fn get_min_xcm_fee)]
	pub type MinXcmFee<T: Config> =
		StorageMap<_, Blake2_128Concat, <T::AssetConfig as AssetConfig<T>>::AssetLocation, u128>;

	/// The count of associated assets for each para id except relaychain.
	#[pallet::storage]
	#[pallet::getter(fn get_para_id)]
	pub type AllowedDestParaIds<T: Config> = StorageMap<_, Blake2_128Concat, ParaId, AssetCount>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new asset in the asset manager.
		///
		/// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
		/// * `location`: Location of the asset.
		/// * `metadata`: Asset metadata.
		/// * `min_balance`: Minimum balance to keep an account alive, used in conjunction with `is_sufficient`.
		/// * `is_sufficient`: Whether this asset needs users to have an existential deposit to hold
		///  this asset.
		#[pallet::weight(T::WeightInfo::register_asset())]
		#[transactional]
		pub fn register_asset(
			origin: OriginFor<T>,
			location: <T::AssetConfig as AssetConfig<T>>::AssetLocation,
			metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				!LocationAssetId::<T>::contains_key(&location),
				Error::<T>::LocationAlreadyExists
			);
			let asset_id = Self::get_next_asset_id()?;
			<T::AssetConfig as AssetConfig<T>>::AssetRegistrar::create_asset(
				asset_id,
				metadata.min_balance(),
				metadata.clone().into(),
				metadata.is_sufficient(),
			)
			.map_err(|_| Error::<T>::ErrorCreatingAsset)?;
			AssetIdLocation::<T>::insert(&asset_id, &location);
			AssetIdMetadata::<T>::insert(&asset_id, &metadata);
			LocationAssetId::<T>::insert(&location, &asset_id);

			// If it's a new para id, which will be inserted with AssetCount as 1.
			// If not, AssetCount will increased by 1.
			if let Some(para_id) =
				Self::get_para_id_from_multilocation(location.clone().into().as_ref())
			{
				Self::increase_count_of_associated_assets(para_id)?;
			}

			Self::deposit_event(Event::<T>::AssetRegistered {
				asset_id,
				asset_address: location,
				metadata,
			});
			Ok(())
		}

		/// Update an asset by its asset id in the asset manager.
		///
		/// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `location`: `location` to update the asset location.
		#[pallet::weight(T::WeightInfo::update_asset_location())]
		#[transactional]
		pub fn update_asset_location(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: AssetId,
			location: <T::AssetConfig as AssetConfig<T>>::AssetLocation,
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

			// 1. If the new location has new para id, insert the new para id,
			// the old para id will be deleted if AssetCount <= 1, or decreased by 1.
			// 2. If the new location doesn't contain a new para id, do nothing to AssetCount
			if let Some(old_para_id) =
				Self::get_para_id_from_multilocation(old_location.into().as_ref())
			{
				if AllowedDestParaIds::<T>::get(old_para_id) <= Some(<AssetCount as One>::one()) {
					AllowedDestParaIds::<T>::remove(old_para_id);
				} else {
					AllowedDestParaIds::<T>::try_mutate(old_para_id, |cnt| -> DispatchResult {
						let new_cnt = cnt
							.map(|c| c - <AssetCount as One>::one())
							.ok_or(Error::<T>::UpdateParaIdError)?;
						*cnt = Some(new_cnt);
						Ok(())
					})?;
				}
			}

			// If it's a new para id, which will be inserted with AssetCount as 1.
			// If not, AssetCount will increased by 1.
			if let Some(para_id) =
				Self::get_para_id_from_multilocation(location.clone().into().as_ref())
			{
				Self::increase_count_of_associated_assets(para_id)?;
			}

			// deposit event.
			Self::deposit_event(Event::<T>::AssetLocationUpdated { asset_id, location });
			Ok(())
		}

		/// Update an asset's metadata by its `asset_id`
		///
		/// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `metadata`: new `metadata` to be associated with `asset_id`.
		#[pallet::weight(T::WeightInfo::update_asset_metadata())]
		#[transactional]
		pub fn update_asset_metadata(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: AssetId,
			metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				asset_id != <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(),
				Error::<T>::CannotUpdateNativeAssetMetadata
			);
			ensure!(
				AssetIdLocation::<T>::contains_key(&asset_id),
				Error::<T>::UpdateNonExistAsset
			);
			<T::AssetConfig as AssetConfig<T>>::AssetRegistrar::update_asset_metadata(
				asset_id,
				metadata.clone().into(),
			)?;
			AssetIdMetadata::<T>::insert(&asset_id, &metadata);
			Self::deposit_event(Event::<T>::AssetMetadataUpdated { asset_id, metadata });
			Ok(())
		}

		/// Update an asset by its asset id in the asset manager.
		///
		/// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `units_per_second`: units per second for `asset_id`
		#[pallet::weight(T::WeightInfo::set_units_per_second())]
		#[transactional]
		pub fn set_units_per_second(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: AssetId,
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

		/// Mint asset by its asset id to a beneficiary.
		///
		/// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
		/// * `asset_id`: AssetId to be updated.
		/// * `beneficiary`: Account to mint the asset.
		/// * `amount`: Amount of asset being minted.
		#[pallet::weight(T::WeightInfo::mint_asset())]
		#[transactional]
		pub fn mint_asset(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: AssetId,
			beneficiary: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			ensure!(
				AssetIdLocation::<T>::contains_key(&asset_id),
				Error::<T>::UpdateNonExistAsset
			);
			ensure!(
				<T::AssetConfig as AssetConfig<T>>::FungibleLedger::mint(
					asset_id,
					&beneficiary,
					amount
				)
				.is_ok(),
				Error::<T>::MintError
			);
			Self::deposit_event(Event::<T>::AssetMinted {
				asset_id,
				beneficiary,
				amount,
			});
			Ok(())
		}

		/// Set min xcm fee for asset/s on their reserve chain.
		///
		/// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
		/// * `reserve_chain`: Multilocation to be haven min xcm fee.
		/// * `min_xcm_fee`: Amount of min_xcm_fee.
		#[pallet::weight(T::WeightInfo::set_min_xcm_fee())]
		#[transactional]
		pub fn set_min_xcm_fee(
			origin: OriginFor<T>,
			reserve_chain: <T::AssetConfig as AssetConfig<T>>::AssetLocation,
			#[pallet::compact] min_xcm_fee: u128,
		) -> DispatchResult {
			T::ModifierOrigin::ensure_origin(origin)?;
			MinXcmFee::<T>::insert(&reserve_chain, &min_xcm_fee);
			Self::deposit_event(Event::<T>::MinXcmFeeUpdated {
				reserve_chain,
				min_xcm_fee,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get and increment the `NextAssetID` by one.
		fn get_next_asset_id() -> Result<AssetId, DispatchError> {
			NextAssetId::<T>::try_mutate(|current| -> Result<AssetId, DispatchError> {
				let id = *current;
				*current = current.checked_add(1u32).ok_or(ArithmeticError::Overflow)?;
				Ok(id)
			})
		}

		/// The account ID of AssetManager
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}

		/// Get para id from asset location
		pub(crate) fn get_para_id_from_multilocation(
			location: Option<&MultiLocation>,
		) -> Option<ParaId> {
			if let Some(MultiLocation { interior, .. }) = location {
				match interior {
					Junctions::X1(Junction::Parachain(para_id))
					| Junctions::X2(Junction::Parachain(para_id), ..)
					| Junctions::X3(Junction::Parachain(para_id), ..)
					| Junctions::X4(Junction::Parachain(para_id), ..)
					| Junctions::X5(Junction::Parachain(para_id), ..)
					| Junctions::X6(Junction::Parachain(para_id), ..)
					| Junctions::X7(Junction::Parachain(para_id), ..)
					| Junctions::X8(Junction::Parachain(para_id), ..) => Some(*para_id),
					_ => None,
				}
			} else {
				None
			}
		}

		/// Increases the count of associated assets for the para id.
		pub(crate) fn increase_count_of_associated_assets(para_id: ParaId) -> DispatchResult {
			// If it's a new para id, which will be inserted with AssetCount as 1.
			// If not, AssetCount will increased by 1.
			if AllowedDestParaIds::<T>::contains_key(para_id) {
				AllowedDestParaIds::<T>::try_mutate(para_id, |count| -> DispatchResult {
					let new_count = count
						.map(|c| c + <AssetCount as One>::one())
						.ok_or(Error::<T>::UpdateParaIdError)?;
					*count = Some(new_count);
					Ok(())
				})
			} else {
				AllowedDestParaIds::<T>::insert(para_id, <AssetCount as One>::one());
				Ok(())
			}
		}
	}

	/// Check the multilocation is supported by calamari/manta.
	impl<T: Config> Contains<MultiLocation> for Pallet<T> {
		fn contains(location: &MultiLocation) -> bool {
			// check parents
			if location.parents != 1 {
				return false;
			}

			match location.interior {
				// Send tokens back to relaychain.
				Junctions::X1(Junction::AccountId32 { .. }) => true,
				// Send tokens to sibling chain.
				Junctions::X2(Junction::Parachain(para_id), Junction::AccountId32 { .. }) => {
					AllowedDestParaIds::<T>::contains_key(para_id)
				}
				// We don't support X3 or longer Junctions.
				_ => false,
			}
		}
	}

	/// Get min-xcm-fee by multilocation.
	impl<T: Config> GetByKey<MultiLocation, u128> for Pallet<T> {
		fn get(location: &MultiLocation) -> u128 {
			let location =
				<T::AssetConfig as AssetConfig<T>>::AssetLocation::from(location.clone());
			match MinXcmFee::<T>::get(&location) {
				Some(min_fee) => min_fee,
				None => u128::MAX,
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			let mut reads: Weight = 0;
			let mut writes: Weight = 0;
			LocationAssetId::<T>::iter().for_each(|(location, _asset_id)| {
				reads += 1;
				if let Some(para_id) =
					Self::get_para_id_from_multilocation(location.into().as_ref())
				{
					if para_id != 2084 {
						let _ = Self::increase_count_of_associated_assets(para_id);
						reads += 1; // There's one read in method increase_count_of_associated_assets.
						writes += 1; // There's one write in method increase_count_of_associated_assets.
					}
				}
			});
			T::DbWeight::get()
				.reads(reads)
				.saturating_add(T::DbWeight::get().writes(writes))
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		/// Direct implementation of `GenesisBuild::build_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
			<Self as GenesisBuild<T>>::build_storage(self)
		}

		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
			<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
		}
	}
}
