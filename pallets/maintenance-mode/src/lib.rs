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

// The pallet-maintenance-mode pallet is forked from Moonbeam: https://github.com/PureStake/moonbeam/tree/master/pallets/maintenance-mode
// The original license is the following - SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

use frame_support::pallet;
pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

#[pallet]
pub mod pallet {
    use cumulus_primitives_core::{
        relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler,
    };
    use frame_support::{
        pallet_prelude::*,
        traits::{
            Contains, EnsureOrigin, OffchainWorker, OnFinalize, OnIdle, OnInitialize,
            OnRuntimeUpgrade,
        },
    };
    use frame_system::pallet_prelude::*;
    use manta_primitives::{
        assets::{AssetFreezer, AssetIdQuerier},
        types::{AssetId, ParaId},
    };
    use sp_runtime::DispatchResult;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    /// Pause and resume execution of XCM
    pub trait PauseXcmExecution {
        fn suspend_xcm_execution() -> DispatchResult;
        fn resume_xcm_execution() -> DispatchResult;
    }

    impl PauseXcmExecution for () {
        fn suspend_xcm_execution() -> DispatchResult {
            Ok(())
        }
        fn resume_xcm_execution() -> DispatchResult {
            Ok(())
        }
    }

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type
        type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

        /// The base call filter to be used in normal operating mode
        /// (When we aren't in the middle of a migration)
        type NormalCallFilter: Contains<Self::Call>;

        /// The base call filter to be used when we are in the middle of migrations
        /// This should be very restrictive. Probably not allowing anything except possibly
        /// something like sudo or other emergency processes
        type MaintenanceCallFilter: Contains<Self::Call>;

        /// The origin from which the call to enter maintenance mode must come
        /// Take care when choosing your maintenance call filter to ensure that you'll still be
        /// able to return to normal mode. For example, if your EnterMaintenanceOrigin is a council, make
        /// sure that your councilors can still cast votes.
        type EnterMaintenanceOrigin: EnsureOrigin<Self::Origin>;

        /// The origin from which the call to exit maintenance mode.
        type ResumeNormalOrigin: EnsureOrigin<Self::Origin>;

        /// Handler to suspend and resume XCM execution
        type XcmExecutionManager: PauseXcmExecution;

        /// The DMP handler to be used in normal operating mode
        type NormalDmpHandler: DmpMessageHandler;

        /// The DMP handler to be used in maintenance mode
        type MaintenanceDmpHandler: DmpMessageHandler;

        /// The executive hooks that will be used in normal operating mode
        /// Important: Use `AllPalletsReversedWithSystemFirst` here if you dont want to modify the
        /// hooks behaviour
        type NormalExecutiveHooks: OnRuntimeUpgrade
            + OnInitialize<Self::BlockNumber>
            + OnIdle<Self::BlockNumber>
            + OnFinalize<Self::BlockNumber>
            + OffchainWorker<Self::BlockNumber>;

        /// The executive hooks that will be used in maintenance mode
        /// Important: Use `AllPalletsReversedWithSystemFirst` here if you dont want to modify the
        /// hooks behaviour
        type MaintenanceExecutiveHooks: OnRuntimeUpgrade
            + OnInitialize<Self::BlockNumber>
            + OnIdle<Self::BlockNumber>
            + OnFinalize<Self::BlockNumber>
            + OffchainWorker<Self::BlockNumber>;

        /// The asset freeze/thaw hook when enter sibling parachain hack mode or resume normal mode.
        type AssetFreezer: AssetFreezer;

        /// The asset should belong to registered parachain on asset manager.
        type AssetIdQuerier: AssetIdQuerier;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event {
        /// The chain was put into Maintenance Mode
        EnteredMaintenanceMode,
        /// Sibling was hacked.
        EnteredSiblingHackMode {
            id: ParaId,
            affected_assets: Vec<AssetId>,
        },
        /// Sibling was resumed.
        ResumedSiblingNormalMode {
            id: ParaId,
            affected_assets: Vec<AssetId>,
        },
        /// The chain returned to its normal operating state
        NormalOperationResumed,
        /// The call to suspend on_idle XCM execution failed with inner error
        FailedToSuspendIdleXcmExecution { error: DispatchError },
        /// The call to resume on_idle XCM execution failed with inner error
        FailedToResumeIdleXcmExecution { error: DispatchError },
    }

    /// An error that can occur while executing this pallet's extrinsics.
    #[pallet::error]
    pub enum Error<T> {
        /// The chain cannot enter maintenance mode because it is already in maintenance mode
        AlreadyInMaintenanceMode,
        /// The chain cannot resume normal operation because it is not in maintenance mode
        NotInMaintenanceMode,
        /// The sibling chain is already in hack mode
        AlreadyInSiblingHackMode,
        /// The sibling chain is not in hack mode
        SiblingNotHack,
        /// The parachain asset is not register to asset manager
        NoAssetRegistForParachain,
    }

    /// Whether the site is in maintenance mode.
    #[pallet::storage]
    #[pallet::getter(fn maintenance_mode)]
    type MaintenanceMode<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Sibling parachain is hacked, use Barrier to failed the cross chain transfer.
    #[pallet::storage]
    #[pallet::getter(fn hacked_sibling_id)]
    type HackedSiblingId<T: Config> = StorageMap<_, Blake2_128Concat, ParaId, bool, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: weight benchmark
        /// Place the chain in maintenance mode, Either we're hack or we manual enter maintain.
        ///
        /// Weight cost is:
        /// * One DB read to ensure we're not already in maintenance mode
        /// * Three DB writes - 1 for the mode, 1 for suspending xcm execution, 1 for the event
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().read + 3 * T::DbWeight::get().write)]
        pub fn enter_maintenance_mode(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::EnterMaintenanceOrigin::ensure_origin(origin)?;

            // Ensure we're not aleady in maintenance mode.
            // This test is not strictly necessary, but seeing the error may help a confused chain
            // operator during an emergency
            ensure!(
                !MaintenanceMode::<T>::get(),
                Error::<T>::AlreadyInMaintenanceMode
            );
            MaintenanceMode::<T>::put(true);

            // Suspend XCM execution
            if let Err(error) = T::XcmExecutionManager::suspend_xcm_execution() {
                <Pallet<T>>::deposit_event(Event::FailedToSuspendIdleXcmExecution { error });
            }

            <Pallet<T>>::deposit_event(Event::EnteredMaintenanceMode);
            Ok(().into())
        }

        /// Return the chain to normal operating mode
        ///
        /// Weight cost is:
        /// * One DB read to ensure we're in maintenance mode
        /// * Three DB writes - 1 for the mode, 1 for resuming xcm execution, 1 for the event
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().read + 3 * T::DbWeight::get().write)]
        pub fn resume_normal_operation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::ResumeNormalOrigin::ensure_origin(origin)?;

            // Ensure we're actually in maintenance mode.
            // This test is not strictly necessary, but seeing the error may help a confused chain
            // operator during an emergency
            ensure!(
                MaintenanceMode::<T>::get(),
                Error::<T>::NotInMaintenanceMode
            );
            MaintenanceMode::<T>::put(false);

            // Resume XCM execution
            if let Err(error) = T::XcmExecutionManager::resume_xcm_execution() {
                <Pallet<T>>::deposit_event(Event::FailedToResumeIdleXcmExecution { error });
            }

            <Pallet<T>>::deposit_event(Event::NormalOperationResumed);
            Ok(().into())
        }

        /// Place the sibling parachain enter hack mode.
        /// The storage `HackedSiblingId` is used by `Barrier` for intercept xcm from parachain.
        ///
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().read + 3 * T::DbWeight::get().write)]
        pub fn enter_sibling_hack_mode(
            origin: OriginFor<T>,
            hacked_chain_id: ParaId,
            affected_assets: Option<Vec<AssetId>>,
        ) -> DispatchResultWithPostInfo {
            T::EnterMaintenanceOrigin::ensure_origin(origin.clone())?;

            ensure!(
                !HackedSiblingId::<T>::get(&hacked_chain_id),
                Error::<T>::AlreadyInSiblingHackMode
            );
            HackedSiblingId::<T>::insert(&hacked_chain_id, true);

            let affected_assets = Self::get_affected_assets(hacked_chain_id, affected_assets)?;
            for asset in affected_assets.clone() {
                T::AssetFreezer::freeze_asset(asset)?;
            }

            <Pallet<T>>::deposit_event(Event::EnteredSiblingHackMode {
                id: hacked_chain_id,
                affected_assets,
            });
            Ok(().into())
        }

        /// Return the sibling parachain to normal operating mode.
        ///
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().read + 3 * T::DbWeight::get().write)]
        pub fn resume_sibling_normal_mode(
            origin: OriginFor<T>,
            normal_chain_id: ParaId,
            affected_assets: Option<Vec<AssetId>>,
        ) -> DispatchResultWithPostInfo {
            T::ResumeNormalOrigin::ensure_origin(origin.clone())?;

            ensure!(
                HackedSiblingId::<T>::contains_key(&normal_chain_id),
                Error::<T>::SiblingNotHack
            );
            HackedSiblingId::<T>::remove(&normal_chain_id);

            let affected_assets = Self::get_affected_assets(normal_chain_id, affected_assets)?;
            for asset in affected_assets.clone() {
                T::AssetFreezer::thaw_asset(asset)?;
            }

            <Pallet<T>>::deposit_event(Event::ResumedSiblingNormalMode {
                id: normal_chain_id,
                affected_assets,
            });
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// User provided `assets` maybe invalid or not belong to this `parachain_id`.
        /// We only filter out assets belong to `parachain_id`.
        fn get_affected_assets(
            parachain_id: ParaId,
            assets: Option<Vec<AssetId>>,
        ) -> Result<Vec<AssetId>, DispatchError> {
            let assets = if let Some(assets) = assets {
                assets
                    .into_iter()
                    .filter(|asset| T::AssetIdQuerier::contains(&parachain_id, asset))
                    .collect::<Vec<AssetId>>()
            } else {
                T::AssetIdQuerier::asset_ids(&parachain_id)
            };
            ensure!(!assets.is_empty(), Error::<T>::NoAssetRegistForParachain);
            Ok(assets)
        }
    }

    /// Genesis config for maintenance mode pallet
    #[derive(Default)]
    #[pallet::genesis_config]
    pub struct GenesisConfig {
        /// Whether to launch in maintenance mode
        pub start_in_maintenance_mode: bool,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            if self.start_in_maintenance_mode {
                MaintenanceMode::<T>::put(true);
            }
        }
    }

    impl<T: Config> Contains<T::Call> for Pallet<T> {
        fn contains(call: &T::Call) -> bool {
            if MaintenanceMode::<T>::get() {
                T::MaintenanceCallFilter::contains(call)
            } else {
                T::NormalCallFilter::contains(call)
            }
        }
    }

    impl<T: Config> DmpMessageHandler for Pallet<T> {
        fn handle_dmp_messages(
            iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
            limit: Weight,
        ) -> Weight {
            if MaintenanceMode::<T>::get() {
                T::MaintenanceDmpHandler::handle_dmp_messages(iter, limit)
            } else {
                T::NormalDmpHandler::handle_dmp_messages(iter, limit)
            }
        }
    }
}
