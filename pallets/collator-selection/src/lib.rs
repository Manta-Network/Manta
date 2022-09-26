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
//
// The pallet-collator-selection pallet is forked from Parity's cumulus module:
// https://github.com/paritytech/cumulus/tree/master/pallets/collator-selection
// The original license is Apache-2.0.

//! Collator Selection pallet.
//!
//! A pallet to manage collators in a parachain.
//!
//! ## Overview
//!
//! The Collator Selection pallet manages the collators of a parachain. **Collation is _not_ a
//! secure activity** and this pallet does not implement any game-theoretic mechanisms to meet BFT
//! safety assumptions of the chosen set.
//!
//! ## Terminology
//!
//! - Collator: A parachain block producer.
//! - Bond: An amount of `Balance` _reserved_ for candidate registration.
//! - Invulnerable: An account guaranteed to be in the collator set.
//!
//! ## Implementation
//!
//! The final [`Collators`] are aggregated from two individual lists:
//!
//! 1. [`Invulnerables`]: a set of collators appointed by governance. These accounts will always be
//!    collators.
//! 2. [`Candidates`]: these are *candidates to the collation task* and may or may not be elected as
//!    a final collator.
//!
//! The current implementation resolves congestion of [`Candidates`] in a first-come-first-serve
//! manner.
//!
//! ### Rewards
//!
//! The Collator Selection pallet maintains an on-chain account (the "Pot"). In each block, the
//! collator who authored it receives:
//!
//! - Half the value of the Pot.
//! - Half the value of the transaction fees within the block. The other half of the transaction
//!   fees are deposited into the Pot.
//!
//! To initiate rewards an ED needs to be transferred to the pot address.
//!
//! Note: Eventually the Pot distribution may be modified as discussed in
//! [this issue](https://github.com/paritytech/statemint/issues/21#issuecomment-810481073).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod migrations;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    pub use crate::weights::WeightInfo;
    use core::ops::Div;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        inherent::Vec,
        pallet_prelude::*,
        sp_runtime::{
            traits::{AccountIdConversion, CheckedSub, Convert, One, Zero},
            RuntimeAppPublic, RuntimeDebug,
        },
        traits::{
            Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, ReservableCurrency,
            StorageVersion, ValidatorRegistration, ValidatorSet,
        },
        weights::DispatchClass,
        PalletId,
    };
    use frame_system::{pallet_prelude::*, Config as SystemConfig};
    use nimbus_primitives::{AccountLookup, CanAuthor, NimbusId};
    use pallet_session::SessionManager;
    use sp_arithmetic::Percent;
    use sp_staking::SessionIndex;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

    /// A convertor from collators id. Since this pallet does not have stash/controller, this is
    /// just identity.
    pub struct IdentityCollator;
    impl<T> sp_runtime::traits::Convert<T, Option<T>> for IdentityCollator {
        fn convert(t: T) -> Option<T> {
            Some(t)
        }
    }
    impl<T> sp_runtime::traits::Convert<T, T> for IdentityCollator {
        fn convert(t: T) -> T {
            t
        }
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Origin that can dictate updating parameters of this pallet.
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// Account Identifier from which the internal Pot is generated.
        type PotId: Get<PalletId>;

        /// Maximum number of candidates that we should have. This is used for benchmarking and is not
        /// enforced.
        ///
        /// This does not take into account the invulnerables.
        type MaxCandidates: Get<u32>;

        /// Maximum number of invulnerables.
        ///
        /// Used only for benchmarking.
        type MaxInvulnerables: Get<u32>;

        /// A stable ID for a validator.
        type ValidatorId: Member
            + Parameter
            + From<<Self::ValidatorRegistration as ValidatorSet<Self::AccountId>>::ValidatorId>;

        /// A conversion from account ID to validator ID.
        ///
        /// Its cost must be at most one storage read.
        type ValidatorIdOf: Convert<Self::AccountId, Option<Self::ValidatorId>>;
        type AccountIdOf: Convert<Self::ValidatorId, Self::AccountId>;

        /// Validate a user is registered
        type ValidatorRegistration: ValidatorRegistration<Self::ValidatorId>
            + ValidatorSet<Self::AccountId>;

        /// The weight information of this pallet.
        type WeightInfo: WeightInfo;

        /// The final word on whether the reported author can author at this height.
        /// If the pallet that implements this trait depends on an inherent, that inherent **must**
        /// be included before this one.
        type CanAuthor: CanAuthor<Self::AccountId>;
    }

    /// Basic information about a collation candidate.
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, scale_info::TypeInfo)]
    pub struct CandidateInfo<AccountId, Balance> {
        /// Account identifier.
        pub who: AccountId,
        /// Reserved deposit.
        pub deposit: Balance,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// The invulnerable, fixed collators.
    #[pallet::storage]
    #[pallet::getter(fn invulnerables)]
    pub type Invulnerables<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// The (community, limited) collation candidates.
    #[pallet::storage]
    #[pallet::getter(fn candidates)]
    pub type Candidates<T: Config> =
        StorageValue<_, Vec<CandidateInfo<T::AccountId, BalanceOf<T>>>, ValueQuery>;

    pub(super) type BlockCount = u32;
    #[pallet::type_value]
    pub(super) fn StartingBlockCount() -> BlockCount {
        Zero::zero()
    }
    #[pallet::storage]
    pub(super) type BlocksPerCollatorThisSession<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockCount, ValueQuery, StartingBlockCount>;

    /// Performance percentile to use as baseline for collator eviction
    #[pallet::storage]
    #[pallet::getter(fn eviction_baseline)]
    pub type EvictionBaseline<T: Config> = StorageValue<_, Percent, ValueQuery>;

    /// Percentage of underperformance to _tolerate_ before evicting a collator
    ///
    /// i.e. A collator gets evicted if it produced _less_ than x% fewer blocks than the collator at EvictionBaseline
    #[pallet::storage]
    #[pallet::getter(fn eviction_tolerance)]
    pub type EvictionTolerance<T: Config> = StorageValue<_, Percent, ValueQuery>;

    /// Desired number of candidates.
    ///
    /// This should ideally always be less than [`Config::MaxCandidates`] for weights to be correct.
    #[pallet::storage]
    #[pallet::getter(fn desired_candidates)]
    pub type DesiredCandidates<T> = StorageValue<_, u32, ValueQuery>;

    /// Fixed deposit bond for each candidate.
    #[pallet::storage]
    #[pallet::getter(fn candidacy_bond)]
    pub type CandidacyBond<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub invulnerables: Vec<T::AccountId>,
        pub candidacy_bond: BalanceOf<T>,
        pub eviction_baseline: Percent,
        pub eviction_tolerance: Percent,
        pub desired_candidates: u32,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                invulnerables: Default::default(),
                candidacy_bond: Default::default(),
                eviction_baseline: Percent::zero(), // Note: eviction disabled by default
                eviction_tolerance: Percent::one(), // Note: eviction disabled by default
                desired_candidates: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let duplicate_invulnerables = self
                .invulnerables
                .iter()
                .collect::<std::collections::BTreeSet<_>>();
            assert!(
                duplicate_invulnerables.len() == self.invulnerables.len(),
                "duplicate invulnerables in genesis."
            );

            assert!(
                T::MaxInvulnerables::get() >= (self.invulnerables.len() as u32),
                "genesis invulnerables are more than T::MaxInvulnerables",
            );
            assert!(
                T::MaxCandidates::get() >= self.desired_candidates,
                "genesis desired_candidates are more than T::MaxCandidates",
            );
            assert!(
                self.eviction_baseline <= Percent::one(),
                "Eviction baseline must be given as a percentile - number between 0 and 100",
            );
            assert!(
                self.eviction_tolerance <= Percent::one(),
                "Eviction tolerance must be given as a percentage - number between 0 and 100",
            );
            <DesiredCandidates<T>>::put(self.desired_candidates);
            <CandidacyBond<T>>::put(self.candidacy_bond);
            <EvictionBaseline<T>>::put(self.eviction_baseline);
            <EvictionTolerance<T>>::put(self.eviction_tolerance);
            <Invulnerables<T>>::put(&self.invulnerables);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewInvulnerables(Vec<T::AccountId>),
        NewDesiredCandidates(u32),
        NewCandidacyBond(BalanceOf<T>),
        CandidateAdded(T::AccountId, BalanceOf<T>),
        CandidateRemoved(T::AccountId),
        NewEvictionBaseline(Percent),
        NewEvictionTolerance(Percent),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Too many candidates
        TooManyCandidates,
        /// Unknown error
        Unknown,
        /// Permission issue
        Permission,
        /// User is already a candidate
        AlreadyCandidate,
        /// User is not a candidate
        NotCandidate,
        /// User is already an Invulnerable
        AlreadyInvulnerable,
        /// Account has no associated validator ID
        NoAssociatedValidatorId,
        /// Validator ID is not yet registered
        ValidatorNotRegistered,
        /// Removing invulnerable collators is not allowed
        NotAllowRemoveInvulnerable,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set candidate collator as invulnerable.
        ///
        /// `new`: candidate collator.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_invulnerables(new.len() as u32))]
        pub fn set_invulnerables(
            origin: OriginFor<T>,
            new: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            // we trust origin calls, this is just a for more accurate benchmarking
            if (new.len() as u32) > T::MaxInvulnerables::get() {
                log::warn!(
                    "invulnerables > T::MaxInvulnerables; you might need to run benchmarks again"
                );
            }
            <Invulnerables<T>>::put(&new);
            Self::deposit_event(Event::NewInvulnerables(new));
            Ok(().into())
        }

        /// Set how many candidate collator are allowed.
        ///
        /// `max`: The max number of candidates.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_desired_candidates())]
        pub fn set_desired_candidates(
            origin: OriginFor<T>,
            max: u32,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            // we trust origin calls, this is just a for more accurate benchmarking
            if max > T::MaxCandidates::get() {
                log::warn!("max > T::MaxCandidates; you might need to run benchmarks again");
            }
            <DesiredCandidates<T>>::put(max);
            Self::deposit_event(Event::NewDesiredCandidates(max));
            Ok(().into())
        }

        /// Set the amount held on reserved for candidate collator.
        ///
        /// `bond`: The amount held on reserved.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_candidacy_bond())]
        pub fn set_candidacy_bond(
            origin: OriginFor<T>,
            bond: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            <CandidacyBond<T>>::put(bond);
            Self::deposit_event(Event::NewCandidacyBond(bond));
            Ok(().into())
        }

        /// Register as candidate collator.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::register_as_candidate(T::MaxCandidates::get()))]
        pub fn register_as_candidate(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // ensure we are below limit.
            let length = <Candidates<T>>::decode_len().unwrap_or_default();
            ensure!(
                (length as u32) < Self::desired_candidates(),
                Error::<T>::TooManyCandidates
            );
            ensure!(
                !Self::invulnerables().contains(&who),
                Error::<T>::AlreadyInvulnerable
            );

            let validator_key = T::ValidatorIdOf::convert(who.clone())
                .ok_or(Error::<T>::NoAssociatedValidatorId)?;
            ensure!(
                T::ValidatorRegistration::is_registered(&validator_key),
                Error::<T>::ValidatorNotRegistered
            );

            let deposit = Self::candidacy_bond();
            // First authored block is current block plus kick threshold to handle session delay
            let incoming = CandidateInfo {
                who: who.clone(),
                deposit,
            };

            let current_count =
                <Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
                    if candidates.iter_mut().any(|candidate| candidate.who == who) {
                        Err(Error::<T>::AlreadyCandidate.into())
                    } else {
                        T::Currency::reserve(&who, deposit)?;
                        candidates.push(incoming);
                        Ok(candidates.len())
                    }
                })?;

            Self::deposit_event(Event::CandidateAdded(who, deposit));
            Ok(Some(T::WeightInfo::register_as_candidate(current_count as u32)).into())
        }

        /// Register an specified candidate as collator.
        ///
        /// - `new_candidate`: Who is going to be collator.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::register_candidate(T::MaxCandidates::get()))]
        pub fn register_candidate(
            origin: OriginFor<T>,
            new_candidate: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            // ensure we are below limit.
            let length = <Candidates<T>>::decode_len().unwrap_or_default();
            ensure!(
                (length as u32) < Self::desired_candidates(),
                Error::<T>::TooManyCandidates
            );
            ensure!(
                !Self::invulnerables().contains(&new_candidate),
                Error::<T>::AlreadyInvulnerable
            );

            let validator_key = T::ValidatorIdOf::convert(new_candidate.clone())
                .ok_or(Error::<T>::NoAssociatedValidatorId)?;
            ensure!(
                T::ValidatorRegistration::is_registered(&validator_key),
                Error::<T>::ValidatorNotRegistered
            );

            let deposit = Self::candidacy_bond();
            let incoming = CandidateInfo {
                who: new_candidate.clone(),
                deposit,
            };

            let current_count =
                <Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
                    if candidates
                        .iter_mut()
                        .any(|candidate| candidate.who == new_candidate)
                    {
                        Err(Error::<T>::AlreadyCandidate.into())
                    } else {
                        T::Currency::reserve(&new_candidate, deposit)?;
                        candidates.push(incoming);
                        Ok(candidates.len())
                    }
                })?;

            Self::deposit_event(Event::CandidateAdded(new_candidate, deposit));
            Ok(Some(T::WeightInfo::register_candidate(current_count as u32)).into())
        }

        /// Leave from collator set.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::leave_intent(T::MaxCandidates::get()))]
        pub fn leave_intent(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let current_count = Self::try_remove_candidate(&who)?;

            Ok(Some(T::WeightInfo::leave_intent(current_count as u32)).into())
        }

        /// Remove an specified collator.
        ///
        /// - `collator`: Who is going to be remove from collators set.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::remove_collator(T::MaxCandidates::get()))]
        pub fn remove_collator(
            origin: OriginFor<T>,
            collator: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            // not allow to remove invulnerables
            ensure!(
                !<Invulnerables<T>>::get().contains(&collator),
                Error::<T>::NotAllowRemoveInvulnerable
            );

            let current_count = Self::try_remove_candidate(&collator)?;

            Ok(Some(T::WeightInfo::remove_collator(current_count as u32)).into())
        }

        /// Set the collator performance percentile used as baseline for eviction
        ///
        /// `percentile`: x-th percentile of collator performance to use as eviction baseline
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::set_eviction_baseline())]
        pub fn set_eviction_baseline(
            origin: OriginFor<T>,
            percentile: Percent,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            <EvictionBaseline<T>>::put(percentile); // NOTE: from_percent saturates at 100
            Self::deposit_event(Event::NewEvictionBaseline(percentile));
            Ok(().into())
        }

        /// Set the tolerated underperformance percentage before evicting
        ///
        /// `percentage`: x% of missed blocks under eviction_baseline to tolerate
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::set_eviction_tolerance())]
        pub fn set_eviction_tolerance(
            origin: OriginFor<T>,
            percentage: Percent,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            <EvictionTolerance<T>>::put(percentage); // NOTE: from_percent saturates at 100
            Self::deposit_event(Event::NewEvictionTolerance(percentage));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get a unique, inaccessible account id from the `PotId`.
        pub fn account_id() -> T::AccountId {
            T::PotId::get().into_account_truncating()
        }

        /// Removes a candidate if they exist and sends them back their deposit
        fn try_remove_candidate(who: &T::AccountId) -> Result<usize, DispatchError> {
            let current_count =
                <Candidates<T>>::try_mutate(|candidates| -> Result<usize, DispatchError> {
                    let index = candidates
                        .iter()
                        .position(|candidate| candidate.who == *who)
                        .ok_or(Error::<T>::NotCandidate)?;
                    T::Currency::unreserve(who, candidates[index].deposit);
                    candidates.remove(index);
                    Ok(candidates.len())
                })?;
            Self::deposit_event(Event::CandidateRemoved(who.clone()));
            Ok(current_count)
        }

        /// Assemble the current set of candidates and invulnerables into the next collator set.
        ///
        /// This is done on the fly, as frequent as we are told to do so, as the session manager.
        pub fn assemble_collators(candidates: Vec<T::AccountId>) -> Vec<T::AccountId> {
            let mut collators = Self::invulnerables();
            collators.extend(candidates.into_iter().collect::<Vec<_>>());
            collators
        }

        /// Removes collators with unsatisfactory performance
        /// Returns the removed AccountIds
        pub fn evict_bad_collators(
            candidates: Vec<CandidateInfo<T::AccountId, BalanceOf<T>>>,
        ) -> Vec<T::AccountId> {
            use sp_runtime::PerThing;

            // 0. Storage reads and precondition checks
            if candidates.is_empty() {
                return Vec::new(); // No candidates means we're running invulnerables only
            }
            let percentile_for_kick = Self::eviction_baseline();
            if percentile_for_kick == Percent::zero() {
                return Vec::new(); // Selecting 0-th percentile disables kicking. Upper bound check in fn build()
            }
            let underperformance_tolerated = Self::eviction_tolerance();
            if underperformance_tolerated == Percent::one() {
                return Vec::new(); // tolerating 100% underperformance disables kicking
            }
            let mut collator_perf_this_session =
                <BlocksPerCollatorThisSession<T>>::iter().collect::<Vec<_>>();
            if collator_perf_this_session.is_empty() {
                return Vec::new(); // no validator performance recorded ( should not happen )
            }

            // 1. Ascending sort of collator performance list by number of produced blocks
            collator_perf_this_session.sort_unstable_by_key(|k| k.1);
            let collator_count = collator_perf_this_session.len();

            // 2. get percentile by _exclusive_ nearest rank method https://en.wikipedia.org/wiki/Percentile#The_nearest-rank_method (rust percentile API is feature gated and unstable)
            let ordinal_rank = percentile_for_kick.mul_ceil(collator_count);
            let index_at_ordinal_rank = ordinal_rank.saturating_sub(One::one()); // -1 to accommodate 0-index counting, should not saturate due to precondition check and round up multiplication

            // 3. Block number at rank is the percentile and our kick performance benchmark
            let blocks_created_at_baseline: BlockCount =
                collator_perf_this_session[index_at_ordinal_rank].1;

            // 4. We kick if a collator produced fewer than (EvictionTolerance * EvictionBaseline rounded up) blocks than the percentile
            let evict_below_blocks = (underperformance_tolerated
                .left_from_one()
                .mul_ceil(blocks_created_at_baseline))
                as BlockCount;
            log::trace!(
                "Session Performance stats: {}-th percentile: {:?} blocks. Evicting collators who produced less than {} blocks",
                percentile_for_kick.mul_ceil(100u8),
                blocks_created_at_baseline,
                evict_below_blocks
            );

            // 5. Walk the percentile slice, call try_remove_candidate if a collator is under threshold
            let kick_candidates = &collator_perf_this_session[..index_at_ordinal_rank]; // ordinal-rank exclusive, the collator at percentile is safe
            let mut removed_account_ids: Vec<T::AccountId> =
                Vec::with_capacity(kick_candidates.len());
            kick_candidates.iter().for_each(|(acc_id, my_blocks_this_session)| {
                if *my_blocks_this_session < evict_below_blocks {
                    // If our validator is not also a candidate we're invulnerable or already kicked
                    if candidates.iter().any(|x| x.who == *acc_id) {
                        #[allow(clippy::bind_instead_of_map)] Self::try_remove_candidate(acc_id)
                            .and_then(|_| {
                                removed_account_ids.push(acc_id.clone());
                                log::info!("Removed collator of account {:?} as it only produced {} blocks this session which is below acceptable threshold of {}", &acc_id, my_blocks_this_session,evict_below_blocks);
                                Ok(())
                            })
                            .unwrap_or_else(|why| {
                                log::warn!("Failed to remove candidate due to underperformance {:?}", why);
                                debug_assert!(false, "failed to remove candidate {:?}", why);
                            });
                    }
                }
            });
            removed_account_ids.shrink_to_fit();
            removed_account_ids
        }

        /// Reset the performance map to the currently active validators at 0 blocks
        pub fn reset_collator_performance() {
            let validators = T::ValidatorRegistration::validators();
            let validators_len = validators.len() as u32;
            let mut clear_res = <BlocksPerCollatorThisSession<T>>::clear(validators_len, None);
            let mut old_cursor = Vec::new();
            while let Some(cursor) = clear_res.maybe_cursor {
                clear_res = <BlocksPerCollatorThisSession<T>>::clear(validators_len, Some(&cursor));
                if cursor == old_cursor {
                    // As per the documentation the cursor may not advance after every operation
                    break;
                }
                old_cursor = cursor;
            }
            for validator_id in validators {
                let account_id = T::AccountIdOf::convert(validator_id.clone().into());
                <BlocksPerCollatorThisSession<T>>::insert(account_id.clone(), 0u32);
            }
        }
    }

    /// Checks if a provided NimbusId SessionKey has an associated AccountId
    impl<T> AccountLookup<T::AccountId> for Pallet<T>
    where
        T: pallet_session::Config + Config,
        // Implemented only where Session's ValidatorId is directly convertible to collator_selection's ValidatorId
        <T as Config>::ValidatorId: From<<T as pallet_session::Config>::ValidatorId>,
    {
        fn lookup_account(author: &NimbusId) -> Option<T::AccountId>
        where
            <T as Config>::ValidatorId: From<<T as pallet_session::Config>::ValidatorId>,
        {
            use sp_runtime::traits::Convert;
            #[allow(clippy::bind_instead_of_map)]
            pallet_session::Pallet::<T>::key_owner(
                nimbus_primitives::NIMBUS_KEY_ID,
                &author.to_raw_vec(),
            )
            .and_then(|vid| Some(T::AccountIdOf::convert(vid.into())))
        }
    }

    /// Fetch list of all possibly eligible authors to use in nimbus consensus filters
    ///
    /// NOTE: This should really be in pallet_session as we only use its storage, but since we haven't
    /// forked that one, this is the next best place.
    impl<T> Get<Vec<T::AccountId>> for Pallet<T>
    where
        T: Config + pallet_session::Config,
        // Implemented only where Session's ValidatorId is directly convertible to collator_selection's ValidatorId
        <T as Config>::ValidatorId: From<<T as pallet_session::Config>::ValidatorId>,
    {
        /// Return the set of eligible collator accounts as registered with pallet session
        fn get() -> Vec<T::AccountId>
        where
            <T as Config>::ValidatorId: From<<T as pallet_session::Config>::ValidatorId>,
        {
            use sp_runtime::traits::Convert;
            pallet_session::Pallet::<T>::validators()
                .into_iter()
                .map(
                    |session_validator_id: <T as pallet_session::Config>::ValidatorId| {
                        <T as Config>::AccountIdOf::convert(session_validator_id.into())
                    },
                )
                .collect::<Vec<T::AccountId>>()
        }
    }

    /// Returns whether an account is part of pallet_session::Validators
    impl<T> nimbus_primitives::CanAuthor<T::AccountId> for Pallet<T>
    where
        T: Config + pallet_session::Config,
        // Implemented only where Session's ValidatorId is directly convertible to collator_selection's ValidatorId
        <T as Config>::ValidatorId: From<<T as pallet_session::Config>::ValidatorId>,
    {
        fn can_author(account: &T::AccountId, slot: &u32) -> bool {
            let validator_key = <T as Config>::ValidatorIdOf::convert(account.clone());
            if validator_key.is_none()
                || !T::ValidatorRegistration::is_registered(
                    &validator_key.expect("we checked against none before. qed"),
                )
            {
                return false;
            }
            T::CanAuthor::can_author(account, slot) // filter passed, hand execution to the next pipeline step
        }
        #[cfg(feature = "runtime-benchmarks")]
        fn get_authors(_slot: &u32) -> Vec<T::AccountId> {
            use sp_runtime::traits::Convert;
            pallet_session::Pallet::<T>::validators()
                .into_iter()
                .map(
                    |session_validator_id: <T as pallet_session::Config>::ValidatorId| {
                        <T as Config>::AccountIdOf::convert(session_validator_id.into())
                    },
                )
                .collect::<Vec<T::AccountId>>()
        }
    }

    /// Keep track of number of authored blocks per authority, uncles are counted as well since
    /// they're a valid proof of being online.
    impl<T: Config + pallet_authorship::Config>
        pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
    {
        fn note_author(author: T::AccountId) {
            let pot = Self::account_id();
            // assumes an ED will be sent to pot.
            let reward = T::Currency::free_balance(&pot)
                .checked_sub(&T::Currency::minimum_balance())
                .unwrap_or_else(Zero::zero)
                .div(2u32.into());
            // `reward` is half of pot account minus ED, this should never fail.
            let _success = T::Currency::transfer(&pot, &author, reward, KeepAlive);
            debug_assert!(_success.is_ok());

            // increment blocks this node authored
            <BlocksPerCollatorThisSession<T>>::mutate(&author, |blocks| {
                *blocks = blocks.saturating_add(One::one());
            });

            frame_system::Pallet::<T>::register_extra_weight_unchecked(
                T::WeightInfo::note_author(),
                DispatchClass::Mandatory,
            );
        }

        fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {
            //TODO can we ignore this?
        }
    }

    /// Play the role of the session manager.
    impl<T: Config> SessionManager<T::AccountId> for Pallet<T> {
        fn new_session(index: SessionIndex) -> Option<Vec<T::AccountId>> {
            log::info!(
                "assembling new collators for new session {} at #{:?}",
                index,
                <frame_system::Pallet<T>>::block_number(),
            );

            let candidates = Self::candidates();
            let candidates_len_before = candidates.len();
            let removed_candidate_ids = Self::evict_bad_collators(candidates.clone());
            let active_candidate_ids = candidates
                .iter()
                .filter_map(|x| {
                    if removed_candidate_ids.contains(&x.who) {
                        None
                    } else {
                        Some(x.who.clone())
                    }
                })
                .collect::<Vec<_>>();
            let result = Self::assemble_collators(active_candidate_ids);
            frame_system::Pallet::<T>::register_extra_weight_unchecked(
                T::WeightInfo::new_session(candidates_len_before as u32),
                DispatchClass::Mandatory,
            );

            Self::reset_collator_performance(); // Reset performance map for the now starting session's active validator set
            Some(result)
        }
        fn start_session(_: SessionIndex) {
            // we don't care.
        }
        fn end_session(_: SessionIndex) {
            // we don't care.
        }
    }
}
