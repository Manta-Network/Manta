// Copyright 2020-2024 Manta Network.
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

//! # Chameleon Staking Pallet
//!
//! Enhanced staking with delegation and rewards.
//!
//! ## Features
//! - Delegation without minimums
//! - 14-day unbonding period
//! - Slashing (0.1% downtime, 5% double-sign)
//! - Era-based rewards

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::{Zero, Saturating}, Perbill};

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
    const STAKING_ID: LockIdentifier = *b"chmlstak";

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: LockableCurrency<Self::AccountId>;
        
        #[pallet::constant]
        type MinValidatorStake: Get<BalanceOf<Self>>;
        
        #[pallet::constant]
        type UnbondingPeriod: Get<BlockNumberFor<Self>>;
        
        #[pallet::constant]
        type MaxDelegatorsPerValidator: Get<u32>;
        
        #[pallet::constant]
        type MaxDelegationsPerDelegator: Get<u32>;
    }

    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    pub enum ValidatorStatus {
        Active,
        Waiting,
        Unbonding,
        Slashed,
    }

    impl Default for ValidatorStatus {
        fn default() -> Self { ValidatorStatus::Waiting }
    }

    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    pub enum SlashingOffense {
        ExtendedDowntime,
        DoubleSigning,
    }

    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen, Default)]
    pub struct ValidatorInfo<AccountId, Balance> {
        pub controller: AccountId,
        pub self_stake: Balance,
        pub total_stake: Balance,
        pub delegator_count: u32,
        pub commission: Perbill,
        pub status: ValidatorStatus,
    }

    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen, Default)]
    pub struct UnbondingRequest<Balance, BlockNumber> {
        pub amount: Balance,
        pub unlock_at: BlockNumber,
    }

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorInfo<T::AccountId, BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn delegations)]
    pub type Delegations<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn unbonding_requests)]
    pub type UnbondingRequests<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<UnbondingRequest<BalanceOf<T>, BlockNumberFor<T>>, ConstU32<100>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_staked)]
    pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_rewards)]
    pub type PendingRewards<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorJoined { validator: T::AccountId, stake: BalanceOf<T> },
        Delegated { delegator: T::AccountId, validator: T::AccountId, amount: BalanceOf<T> },
        Undelegated { delegator: T::AccountId, validator: T::AccountId, amount: BalanceOf<T> },
        UnbondingStarted { who: T::AccountId, amount: BalanceOf<T>, unlock_at: BlockNumberFor<T> },
        Withdrawn { who: T::AccountId, amount: BalanceOf<T> },
        RewardsDistributed { era: u32, total_reward: BalanceOf<T> },
        ValidatorSlashed { validator: T::AccountId, amount: BalanceOf<T>, offense: SlashingOffense },
        CommissionSet { validator: T::AccountId, commission: Perbill },
        RewardsClaimed { who: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorNotFound,
        InsufficientStake,
        TooManyDelegators,
        TooManyDelegations,
        AlreadyValidator,
        NotDelegated,
        NothingToWithdraw,
        InvalidCommission,
        NothingToClaim,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join as validator
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn join_candidates(origin: OriginFor<T>, stake: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(stake >= T::MinValidatorStake::get(), Error::<T>::InsufficientStake);
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::AlreadyValidator);

            T::Currency::set_lock(STAKING_ID, &who, stake, WithdrawReasons::all());

            let info = ValidatorInfo {
                controller: who.clone(),
                self_stake: stake,
                total_stake: stake,
                delegator_count: 0,
                commission: Perbill::from_percent(10),
                status: ValidatorStatus::Active,
            };

            Validators::<T>::insert(&who, info);
            TotalStaked::<T>::mutate(|t| *t = t.saturating_add(stake));

            Self::deposit_event(Event::ValidatorJoined { validator: who, stake });
            Ok(())
        }

        /// Delegate to a validator
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn delegate(origin: OriginFor<T>, validator: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!amount.is_zero(), Error::<T>::InsufficientStake);

            Validators::<T>::try_mutate(&validator, |maybe_info| {
                let info = maybe_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                ensure!(info.delegator_count < T::MaxDelegatorsPerValidator::get(), Error::<T>::TooManyDelegators);

                T::Currency::set_lock(STAKING_ID, &who, amount, WithdrawReasons::all());

                let current = Delegations::<T>::get(&who, &validator);
                if current.is_zero() {
                    info.delegator_count += 1;
                }
                Delegations::<T>::insert(&who, &validator, current.saturating_add(amount));
                info.total_stake = info.total_stake.saturating_add(amount);
                TotalStaked::<T>::mutate(|t| *t = t.saturating_add(amount));

                Self::deposit_event(Event::Delegated { delegator: who, validator: validator.clone(), amount });
                Ok(())
            })
        }

        /// Undelegate and start unbonding
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn undelegate(origin: OriginFor<T>, validator: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current = Delegations::<T>::get(&who, &validator);
            ensure!(!current.is_zero(), Error::<T>::NotDelegated);

            let unbond_amount = amount.min(current);
            let remaining = current.saturating_sub(unbond_amount);

            Validators::<T>::try_mutate(&validator, |maybe_info| {
                let info = maybe_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;

                if remaining.is_zero() {
                    Delegations::<T>::remove(&who, &validator);
                    info.delegator_count = info.delegator_count.saturating_sub(1);
                } else {
                    Delegations::<T>::insert(&who, &validator, remaining);
                }

                info.total_stake = info.total_stake.saturating_sub(unbond_amount);
                TotalStaked::<T>::mutate(|t| *t = t.saturating_sub(unbond_amount));

                let unlock_at = frame_system::Pallet::<T>::block_number() + T::UnbondingPeriod::get();
                UnbondingRequests::<T>::try_mutate(&who, |reqs| {
                    reqs.try_push(UnbondingRequest { amount: unbond_amount, unlock_at })
                }).map_err(|_| Error::<T>::TooManyDelegations)?;

                Self::deposit_event(Event::Undelegated { delegator: who.clone(), validator: validator.clone(), amount: unbond_amount });
                Self::deposit_event(Event::UnbondingStarted { who, amount: unbond_amount, unlock_at });
                Ok(())
            })
        }

        /// Withdraw unbonded tokens
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();

            let mut total_withdrawn = BalanceOf::<T>::zero();
            UnbondingRequests::<T>::mutate(&who, |reqs| {
                reqs.retain(|req| {
                    if req.unlock_at <= now {
                        total_withdrawn = total_withdrawn.saturating_add(req.amount);
                        false
                    } else {
                        true
                    }
                });
            });

            ensure!(!total_withdrawn.is_zero(), Error::<T>::NothingToWithdraw);
            T::Currency::remove_lock(STAKING_ID, &who);

            Self::deposit_event(Event::Withdrawn { who, amount: total_withdrawn });
            Ok(())
        }

        /// Claim pending rewards
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let rewards = PendingRewards::<T>::take(&who);
            ensure!(!rewards.is_zero(), Error::<T>::NothingToClaim);

            Self::deposit_event(Event::RewardsClaimed { who, amount: rewards });
            Ok(())
        }

        /// Set validator commission
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn set_commission(origin: OriginFor<T>, commission: Perbill) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(commission <= Perbill::from_percent(100), Error::<T>::InvalidCommission);

            Validators::<T>::try_mutate(&who, |maybe_info| {
                let info = maybe_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                info.commission = commission;
                Self::deposit_event(Event::CommissionSet { validator: who, commission });
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Distribute rewards for an era
        pub fn distribute_rewards(total_reward: BalanceOf<T>) -> DispatchResult {
            let era = CurrentEra::<T>::get();
            let total = TotalStaked::<T>::get();
            if total.is_zero() { return Ok(()); }

            for (validator_id, info) in Validators::<T>::iter() {
                if info.status != ValidatorStatus::Active { continue; }
                // Simplified: give proportional reward to validator
                PendingRewards::<T>::mutate(&validator_id, |r| *r = r.saturating_add(total_reward));
            }

            CurrentEra::<T>::put(era + 1);
            Self::deposit_event(Event::RewardsDistributed { era, total_reward });
            Ok(())
        }

        /// Slash validator for offense
        pub fn slash_validator(validator: &T::AccountId, offense: SlashingOffense) -> DispatchResult {
            Validators::<T>::try_mutate(validator, |maybe_info| {
                let info = maybe_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;

                let slash_rate = match offense {
                    SlashingOffense::ExtendedDowntime => Perbill::from_parts(1_000_000), // 0.1%
                    SlashingOffense::DoubleSigning => Perbill::from_parts(50_000_000),   // 5%
                };

                let slash_amount = slash_rate * info.self_stake;
                info.self_stake = info.self_stake.saturating_sub(slash_amount);
                info.total_stake = info.total_stake.saturating_sub(slash_amount);
                info.status = ValidatorStatus::Slashed;

                TotalStaked::<T>::mutate(|t| *t = t.saturating_sub(slash_amount));

                Self::deposit_event(Event::ValidatorSlashed {
                    validator: validator.clone(),
                    amount: slash_amount,
                    offense,
                });
                Ok(())
            })
        }
    }
}
