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

//! # Randomness Pallet
//!
//! This pallet provides access to 1 sources of randomness:
//! 1. relay chain BABE one epoch ago randomness, produced by the relay chain per relay chain epoch
//! These options are represented as `type::RequestType`.
//!
//! There are no extrinsics for this pallet. Instead, public functions on `Pallet<T: Config>` expose
//! user actions for the precompile i.e. `request_randomness`.
//!
//! ## Babe Epoch Randomness
//! Babe epoch randomness is retrieved once every relay chain epoch.
//!
//! The `set_babe_randomness_results` mandatory inherent reads the Babe epoch randomness from the
//! relay chain state proof and fills any pending `RandomnessResults` for this epoch randomness.
//!
//! `Config::BabeDataGetter` is responsible for reading the epoch index and epoch randomness
//! from the relay chain state proof. The moonbeam `GetBabeData` implementation is in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;
use sp_std::vec::Vec;
pub use weights::WeightInfo;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
pub mod types;
pub use types::*;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Read babe randomness info from the relay chain state proof
pub trait GetBabeData<EpochIndex, Randomness> {
    fn get_epoch_index() -> EpochIndex;
    fn get_epoch_randomness() -> Randomness;
}

#[pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use session_key_primitives::inherent::{InherentError, INHERENT_IDENTIFIER};
    use sp_runtime::traits::Hash;
    use sp_std::convert::TryInto;
    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Get the BABE data from the runtime
        type BabeDataGetter: GetBabeData<u64, Option<Self::Hash>>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        CannotRequestRandomnessAfterMaxDelay,
    }

    /// Relay epoch
    #[pallet::storage]
    #[pallet::getter(fn relay_epoch)]
    pub(crate) type RelayEpoch<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Ensures the mandatory inherent was included in the block
    #[pallet::storage]
    #[pallet::getter(fn inherent_included)]
    pub(crate) type InherentIncluded<T: Config> = StorageValue<_, ()>;

    /// Snapshot of randomness to fulfill all requests that are for the same raw randomness
    /// Removed once $value.request_count == 0
    #[pallet::storage]
    #[pallet::getter(fn randomness_results)]
    pub type RandomnessResults<T: Config> =
        StorageMap<_, Twox64Concat, RequestType, RandomnessResult<T::Hash>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Populates `RandomnessResults` due this epoch with BABE epoch randomness
        #[pallet::call_index(0)]
        #[pallet::weight((
            <T as Config>::WeightInfo::set_babe_randomness_results(),
			DispatchClass::Mandatory
		))]
        pub fn set_babe_randomness_results(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            let last_relay_epoch_index = <RelayEpoch<T>>::get();
            let relay_epoch_index = T::BabeDataGetter::get_epoch_index();
            if relay_epoch_index > last_relay_epoch_index {
                // NOTE: Whether n = 1 or 2 depends on the trait implementation of BabeDataGetter
                let babe_n_epochs_ago_this_block = RequestType::BabeEpoch(relay_epoch_index);
                if let Some(randomness) = T::BabeDataGetter::get_epoch_randomness() {
                    let result = RandomnessResult {
                        request_count: 1,
                        randomness: Some(randomness),
                    };
                    <RandomnessResults<T>>::insert(babe_n_epochs_ago_this_block, result);
                } else {
                    log::warn!(
                        "Failed to fill BABE epoch randomness results \
							REQUIRE HOTFIX TO FILL EPOCH RANDOMNESS RESULTS FOR EPOCH {:?}",
                        relay_epoch_index
                    );
                }
            }
            <RelayEpoch<T>>::put(relay_epoch_index);
            <InherentIncluded<T>>::put(());
            Ok(Pays::No.into())
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = InherentError;
        const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

        fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
            // Return Ok(Some(_)) unconditionally because this inherent is required in every block
            // If it is not found, throw a VrfInherentRequired error.
            Ok(Some(InherentError::Other(
                sp_runtime::RuntimeString::Borrowed(
                    "Inherent required to set babe randomness results",
                ),
            )))
        }

        // The empty-payload inherent extrinsic.
        fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
            Some(Call::set_babe_randomness_results {})
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::set_babe_randomness_results { .. })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_now: BlockNumberFor<T>) {
            // Ensure the mandatory inherent was included in the block or the block is invalid
            assert!(
				<InherentIncluded<T>>::take().is_some(),
				"Mandatory randomness inherent not included; InherentIncluded storage item is empty"
			);
        }
    }

    // Randomness trait
    impl<T: Config> frame_support::traits::Randomness<T::Hash, BlockNumberFor<T>> for Pallet<T> {
        /// Uses the BABE randomness to generate a random seed.
        fn random(subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
            let relay_epoch_index = <RelayEpoch<T>>::get();
            let randomness_output =
                RandomnessResults::<T>::get(RequestType::BabeEpoch(relay_epoch_index))
                    .unwrap_or_else(|| {
                        log::error!(
                    "FATAL Could not find the included Babe randomness for {:?}. Using None",
                    relay_epoch_index
                );
                        RandomnessResult::<T::Hash>::new()
                    })
                    .randomness
                    .unwrap_or_else(|| {
                        log::error!("FATAL included BABE randomness is `None`. Using default hash");
                        T::Hash::default()
                    });
            let mut digest = Vec::new();
            digest.extend_from_slice(randomness_output.as_ref());
            digest.extend_from_slice(subject);
            let randomness = T::Hashing::hash(digest.as_slice());
            // TODO: Randomness Established at start of Epoch! This is nontrivial to implement
            // because we need to map the start-of-epoch relayblock to its matching parablock
            // in its current form block_number is meaningless and should not be relied upon
            let randomness_established_at = 0u32.into();
            (randomness, randomness_established_at)
        }
    }
}
