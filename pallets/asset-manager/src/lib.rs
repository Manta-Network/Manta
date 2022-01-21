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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use codec::HasCompact;
	use scale_info::TypeInfo;

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
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
		
		/// Metadata type that required in token storage: e.g. AssetMetadata in Pallet-Assets.
		type StorageMetadata: Member + Parameter + Default;

		/// The Asset Metadata type stored in this pallet.
		type AssetRegistrarMetadata: Member + Parameter + Default + Into<Self::StorageMetadata>;

		/// The MultiLocation type.
		type MultiLocation: Member + Parameter + Default;
		
		/// The origin which may forcibly create or destroy an asset or otherwise alter privileged
		/// attributes.
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// The maximum number of assets this pallet can manage
		#[pallet::constant]
		type Capacity: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new asset registered.
		AssetRegistered {
			asset_id: T::AssetId,
			asset_address: T::MultiLocation,
			metadata: T::AssetRegistrarMetadata,
		},
		/// An asset has been updated.
		AssetUpdated {
			asset_id: T::AssetId,
			asset_address: T::MultiLocation,
			metadata: T::AssetRegistrarMetadata,
		},
		/// Asset frozen.
		AssetFrozen {
			asset_id: T::AssetId,
		},
		/// Asset unfrozen.
		AssetUnfrozen {
			asset_id: T::AssetId,
		}
	}

	/// Error for the nicks pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShort,
		/// A name is too long.
		TooLong,
		/// An account isn't named.
		Unnamed,
	}

	/// AssetId to MultiLocation Map.
	/// This is mostly useful when sending an asset to a foreign location.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_location)]
	pub(super) type AssetIdLocation<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::MultiLocation>;
	
	/// MultiLocation to AssetId Map.
	/// This is mostly useful when receiving an asset from a foreign location.
	#[pallet::storage]
	#[pallet::getter(fn location_asset_id)]
	pub(super) type LocationAssetId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::MultiLocation, T::AssetId>;

	/// AssetId to AssetRegistrar Map.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_metadata)]
	pub(super) type AssetIdMetadata<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetRegistrarMetadata>;
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 
		/// # <weight>
		/// TODO: get actual weight
		/// # </weight>
		#[pallet::weight(50_000_000)]
		pub fn set_name(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			
			Ok(())
		}

	}
}
