// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Balances Pallet
//!
//! The Balances pallet provides functionality for handling accounts and balances.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The Balances pallet provides functions for:
//!
//! - Getting and setting free balances.
//! - Retrieving total, reserved and unreserved balances.
//! - Repatriating a reserved balance to a beneficiary account that exists.
//! - Transferring a balance between accounts (when not reserved).
//! - Slashing an account balance.
//! - Account creation and removal.
//! - Managing total issuance.
//! - Setting and managing locks.
//!
//! ### Terminology
//!
//! - **Existential Deposit:** The minimum balance required to create or keep an account open. This
//!   prevents "dust accounts" from filling storage. When the free plus the reserved balance (i.e.
//!   the total balance) fall below this, then the account is said to be dead; and it loses its
//!   functionality as well as any prior history and all information on it is removed from the
//!   chain's state. No account should ever have a total balance that is strictly between 0 and the
//!   existential deposit (exclusive). If this ever happens, it indicates either a bug in this
//!   pallet or an erroneous raw mutation of storage.
//!
//! - **Total Issuance:** The total number of units in existence in a system.
//!
//! - **Reaping an account:** The act of removing an account by resetting its nonce. Happens after
//!   its
//! total balance has become zero (or, strictly speaking, less than the Existential Deposit).
//!
//! - **Free Balance:** The portion of a balance that is not reserved. The free balance is the only
//!   balance that matters for most operations.
//!
//! - **Reserved Balance:** Reserved balance still belongs to the account holder, but is suspended.
//!   Reserved balance can still be slashed, but only after all the free balance has been slashed.
//!
//! - **Imbalance:** A condition when some funds were credited or debited without equal and opposite
//!   accounting
//! (i.e. a difference between total issuance and account balances). Functions that result in an
//! imbalance will return an object of the `Imbalance` trait that can be managed within your runtime
//! logic. (If an imbalance is simply dropped, it should automatically maintain any book-keeping
//! such as total issuance.)
//!
//! - **Lock:** A freeze on a specified amount of an account's free balance until a specified block
//!   number. Multiple
//! locks always operate over the same funds, so they "overlay" rather than "stack".
//!
//! ### Implementations
//!
//! The Balances pallet provides implementations for the following traits. If these traits provide
//! the functionality that you need, then you can avoid coupling with the Balances pallet.
//!
//! - [`Currency`](frame_support::traits::Currency): Functions for dealing with a
//! fungible assets system.
//! - [`ReservableCurrency`](frame_support::traits::ReservableCurrency):
//! - [`NamedReservableCurrency`](frame_support::traits::NamedReservableCurrency):
//! Functions for dealing with assets that can be reserved from an account.
//! - [`LockableCurrency`](frame_support::traits::LockableCurrency): Functions for
//! dealing with accounts that allow liquidity restrictions.
//! - [`Imbalance`](frame_support::traits::Imbalance): Functions for handling
//! imbalances between total issuance in the system and account balances. Must be used when a
//! function creates new funds (e.g. a reward) or destroys some funds (e.g. a system fee).
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some liquid free balance to another account.
//! - `set_balance` - Set the balances of a given account. The origin of this call must be root.
//!
//! ## Usage
//!
//! The following examples show how to use the Balances pallet in your custom pallet.
//!
//! ### Examples from the FRAME
//!
//! The Contract pallet uses the `Currency` trait to handle gas payment, and its types inherit from
//! `Currency`:
//!
//! ```
//! use frame_support::traits::Currency;
//! # pub trait Config: frame_system::Config {
//! # 	type Currency: Currency<Self::AccountId>;
//! # }
//!
//! pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
//! pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
//!
//! # fn main() {}
//! ```
//!
//! The Staking pallet uses the `LockableCurrency` trait to lock a stash account's funds:
//!
//! ```
//! use frame_support::traits::{WithdrawReasons, LockableCurrency};
//! use sp_runtime::traits::Bounded;
//! pub trait Config: frame_system::Config {
//! 	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
//! }
//! # struct StakingLedger<T: Config> {
//! # 	stash: <T as frame_system::Config>::AccountId,
//! # 	total: <<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance,
//! # 	phantom: std::marker::PhantomData<T>,
//! # }
//! # const STAKING_ID: [u8; 8] = *b"staking ";
//!
//! fn update_ledger<T: Config>(
//! 	controller: &T::AccountId,
//! 	ledger: &StakingLedger<T>
//! ) {
//! 	T::Currency::set_lock(
//! 		STAKING_ID,
//! 		&ledger.stash,
//! 		ledger.total,
//! 		WithdrawReasons::all()
//! 	);
//! 	// <Ledger<T>>::insert(controller, ledger); // Commented out as we don't have access to Staking's storage here.
//! }
//! # fn main() {}
//! ```
//!
//! ## Genesis config
//!
//! The Balances pallet depends on the [`GenesisConfig`].
//!
//! ## Assumptions
//!
//! * Total issued balanced of all accounts should be less than `Config::Balance::max_value()`.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod tests;
mod benchmarking;
pub mod weights;

use codec::{Codec, MaxEncodedLen};
#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use frame_support::{ensure, pallet_prelude::DispatchResult, traits::UnixTime};
use frame_system as system;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
    FixedPointOperand, Saturating,
};
use sp_std::{fmt::Debug, prelude::*};
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The balance of an account.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo
            + FixedPointOperand;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Timestamp provider
        type UnixTime: UnixTime;
    }

    /// The current storage version.
    const STORAGE_VERSION: frame_support::traits::StorageVersion =
        frame_support::traits::StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<(T)>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Transfer the entire transferable balance from the caller account.
        ///
        /// NOTE: This function only attempts to transfer _transferable_ balances. This means that
        /// any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
        /// transferred by this function. To ensure that this function results in a killed account,
        /// you might need to prepare the account by removing any reference counters, storage
        /// deposits, etc...
        ///
        /// The dispatch origin of this call must be Signed.
        ///
        /// - `dest`: The recipient of the transfer.
        /// - `keep_alive`: A boolean to determine if the `transfer_all` operation should send all
        ///   of the funds the account has, causing the sender account to be killed (false), or
        ///   transfer everything except at least the existential deposit, which will guarantee to
        ///   keep the sender account alive (true). # <weight>
        /// - O(1). Just like transfer, but reading the user's transferable balance first.
        ///   #</weight>
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn add_address_to_barrier(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            // todo: check if account exists ?
            <XcmBarrierList<T>>::insert(account, ());
            Ok(())
        }

        ///   #</weight>
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn set_start_unix_time(
            origin: OriginFor<T>,
            start_unix_time: Option<core::time::Duration>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <StartUnixTime<T>>::set(start_unix_time);
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An account was created with some free balance.
        Endowed, // TODO: actual event
    }

    #[pallet::error]
    pub enum Error<T> {
        ///
        XcmTransfersLimitExceeded,
        ///
        XcmTransfersNotAllowedForAccount,
    }
    /// Stores amount of native asset XCM transfers and timestamp of last transfer
    #[pallet::storage]
    pub type XcmNativeTransfers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (T::Balance, u64), OptionQuery>;

    /// Stores limit value
    #[pallet::storage]
    pub type DailyXcmLimit<T: Config> = StorageValue<_, T::Balance, OptionQuery>;

    #[pallet::storage]
    pub type XcmBarrierList<T: Config> = StorageMap<_, Identity, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    pub type RemainingXcmLimit<T: Config> =
        StorageMap<_, Identity, T::AccountId, T::Balance, OptionQuery>;

    /// Stores limit value
    #[pallet::storage]
    pub type LastDayProcessed<T: Config> = StorageValue<_, u64, OptionQuery>;

    /// Stores limit value
    #[pallet::storage]
    pub type StartUnixTime<T: Config> = StorageValue<_, core::time::Duration, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        // TODO: real genesis
        pub balances: Vec<(T::AccountId, T::Balance)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                balances: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            let now = T::UnixTime::now();
            if let Some(start_unix_time) = StartUnixTime::<T>::get() {
                if start_unix_time <= now {
                    let days_since_start =
                        (now.as_secs() - start_unix_time.as_secs()) / (24 * 60 * 60);

                    // TODO: is this default ok ?
                    let last_day_processed = <LastDayProcessed<T>>::get().unwrap_or(0);

                    if days_since_start > last_day_processed {
                        Self::reset_remaining_xcm_limit();
                        <LastDayProcessed<T>>::put(days_since_start);
                    }
                }
            }

            //T::WeightInfo::on_initialize()
            Weight::from_ref_time(0) // TODO: use the commented out line
        }
    }
}

#[cfg(feature = "std")]
impl<T: Config> GenesisConfig<T> {
    /// Direct implementation of `GenesisBuild::build_storage`.
    ///
    /// Kept in order not to break dependency.
    pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
        <Self as GenesisBuild<T>>::build_storage(self)
    }

    /// Direct implementation of `GenesisBuild::assimilate_storage`.
    ///
    /// Kept in order not to break dependency.
    pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
        <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
    }
}

const XCM_LIMIT_PERIOD_IN_SEC: u64 = 86400; // 1 day

impl<T: Config> orml_traits::xcm_transfer::NativeBarrier<T::AccountId, T::Balance> for Pallet<T> {
    fn ensure_xcm_transfer_limit_not_exceeded(
        account_id: &T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult {
        if let Some(transfer_limit) = DailyXcmLimit::<T>::get() {
            // The address is not in the barrier list, so we don't care about it
            if <XcmBarrierList<T>>::get(account_id).is_none() {
                return Ok(());
            }

            let now = T::UnixTime::now().as_secs();
            let current_period = (now / XCM_LIMIT_PERIOD_IN_SEC) * XCM_LIMIT_PERIOD_IN_SEC;
            let (mut transferred, last_transfer) = XcmNativeTransfers::<T>::get(account_id)
                .ok_or(Error::<T>::XcmTransfersNotAllowedForAccount)?;
            let remaining_limit = RemainingXcmLimit::<T>::get(account_id).unwrap_or(transfer_limit);

            if last_transfer < current_period {
                transferred = Default::default();
                XcmNativeTransfers::<T>::insert(account_id, (transferred, now));
            };

            ensure!(
                transferred + amount <= remaining_limit,
                Error::<T>::XcmTransfersLimitExceeded
            );

            // If the ensure didn't return an error, update the native transfers
            Self::update_xcm_native_transfers(account_id, amount);
        }

        Ok(())
    }

    fn update_xcm_native_transfers(account_id: &T::AccountId, amount: T::Balance) {
        <XcmNativeTransfers<T>>::mutate_exists(account_id, |maybe_transfer| match maybe_transfer {
            Some((current_amount, last_transfer)) => {
                *current_amount = *current_amount + amount;
                *last_transfer = T::UnixTime::now().as_secs();
            }
            None => {}
        });

        <RemainingXcmLimit<T>>::mutate_exists(
            account_id,
            |maybe_remainder| match maybe_remainder {
                Some(remainder) => {
                    *remainder = remainder.saturating_sub(amount);
                }
                None => {}
            },
        );
    }
}

impl<T: Config> Pallet<T> {
    fn reset_remaining_xcm_limit() {
        if let Some(daily_limit) = <DailyXcmLimit<T>>::get() {
            for (account_id, _) in XcmBarrierList::<T>::iter() {
                let mut remaining_limit =
                    <RemainingXcmLimit<T>>::get(&account_id).unwrap_or(daily_limit);
                remaining_limit += daily_limit;
                <RemainingXcmLimit<T>>::insert(&account_id, remaining_limit);
            }
        }
    }
}
