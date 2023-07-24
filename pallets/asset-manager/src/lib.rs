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
    use ascii::AsciiStr;
    use frame_support::{
        pallet_prelude::*,
        traits::{Contains, ExistenceRequirement, StorageVersion},
        transactional, PalletId,
    };
    use frame_system::pallet_prelude::*;
    use manta_primitives::assets::{
        self, AssetConfig, AssetIdLocationMap, AssetIdLpMap, AssetIdType, AssetMetadata,
        AssetRegistry, FungibleLedger, LocationType,
    };
    use orml_traits::GetByKey;
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, One,
            Zero,
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

    pub type BalanceOf<T> = <T as Config>::Balance;

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
            + Copy
            + Sync
            + Send;

        /// Balance Type
        type Balance: Default
            + Member
            + Parameter
            + TypeInfo
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + AtLeast32BitUnsigned
            + Copy;

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

        type SuspenderOrigin: EnsureOrigin<Self::RuntimeOrigin>;

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

    impl<T> AssetIdLpMap for Pallet<T>
    where
        T: Config,
    {
        fn lp_asset_id(
            asset_id0: &Self::AssetId,
            asset_id1: &Self::AssetId,
        ) -> Option<Self::AssetId> {
            AssetIdPairToLp::<T>::get((asset_id0, asset_id1))
        }

        fn lp_asset_pool(pool_id: &Self::AssetId) -> Option<Self::AssetId> {
            LpToAssetIdPair::<T>::get(pool_id).map(|_| *pool_id)
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

        /// A LP asset was registered
        LPAssetRegistered {
            /// Asset Id of new Asset
            asset_id0: T::AssetId,

            /// Asset Id of new Asset
            asset_id1: T::AssetId,

            /// Asset Id of new Asset
            asset_id: T::AssetId,

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

        /// An asset location has been filtered from outgoing transfers
        AssetLocationFilteredForOutgoingTransfers {
            /// The asset location which can't be transferred out
            filtered_location: T::Location,
        },

        /// An asset location has been unfiltered from outgoing transfers
        AssetLocationUnfilteredForOutgoingTransfers {
            /// The asset location which can be transferred out
            filtered_location: T::Location,
        },
    }

    /// Asset Manager Error
    #[pallet::error]
    pub enum Error<T> {
        /// Location Already Exists
        LocationAlreadyExists,

        /// An error occurred while creating a new asset at the [`AssetRegistry`].
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

        /// Two asset that used for generate LP asset should different
        AssetIdNotDifferent,

        /// Two asset that used for generate LP asset should exist
        AssetIdNotExist,

        InvalidTokenName,
        InvalidDecimals,
        ContractError,
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

    /// Multilocation of assets that should not be transfered out of the chain
    #[pallet::storage]
    #[pallet::getter(fn get_filtered_location)]
    pub type FilteredOutgoingAssetLocations<T: Config> =
        StorageMap<_, Blake2_128Concat, Option<MultiLocation>, ()>;

    /// AssetId pair to LP asset id mapping.
    #[pallet::storage]
    #[pallet::getter(fn asset_id_pair_to_lp)]
    pub(super) type AssetIdPairToLp<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AssetId, T::AssetId), T::AssetId>;

    /// LP asset id to asset id pair mapping.
    #[pallet::storage]
    #[pallet::getter(fn lp_to_asset_id_pair)]
    pub(super) type LpToAssetIdPair<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AssetId, (T::AssetId, T::AssetId)>;

    /// used by the chainbridge. access should be permissioned
    #[pallet::storage]
    #[pallet::getter(fn get_token_from_chainbridge)]
    pub type AssetByContract<T: Config> =
        StorageMap<_, Blake2_128Concat, (ChainId, Vec<u8>), T::AssetId, OptionQuery>;

    /// used by the octbridge. the chainid is omited. avoid to use the storage directly in case mess everything
    #[pallet::storage]
    #[pallet::getter(fn get_token_from_octopus)]
    pub type TokenByName<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, T::AssetId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_token_info)]
    pub type Tokens<T: Config> =
        StorageMap<_, Twox64Concat, T::AssetId, XToken<BalanceOf<T>>, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new asset in the asset manager.
        ///
        /// * `origin`: Caller of this extrinsic, the access control is specified by `ModifierOrigin`.
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

            let asset_id = Self::do_register_asset(Some(&location), &metadata)?;

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
        /// * `origin`: Caller of this extrinsic, the access control is specified by `ModifierOrigin`.
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

        /// Set min xcm fee for asset/s on their reserve chain.
        ///
        /// * `origin`: Caller of this extrinsic, the access control is specified by `ForceOrigin`.
        /// * `filtered_location`: Multilocation to be filtered.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::update_outgoing_filtered_assets())]
        #[transactional]
        pub fn update_outgoing_filtered_assets(
            origin: OriginFor<T>,
            filtered_location: T::Location,
            should_add: bool,
        ) -> DispatchResult {
            T::SuspenderOrigin::ensure_origin(origin)?;
            if should_add {
                FilteredOutgoingAssetLocations::<T>::insert(filtered_location.clone().into(), ());
                Self::deposit_event(Event::<T>::AssetLocationFilteredForOutgoingTransfers {
                    filtered_location,
                });
            } else {
                FilteredOutgoingAssetLocations::<T>::remove(filtered_location.clone().into());
                Self::deposit_event(Event::<T>::AssetLocationUnfilteredForOutgoingTransfers {
                    filtered_location,
                });
            }
            Ok(())
        }

        /// Register a LP(liquidity provider) asset in the asset manager based on two given already exist asset.
        ///
        /// * `origin`: Caller of this extrinsic, the access control is specified by `ModifierOrigin`.
        /// * `asset_0`: First assetId.
        /// * `asset_1`: Second assetId.
        /// * `location`: Location of the LP asset.
        /// * `metadata`: LP Asset metadata.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::register_lp_asset())]
        #[transactional]
        pub fn register_lp_asset(
            origin: OriginFor<T>,
            asset_0: T::AssetId,
            asset_1: T::AssetId,
            metadata: <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(asset_0 != asset_1, Error::<T>::AssetIdNotDifferent);
            ensure!(
                AssetIdLocation::<T>::contains_key(asset_0)
                    && AssetIdLocation::<T>::contains_key(asset_1),
                Error::<T>::AssetIdNotExist
            );

            let (asset_id0, asset_id1) = Self::sort_asset_id(asset_0, asset_1);
            ensure!(
                !AssetIdPairToLp::<T>::contains_key((&asset_id0, &asset_id1)),
                Error::<T>::AssetAlreadyRegistered
            );

            let asset_id = Self::do_register_asset(None, &metadata)?;

            AssetIdPairToLp::<T>::insert((asset_id0, asset_id1), asset_id);
            LpToAssetIdPair::<T>::insert(asset_id, (asset_id0, asset_id1));

            Self::deposit_event(Event::<T>::LPAssetRegistered {
                asset_id0,
                asset_id1,
                asset_id,
                metadata,
            });
            Ok(())
        }

        #[pallet::weight(0)]
        #[pallet::call_index(8)]
        #[transactional]
        pub fn associate_asset(
            origin: OriginFor<T>,
            chain_id: ChainId,
            contract_id: Vec<u8>,
            asset_id: T::AssetId,
        ) -> DispatchResult {
            let _ = T::ModifierOrigin::ensure_origin(origin)?;
            AssetByContract::<T>::insert((chain_id, contract_id), asset_id);
            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        /// Register asset by providing optional location and metadata.
        pub fn do_register_asset(
            location: Option<&T::Location>,
            metadata: &<T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata,
        ) -> Result<T::AssetId, DispatchError> {
            if let Some(location) = location {
                ensure!(
                    !LocationAssetId::<T>::contains_key(location),
                    Error::<T>::LocationAlreadyExists
                );
            }
            let asset_id = Self::next_asset_id_and_increment()?;
            <T::AssetConfig as AssetConfig<T>>::AssetRegistry::create_asset(
                asset_id,
                metadata.clone().into(),
                metadata.min_balance().clone(),
                metadata.is_sufficient(),
            )
            .map_err(|_| Error::<T>::ErrorCreatingAsset)?;
            AssetIdMetadata::<T>::insert(asset_id, metadata);
            if let Some(location) = location {
                AssetIdLocation::<T>::insert(asset_id, location);
                LocationAssetId::<T>::insert(location, asset_id);
            }
            Ok(asset_id)
        }

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

        pub fn check_outgoing_assets_filter(asset_location: &Option<MultiLocation>) -> bool {
            FilteredOutgoingAssetLocations::<T>::contains_key(asset_location)
        }

        /// Sorted the assets pair
        pub fn sort_asset_id(asset_0: T::AssetId, asset_1: T::AssetId) -> (T::AssetId, T::AssetId) {
            if asset_0 < asset_1 {
                (asset_0, asset_1)
            } else {
                (asset_1, asset_0)
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

    use fuso_support::{
        chainbridge::{AssetIdResourceIdProvider, EthereumCompatibleAddress},
        constants::{MAX_DECIMALS, STANDARD_DECIMALS},
        external_chain::XToken,
        traits::{DecimalsTransformer, PriceOracle, Token},
        ChainId,
    };

    /// used by the chainbridge
    impl<T: Config> fuso_support::chainbridge::AssetIdResourceIdProvider<T::AssetId> for Pallet<T> {
        type Err = ();

        fn try_get_asset_id(
            chain_id: ChainId,
            contract_id: impl AsRef<[u8]>,
        ) -> Result<T::AssetId, Self::Err> {
            Self::get_token_from_chainbridge((chain_id, contract_id.as_ref().to_vec())).ok_or(())
        }
    }

    impl<T: Config> Token<T::AccountId> for Pallet<T> {
        type Balance = T::Balance;
        type TokenId = T::AssetId;

        #[transactional]
        fn create(mut token_info: XToken<T::Balance>) -> Result<Self::TokenId, DispatchError> {
            let id = Self::next_asset_id_and_increment().unwrap();
            match token_info {
                XToken::NEP141(
                    ref symbol,
                    ref contract,
                    ref mut total,
                    ref mut stable,
                    decimals,
                ) => {
                    ensure!(decimals <= MAX_DECIMALS, Error::<T>::InvalidDecimals);
                    let name = AsciiStr::from_ascii(&symbol);
                    ensure!(name.is_ok(), Error::<T>::InvalidTokenName);
                    let name = name.unwrap();
                    ensure!(
                        name.len() >= 2 && name.len() <= 8,
                        Error::<T>::InvalidTokenName
                    );
                    ensure!(
                        contract.len() < 120 && contract.len() > 2,
                        Error::<T>::ContractError
                    );
                    ensure!(
                        !TokenByName::<T>::contains_key(&contract),
                        Error::<T>::ContractError
                    );
                    *total = Zero::zero();
                    *stable = false;
                    TokenByName::<T>::insert(contract.clone(), id);
                }
                XToken::ERC20(
                    ref symbol,
                    ref contract,
                    ref mut total,
                    ref mut stable,
                    decimals,
                )
                | XToken::POLYGON(
                    ref symbol,
                    ref contract,
                    ref mut total,
                    ref mut stable,
                    decimals,
                )
                | XToken::BEP20(
                    ref symbol,
                    ref contract,
                    ref mut total,
                    ref mut stable,
                    decimals,
                ) => {
                    ensure!(decimals <= MAX_DECIMALS, Error::<T>::InvalidDecimals);
                    let name = AsciiStr::from_ascii(&symbol);
                    ensure!(name.is_ok(), Error::<T>::InvalidTokenName);
                    let name = name.unwrap();
                    ensure!(
                        name.len() >= 2 && name.len() <= 8,
                        Error::<T>::InvalidTokenName
                    );
                    ensure!(contract.len() == 20, Error::<T>::ContractError);
                    *total = Zero::zero();
                    *stable = false;
                }
                XToken::FND10(ref symbol, ref mut total) => {
                    let name = AsciiStr::from_ascii(&symbol);
                    ensure!(name.is_ok(), Error::<T>::InvalidTokenName);
                    let name = name.unwrap();
                    ensure!(
                        name.len() >= 2 && name.len() <= 8,
                        Error::<T>::InvalidTokenName
                    );
                    *total = Zero::zero();
                }
            }

            // TODO:
            // update our current asset-id maps
            // increment next-asset-id properly

            Tokens::<T>::insert(id, token_info);

            Ok(id)
        }

        #[transactional]
        fn transfer_token(
            origin: &T::AccountId,
            token: Self::TokenId,
            amount: Self::Balance,
            target: &T::AccountId,
        ) -> Result<Self::Balance, DispatchError> {
            // if amount.is_zero() {
            //     return Ok(amount);
            // }
            // if origin == target {
            //     return Ok(amount);
            // }
            // if token == Self::native_token_id() {
            //     return <pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(
            //         origin,
            //         target,
            //         amount,
            //         ExistenceRequirement::KeepAlive,
            //     )
            //     .map(|_| amount);
            // }
            // Balances::<T>::try_mutate_exists((&token, &origin), |from| -> DispatchResult {
            //     ensure!(from.is_some(), Error::<T>::BalanceZero);
            //     let mut account = from.take().unwrap();
            //     account.free = account
            //         .free
            //         .checked_sub(&amount)
            //         .ok_or(Error::<T>::InsufficientBalance)?;
            //     match account.free == Zero::zero() && account.reserved == Zero::zero() {
            //         true => {}
            //         false => {
            //             from.replace(account);
            //         }
            //     }
            //     Balances::<T>::try_mutate_exists((&token, &target), |to| -> DispatchResult {
            //         let mut account = to.take().unwrap_or(TokenAccountData {
            //             free: Zero::zero(),
            //             reserved: Zero::zero(),
            //         });
            //         account.free = account
            //             .free
            //             .checked_add(&amount)
            //             .ok_or(Error::<T>::Overflow)?;
            //         to.replace(account);
            //         Ok(())
            //     })?;
            //     Ok(())
            // })?;
            // Self::deposit_event(Event::TokenTransfered(
            //     token,
            //     origin.clone(),
            //     target.clone(),
            //     amount,
            // ));
            Ok(amount)
        }

        fn try_mutate_account<R>(
            token: &Self::TokenId,
            who: &T::AccountId,
            f: impl FnOnce(&mut (Self::Balance, Self::Balance)) -> Result<R, DispatchError>,
        ) -> Result<R, DispatchError> {
            // if *token == Self::native_token_id() {
            // // We can just use transfer to special account instead of reserving/unreserving

            // pallet_balances::Pallet::<T>::mutate_account(who, |b| -> Result<R, DispatchError> {
            //     let mut v = (b.free, b.reserved);
            //     let r = f(&mut v)?;
            //     b.free = v.0;
            //     b.reserved = v.1;
            //     Ok(r)
            // })?
            // } else {
            // // We can just use transfer to reserve account instead of reserving/unreserving

            // Balances::<T>::try_mutate_exists((token, who), |t| -> Result<R, DispatchError> {
            //     let mut b = t.take().unwrap_or_default();
            //     let mut v = (b.free, b.reserved);
            //     let r = f(&mut v)?;
            //     b.free = v.0;
            //     b.reserved = v.1;
            //     match b.free == Zero::zero() && b.reserved == Zero::zero() {
            //         true => {}
            //         false => {
            //             t.replace(b);
            //         }
            //     }
            //     Ok(r)
            // })
            //}

            let mut v = (
                <T as Config>::Balance::from(1u32),
                <T as Config>::Balance::from(1u32),
            );
            let r = f(&mut v)?;
            Ok(r)
        }

        fn try_mutate_issuance(
            token: &Self::TokenId,
            f: impl FnOnce(&mut Self::Balance) -> Result<(), DispatchError>,
        ) -> Result<(), DispatchError> {
            // if *token == Self::native_token_id() {
            //     <pallet_balances::TotalIssuance<T>>::try_mutate(|total| f(total))
            // } else {
            //     Err(DispatchError::Other("can't update the token issuance"))
            // }
            Err(DispatchError::Other("can't update the token issuance"))
        }

        fn exists(token: &Self::TokenId) -> bool {
            //*token == Self::native_token_id() || Tokens::<T>::contains_key(token)
            true
        }

        fn native_token_id() -> Self::TokenId {
            //T::NativeTokenId::get()
            <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()
        }

        fn is_stable(token: &Self::TokenId) -> bool {
            // if *token == Self::native_token_id() {
            //     false
            // } else {
            //     Self::get_token_info(token)
            //         .map(|t| t.is_stable())
            //         .unwrap_or(false)
            // }
            true
        }

        fn free_balance(token: &Self::TokenId, who: &T::AccountId) -> Self::Balance {
            // if *token == Self::native_token_id() {
            //     return pallet_balances::Pallet::<T>::free_balance(who);
            // }
            // Self::get_token_balance((token, who)).free
            <T as Config>::Balance::from(1u32)
        }

        fn total_issuance(token: &Self::TokenId) -> Self::Balance {
            // if *token == Self::native_token_id() {
            //     return pallet_balances::Pallet::<T>::total_issuance();
            // }
            // let token_info = Self::get_token_info(token);
            // if token_info.is_some() {
            //     let token = token_info.unwrap();
            //     match token {
            //         XToken::NEP141(_, _, total, _, _)
            //         | XToken::ERC20(_, _, total, _, _)
            //         | XToken::POLYGON(_, _, total, _, _)
            //         | XToken::BEP20(_, _, total, _, _) => total,
            //         XToken::FND10(_, total) => total,
            //     }
            // } else {
            //     Zero::zero()
            // }
            <T as Config>::Balance::from(1u32)
        }

        fn token_external_decimals(token: &Self::TokenId) -> Result<u8, DispatchError> {
            // if *token == Self::native_token_id() {
            //     return Ok(STANDARD_DECIMALS);
            // }
            // let token_info = Self::get_token_info(token);
            // if token_info.is_some() {
            //     let token = token_info.unwrap();
            //     match token {
            //         XToken::NEP141(_, _, _, _, decimals)
            //         | XToken::ERC20(_, _, _, _, decimals)
            //         | XToken::POLYGON(_, _, _, _, decimals)
            //         | XToken::BEP20(_, _, _, _, decimals) => Ok(decimals),
            //         XToken::FND10(_, _) => Err(Error::<T>::TokenNotFound.into()),
            //     }
            // } else {
            //     Err(Error::<T>::TokenNotFound.into())
            // }
            Ok(0u8)
        }
    }

    impl<T: Config> DecimalsTransformer<T::Balance> for Pallet<T>
    where
        T::Balance: From<u128> + Into<u128>,
    {
        fn transform_decimals_to_standard(amount: T::Balance, external_decimals: u8) -> T::Balance
        where
            T::Balance: From<u128> + Into<u128>,
        {
            let mut amount: u128 = amount.into();
            if external_decimals > STANDARD_DECIMALS {
                let diff = external_decimals - STANDARD_DECIMALS;
                for _i in 0..diff {
                    amount /= 10
                }
            } else {
                let diff = STANDARD_DECIMALS - external_decimals;
                for _i in 0..diff {
                    amount *= 10
                }
            }
            amount.into()
        }

        fn transform_decimals_to_external(amount: T::Balance, external_decimals: u8) -> T::Balance {
            let mut amount: u128 = amount.into();
            if external_decimals > STANDARD_DECIMALS {
                let diff = external_decimals - STANDARD_DECIMALS;
                for _i in 0..diff {
                    amount *= 10
                }
            } else {
                let diff = STANDARD_DECIMALS - external_decimals;
                for _i in 0..diff {
                    amount /= 10
                }
            }
            amount.into()
        }
    }
}
