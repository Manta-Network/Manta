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

//! # MantaSBT Module
//!
//! MantaSBT creates non-transferable nfts as unspendable UTXOs
//!
//! ## Overview
//!
//! Uses `pallet-uniques` to store NFT data. NFTs are created and Ownership is recorded as an UTXO

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]

extern crate alloc;

use alloc::vec;
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::*,
    traits::{
        tokens::nonfungibles::Mutate, Currency, ExistenceRequirement, ReservableCurrency,
        StorageVersion,
    },
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use manta_support::manta_pay::{
    id_from_field, AssetType, IncrementAssetId, PostToLedger, StandardAssetId, TransferPost,
};
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    ArithmeticError,
};

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

/// This is needed because ItemId is generic and doesn't have Numeric traits implemented
pub trait ItemIdConvert<ItemId> {
    fn asset_id_to_item_id(asset_id: StandardAssetId) -> ItemId;
}

/// Type alias for currency balance.
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// MantaSBT Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Pallet
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// The module configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_uniques::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// SBT CollectionId
        type PalletCollectionId: Get<Self::CollectionId>;

        /// Converts to ItemId type
        type ConvertItemId: ItemIdConvert<Self::ItemId>;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Pallet ID
        type PalletId: Get<PalletId>;

        /// Number of mints reserved per `reserve_sbt` call
        #[pallet::constant]
        type MintsPerReserve: Get<u16>;

        /// Price to reserve
        type ReservePrice: Get<BalanceOf<Self>>;

        /// Ledger to Post
        type Ledger: PostToLedger<Self::AccountId>;

        /// Increment Asset Id
        type IncrementAssetId: IncrementAssetId<StandardAssetId>;
    }

    /// Whitelists accounts to be able to mint SBTs with designated `StandardAssetId`
    #[pallet::storage]
    pub type ReservedIds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (StandardAssetId, StandardAssetId),
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mints a zkSBT
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::to_private())]
        #[transactional]
        pub fn to_private(
            origin: OriginFor<T>,
            post: TransferPost,
            metadata: BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            // Only one UTXO allowed to be inserted per transaction
            ensure!(
                post.sources.len() == 1
                    && post.sender_posts.is_empty()
                    && post.receiver_posts.len() == 1
                    && post.sinks.is_empty(),
                Error::<T>::NoSenderLedger
            );

            let (start_id, end_id) =
                ReservedIds::<T>::get(&origin).ok_or(Error::<T>::NotReserved)?;
            let asset_id: StandardAssetId = post
                .asset_id
                .map(id_from_field)
                .ok_or(Error::<T>::InvalidAssetId)?
                .ok_or(Error::<T>::InvalidAssetId)?;
            // Ensure asset id is correct
            ensure!(asset_id == start_id, Error::<T>::InvalidAssetId);

            pallet_uniques::Pallet::<T>::mint_into(
                &T::PalletCollectionId::get(),
                &T::ConvertItemId::asset_id_to_item_id(start_id),
                &Self::account_id(),
            )?;

            pallet_uniques::Pallet::<T>::set_metadata(
                frame_system::RawOrigin::Root.into(),
                T::PalletCollectionId::get(),
                T::ConvertItemId::asset_id_to_item_id(start_id),
                metadata,
                true,
            )?;

            // Increments id by one, remove from storage if reserved asset_ids are exhausted
            let increment_start_id = start_id
                .checked_add(One::one())
                .ok_or(ArithmeticError::Overflow)?;
            if increment_start_id == end_id {
                ReservedIds::<T>::remove(&origin)
            } else {
                ReservedIds::<T>::insert(&origin, (increment_start_id, end_id))
            }

            Self::deposit_event(Event::<T>::MintSbt {
                source: origin.clone(),
                asset: asset_id,
            });
            T::Ledger::post_transaction(None, vec![origin], vec![], post, AssetType::SBT)
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::reserve_sbt())]
        #[transactional]
        pub fn reserve_sbt(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <T as pallet::Config>::Currency::transfer(
                &who,
                &Self::account_id(),
                T::ReservePrice::get(),
                ExistenceRequirement::KeepAlive,
            )?;
            let first_id = T::IncrementAssetId::next_asset_id_and_increment()?;
            let mut stop_id: StandardAssetId = 0;
            for _ in 1..T::MintsPerReserve::get() {
                stop_id = T::IncrementAssetId::next_asset_id_and_increment()?;
            }
            // Increment by one because it stops here, will not submit a post with this asset_id
            stop_id = stop_id
                .checked_add(One::one())
                .ok_or(ArithmeticError::Overflow)?;

            ReservedIds::<T>::insert(&who, (first_id, stop_id));
            Self::deposit_event(Event::<T>::SBTReserved {
                who,
                start_id: first_id,
                stop_id,
            });
            Ok(())
        }
    }

    /// Event
    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Mint SBT Event
        MintSbt {
            /// AssetId on private leger
            asset: StandardAssetId,

            /// Source Account
            source: T::AccountId,
        },
        /// Reserve SBT
        SBTReserved {
            /// Public Account reserving sbt mints
            who: T::AccountId,
            /// Start of reserved ids
            start_id: StandardAssetId,
            /// end of reserved ids
            stop_id: StandardAssetId,
        },
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T>(PhantomData<T>);

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self(std::marker::PhantomData::<T>)
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        #[inline]
        fn build(&self) {
            pallet_uniques::Pallet::<T>::do_create_collection(
                T::PalletCollectionId::get(),
                Pallet::<T>::account_id(),
                Pallet::<T>::account_id(),
                Zero::zero(),
                true,
                pallet_uniques::Event::<T>::ForceCreated {
                    collection: T::PalletCollectionId::get(),
                    owner: Pallet::<T>::account_id(),
                },
            )
            .expect("create SBT collection on genesis failed");
        }
    }

    /// Error
    #[pallet::error]
    pub enum Error<T> {
        /// Invalid Asset Id
        ///
        /// The asset id of the transfer could not be converted correctly to the standard format.
        InvalidAssetId,

        /// No Sender Ledger in SBT, Private Transfers are disabled
        NoSenderLedger,

        /// Need to first reserve SBT before minting
        NotReserved,
    }
}

impl<T: Config> Pallet<T> {
    /// Returns the account ID of this pallet.
    #[inline]
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }
}
