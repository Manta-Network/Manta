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
//! Enhanced staking mechanism for Chameleon Network with advanced delegation features
//! and reward optimization.
//!
//! ## Overview
//!
//! This pallet extends the base parachain-staking functionality with:
//! - Enhanced delegation with no minimum amounts
//! - Performance-based reward distribution
//! - Automated slashing for poor performance
//! - Governance participation bonuses
//!
//! ## Features
//!
//! ### Delegation
//! - No minimum delegation amounts (accessible to all users)
//! - Instant delegation, 14-day unbonding
//! - Proportional rewards minus validator commission
//!
//! ### Reward Optimization
//! - Rewards based on stake + performance + uptime
//! - Bonus for governance participation
//! - Automatic slashing for downtime and double-signing
//!
//! ### Validator Performance
//! - Uptime monitoring
//! - Block production tracking
//! - Governance participation tracking

#![cfg_attr(not(feature = "std"), no_std)]

mod delegation;
mod rewards;
mod slashing;
mod types;
mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::pallet;
pub use delegation::*;
pub use rewards::*;
pub use slashing::*;
pub use types::*;
pub use weights::WeightInfo;

#[pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{
            Currency, Get, LockIdentifier, LockableCurrency, ReservableCurrency,
            WithdrawReasons,
        },
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use manta_primitives::chameleon_constants::{
        emission::*,
        staking::*,
        time::*,
        CHAMELEON_STAKING_PALLET_ID,
    };
    use pallet_parachain_staking::{self as parachain_staking};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
        Perbill, Percent,
    };
    use sp_std::vec::Vec;

    /// Lock identifier for staking
    pub const STAKING_ID: LockIdentifier = *b"chmlstak";

    /// The current storage version
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Balance type alias
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Block number type alias
    pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

    #[pallet::config]
    pub trait Config: frame_system::Config + parachain_staking::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency used for staking
        type Currency: Currency<Self::AccountId>
            + ReservableCurrency<Self::AccountId>
            + LockableCurrency<Self::AccountId>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// The pallet ID for the staking pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Maximum number of delegations per delegator
        #[pallet::constant]
        type MaxDelegationsPerDelegator: Get<u32>;

        /// Maximum number of delegators per validator
        #[pallet::constant]
        type MaxDelegatorsPerValidator: Get<u32>;

        /// Minimum validator stake
        #[pallet::constant]
        type MinValidatorStake: Get<BalanceOf<Self>>;

        /// Unbonding period in blocks
        #[pallet::constant]
        type UnbondingPeriod: Get<BlockNumberOf<Self>>;

        /// Slashing rate for downtime
        #[pallet::constant]
        type SlashingDowntime: Get<Perbill>;

        /// Slashing rate for double signing
        #[pallet::constant]
        type SlashingDoubleSign: Get<Perbill>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A delegation was made [delegator, validator, amount]
        Delegated {
            delegator: T::AccountId,
            validator: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// A delegation was removed [delegator, validator, amount]
        Undelegated {
            delegator: T::AccountId,
            validator: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// Unbonding started [who, amount, unlock_at]
        UnbondingStarted {
            who: T::AccountId,
            amount: BalanceOf<T>,
            unlock_at: BlockNumberOf<T>,
        },
        /// Unbonding completed [who, amount]
        UnbondingCompleted {
            who: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// Rewards distributed [era, total_reward]
        RewardsDistributed {
            era: u32,
            total_reward: BalanceOf<T>,
        },
        /// Validator slashed [validator, amount, offense]
        ValidatorSlashed {
            validator: T::AccountId,
            amount: BalanceOf<T>,
            offense: SlashingOffense,
        },
        /// Validator commission set [validator, commission]
        CommissionSet {
            validator: T::AccountId,
            commission: Perbill,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Validator not found
        ValidatorNotFound,
        /// Delegation not found
        DelegationNotFound,
        /// Invalid amount (zero or negative)
        InvalidAmount,
        /// Insufficient balance
        InsufficientBalance,
        /// Too many delegators for this validator
        TooManyDelegators,
        /// Too many delegations for this delegator
        TooManyDelegations,
        /// Validator stake below minimum
        ValidatorStakeBelowMin,
        /// Unbonding request not found
        UnbondingRequestNotFound,
        /// Unbonding not ready yet
        UnbondingNotReady,
        /// Invalid commission rate
        InvalidCommission,
        /// Validator already exists
        ValidatorAlreadyExists,
        /// Not a validator
        NotAValidator,
    }

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

    /// Delegation records storage
    #[pallet::storage]
    #[pallet::getter(fn delegations)]
    pub type Delegations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // delegator
        Blake2_128Concat,
        T::AccountId, // validator
        Delegation<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    /// Unbonding requests storage
    #[pallet::storage]
    #[pallet::getter(fn unbonding_requests)]
    pub type UnbondingRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Vec<UnbondingRequest<BalanceOf<T>, BlockNumberOf<T>>>,
        ValueQuery,
    >;

    /// Current era index
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Total staked amount in the network
    #[pallet::storage]
    #[pallet::getter(fn total_staked)]
    pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Initial validators with their stakes
        pub validators: Vec<(T::AccountId, BalanceOf<T>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                validators: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (validator, stake) in &self.validators {
                let validator_info = ValidatorInfo {
                    controller: validator.clone(),
                    stash: validator.clone(),
                    self_stake: *stake,
                    total_stake: *stake,
                    delegator_count: 0,
                    commission: Perbill::from_percent(10), // Default 10% commission
                    status: ValidatorStatus::Active,
                    performance: ValidatorPerformance {
                        uptime_percent: Perbill::one(),
                        blocks_produced: 0,
                        blocks_missed: 0,
                        last_active_era: 0,
                    },
                };
                Validators::<T>::insert(validator, validator_info);
                TotalStaked::<T>::mutate(|total| *total = total.saturating_add(*stake));
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Delegate CHML to a validator
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::delegate())]
        pub fn delegate(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let delegator = ensure_signed(origin)?;
            Self::do_delegate(delegator, validator, amount)
        }

        /// Remove delegation from a validator
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::undelegate())]
        pub fn undelegate(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResult {
            let delegator = ensure_signed(origin)?;
            Self::do_undelegate(delegator, validator)
        }

        /// Start unbonding process
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::unbond())]
        pub fn unbond(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_unbond(who, amount)
        }

        /// Withdraw unbonded funds
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::withdraw_unbonded())]
        pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_withdraw_unbonded(who)
        }

        /// Set validator commission rate
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::set_commission())]
        pub fn set_commission(
            origin: OriginFor<T>,
            commission: Perbill,
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            Self::do_set_commission(validator, commission)
        }

        /// Join as validator candidate
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::join_candidates())]
        pub fn join_candidates(
            origin: OriginFor<T>,
            bond: BalanceOf<T>,
        ) -> DispatchResult {
            let candidate = ensure_signed(origin)?;
            Self::do_join_candidates(candidate, bond)
        }

        /// Leave validator candidate pool
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::leave_candidates())]
        pub fn leave_candidates(origin: OriginFor<T>) -> DispatchResult {
            let candidate = ensure_signed(origin)?;
            Self::do_leave_candidates(candidate)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberOf<T>> for Pallet<T> {
        fn on_initialize(block_number: BlockNumberOf<T>) -> Weight {
            // Check if we need to distribute rewards (every era)
            let era_length = T::UnbondingPeriod::get() / 14u32.into(); // Approximate era length
            if block_number % era_length == Zero::zero() {
                let current_era = Self::current_era();
                let _ = Self::distribute_era_rewards(current_era);
                CurrentEra::<T>::put(current_era.saturating_add(1));
            }

            // Process unbonding requests
            let _ = Self::process_unbonding_requests(block_number);

            T::WeightInfo::on_initialize()
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the pallet account ID
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}