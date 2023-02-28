// Copyright 2020-2023 Manta Network.
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
pub mod migrations;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use crate::weights::WeightInfo;
pub use pallet::*;

/// Asset Manager Pallet
#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::{
        pallet_prelude::*,
        traits::{Contains, StorageVersion},
        transactional, PalletId,
    };
    use frame_system::pallet_prelude::*;
    use manta_primitives::assets::{
        self, AssetConfig, AssetIdLocationMap, AssetIdType, AssetMetadata, AssetRegistry,
        FungibleLedger, LocationType,
    };
    use orml_traits::GetByKey;
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, One,
        },
        ArithmeticError,
    };
    use xcm::latest::prelude::*;

    /// Storage Version
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    /// Alias for the junction type `Parachain(#[codec(compact)] u32)`
    pub(crate) type ParaId = u32;

    /// Asset Count Type
    pub(crate) type AssetCount = u32;

    /// Pallet Configuration
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Asset Id Type
        type AssetId: AtLeast32BitUnsigned
            + Default
            + Parameter
            + MaybeSerializeDeserialize
            + TypeInfo
            + Copy;

        /// Balance Type
        type Balance: Default + Member + Parameter + TypeInfo;

        /// Location Type
        type Location: Default
            + Parameter
            + TypeInfo
            + From<MultiLocation>
            + Into<Option<MultiLocation>>;

        /// Asset Configuration
        type AssetConfig: AssetConfig<
            Self,
            AssetId = Self::AssetId,
            Balance = Self::Balance,
            Location = Self::Location,
        >;

        /// The origin which may forcibly create or destroy an asset or otherwise alter privileged
        /// attributes.
        type ModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Pallet ID
        type PalletId: Get<PalletId>;

        /// Weight information for the extrinsics in this pallet.
        type WeightInfo: crate::weights::WeightInfo;
    }

    /// Asset Manager Pallet
    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    impl<T> AssetIdType for Pallet<T>
    where
        T: Config,
    {
        type AssetId = T::AssetId;
    }

    impl<T> LocationType for Pallet<T>
    where
        T: Config,
    {
        type Location = T::Location;
    }

    impl<T> AssetIdLocationMap for Pallet<T>
    where
        T: Config,
    {
        #[inline]
        fn location(asset_id: &Self::AssetId) -> Option<Self::Location> {
            AssetIdLocation::<T>::get(asset_id)
        }

        #[inline]
        fn asset_id(location: &Self::Location) -> Option<Self::AssetId> {
            LocationAssetId::<T>::get(location)
        }
    }

    impl<T> assets::UnitsPerSecond for Pallet<T>
    where
        T: Config,
    {
        #[inline]
        fn units_per_second(id: &Self::AssetId) -> Option<u128> {
            UnitsPerSecond::<T>::get(id)
        }
    }

    /// Genesis Configuration
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub start_id: T::AssetId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        #[inline]
        fn default() -> Self {
            Self {
                start_id: <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        #[inline]
        fn build(&self) {
            NextAssetId::<T>::set(self.start_id);
            let asset_id = <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get();
            let metadata = <T::AssetConfig as AssetConfig<T>>::NativeAssetMetadata::get();
            let location = <T::AssetConfig as AssetConfig<T>>::NativeAssetLocation::get();
            AssetIdLocation::<T>::insert(asset_id, &location);
            AssetIdMetadata::<T>::insert(asset_id, &metadata);
            LocationAssetId::<T>::insert(&location, asset_id);
        }
    }

    /// Asset Manager Event
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new asset was registered
        AssetRegistered {
            /// Asset Id of new Asset
            asset_id: T::AssetId,

            /// Location of the new Asset
            location: T::Location,

            /// Metadata Registered to Asset Manager
            metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
        },

        /// Updated the location of an asset
        AssetLocationUpdated {
            /// Asset Id of the updated Asset
            asset_id: T::AssetId,

            /// Updated Location for the Asset
            location: T::Location,
        },

        /// Updated the metadata of an asset
        AssetMetadataUpdated {
            /// Asset Id of the updated Asset
            asset_id: T::AssetId,

            /// Updated Metadata for the Asset
            metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
        },

        /// Updated the units-per-second for an asset
        UnitsPerSecondUpdated {
            /// Asset Id of the updated Asset
            asset_id: T::AssetId,

            /// Updated units-per-second for the Asset
            units_per_second: u128,
        },

        /// An asset was minted
        AssetMinted {
            /// Asset Id of the minted Asset
            asset_id: T::AssetId,

            /// Beneficiary Account
            beneficiary: T::AccountId,

            /// Amount Minted
            amount: T::Balance,
        },

        /// Updated the minimum XCM fee for an asset
        MinXcmFeeUpdated {
            /// Reserve Chain Location
            reserve_chain: T::Location,

            /// Updated Minimum XCM Fee
            min_xcm_fee: u128,
        },
    }

    /// Asset Manager Error
    #[pallet::error]
    pub enum Error<T> {
        /// Location Already Exists
        LocationAlreadyExists,

        /// An error occured while creating a new asset at the [`AssetRegistry`].
        ErrorCreatingAsset,

        /// There was an attempt to update a non-existent asset.
        UpdateNonExistentAsset,

        /// Cannot Update Native Asset Metadata
        CannotUpdateNativeAssetMetadata,

        /// Asset Already Registered
        AssetAlreadyRegistered,

        /// An error occurred while minting an asset.
        MintError,

        /// An error occurred while updating the parachain id.
        UpdateParaIdError,
    }

    /// [`AssetId`](AssetConfig::AssetId) to [`MultiLocation`] Map
    ///
    /// This is mostly useful when sending an asset to a foreign location.
    #[pallet::storage]
    #[pallet::getter(fn asset_id_location)]
    pub(super) type AssetIdLocation<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AssetId, T::Location>;

    /// [`MultiLocation`] to [`AssetId`](AssetConfig::AssetId) Map
    ///
    /// This is mostly useful when receiving an asset from a foreign location.
    #[pallet::storage]
    #[pallet::getter(fn location_asset_id)]
    pub(super) type LocationAssetId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Location, T::AssetId>;

    /// AssetId to AssetRegistry Map.
    #[pallet::storage]
    #[pallet::getter(fn asset_id_metadata)]
    pub(super) type AssetIdMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
    >;

    /// The Next Available [`AssetId`](AssetConfig::AssetId)
    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

    /// XCM transfer cost for each [`AssetId`](AssetConfig::AssetId)
    #[pallet::storage]
    pub type UnitsPerSecond<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, u128>;

    /// Minimum xcm execution fee paid on destination chain.
    #[pallet::storage]
    #[pallet::getter(fn get_min_xcm_fee)]
    pub type MinXcmFee<T: Config> = StorageMap<_, Blake2_128Concat, T::Location, u128>;

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
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_asset())]
        #[transactional]
        pub fn register_asset(
            origin: OriginFor<T>,
            location: T::Location,
            metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(
                !LocationAssetId::<T>::contains_key(&location),
                Error::<T>::LocationAlreadyExists
            );
            let asset_id = Self::next_asset_id_and_increment()?;
            <T::AssetConfig as AssetConfig<T>>::AssetRegistry::create_asset(
                asset_id,
                metadata.clone().into(),
                metadata.min_balance().clone(),
                metadata.is_sufficient(),
            )
            .map_err(|_| Error::<T>::ErrorCreatingAsset)?;
            AssetIdLocation::<T>::insert(asset_id, &location);
            AssetIdMetadata::<T>::insert(asset_id, &metadata);
            LocationAssetId::<T>::insert(&location, asset_id);

            // If it's a new para id, which will be inserted with AssetCount as 1.
            // If not, AssetCount will increased by 1.
            if let Some(para_id) =
                Self::para_id_from_multilocation(location.clone().into().as_ref())
            {
                Self::increase_count_of_associated_assets(*para_id)?;
            }

            Self::deposit_event(Event::<T>::AssetRegistered {
                asset_id,
                location,
                metadata,
            });
            Ok(())
        }

        /// Update an asset by its asset id in the asset manager.
        ///
        /// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
        /// * `asset_id`: AssetId to be updated.
        /// * `location`: `location` to update the asset location.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_asset_location())]
        #[transactional]
        pub fn update_asset_location(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            location: T::Location,
        ) -> DispatchResult {
            // checks validity
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(
                AssetIdLocation::<T>::contains_key(asset_id),
                Error::<T>::UpdateNonExistentAsset
            );
            ensure!(
                !LocationAssetId::<T>::contains_key(&location),
                Error::<T>::LocationAlreadyExists
            );
            // change the ledger state.
            let old_location =
                AssetIdLocation::<T>::get(asset_id).ok_or(Error::<T>::UpdateNonExistentAsset)?;
            LocationAssetId::<T>::remove(&old_location);
            LocationAssetId::<T>::insert(&location, asset_id);
            AssetIdLocation::<T>::insert(asset_id, &location);

            // 1. If the new location has new para id, insert the new para id,
            // the old para id will be deleted if AssetCount <= 1, or decreased by 1.
            // 2. If the new location doesn't contain a new para id, do nothing to AssetCount
            if let Some(old_para_id) =
                Self::para_id_from_multilocation(old_location.into().as_ref())
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
                Self::para_id_from_multilocation(location.clone().into().as_ref())
            {
                Self::increase_count_of_associated_assets(*para_id)?;
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
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_asset_metadata())]
        #[transactional]
        pub fn update_asset_metadata(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(
                asset_id != <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(),
                Error::<T>::CannotUpdateNativeAssetMetadata
            );
            ensure!(
                AssetIdLocation::<T>::contains_key(asset_id),
                Error::<T>::UpdateNonExistentAsset
            );
            <T::AssetConfig as AssetConfig<T>>::AssetRegistry::update_asset_metadata(
                &asset_id,
                metadata.clone().into(),
            )?;
            AssetIdMetadata::<T>::insert(asset_id, &metadata);
            Self::deposit_event(Event::<T>::AssetMetadataUpdated { asset_id, metadata });
            Ok(())
        }

        /// Update an asset by its asset id in the asset manager.
        ///
        /// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
        /// * `asset_id`: AssetId to be updated.
        /// * `units_per_second`: units per second for `asset_id`
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::set_units_per_second())]
        #[transactional]
        pub fn set_units_per_second(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] units_per_second: u128,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(
                AssetIdLocation::<T>::contains_key(asset_id),
                Error::<T>::UpdateNonExistentAsset
            );
            UnitsPerSecond::<T>::insert(asset_id, units_per_second);
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
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::mint_asset())]
        #[transactional]
        pub fn mint_asset(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            beneficiary: T::AccountId,
            amount: T::Balance,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(
                AssetIdLocation::<T>::contains_key(asset_id),
                Error::<T>::UpdateNonExistentAsset
            );
            <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(
                asset_id,
                &beneficiary,
                amount.clone(),
                true,
            )
            .map_err(|_| Error::<T>::MintError)?;
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
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::set_min_xcm_fee())]
        #[transactional]
        pub fn set_min_xcm_fee(
            origin: OriginFor<T>,
            reserve_chain: T::Location,
            #[pallet::compact] min_xcm_fee: u128,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            MinXcmFee::<T>::insert(&reserve_chain, min_xcm_fee);
            Self::deposit_event(Event::<T>::MinXcmFeeUpdated {
                reserve_chain,
                min_xcm_fee,
            });
            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        /// Returns and increments the [`NextAssetId`] by one.
        #[inline]
        fn next_asset_id_and_increment() -> Result<T::AssetId, DispatchError> {
            NextAssetId::<T>::try_mutate(|current| {
                let id = *current;
                *current = current
                    .checked_add(&One::one())
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(id)
            })
        }

        /// Returns the account identifier of the [`AssetManager`] pallet.
        #[inline]
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Returns the [`ParaId`] associated to `location`.
        pub fn para_id_from_multilocation(location: Option<&MultiLocation>) -> Option<&ParaId> {
            location.and_then(|location| {
                if let Some(Parachain(para_id)) = location.first_interior() {
                    Some(para_id)
                } else {
                    None
                }
            })
        }

        /// Increases the count of associated assets for the para id.
        pub fn increase_count_of_associated_assets(para_id: ParaId) -> DispatchResult {
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

    /// Check the multilocation destination is supported by calamari/manta.
    impl<T> Contains<MultiLocation> for Pallet<T>
    where
        T: Config,
    {
        #[inline]
        fn contains(location: &MultiLocation) -> bool {
            // check parents
            if location.parents != 1 {
                return false;
            }

            match location.interior {
                // Send tokens back to relaychain.
                Junctions::X1(Junction::AccountId32 { .. }) => true,
                // Send tokens to sibling chain.
                Junctions::X2(Junction::Parachain(para_id), Junction::AccountId32 { .. })
                | Junctions::X2(Junction::Parachain(para_id), Junction::AccountKey20 { .. }) => {
                    AllowedDestParaIds::<T>::contains_key(para_id)
                }
                // We don't support X3 or longer Junctions.
                _ => false,
            }
        }
    }

    /// Get min-xcm-fee for reserve chain by multilocation.
    impl<T> GetByKey<MultiLocation, Option<u128>> for Pallet<T>
    where
        T: Config,
    {
        #[inline]
        fn get(location: &MultiLocation) -> Option<u128> {
            MinXcmFee::<T>::get(&T::Location::from(location.clone()))
        }
    }
}
