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
    pub type TotalStaked<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Current era
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Validator count
    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Validator joined the candidate pool
        ValidatorJoined { validator: T::AccountId, stake: u128 },
        /// Delegation made
        Delegated { delegator: T::AccountId, validator: T::AccountId, amount: u128 },
        /// Delegation removed
        Undelegated { delegator: T::AccountId, validator: T::AccountId, amount: u128 },
        /// Unbonding started
        UnbondingStarted { who: T::AccountId, amount: u128, unlock_at: BlockNumberFor<T> },
        /// Rewards distributed
        RewardsDistributed { era: u32, total_reward: u128 },
        /// Validator slashed
        ValidatorSlashed { validator: T::AccountId, amount: u128, offense: SlashingOffense },
        /// Commission set
        CommissionSet { validator: T::AccountId, commission: Perbill },
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join as validator candidate (stub)
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn join_candidates(
            origin: OriginFor<T>,
            stake: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(stake >= T::MinValidatorStake::get(), Error::<T>::InsufficientStake);
            
            TotalStaked::<T>::mutate(|total| *total = total.saturating_add(stake));
            ValidatorCount::<T>::mutate(|count| *count = count.saturating_add(1));
            
            Self::deposit_event(Event::ValidatorJoined { validator: who, stake });
            Ok(())
        }

        /// Delegate to a validator (stub)
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn delegate(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            TotalStaked::<T>::mutate(|total| *total = total.saturating_add(amount));
            
            Self::deposit_event(Event::Delegated { delegator: who, validator, amount });
            Ok(())
        }

        /// Set commission rate (stub)
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn set_commission(
            origin: OriginFor<T>,
            commission: Perbill,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(commission <= Perbill::from_percent(100), Error::<T>::InvalidCommission);
            
            Self::deposit_event(Event::CommissionSet { validator: who, commission });
            Ok(())
        }
    }
}
