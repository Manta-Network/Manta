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
//! Uses `pallet-asset-manager` to store SBT metadata. Ownership is recorded as a corresponding UTXO.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]

extern crate alloc;

use alloc::{boxed::Box, vec};
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency, StorageVersion},
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use manta_primitives::assets::{AssetMetadata, SbtBound, UpdateMetadata};
use manta_support::manta_pay::{
    id_from_field, AssetType, PostToLedger, StandardAssetId, TransferPost,
};
use sp_runtime::{
    traits::{AccountIdConversion, One},
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

// One in encoded form, used to check that value input in `ToPrivate` post is one
const ENCODED_ONE: [u8; 16] = 1u128.to_le_bytes();

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
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Balance Type
        type Balance: Default + Member + Parameter + TypeInfo;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Pallet ID
        type PalletId: Get<PalletId>;

        /// Number of unique Asset Ids reserved per `reserve_sbt` call, cannot be zero
        #[pallet::constant]
        type MintsPerReserve: Get<u16>;

        /// Price to reserve Asset Ids
        type ReservePrice: Get<BalanceOf<Self>>;

        /// Private Ledger to Post to
        type Ledger: PostToLedger<Self::AccountId>;

        /// Register SBTs to `AssetRegistry`
        type UpdateMetadata: UpdateMetadata<StandardAssetId, Self::Balance>;
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
        ///
        /// `TransferPost` is posted to private ledger and SBT metadata is stored onchain.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::to_private())]
        #[transactional]
        pub fn to_private(
            origin: OriginFor<T>,
            post: Box<TransferPost>,
            metadata: BoundedVec<u8, SbtBound>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            // Checks that it is indeed a to_private post with a value of 1
            ensure!(
                post.sources.len() == 1
                    && post.sender_posts.is_empty()
                    && post.receiver_posts.len() == 1
                    && post.sinks.is_empty(),
                Error::<T>::NoSenderLedger
            );
            // Checks that value is one, this is defensive as value is not used for SBT.
            ensure!(
                post.sources.first() == Some(&ENCODED_ONE),
                Error::<T>::ValueNotOne
            );

            let (start_id, end_id) =
                ReservedIds::<T>::get(&origin).ok_or(Error::<T>::NotReserved)?;
            let asset_id: StandardAssetId = post
                .asset_id
                .and_then(id_from_field)
                .ok_or(Error::<T>::InvalidAssetId)?;
            // Ensure asset id is correct, only a single unique asset_id mapped to account is valid
            ensure!(asset_id == start_id, Error::<T>::InvalidAssetId);

            // Updates SbtMetadata
            T::UpdateMetadata::update_metadata(&asset_id, AssetMetadata::SBT(metadata))?;
            // Increments id by one, remove from storage if reserved asset_ids are exhausted
            let increment_start_id = start_id
                .checked_add(One::one())
                .ok_or(ArithmeticError::Overflow)?;
            if increment_start_id == end_id {
                ReservedIds::<T>::remove(&origin)
            } else {
                ReservedIds::<T>::insert(&origin, (increment_start_id, end_id))
            }

            T::Ledger::post_transaction(None, vec![origin.clone()], vec![], *post, AssetType::SBT)?;
            Self::deposit_event(Event::<T>::MintSbt {
                source: origin,
                asset: asset_id,
            });
            Ok(().into())
        }

        /// Reserves AssetIds to be used subsequently in `to_private` above.
        ///
        /// Increments AssetManager's AssetId counter.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::reserve_sbt())]
        #[transactional]
        pub fn reserve_sbt(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Charges fee to reserve AssetIds
            <T as pallet::Config>::Currency::transfer(
                &who,
                &Self::account_id(),
                T::ReservePrice::get(),
                ExistenceRequirement::KeepAlive,
            )?;

            // Reserves uniques AssetIds to be used later to mint SBTs
            let start_id =
                T::UpdateMetadata::create_asset(AssetMetadata::SBT(BoundedVec::default()))?;
            for _ in 1..T::MintsPerReserve::get() {
                T::UpdateMetadata::create_asset(AssetMetadata::SBT(BoundedVec::default()))?;
            }
            // Asset_id to stop minting at, goes up to, but not including this value
            let stop_id = start_id
                .checked_add(T::MintsPerReserve::get().into())
                .ok_or(ArithmeticError::Overflow)?;

            ReservedIds::<T>::insert(&who, (start_id, stop_id));
            Self::deposit_event(Event::<T>::SBTReserved {
                who,
                start_id,
                stop_id,
            });
            Ok(())
        }
    }

    /// Event
    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Mints SBTs to private ledger
        MintSbt {
            /// AssetId on private leger
            asset: StandardAssetId,
            /// Source Account
            source: T::AccountId,
        },
        /// Reserve `AssetIds` as SBT
        SBTReserved {
            /// Public Account reserving SBT mints
            who: T::AccountId,
            /// Start of `AssetIds` reserved for use on private ledger
            start_id: StandardAssetId,
            /// End of `AssetIds` reserved for use private ledger, does not include this value
            stop_id: StandardAssetId,
        },
    }

    /// Error
    #[pallet::error]
    pub enum Error<T> {
        /// Invalid Asset Id
        ///
        /// The asset id of the `TransferPost` is incorrect. It could not be converted correctly to `StandardAssetId`
        /// or is not the designated `ReservedIds`
        InvalidAssetId,

        /// No Sender Ledger in SBT, Private Transfers are disabled
        NoSenderLedger,

        /// Need to first call `reserve_sbt` before minting
        NotReserved,

        /// `ToPrivate` post can only have value of 1. This is defensive to not allow large numbers on private ledger.
        ValueNotOne,
    }
}

impl<T: Config> Pallet<T> {
    /// Returns the account ID of this pallet.
    #[inline]
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }
}
