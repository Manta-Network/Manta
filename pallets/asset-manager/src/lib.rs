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
        traits::{tokens::ExistenceRequirement, Contains, StorageVersion},
        transactional, PalletId,
    };
    use frame_system::pallet_prelude::*;
    use manta_primitives::{
        assets::{
            self, AssetConfig, AssetIdLocationMap, AssetIdLpMap, AssetIdType, AssetMetadata,
            AssetRegistry, AssetRegistryMetadata, AssetStorageMetadata, FungibleLedger,
            LocationType,
        },
        types::Balance,
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

    /// Used to set the minimum balance for permissionless assets.
    pub const POSSIBLE_ACCOUNTS_PER_ASSET: Balance = 10_000_000_000;

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
            Balance = Balance,
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

        /// AssetId where Permissionless Assets start, must be greater than `StartNonNativeAssetId`
        type PermissionlessStartId: Get<Self::AssetId>;

        /// Max length of token name
        type TokenNameMaxLen: Get<u32>;

        /// Max length of token symbol
        type TokenSymbolMaxLen: Get<u32>;

        /// Cost of registering a permissionless asset in native token
        type PermissionlessAssetRegistryCost: Get<Balance>;
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
            assert!(
                T::PermissionlessStartId::get()
                    > <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(),
                "Permissionless start id must be greater than non native id start"
            );
            NextAssetId::<T>::set(self.start_id);
            let asset_id = <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get();
            let metadata = <T::AssetConfig as AssetConfig<T>>::NativeAssetMetadata::get();
            let location = <T::AssetConfig as AssetConfig<T>>::NativeAssetLocation::get();
            AssetIdLocation::<T>::insert(asset_id, &location);
            AssetIdMetadata::<T>::insert(asset_id, metadata);
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
            metadata: AssetRegistryMetadata<Balance>,
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
            metadata: AssetRegistryMetadata<Balance>,
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
            metadata: AssetRegistryMetadata<Balance>,
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
            amount: Balance,
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
        /// A new asset was registered
        PermissionlessAssetRegistered {
            /// Asset Id of new Asset
            asset_id: T::AssetId,

            /// Metadata Registered to Asset Manager
            metadata: AssetRegistryMetadata<Balance>,
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

        /// AssetIdOverflow
        AssetIdOverflow,

        /// Account does not have enough native funds
        NotEnoughNativeFunds,

        /// Total Supply is less than decimal value
        TotalSupplyTooLow,

        /// Decimals cannot be set to zero
        DecimalIsZero,
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
    pub(super) type AssetIdMetadata<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AssetId, AssetRegistryMetadata<Balance>>;

    /// The Next Available [`AssetId`](AssetConfig::AssetId)
    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

    /// The Next Available Permissionless [`AssetId`](AssetConfig::AssetId)
    #[pallet::storage]
    #[pallet::getter(fn next_permissionless_asset_id)]
    pub type NextPermissionlessAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

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
            metadata: AssetRegistryMetadata<Balance>,
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
        /// * `metadata`: new `metadata` to be associated with `asset_id`, note `is_frozen`
        /// flag in metadata will have no effect and and cannot be changed.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_asset_metadata())]
        #[transactional]
        pub fn update_asset_metadata(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            metadata: AssetStorageMetadata,
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
                metadata.clone(),
            )?;

            let mut registered_metadata =
                AssetIdMetadata::<T>::get(asset_id).ok_or(Error::<T>::UpdateNonExistentAsset)?;
            let new_metadata = AssetStorageMetadata {
                name: metadata.name,
                symbol: metadata.symbol,
                decimals: metadata.decimals,
                // is frozen flag doesn't do anything in metadata
                is_frozen: registered_metadata.metadata.is_frozen,
            };
            registered_metadata.metadata = new_metadata;

            AssetIdMetadata::<T>::insert(asset_id, &registered_metadata);
            Self::deposit_event(Event::<T>::AssetMetadataUpdated {
                asset_id,
                metadata: registered_metadata,
            });
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
            amount: Balance,
        ) -> DispatchResult {
            T::ModifierOrigin::ensure_origin(origin)?;
            ensure!(
                AssetIdLocation::<T>::contains_key(asset_id),
                Error::<T>::UpdateNonExistentAsset
            );
            <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(
                asset_id,
                &beneficiary,
                amount,
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
            metadata: AssetRegistryMetadata<Balance>,
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

        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::permissionless_register_asset())]
        #[transactional]
        pub fn permissionless_register_asset(
            origin: OriginFor<T>,
            name: BoundedVec<u8, T::TokenNameMaxLen>,
            symbol: BoundedVec<u8, T::TokenSymbolMaxLen>,
            decimals: u8,
            total_supply: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;
            let decimal_num =
                u128::checked_pow(10, decimals.into()).ok_or(ArithmeticError::Overflow)?;

            ensure!(total_supply >= decimal_num, Error::<T>::TotalSupplyTooLow);
            ensure!(!decimals.is_zero(), Error::<T>::DecimalIsZero);

            let native_asset_id = <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get();
            <T::AssetConfig as AssetConfig<T>>::FungibleLedger::transfer(
                native_asset_id,
                &who,
                &Self::account_id(),
                T::PermissionlessAssetRegistryCost::get(),
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::NotEnoughNativeFunds)?;

            let asset_id = Self::next_permissionless_asset_id_and_increment()?;
            let mut min_balance: Balance = total_supply
                .checked_div(POSSIBLE_ACCOUNTS_PER_ASSET)
                .ok_or(ArithmeticError::DivisionByZero)?;
            if min_balance.is_zero() {
                min_balance = One::one();
            }

            let metadata = AssetStorageMetadata {
                name: name.into(),
                symbol: symbol.into(),
                decimals,
                is_frozen: false,
            };
            // create asset and mint total supply to creator
            <T::AssetConfig as AssetConfig<T>>::AssetRegistry::create_asset(
                asset_id,
                metadata.clone(),
                min_balance,
                true,
            )
            .map_err(|_| Error::<T>::ErrorCreatingAsset)?;

            let register_metadata = AssetRegistryMetadata::<Balance> {
                metadata,
                min_balance,
                is_sufficient: true,
            };
            AssetIdMetadata::<T>::insert(asset_id, &register_metadata);
            <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(
                asset_id,
                &who,
                total_supply,
                true,
            )
            .map_err(|_| Error::<T>::MintError)?;

            Self::deposit_event(Event::<T>::PermissionlessAssetRegistered {
                asset_id,
                metadata: register_metadata,
            });
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
            metadata: &AssetRegistryMetadata<Balance>,
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
                *metadata.min_balance(),
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

        /// Returns and increments the [`NextAssetId`] by one. Fails if it hits the upper limit of `PermissionlessStartId`
        #[inline]
        pub(super) fn next_asset_id_and_increment() -> Result<T::AssetId, DispatchError> {
            NextAssetId::<T>::try_mutate(|current| {
                if *current >= T::PermissionlessStartId::get() {
                    Err(Error::<T>::AssetIdOverflow.into())
                } else {
                    let id = *current;
                    *current = current
                        .checked_add(&One::one())
                        .ok_or(ArithmeticError::Overflow)?;
                    Ok(id)
                }
            })
        }

        /// Returns and increments the [`NextPermssionlessAssetId`] by one.
        #[inline]
        pub(super) fn next_permissionless_asset_id_and_increment(
        ) -> Result<T::AssetId, DispatchError> {
            NextPermissionlessAssetId::<T>::try_mutate(|current| {
                if current.is_zero() {
                    let id = T::PermissionlessStartId::get();
                    *current = id
                        .checked_add(&One::one())
                        .ok_or(ArithmeticError::Overflow)?;
                    Ok(id)
                } else {
                    let id = *current;
                    *current = current
                        .checked_add(&One::one())
                        .ok_or(ArithmeticError::Overflow)?;
                    Ok(id)
                }
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
}
