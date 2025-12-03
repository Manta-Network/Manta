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
//! Enhanced staking mechanism with delegation and reward optimization.
//!
//! ## Overview
//!
//! This pallet provides:
//! - Enhanced delegation system with no minimum amounts
//! - Performance-based reward distribution
//! - Slashing conditions (0.1% downtime, 5% double-sign)
//! - 14-day unbonding period
//!
//! ## Features
//!
//! - **Validator Management**: Join/leave validator set with minimum stake requirements
//! - **Delegation**: Delegate tokens to validators without running a node
//! - **Unbonding**: 14-day unbonding period for security
//! - **Rewards**: Era-based reward distribution with commission
//! - **Slashing**: Penalties for downtime and double-signing

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{
            Currency, LockIdentifier, LockableCurrency, WithdrawReasons,
            ReservableCurrency, ExistenceRequirement,
        },
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{Zero, Saturating, CheckedAdd, CheckedSub},
        Perbill, ArithmeticError,
    };
    use sp_std::vec::Vec;

    /// The current storage version
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Lock identifier for staking
    const STAKING_ID: LockIdentifier = *b"chmlstak";

    /// Balance type alias
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type for staking operations
        type Currency: LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Minimum validator stake (1,750 CHML)
        #[pallet::constant]
        type MinValidatorStake: Get<BalanceOf<Self>>;

        /// Unbonding period in blocks (14 days)
        #[pallet::constant]
        type UnbondingPeriod: Get<BlockNumberFor<Self>>;

        /// Maximum delegators per validator
        #[pallet::constant]
        type MaxDelegatorsPerValidator: Get<u32>;

        /// Maximum delegations per delegator
        #[pallet::constant]
        type MaxDelegationsPerDelegator: Get<u32>;

        /// Maximum unbonding requests per account
        #[pallet::constant]
        type MaxUnbondingRequests: Get<u32>;
    }

    /// Validator status
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    pub enum ValidatorStatus {
        /// Validator is active and producing blocks
        Active,
        /// Validator is waiting to be activated
        Waiting,
        /// Validator is unbonding
        Unbonding,
        /// Validator has been slashed
        Slashed,
    }

    impl Default for ValidatorStatus {
        fn default() -> Self {
            ValidatorStatus::Waiting
        }
    }

    /// Slashing offense type
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    pub enum SlashingOffense {
        /// Extended downtime (>12 hours)
        ExtendedDowntime,
        /// Double signing
        DoubleSigning,
    }

    /// Validator information
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    pub struct ValidatorInfo<AccountId, Balance> {
        /// Controller account
        pub controller: AccountId,
        /// Self-staked amount
        pub self_stake: Balance,
        /// Total stake (self + delegated)
        pub total_stake: Balance,
        /// Number of delegators
        pub delegator_count: u32,
        /// Commission rate (percentage taken from delegator rewards)
        pub commission: Perbill,
        /// Current status
        pub status: ValidatorStatus,
    }

    /// Unbonding request
    #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
    pub struct UnbondingRequest<Balance, BlockNumber> {
        /// Amount being unbonded
        pub amount: Balance,
        /// Block number when unbonding completes
        pub unlock_at: BlockNumber,
    }

    /// Total staked in the network
    #[pallet::storage]
    #[pallet::getter(fn total_staked)]
    pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Current era
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Validator count
    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Validator information storage
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorInfo<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    /// Delegations: (delegator, validator) -> amount
    #[pallet::storage]
    #[pallet::getter(fn delegations)]
    pub type Delegations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,  // delegator
        Blake2_128Concat,
        T::AccountId,  // validator
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Unbonding requests per account
    #[pallet::storage]
    #[pallet::getter(fn unbonding_requests)]
    pub type UnbondingRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<UnbondingRequest<BalanceOf<T>, BlockNumberFor<T>>, T::MaxUnbondingRequests>,
        ValueQuery,
    >;

    /// Pending rewards per account
    #[pallet::storage]
    #[pallet::getter(fn pending_rewards)]
    pub type PendingRewards<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Delegator count per validator
    #[pallet::storage]
    #[pallet::getter(fn delegator_count)]
    pub type DelegatorCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Delegation count per delegator
    #[pallet::storage]
    #[pallet::getter(fn delegation_count)]
    pub type DelegationCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Validator joined the candidate pool
        ValidatorJoined { validator: T::AccountId, stake: BalanceOf<T> },
        /// Delegation made
        Delegated { delegator: T::AccountId, validator: T::AccountId, amount: BalanceOf<T> },
        /// Delegation removed
        Undelegated { delegator: T::AccountId, validator: T::AccountId, amount: BalanceOf<T> },
        /// Unbonding started
        UnbondingStarted { who: T::AccountId, amount: BalanceOf<T>, unlock_at: BlockNumberFor<T> },
        /// Rewards distributed
        RewardsDistributed { era: u32, total_reward: BalanceOf<T> },
        /// Validator slashed
        ValidatorSlashed { validator: T::AccountId, amount: BalanceOf<T>, offense: SlashingOffense },
        /// Commission set
        CommissionSet { validator: T::AccountId, commission: Perbill },
        /// Rewards claimed
        RewardsClaimed { who: T::AccountId, amount: BalanceOf<T> },
        /// Unbonded tokens withdrawn
        UnbondedWithdrawn { who: T::AccountId, amount: BalanceOf<T> },
        /// Validator left the candidate pool
        ValidatorLeft { validator: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Validator not found
        ValidatorNotFound,
        /// Insufficient stake
        InsufficientStake,
        /// Too many delegators
        TooManyDelegators,
        /// Too many delegations
        TooManyDelegations,
        /// Already delegated
        AlreadyDelegated,
        /// Not delegated
        NotDelegated,
        /// Unbonding in progress
        UnbondingInProgress,
        /// Invalid commission
        InvalidCommission,
        /// Already a validator
        AlreadyValidator,
        /// Cannot delegate to self
        CannotDelegateToSelf,
        /// Insufficient balance
        InsufficientBalance,
        /// Too many unbonding requests
        TooManyUnbondingRequests,
        /// No unbonded tokens to withdraw
        NoUnbondedTokens,
        /// No rewards to claim
        NoRewardsToClaim,
        /// Arithmetic overflow
        ArithmeticOverflow,
        /// Arithmetic underflow
        ArithmeticUnderflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join as validator candidate
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn join_candidates(
            origin: OriginFor<T>,
            stake: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Ensure minimum stake requirement
            ensure!(stake >= T::MinValidatorStake::get(), Error::<T>::InsufficientStake);
            
            // Ensure not already a validator
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::AlreadyValidator);
            
            // Lock the stake
            T::Currency::set_lock(STAKING_ID, &who, stake, WithdrawReasons::all());
            
            // Create validator info
            let validator_info = ValidatorInfo {
                controller: who.clone(),
                self_stake: stake,
                total_stake: stake,
                delegator_count: 0,
                commission: Perbill::from_percent(10), // Default 10% commission
                status: ValidatorStatus::Waiting,
            };
            
            // Store validator info
            Validators::<T>::insert(&who, validator_info);
            
            // Update global counters
            TotalStaked::<T>::mutate(|total| *total = total.saturating_add(stake));
            ValidatorCount::<T>::mutate(|count| *count = count.saturating_add(1));
            
            Self::deposit_event(Event::ValidatorJoined { validator: who, stake });
            Ok(())
        }

        /// Delegate to a validator
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn delegate(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Ensure amount is not zero
            ensure!(!amount.is_zero(), Error::<T>::InsufficientStake);
            
            // Ensure validator exists
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotFound);
            
            // Ensure not delegating to self
            ensure!(who != validator, Error::<T>::CannotDelegateToSelf);
            
            // Ensure not already delegated to this validator
            ensure!(!Delegations::<T>::contains_key(&who, &validator), Error::<T>::AlreadyDelegated);
            
            // Check delegation limits
            let delegator_count = DelegatorCount::<T>::get(&validator);
            ensure!(
                delegator_count < T::MaxDelegatorsPerValidator::get(),
                Error::<T>::TooManyDelegators
            );
            
            let delegation_count = DelegationCount::<T>::get(&who);
            ensure!(
                delegation_count < T::MaxDelegationsPerDelegator::get(),
                Error::<T>::TooManyDelegations
            );
            
            // Lock the delegation amount
            T::Currency::set_lock(STAKING_ID, &who, amount, WithdrawReasons::all());
            
            // Store delegation
            Delegations::<T>::insert(&who, &validator, amount);
            
            // Update validator info
            Validators::<T>::try_mutate(&validator, |validator_info| -> DispatchResult {
                let info = validator_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                info.total_stake = info.total_stake.checked_add(&amount)
                    .ok_or(Error::<T>::ArithmeticOverflow)?;
                info.delegator_count = info.delegator_count.saturating_add(1);
                Ok(())
            })?;
            
            // Update counters
            DelegatorCount::<T>::mutate(&validator, |count| *count = count.saturating_add(1));
            DelegationCount::<T>::mutate(&who, |count| *count = count.saturating_add(1));
            TotalStaked::<T>::mutate(|total| *total = total.saturating_add(amount));
            
            Self::deposit_event(Event::Delegated { delegator: who, validator, amount });
            Ok(())
        }

        /// Remove delegation (start unbonding)
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn undelegate(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Get delegation amount
            let amount = Delegations::<T>::get(&who, &validator);
            ensure!(!amount.is_zero(), Error::<T>::NotDelegated);
            
            // Check unbonding request limit
            let mut unbonding_requests = UnbondingRequests::<T>::get(&who);
            ensure!(
                unbonding_requests.len() < T::MaxUnbondingRequests::get() as usize,
                Error::<T>::TooManyUnbondingRequests
            );
            
            // Calculate unlock block
            let current_block = frame_system::Pallet::<T>::block_number();
            let unlock_at = current_block + T::UnbondingPeriod::get();
            
            // Add unbonding request
            let unbonding_request = UnbondingRequest {
                amount,
                unlock_at,
            };
            
            unbonding_requests.try_push(unbonding_request)
                .map_err(|_| Error::<T>::TooManyUnbondingRequests)?;
            
            UnbondingRequests::<T>::insert(&who, unbonding_requests);
            
            // Remove delegation
            Delegations::<T>::remove(&who, &validator);
            
            // Update validator info
            Validators::<T>::try_mutate(&validator, |validator_info| -> DispatchResult {
                let info = validator_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                info.total_stake = info.total_stake.checked_sub(&amount)
                    .ok_or(Error::<T>::ArithmeticUnderflow)?;
                info.delegator_count = info.delegator_count.saturating_sub(1);
                Ok(())
            })?;
            
            // Update counters
            DelegatorCount::<T>::mutate(&validator, |count| *count = count.saturating_sub(1));
            DelegationCount::<T>::mutate(&who, |count| *count = count.saturating_sub(1));
            TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(amount));
            
            Self::deposit_event(Event::Undelegated { delegator: who.clone(), validator, amount });
            Self::deposit_event(Event::UnbondingStarted { who, amount, unlock_at });
            Ok(())
        }

        /// Withdraw unbonded tokens
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();
            let mut unbonding_requests = UnbondingRequests::<T>::get(&who);
            let mut total_withdrawn = BalanceOf::<T>::zero();
            
            // Filter out completed unbonding requests
            unbonding_requests.retain(|request| {
                if request.unlock_at <= current_block {
                    total_withdrawn = total_withdrawn.saturating_add(request.amount);
                    false
                } else {
                    true
                }
            });
            
            ensure!(!total_withdrawn.is_zero(), Error::<T>::NoUnbondedTokens);
            
            // Update unbonding requests
            UnbondingRequests::<T>::insert(&who, unbonding_requests);
            
            // Remove lock for withdrawn amount
            T::Currency::remove_lock(STAKING_ID, &who);
            
            Self::deposit_event(Event::UnbondedWithdrawn { who, amount: total_withdrawn });
            Ok(())
        }

        /// Claim pending rewards
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn claim_rewards(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let reward_amount = PendingRewards::<T>::get(&who);
            ensure!(!reward_amount.is_zero(), Error::<T>::NoRewardsToClaim);
            
            // Clear pending rewards
            PendingRewards::<T>::remove(&who);
            
            // Transfer rewards
            T::Currency::deposit_creating(&who, reward_amount);
            
            Self::deposit_event(Event::RewardsClaimed { who, amount: reward_amount });
            Ok(())
        }

        /// Set commission rate
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn set_commission(
            origin: OriginFor<T>,
            commission: Perbill,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Ensure valid commission (max 100%)
            ensure!(commission <= Perbill::from_percent(100), Error::<T>::InvalidCommission);
            
            // Update validator commission
            Validators::<T>::try_mutate(&who, |validator_info| -> DispatchResult {
                let info = validator_info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                info.commission = commission;
                Ok(())
            })?;
            
            Self::deposit_event(Event::CommissionSet { validator: who, commission });
            Ok(())
        }

        /// Leave validator set (start unbonding self-stake)
        #[pallet::call_index(6)]
        #[pallet::weight(10_000)]
        pub fn leave_candidates(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Get validator info
            let validator_info = Validators::<T>::get(&who)
                .ok_or(Error::<T>::ValidatorNotFound)?;
            
            // Ensure no delegators
            ensure!(validator_info.delegator_count == 0, Error::<T>::TooManyDelegators);
            
            // Check unbonding request limit
            let mut unbonding_requests = UnbondingRequests::<T>::get(&who);
            ensure!(
                unbonding_requests.len() < T::MaxUnbondingRequests::get() as usize,
                Error::<T>::TooManyUnbondingRequests
            );
            
            // Calculate unlock block
            let current_block = frame_system::Pallet::<T>::block_number();
            let unlock_at = current_block + T::UnbondingPeriod::get();
            
            // Add unbonding request for self-stake
            let unbonding_request = UnbondingRequest {
                amount: validator_info.self_stake,
                unlock_at,
            };
            
            unbonding_requests.try_push(unbonding_request)
                .map_err(|_| Error::<T>::TooManyUnbondingRequests)?;
            
            UnbondingRequests::<T>::insert(&who, unbonding_requests);
            
            // Remove validator
            Validators::<T>::remove(&who);
            
            // Update global counters
            TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(validator_info.self_stake));
            ValidatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
            
            Self::deposit_event(Event::ValidatorLeft { validator: who.clone() });
            Self::deposit_event(Event::UnbondingStarted { 
                who, 
                amount: validator_info.self_stake, 
                unlock_at 
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Distribute rewards for the current era
        pub fn distribute_rewards(total_reward: BalanceOf<T>) -> DispatchResult {
            let current_era = CurrentEra::<T>::get();
            let total_staked = TotalStaked::<T>::get();
            
            if total_staked.is_zero() {
                return Ok(());
            }
            
            // Iterate through all validators and distribute rewards
            for (validator_id, validator_info) in Validators::<T>::iter() {
                if validator_info.status != ValidatorStatus::Active {
                    continue;
                }
                
                // Calculate validator's share based on total stake
                let validator_reward = total_reward
                    .saturating_mul(validator_info.total_stake)
                    .saturating_div(total_staked);
                
                if validator_reward.is_zero() {
                    continue;
                }
                
                // Calculate commission
                let commission_amount = validator_info.commission * validator_reward;
                let delegator_reward_pool = validator_reward.saturating_sub(commission_amount);
                
                // Validator gets commission + their proportional share of delegator rewards
                let validator_share = if validator_info.total_stake.is_zero() {
                    validator_reward
                } else {
                    let validator_delegator_share = delegator_reward_pool
                        .saturating_mul(validator_info.self_stake)
                        .saturating_div(validator_info.total_stake);
                    commission_amount.saturating_add(validator_delegator_share)
                };
                
                // Add to validator's pending rewards
                PendingRewards::<T>::mutate(&validator_id, |rewards| {
                    *rewards = rewards.saturating_add(validator_share);
                });
                
                // Distribute remaining rewards to delegators
                let remaining_delegator_rewards = delegator_reward_pool.saturating_sub(
                    delegator_reward_pool
                        .saturating_mul(validator_info.self_stake)
                        .saturating_div(validator_info.total_stake)
                );
                
                if !remaining_delegator_rewards.is_zero() {
                    Self::distribute_delegator_rewards(
                        &validator_id,
                        &validator_info,
                        remaining_delegator_rewards,
                    )?;
                }
            }
            
            // Increment era
            CurrentEra::<T>::mutate(|era| *era = era.saturating_add(1));
            
            Self::deposit_event(Event::RewardsDistributed { 
                era: current_era, 
                total_reward 
            });
            Ok(())
        }
        
        /// Distribute rewards to delegators of a specific validator
        fn distribute_delegator_rewards(
            validator_id: &T::AccountId,
            validator_info: &ValidatorInfo<T::AccountId, BalanceOf<T>>,
            total_delegator_rewards: BalanceOf<T>,
        ) -> DispatchResult {
            let delegated_stake = validator_info.total_stake.saturating_sub(validator_info.self_stake);
            
            if delegated_stake.is_zero() {
                return Ok(());
            }
            
            // Iterate through all delegations to find ones for this validator
            let delegations: Vec<_> = Delegations::<T>::iter()
                .filter(|((_, validator), _)| validator == validator_id)
                .collect();
            
            for ((delegator, _), delegation_amount) in delegations {
                let delegator_reward = total_delegator_rewards
                    .saturating_mul(delegation_amount)
                    .saturating_div(delegated_stake);
                
                if !delegator_reward.is_zero() {
                    PendingRewards::<T>::mutate(&delegator, |rewards| {
                        *rewards = rewards.saturating_add(delegator_reward);
                    });
                }
            }
            
            Ok(())
        }
        
        /// Slash a validator for an offense
        pub fn slash_validator(
            validator: &T::AccountId,
            offense: SlashingOffense,
        ) -> DispatchResult {
            let validator_info = Validators::<T>::get(validator)
                .ok_or(Error::<T>::ValidatorNotFound)?;
            
            // Determine slash percentage based on offense
            let slash_percent = match offense {
                SlashingOffense::ExtendedDowntime => Perbill::from_parts(1_000_000),   // 0.1%
                SlashingOffense::DoubleSigning => Perbill::from_parts(50_000_000),     // 5%
            };
            
            // Calculate slash amount
            let slash_amount = slash_percent * validator_info.total_stake;
            
            if slash_amount.is_zero() {
                return Ok(());
            }
            
            // Slash validator's self-stake first
            let validator_slash = slash_percent * validator_info.self_stake;
            
            // Update validator info
            Validators::<T>::try_mutate(validator, |info| -> DispatchResult {
                let validator_info = info.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                validator_info.self_stake = validator_info.self_stake.saturating_sub(validator_slash);
                validator_info.total_stake = validator_info.total_stake.saturating_sub(slash_amount);
                validator_info.status = ValidatorStatus::Slashed;
                Ok(())
            })?;
            
            // Slash delegators proportionally
            let delegated_slash = slash_amount.saturating_sub(validator_slash);
            if !delegated_slash.is_zero() {
                Self::slash_delegators(validator, delegated_slash, &validator_info)?;
            }
            
            // Update total staked
            TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(slash_amount));
            
            Self::deposit_event(Event::ValidatorSlashed { 
                validator: validator.clone(), 
                amount: slash_amount, 
                offense 
            });
            Ok(())
        }
        
        /// Slash delegators of a validator proportionally
        fn slash_delegators(
            validator: &T::AccountId,
            total_delegator_slash: BalanceOf<T>,
            validator_info: &ValidatorInfo<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            let delegated_stake = validator_info.total_stake.saturating_sub(validator_info.self_stake);
            
            if delegated_stake.is_zero() {
                return Ok(());
            }
            
            // Slash each delegator proportionally
            for ((delegator, validator_key), delegation_amount) in Delegations::<T>::iter() {
                if validator_key != *validator {
                    continue;
                }
                let delegator_slash = total_delegator_slash
                    .saturating_mul(delegation_amount)
                    .saturating_div(delegated_stake);
                
                if !delegator_slash.is_zero() {
                    let new_delegation = delegation_amount.saturating_sub(delegator_slash);
                    
                    if new_delegation.is_zero() {
                        // Remove delegation if fully slashed
                        Delegations::<T>::remove(&delegator, validator);
                        DelegationCount::<T>::mutate(&delegator, |count| *count = count.saturating_sub(1));
                        DelegatorCount::<T>::mutate(validator, |count| *count = count.saturating_sub(1));
                    } else {
                        // Update delegation amount
                        Delegations::<T>::insert(&delegator, validator, new_delegation);
                    }
                }
            }
            
            Ok(())
        }
        
        /// Check if an account is a validator
        pub fn is_validator(account: &T::AccountId) -> bool {
            Validators::<T>::contains_key(account)
        }
        
        /// Get validator information
        pub fn get_validator_info(account: &T::AccountId) -> Option<ValidatorInfo<T::AccountId, BalanceOf<T>>> {
            Validators::<T>::get(account)
        }
        
        /// Get delegation amount
        pub fn get_delegation(delegator: &T::AccountId, validator: &T::AccountId) -> BalanceOf<T> {
            Delegations::<T>::get(delegator, validator)
        }
    }
}
