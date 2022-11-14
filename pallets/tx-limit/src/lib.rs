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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use manta_primitives::assets::TransactionLimitation;
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchResult};

mod mock;
mod tests;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which may set asset limit.
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// Asset Id Type
        type AssetId: AtLeast32BitUnsigned
            + Default
            + Parameter
            + MaybeSerializeDeserialize
            + TypeInfo
            + Copy;

        /// Balance Type
        type Balance: AtLeast32BitUnsigned + Default + Member + Parameter + TypeInfo;

        /// Weight information for the extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Setting asset limit on each tx
        TransactionLimitSet {
            asset_id: T::AssetId,
            amount: T::Balance,
        },
        /// Unset asset limit on each tx
        TransactionLimitUnset { asset_id: T::AssetId },
    }

    /// The asset limitation map
    ///
    /// map AssetId => Option<Balance>
    #[pallet::storage]
    #[pallet::getter(fn asset_limits)]
    pub type AssetLimits<T: Config> =
        StorageMap<_, Twox64Concat, T::AssetId, T::Balance, ValueQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set transfer amount limit of specify asset
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_asset_limit())]
        #[transactional]
        pub fn set_asset_limit(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            amount: T::Balance,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            AssetLimits::<T>::insert(asset_id, amount.clone());
            Self::deposit_event(Event::TransactionLimitSet { asset_id, amount });

            Ok(())
        }

        /// Unset transfer amount limit of specify asset
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unset_asset_limit())]
        #[transactional]
        pub fn unset_asset_limit(origin: OriginFor<T>, asset_id: T::AssetId) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            AssetLimits::<T>::remove(asset_id);
            Self::deposit_event(Event::TransactionLimitUnset { asset_id });

            Ok(())
        }
    }
}

impl<T: Config> TransactionLimitation<T::AssetId, T::Balance> for Pallet<T> {
    fn ensure_valid(asset_id: T::AssetId, amount: T::Balance) -> bool {
        !AssetLimits::<T>::contains_key(asset_id) || amount < AssetLimits::<T>::get(asset_id)
    }
}
