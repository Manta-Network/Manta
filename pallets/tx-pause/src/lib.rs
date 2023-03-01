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
//
// The pallet-tx-pause pallet is forked from Acala's transaction-pause module https://github.com/AcalaNetwork/Acala/tree/master/modules/transaction-pause
// The original license is the following - SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
    dispatch::{CallMetadata, GetCallMetadata},
    pallet_prelude::*,
    traits::{Contains, PalletInfoAccess},
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};

mod mock;
mod tests;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

type CallOf<T> = <T as Config>::RuntimeCall;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::StorageVersion;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type RuntimeCall: Parameter + GetCallMetadata;

        type MaxCallNames: Get<u32>;

        /// The origin which may add to filter.
        type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The origin which may remove from filter.
        type UnpauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Names of pallets which cannot be paused.
        type NonPausablePallets: Contains<Vec<u8>>;

        /// Weight information for the extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// can not pause
        CannotPause,
        /// invalid character encoding
        InvalidCharacter,
        /// call of pallet too many
        TooManyCalls,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Paused transaction . \[pallet_name_bytes, function_name_bytes\]
        TransactionPaused(Vec<u8>, Vec<u8>),
        /// Unpaused transaction . \[pallet_name_bytes, function_name_bytes\]
        TransactionUnpaused(Vec<u8>, Vec<u8>),
        /// Paused pallet
        PalletPaused(Vec<u8>),
        /// Unpaused pallet
        PalletUnpaused(Vec<u8>),
    }

    /// The paused transaction map
    ///
    /// map (PalletNameBytes, FunctionNameBytes) => Option<()>
    #[pallet::storage]
    #[pallet::getter(fn paused_transactions)]
    pub type PausedTransactions<T: Config> =
        StorageMap<_, Twox64Concat, (Vec<u8>, Vec<u8>), (), OptionQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Pause an extrinsic by passing the extrinsic and corresponding pallet names.
        /// Use names as they are written in the source code of the pallet.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::pause_transaction())]
        #[transactional]
        pub fn pause_transaction(
            origin: OriginFor<T>,
            pallet_name: Vec<u8>,
            function_name: Vec<u8>,
        ) -> DispatchResult {
            T::PauseOrigin::ensure_origin(origin)?;

            Self::ensure_can_pause(&pallet_name)?;

            Self::pause_one(&pallet_name, &function_name, true)?;

            Ok(())
        }

        /// Unpause an extrinsic by passing the extrinsic and corresponding pallet names.
        /// Use names as they are written in the source code of the pallet.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unpause_transaction())]
        #[transactional]
        pub fn unpause_transaction(
            origin: OriginFor<T>,
            pallet_name: Vec<u8>,
            function_name: Vec<u8>,
        ) -> DispatchResult {
            T::UnpauseOrigin::ensure_origin(origin)?;

            Self::unpause_one(&pallet_name, &function_name)?;

            Ok(())
        }

        /// Pause extrinsics by passing the extrinsic and corresponding pallet names.
        /// Use names as they are written in the source code of the pallet.
        #[pallet::call_index(2)]
        #[pallet::weight({
            let len = pallet_and_funcs.iter().flat_map(|item| {item.clone().1}).count();
            T::WeightInfo::pause_transaction().saturating_mul(len as u64)
        })]
        #[transactional]
        pub fn pause_transactions(
            origin: OriginFor<T>,
            pallet_and_funcs: Vec<(Vec<u8>, Vec<Vec<u8>>)>,
        ) -> DispatchResult {
            T::PauseOrigin::ensure_origin(origin)?;

            for (pallet_name, function_name) in pallet_and_funcs {
                Self::ensure_can_pause(&pallet_name)?;

                for call_name in function_name {
                    Self::pause_one(&pallet_name, &call_name, true)?;
                }
            }

            Ok(())
        }

        /// Unpause extrinsics by passing the extrinsic and corresponding pallet names.
        /// Use names as they are written in the source code of the pallet.
        #[pallet::call_index(3)]
        #[pallet::weight({
            let len = pallet_and_funcs.iter().flat_map(|item| {item.clone().1}).count();
            T::WeightInfo::unpause_transaction().saturating_mul(len as u64)
        })]
        #[transactional]
        pub fn unpause_transactions(
            origin: OriginFor<T>,
            pallet_and_funcs: Vec<(Vec<u8>, Vec<Vec<u8>>)>,
        ) -> DispatchResult {
            T::UnpauseOrigin::ensure_origin(origin)?;

            for (pallet_name, function_name) in pallet_and_funcs {
                for call_name in function_name {
                    Self::unpause_one(&pallet_name, &call_name)?;
                }
            }

            Ok(())
        }

        /// Pause all the calls of the listed pallets in `pallet_names`.
        /// This logic is in its own extrinsic in order to not have to pause calls 1 by 1.
        #[pallet::call_index(4)]
        #[pallet::weight({
            let len = pallet_names.len();
            let max = T::MaxCallNames::get();
            T::WeightInfo::pause_transaction().saturating_mul(len as u64).saturating_mul(max as u64)
        })]
        #[transactional]
        pub fn pause_pallets(
            origin: OriginFor<T>,
            pallet_names: Vec<Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            T::PauseOrigin::ensure_origin(origin)?;
            let mut sum = 0;

            for pallet_name in pallet_names {
                Self::ensure_can_pause(&pallet_name)?;

                let pallet_name_string = sp_std::str::from_utf8(&pallet_name)
                    .map_err(|_| Error::<T>::InvalidCharacter)?;

                let function_name =
                    <CallOf<T> as GetCallMetadata>::get_call_names(pallet_name_string);
                ensure!(
                    function_name.len() < T::MaxCallNames::get() as usize,
                    Error::<T>::TooManyCalls
                );

                for call_name in function_name {
                    let call_name = call_name.as_bytes().to_vec();

                    Self::pause_one(&pallet_name, &call_name, false)?;

                    sum += 1;
                }

                // deposit event for each pallet
                Self::deposit_event(Event::PalletPaused(pallet_name));
            }

            Ok(Some(T::WeightInfo::pause_transaction().saturating_mul(sum as u64)).into())
        }

        /// Unpause all the calls of the listed pallets in `pallet_names`.
        /// This logic is in its own extrinsic in order to not have to pause calls 1 by 1.
        #[pallet::call_index(5)]
        #[pallet::weight({
            let len = pallet_names.len();
            let max = T::MaxCallNames::get();
            T::WeightInfo::pause_transaction().saturating_mul(len as u64).saturating_mul(max as u64)
        })]
        #[transactional]
        pub fn unpause_pallets(
            origin: OriginFor<T>,
            pallet_names: Vec<Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            T::UnpauseOrigin::ensure_origin(origin)?;
            let mut sum = 0;

            for pallet_name in pallet_names {
                let pallet_name_string = sp_std::str::from_utf8(&pallet_name)
                    .map_err(|_| Error::<T>::InvalidCharacter)?;

                let function_name =
                    <CallOf<T> as GetCallMetadata>::get_call_names(pallet_name_string);
                for call_name in function_name {
                    let call_name = call_name.as_bytes().to_vec();

                    PausedTransactions::<T>::take((&pallet_name, call_name));

                    sum += 1;
                }

                // deposit event for each pallet
                Self::deposit_event(Event::PalletUnpaused(pallet_name));
            }

            Ok(Some(T::WeightInfo::pause_transaction().saturating_mul(sum as u64)).into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn ensure_can_pause(pallet_name: &Vec<u8>) -> DispatchResult {
        let pallet_name_string =
            sp_std::str::from_utf8(pallet_name).map_err(|_| Error::<T>::InvalidCharacter)?;

        // not allowed to pause calls of this pallet to ensure safe
        ensure!(
            pallet_name_string != <Self as PalletInfoAccess>::name(),
            Error::<T>::CannotPause
        );

        // not allowed to pause `NonPausablePallets`
        ensure!(
            !T::NonPausablePallets::contains(pallet_name),
            Error::<T>::CannotPause
        );

        Ok(())
    }

    fn pause_one(
        pallet_name: &Vec<u8>,
        function_name: &Vec<u8>,
        deposit_event: bool,
    ) -> DispatchResult {
        PausedTransactions::<T>::mutate_exists((pallet_name, function_name), |maybe_paused| {
            if maybe_paused.is_none() {
                *maybe_paused = Some(());
                if deposit_event {
                    Self::deposit_event(Event::TransactionPaused(
                        pallet_name.clone(),
                        function_name.clone(),
                    ));
                }
            }
        });
        Ok(())
    }

    fn unpause_one(pallet_name: &Vec<u8>, function_name: &Vec<u8>) -> DispatchResult {
        if PausedTransactions::<T>::take((pallet_name, function_name)).is_some() {
            Self::deposit_event(Event::TransactionUnpaused(
                pallet_name.clone(),
                function_name.clone(),
            ));
        };
        Ok(())
    }
}

pub struct PausedTransactionFilter<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> Contains<CallOf<T>> for PausedTransactionFilter<T>
where
    CallOf<T>: GetCallMetadata,
{
    fn contains(call: &CallOf<T>) -> bool {
        let CallMetadata {
            function_name,
            pallet_name,
        } = call.get_call_metadata();
        PausedTransactions::<T>::contains_key((pallet_name.as_bytes(), function_name.as_bytes()))
    }
}
