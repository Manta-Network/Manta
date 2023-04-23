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

//! # No-Loss-Lottery Module
//!
//! ## Overview
//!
//! This pallet implements a no-loss-lottery by taking user deposits, generating excess funds by staking funds with [`pallet_parachain_staking`]
//! and periodically selects a winner from participating users weighted by their deposit amount to receive a claim to the
//! accrued excess funds.
//! Funds withdrawn from the the lottery are subject to a timelock determined by parachain-staking before they can be claimed.
//!
//! ### Lottery Rules
//! 1. A drawing is scheduled to happen every [`Config::DrawingInterval`] blocks.
//! 2. A designated manager can start & stop the drawings as well as rebalance the stake to improve the yield generated through staking
//! 3. In order to prevent gaming of the lottery drawing mechanism, no modifications to this pallet are allowed [`Config::DrawingFreezeout`] blocks before a drawing
//!     This is needed e.g. using BABE Randomness, where the randomness will be known a day before the scheduled drawing
//! 4. Winnings must be claimed manually by the winner but there is no time limit for claiming winnings
//! 5. Deposits are instantly staked by the pallet
//! 6. Withdrawals must wait for a timelock imposed by [`pallet_parachain_staking`] and are paid out automatically (via scheduler) in the first lottery drawing after it expires
//! 7. The [`Config::ManageOrigin`] must at the same time be allowed to use [`frame_support::traits::schedule::Named`] e.g. `ScheduleOrigin` in `pallet_scheduler`
//!
//! ## Dependencies
//! 1. To enable fair winner selection, a fair and low-influience randomness provider implementing [`frame_support::traits::Randomness`], e.g. pallet_randomness
//! 2. To schedule automatic drawings, a scheduling pallet implementing [`frame_support::traits::schedule::Named`], e.g. pallet_scheduler
//! 3. To generate lottery revenue, [`pallet_parachain_staking`]
//!
//! ## Interface
//!
//! This pallet contains extrinsics callable by any user and a second set of extrinsic callable only by a *Lottery Manager* Origin
//! configurable as [`Config::ManageOrigin`] in the [`Config`] struct of this pallet.
//!
//! ### User Dispatchable Functions
//! * [`Call::deposit`]: Allows any user to deposit tokens into the lottery
//! * [`Call::request_withdraw`]: Allows any user to request return of their deposited tokens to own wallet
//! * [`Call::claim_my_winnings`]: Allows any user to transfer any accrued winnings into their wallet
//!
//! ### Manager Dispatchable Functions
//! * [`Call::start_lottery`]: Schedules periodic lottery drawings to occur each [`Config::DrawingInterval`]
//! * [`Call::stop_lottery`]: Cancels the current drawing and stops scheduling new drawings
//! * [`Call::draw_lottery`]: Immediately executes a lottery drawing
//! * [`Call::process_matured_withdrawals`]: Immediately transfer funds of all matured withdrawals to their respective owner's wallets
//! * [`Call::liquidate_lottery`]: Unstakes all lottery funds and schedules [`Call::process_matured_withdrawals`] after the timelock period
//! * [`Call::rebalance_stake`]: Immediately unstakes overweight collators (with low APY) for later restaking into underweight collators (with high APY)
//!
//! ### Important state queries
//! * [`Pallet::next_drawing_at`]: Block number where the next drawing will happen
//! * [`Pallet::not_in_drawing_freezeout`]: False if deposits/withdrawals are currently frozen
//! * [`Pallet::lottery_funds_surplus_idle`]: Token amount currently in the pallet the winner would get if the drawing was now
//!
//! Please refer to [`Pallet`] for more documentation on each function.
//! Furthermore, the storage items containing all relevant information about lottery state can be queried via e.g. the [polkadot.js API](https://polkadot.js.org/docs/api)

#![cfg_attr(not(feature = "std"), no_std)]

mod staking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    pub use ::function_name::named;
    pub use frame_system::WeightInfo;
    use frame_support::{
        ensure, log,
        pallet_prelude::*,
        traits::{
            schedule::{v2::Named as ScheduleNamed, DispatchTime, MaybeHashed, LOWEST_PRIORITY},
            ExistenceRequirement::KeepAlive,
            *,
        },
        PalletId,
    };
    use frame_system::{pallet_prelude::*, RawOrigin};
    use manta_primitives::constants::time::{DAYS, MINUTES};
    use pallet_parachain_staking::BalanceOf;
    use sp_core::U256;
    use sp_runtime::{
        traits::{
            AccountIdConversion, CheckedAdd, CheckedSub, Dispatchable, Hash, Saturating, Zero,
        },
        ArithmeticError, DispatchResult,
    };
    use sp_std::prelude::*;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    pub type CallOf<T> = <T as Config>::Call;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_parachain_staking::Config {
        type Call: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The Scheduler.
        type Scheduler: ScheduleNamed<
            Self::BlockNumber,
            CallOf<Self>,
            Self::PalletsOrigin,
            Hash = Self::Hash,
        >;
        // Randomness source to use for determining lottery winner
        type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;
        /// Something that can estimate the cost of sending an extrinsic
        type EstimateCallFee: frame_support::traits::EstimateCallFee<
            pallet_parachain_staking::Call<Self>,
            BalanceOf<Self>,
        >;
        /// Origin that can manage lottery parameters and start/stop drawings
        type ManageOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
        /// Overarching type of all pallets origins.
        type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;
        /// Account Identifier from which the internal Pot is generated.
        #[pallet::constant]
        type LotteryPot: Get<PalletId>;
        /// Time in blocks between lottery drawings
        #[pallet::constant]
        type DrawingInterval: Get<Self::BlockNumber>;
        /// Time in blocks *before* a drawing in
        /// Depending on the randomness source, the winner might be established before the drawing, this prevents modification of the eligible winning set after the winner
        /// has been established but before it is selected by [`Call::draw_lottery`] which modifications of the win-eligble pool are prevented
        #[pallet::constant]
        type DrawingFreezeout: Get<Self::BlockNumber>;
        /// Time in blocks until a collator is done unstaking
        #[pallet::constant]
        type UnstakeLockTime: Get<Self::BlockNumber>; // XXX: could maybe alculate this from staking LeaveDelayRounds * DefaultBlocksPerRound
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    // #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    // Configurable (constant) storage items

    /// NOTE: how much KMA to keep in the pallet for gas
    /// This must be initialized at genesis, otherwise the pallet will run out of gas at the first drawing
    #[pallet::storage]
    #[pallet::getter(fn gas_reserve)]
    pub(super) type GasReserve<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn min_deposit)]
    pub(super) type MinDeposit<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn min_withdraw)]
    pub(super) type MinWithdraw<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // Dynamic Storage Items

    /// NOTE: sum of all user's deposits, to ensure balance never drops below
    #[pallet::storage]
    #[pallet::getter(fn sum_of_deposits)]
    pub(super) type SumOfDeposits<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// NOTE: the total pot is the total number of KMA eligible to win in the current drawing cycle
    #[pallet::storage]
    #[pallet::getter(fn total_pot)]
    pub(super) type TotalPot<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_rebalancing)]
    pub(super) type RebalanceInProgress<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    pub(super) type ActiveBalancePerUser<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub(super) type UnclaimedWinningsByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_unclaimed_winnings)]
    pub(super) type TotalUnclaimedWinnings<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[derive(Clone, Encode, Decode, TypeInfo)]
    pub(super) struct UnstakingCollator<AccountId, BlockNumber> {
        pub account: AccountId,
        pub since: BlockNumber,
    }

    #[pallet::storage]
    pub(super) type UnstakingCollators<T: Config> =
        StorageValue<_, Vec<UnstakingCollator<T::AccountId, T::BlockNumber>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn remaining_unstaking_balance)]
    pub(super) type RemainingUnstakingBalance<T: Config> =
        StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[derive(Clone, Encode, Decode, TypeInfo)]
    pub(super) struct Request<AccountId, BlockNumber, Balance> {
        user: AccountId,
        block: BlockNumber,
        balance: Balance,
    }

    #[pallet::storage]
    pub(super) type WithdrawalRequestQueue<T: Config> =
        StorageValue<_, Vec<Request<T::AccountId, T::BlockNumber, BalanceOf<T>>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn staked_collators)]
    pub(super) type StakedCollators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// amount of token to keep in the pot for paying gas fees
        pub gas_reserve: BalanceOf<T>,
        pub min_deposit: BalanceOf<T>,
        pub min_withdraw: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                min_deposit: 1u32.into(),
                min_withdraw: 1u32.into(),
                gas_reserve: 10_000u32.into(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        #[inline]
        fn build(&self) {
            GasReserve::<T>::set(self.gas_reserve);
            MinDeposit::<T>::set(self.min_deposit);
            MinWithdraw::<T>::set(self.min_withdraw);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        LotteryStarted(T::BlockNumber),
        LotteryStopped(T::BlockNumber),
        LotteryWinner(T::AccountId, BalanceOf<T>),
        Deposited(T::AccountId, BalanceOf<T>),
        ScheduledWithdraw(T::AccountId, BalanceOf<T>),
        Withdrawn(T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        LotteryNotStarted,
        LotteryIsRunning,
        LotteryNotScheduled,
        TooEarlyForDrawing,
        TooCloseToDrawing,
        PotBalanceTooLow,
        PotBalanceBelowGasReserve,
        PotBalanceTooLowToPayTxFee,
        NoWinnerFound,
        DepositBelowMinAmount,
        WithdrawBelowMinAmount,
        WithdrawAboveDeposit,
        WithdrawFailed,
        PalletMisconfigured,
        TODO,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Allows any user to deposit tokens into the lottery
        ///
        /// # Arguments
        ///
        /// * `amount` - The amount of tokens to be deposited.
        #[pallet::weight(0)]
        pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let caller_account = ensure_signed(origin)?;

            ensure!(
                amount >= Self::min_deposit(),
                Error::<T>::DepositBelowMinAmount
            );
            ensure!(
                Self::not_in_drawing_freezeout(),
                Error::<T>::TooCloseToDrawing
            );

            // Transfer funds to pot
            <T as pallet_parachain_staking::Config>::Currency::transfer(
                &caller_account.clone(),
                &Self::account_id(),
                amount,
                KeepAlive,
            )?;

            // Attempt to stake them
            for (some_collator, balance) in Self::calculate_deposit_distribution(amount) {
                Self::do_stake_one_collator(some_collator, balance)?;
            }

            // Add to active funds
            ActiveBalancePerUser::<T>::mutate(caller_account.clone(), |balance| *balance += amount);
            TotalPot::<T>::mutate(|balance| *balance += amount);
            SumOfDeposits::<T>::mutate(|balance| *balance += amount);

            Self::deposit_event(Event::Deposited(caller_account.clone(), amount));
            Ok(())
        }

        /// Requests a withdrawal of `amount` from the caller's active funds.
        ///
        /// Withdrawal is not immediate as funds are subject to a timelock imposed by [`pallet_parachain_staking`]
        /// It will be executed with the first [`Call::draw_lottery`] call after timelock expires
        ///
        /// Withdrawals can NOT be cancelled because they inflict ecomomic damage on the lottery through collator unstaking
        /// and the user causing this damage must be subjected to economic consequences for doing so
        ///
        /// The withdrawal is paid from [`RemainingUnstakingBalance`]
        /// If this balance is too low to handle the request, another collator is unstaked
        ///
        /// # Arguments
        ///
        /// * `amount` - the amount of funds to withdraw
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// * `amount` is below the minimum withdraw amount
        /// * `amount` is larger than the user's total deposit
        /// * It is too close to the drawing
        /// * The user has not enough active funds
        /// * There are any arithmetic underflows
        #[pallet::weight(0)]
        pub fn request_withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            ensure!(
                amount >= Self::min_withdraw(),
                Error::<T>::WithdrawBelowMinAmount
            );
            ensure!(
                Self::not_in_drawing_freezeout(),
                Error::<T>::TooCloseToDrawing
            );
            let now = <frame_system::Pallet<T>>::block_number();
            log::debug!("Requesting withdraw of {:?} tokens", amount);
            // Ensure user has enough funds active and mark them as offboarding (remove from `ActiveFundsPerUser`)
            ActiveBalancePerUser::<T>::try_mutate(caller.clone(), |balance| {
                // Withdraw only what's active
                ensure!(*balance >= amount, Error::<T>::WithdrawAboveDeposit);
                // Mark funds as offboarding
                WithdrawalRequestQueue::<T>::mutate(|withdraw_vec| {
                    withdraw_vec.push(Request {
                        user: caller.clone(),
                        block: now,
                        balance: amount,
                    })
                });
                // store reduced balance
                *balance -= amount;
                TotalPot::<T>::try_mutate(|pot| {
                    (*pot)
                        .checked_sub(&amount)
                        .ok_or(ArithmeticError::Underflow)
                })?;
                Ok::<(), DispatchError>(())
            })?;

            // Unstaking workflow
            // 1. See if this withdrawal can be serviced with left-over balance from an already unstaking collator, if so deduct remaining balance and schedule the request
            // 2. If it can't, find the collator with the smallest delegation that is able to handle this withdrawal request and fully unstake it
            // 3. Add balance overshoot to "remaining balance" to handle further requests from

            // If the withdrawal fits in the currently unstaking funds, do nothing else
            RemainingUnstakingBalance::<T>::try_mutate(|remaining_balance| {
                match (*remaining_balance).checked_sub(&amount){
                    Some(subtracted) => {
                        *remaining_balance = subtracted;
                        Ok(())
                    }
                    _ => {
                        Err("not enough left to handle this request from current unstaking funds")
                    }
                }
            })
            .or_else(|_| {
                // Withdrawal needs extra collators to unstake to have enough funds to serve withdrawals, do it
                let reserve = RemainingUnstakingBalance::<T>::get();
                let mut remaining_to_withdraw = amount - reserve;

                // unstake collators as necessary. This updates `RemainingUnstakingBalance`
                for collator_to_unstake in Self::calculate_withdrawal_distribution(remaining_to_withdraw){
                    let our_stake = StakedCollators::<T>::get(collator_to_unstake.clone());
                    remaining_to_withdraw = remaining_to_withdraw.saturating_sub(our_stake);
                    // If this fails, something weird is going on
                    Self::do_unstake_collator(now,collator_to_unstake)?;
                }
                if !remaining_to_withdraw.is_zero() {
                    log::error!("Somehow didn't manage to handle the requested balance. Have {:?} left over",remaining_to_withdraw);
                }
                RemainingUnstakingBalance::<T>::try_mutate(|remaining_balance| {
                    match (*remaining_balance).checked_sub(&amount){
                        Some(subtracted) => {
                            *remaining_balance = subtracted;
                            Ok(())
                        }
                        _ => {
                            Err("not enough left to handle this request after unstaking additional collators")
                        }
                    }
                })
            })?;
            // END UNSTAKING SECTION

            // RAD: What happens if delegation_execute_scheduled_request fails?
            Self::deposit_event(Event::ScheduledWithdraw(caller, amount));
            Ok::<(), DispatchError>(())
        }

        /// Allows the caller to transfer any of the account's previously unclaimed winnings to his their wallet
        ///
        /// # Errors
        ///
        /// CannotLookup: The caller has no unclaimed winnings.
        #[pallet::weight(0)]
        pub fn claim_my_winnings(origin: OriginFor<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            match UnclaimedWinningsByAccount::<T>::take(caller.clone()) {
                Some(winnings) => {
                    // Never pay out winnings if it would reduce pallet funds below total number of user's deposits
                    let all_funds_in_pallet =
                        <T as pallet_parachain_staking::Config>::Currency::total_balance(
                            &Self::account_id(),
                        );
                    ensure!(
                        all_funds_in_pallet.saturating_sub(winnings) >= Self::sum_of_deposits(),
                        Error::<T>::PotBalanceTooLow
                    );

                    TotalUnclaimedWinnings::<T>::try_mutate(|old| {
                        *old = (*old)
                            .checked_sub(&winnings)
                            .ok_or(ArithmeticError::Underflow)?;
                        Ok::<(), ArithmeticError>(())
                    })?;

                    <T as pallet_parachain_staking::Config>::Currency::transfer(
                        &Self::account_id(),
                        &caller,
                        winnings,
                        KeepAlive,
                    ) // NOTE: If the transfer fails, the TXN get rolled back and the winnings stay in the map for claiming later
                }
                None => Err(DispatchError::CannotLookup).into(),
            }
        }

        /// Maximizes staking APY and thus accrued winnings by removing staked tokens from overallocated/inactive
        /// collators and adding to underallocated ones.
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// This function should be called when the pallet's staked tokens are staked with overweight collators
        /// or collators that became inactive or left the staking set.
        /// This will withdraw the tokens from overallocated and inactive collators and wait until the funds are unlocked,
        /// then re-allocate them to underallocated collators.
        ///
        /// Note that this operation can run in parallel with a drawing, but it will reduce the staking revenue
        /// generated in that drawing by the amount of funds being rebalanced.
        ///
        /// # Errors
        ///
        /// * BadOrigin: Caller is not ManageOrigin
        /// * TODO: Amount of tokens to be rebalanced would be too low.
        #[pallet::weight(0)]
        pub fn rebalance_stake(origin: OriginFor<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin)?;

            // withdraw from overallocated collators, wait until funds unlock, re-allocate to underallocated collators
            // TODO: find some balancing algorithm that does this

            // Self::deposit_event(Event::StartedRebalance(amount));
            Ok(())
        }

        /// Starts the lottery by scheduling a [`Call::draw_lottery`] call
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// * BadOrigin: Caller is not ManageOrigin
        /// * The pallet does not have enough funds to pay for gas fees for at least the first drawing.
        /// * The drawing interval is zero or negative.
        /// * The Scheduler implementation failed to schedule the [`Call::draw_lottery`] call.
        ///
        /// # Details
        ///
        /// This function schedules a [`Call::draw_lottery`] call with a delay specified by the [`Config::DrawingInterval`] configuration
        /// using [`frame_support::traits::schedule::Named`] with the lottery pallet's pallet ID configured with [`Config::LotteryPot`] as identifier.
        /// If the lottery is already started, this function will fail.
        ///
        /// You can always learn what block the next drawing - if any - will happen by calling [`Self::next_drawing_at`]
        #[pallet::weight(0)]
        pub fn start_lottery(origin: OriginFor<T>) -> DispatchResult {
            // T::ManageOrigin::ensure_origin(origin.clone())?;

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                Self::next_drawing_at().is_none(),
                Error::<T>::LotteryIsRunning
            );
            // Pallet has enough funds to pay gas fees for at least the first drawing
            ensure!(
                Self::lottery_funds_surplus() >= Self::gas_reserve(),
                Error::<T>::PotBalanceBelowGasReserve
            );

            let drawing_interval = <T as Config>::DrawingInterval::get();
            ensure!(
                drawing_interval > 0u32.into(),
                Error::<T>::PalletMisconfigured
            );
            let lottery_drawing_call: CallOf<T> = Call::draw_lottery {}.into();
            T::Scheduler::schedule_named(
                Self::lottery_schedule_id(),
                DispatchTime::After(drawing_interval),
                Some((drawing_interval, 99999u32)), // XXX: Seems scheduler has no way to schedule infinite amount
                LOWEST_PRIORITY, // TODO: Maybe schedule only one and schedule the next drawing in `draw_lottery`
                frame_support::dispatch::RawOrigin::Root.into(),
                MaybeHashed::Value(lottery_drawing_call),
            )
            .map_err(|_| Error::<T>::TODO)?; // TODO: Error handling

            Self::deposit_event(Event::LotteryStarted(now + drawing_interval));
            Ok(())
        }

        /// Stops the ongoing lottery and cancels the scheduled and any future drawings.
        ///
        /// This function cancels the scheduled drawing and cleans up bookkeeping.
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// # Errors
        ///
        /// * BadOrigin: Caller is not manager
        /// * LotteryNotStarted: Nothing to stop
        ///
        #[pallet::weight(0)]
        pub fn stop_lottery(origin: OriginFor<T>) -> DispatchResult {
            // T::ManageOrigin::ensure_origin(origin.clone())?;

            T::Scheduler::cancel_named(Self::lottery_schedule_id())
                .map_err(|_| Error::<T>::LotteryNotStarted)?;

            let now = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::LotteryStopped(now));
            Ok(())
        }

        /// Draws a lottery winner and allows them to claim their winnings later. Only the [`Config::ManageOrigin`] can execute this function.
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// # Errors
        ///
        /// ## Operational
        /// * BadOrigin: Caller is not ManageOrigin
        /// * TooEarlyForDrawing: The lottery is not ready for drawing yet.
        /// * PotBalanceBelowGasReserve: The balance of the pot is below the gas reserve so no winner will be paid out
        ///
        /// ## Fatal
        /// * ArithmeticError::Underflow: An underflow occurred when calculating the payout.
        /// * PotBalanceTooLow: The balance of the pot is too low.
        /// * NoWinnerFound: Nobody was selected as winner
        #[pallet::weight(0)]
        pub fn draw_lottery(origin: OriginFor<T>) -> DispatchResult {
            // T::ManageOrigin::ensure_origin(origin.clone())?;
            let now = <frame_system::Pallet<T>>::block_number();
            log::trace!("Drawing lottery called at block {:?}", now.clone());
            let total_funds_in_pallet =
                <T as pallet_parachain_staking::Config>::Currency::total_balance(
                    &Self::account_id(),
                );
            let winning_claim = Self::lottery_funds_surplus_idle();
            log::debug!(
                "total funds: {:?}, surplus funds/winner payout: {:?}",
                total_funds_in_pallet.clone(),
                winning_claim.clone()
            );
            if !winning_claim.is_zero(){
                ensure!(
                    // Prevent granting funds to a user that would have to be paid from deposit money (we're a casino, not a bank)
                    total_funds_in_pallet
                        .saturating_sub(winning_claim)
                        .saturating_sub(Self::total_unclaimed_winnings())
                        >= Self::sum_of_deposits(),
                    Error::<T>::PotBalanceTooLow
                );
                Self::select_winner(winning_claim)?;
            }

            // unstake, pay out tokens due for withdrawals and restake excess funds // XXX: This might not work well with multiphase withdrawals
            Self::process_matured_withdrawals(origin)?;
            Self::update_active_funds()?; // TODO: Figure out what to do with this
            Ok(())
        }

        /// This function transfers all withdrawals to user's wallets that are payable from unstaked collators whose timelock expired
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// # Errors
        ///
        /// * BadOrigin: Caller is not ManageOrigin
        /// * errors defined by the do_process_matured_withdrawals function.
        #[pallet::weight(0)]
        pub fn process_matured_withdrawals(origin: OriginFor<T>) -> DispatchResult {
            log::trace!("process_matured_withdrawals");
            // T::ManageOrigin::ensure_origin(origin.clone())?;
            // TODO: these two fns share a duplicate a lot of code, refactor it into here
            Self::finish_unstaking_collators();
            Self::do_process_matured_withdrawals()?;
            Self::do_rebalance_remaining_funds()?;
            Ok(())
        }

        /// Liquidates all funds held in the lottery pallet, unstaking collators, returning user deposits and paying out winnings
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// Due to staking timelock, this schedules the payout of user deposits after timelock has expired.
        /// NOTE: TODO: Any interaction with this pallet is disallowed while a liquidation is ongoing
        ///
        /// # Errors
        ///
        /// * BadOrigin: Caller is not ManageOrigin
        /// * Fails if a lottery has not been stopped and a drawing is ongoing
        #[pallet::weight(0)]
        pub fn liquidate_lottery(origin: OriginFor<T>) -> DispatchResult {
            // T::ManageOrigin::ensure_origin(origin.clone())?;

            ensure!(
                Self::next_drawing_at().is_none(),
                Error::<T>::LotteryIsRunning
            );

            // TODO: Unstake all collators, schedule return of all user deposits
            // for collator in collators_we_staked_to {
            //     do_unstake(collator);
            // }
            // TODO: Lock everything until this process is finished

            // TODO: return user deposits and paying out winnings

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get a unique, inaccessible account id from the `PotId`.
        pub(crate) fn account_id() -> T::AccountId {
            T::LotteryPot::get().into_account_truncating()
        }
        /// Get an identifier for scheduling drawings from the `PotId`.
        fn lottery_schedule_id() -> Vec<u8> {
            T::LotteryPot::get().0.to_vec()
        }

        fn update_active_funds() -> DispatchResult {
            log::trace!("update_active_funds");
            Ok(())
        }

        fn select_winner(payout_for_winner: BalanceOf<T>) -> DispatchResult{
            // Match random number to winner. We select a winning **balance** and then just add up accounts in the order they're stored until the sum of balance exceeds the winning amount
            // IMPORTANT: This order and active balances must be locked to modification after the random seed is created (relay BABE randomness, 2 epochs ago)
            let random = T::RandomnessSource::random(&[0u8, 1]);
            let randomness_established_at_block = random.1;

            // TODO: Ensure freezeout period started before the randomness was known to prevent manipulation of the winning set
            // ensure!(
            //     Self::next_drawing_at().is_some()
            //         && randomness_established_at_block
            //             .saturating_add(<T as Config>::DrawingFreezeout::get())
            //             < Self::next_drawing_at().unwrap(),
            //     Error::<T>::PalletMisconfigured
            // );

            let random_hash = random.0;
            let as_number = U256::from_big_endian(random_hash.as_ref());
            let winning_number = as_number.low_u128();
            let winning_balance: BalanceOf<T> = BalanceOf::<T>::try_from(winning_number)
                .map_err(|_| ArithmeticError::Overflow)?
                % Self::total_pot().into();

            log::debug!(
                "hash: {:?}, winning balance: {:?}",
                random_hash,
                winning_balance
            );
            let mut winner: Option<T::AccountId> = None;
            let mut count: BalanceOf<T> = 0u32.into();
            for (account, balance) in ActiveBalancePerUser::<T>::iter() {
                count += balance;
                if count >= winning_balance {
                    winner = Some(account);
                    break;
                }
            }
            // Should be impossible: If no winner was selected, return Error
            ensure!(winner.is_some(), Error::<T>::NoWinnerFound);

            // Allow winner to manually claim their winnings later
            UnclaimedWinningsByAccount::<T>::mutate(
                winner
                    .clone()
                    .expect("we checked a winner exists before. qed"),
                |maybe_balance| match *maybe_balance {
                    Some(balance) => (*maybe_balance) = Some(balance.saturating_add(payout_for_winner)),
                    None => (*maybe_balance) = Some(payout_for_winner),
                },
            );
            TotalUnclaimedWinnings::<T>::try_mutate(|old| {
                *old = (*old)
                    .checked_add(&payout_for_winner)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok::<(), ArithmeticError>(())
            })?;
            log::debug!(
                "winning of {:?} added to claim for account {:?}",
                payout_for_winner,
                winner
            );
            Self::deposit_event(Event::LotteryWinner(winner.unwrap(), payout_for_winner));
            Ok(())
        }

        /// NOTE: This code assumes UnstakingCollators is sorted
        fn finish_unstaking_collators() {
            let now = <frame_system::Pallet<T>>::block_number();
            let unstaking = UnstakingCollators::<T>::get();
            UnstakingCollators::<T>::try_mutate(|unstaking| {
                let remaining_collators = unstaking.iter().filter_map(|collator| {
                    // Leave collators that are not finished unstaking alone
                    if collator.since + <T as Config>::UnstakeLockTime::get() > now {
                        return Some(collator.clone());
                    };

                    // Recover funds locked in the collator
                    // There can only be one request per collator and it is always a full revoke_delegation call
                    let delegation_requests_against_this_collator = pallet_parachain_staking::Pallet::<T>::delegation_scheduled_requests(collator.account.clone());
                    let balance_to_unstake;
                    match delegation_requests_against_this_collator.iter().find(|request|request.delegator == Self::account_id()){
                        Some(our_request) if matches!(our_request.action, pallet_parachain_staking::DelegationAction::Revoke(_)) => {
                            if T::BlockNumber::from(our_request.when_executable) > now {
                                    log::error!("Collator {:?} finished our unstaking timelock but not the pallet_parachain_staking one. leaving in queue",collator.account.clone());
                                return Some(collator.clone());
                            };
                            balance_to_unstake = our_request.action.amount();
                        }
                        _ => {
                                log::error!( "Expected revoke_delegation request not found on collator {:?}. Leaving in withdraw queue",collator.account.clone() );
                                return Some(collator.clone());
                            }
                    };
                    // Ensure the pallet has enough gas to pay for this
                    let fee_estimate : BalanceOf<T> = T::EstimateCallFee::estimate_call_fee(&pallet_parachain_staking::Call::execute_delegation_request { delegator: Self::account_id() , candidate: collator.account.clone()  }, None.into());
                    if Self::lottery_funds_surplus() <= fee_estimate{
                        log::warn!("could not finish unstaking delegation because the pallet is out of funds to pay TX fees. Skipping");
                        Some(collator.clone());
                    };
                    match pallet_parachain_staking::Pallet::<T>::execute_delegation_request(
                        RawOrigin::Signed(Self::account_id()).into(),
                        Self::account_id(),
                        collator.account.clone(),
                    ){
                        Ok(_) => {
                            // collator was unstaked
                            // TODO: check if other bookkeeping needs updating
                            SumOfDeposits::<T>::mutate(|balance| *balance = (*balance).saturating_sub(balance_to_unstake)); // XXX: Maybe checked_sub
                            log::debug!("Unstaked {:?} from collator {:?}",balance_to_unstake,collator.account.clone());
                            // remove from unstaking collators
                             return None;
                        },
                        Err(e) => {
                            log::error!("Collator finished unstaking timelock but could not be removed with error {:?}",e);
                            return Some(collator.clone());
                        },
                    };
                }).collect::<Vec<_>>();

                if remaining_collators.len() != unstaking.len() {
                    unstaking.clear();
                    for c in remaining_collators {
                        unstaking.push(c.clone());
                    }
                    Ok(())
                } else {
                    Err("no change")
                }
            });
        }

        #[named]
        fn do_rebalance_remaining_funds() -> DispatchResult {
            log::trace!(function_name!());
            // Only restake what isn't needed to service outstanding withdrawal requests

            // TODO: If we have outstanding out-of-timelock withdrawal requests, do nothing
            let stakable_balance = Self::lottery_funds_surplus_idle();
            if stakable_balance.is_zero(){
                log::debug!("Nothing to restake");
                return Ok(());
            }

            for (collator, amount_to_stake) in
                Self::calculate_deposit_distribution(stakable_balance)
            {
                Self::do_stake_one_collator(collator, amount_to_stake)?;
                TotalPot::<T>::mutate(|balance| *balance += amount_to_stake);
            }
            Ok(())
        }

        /// This fn schedules a single shot payout of all matured withdrawals
        /// Main usage: Automatic execution in the course of a drawing
        /// It can also be manually invoke by T::ManageOrigin to reprocess withdrawals that
        /// previously failed, e.g. due to the pallet running out of gas funds
        /// A withdrawal is considered "matured" if its staking timelock expired
        #[named]
        fn do_process_matured_withdrawals() -> DispatchResult {
            log::trace!(function_name!());
            if <WithdrawalRequestQueue<T>>::get().is_empty() {
                return Ok(()); // nothing to do
            }
            let funds_available_to_withdraw = Self::lottery_funds_surplus();
            log::debug!("Withdrawable funds: {:?}",funds_available_to_withdraw.clone());
            if funds_available_to_withdraw.is_zero() {
                return Ok(()); // nothing to do
            }
            let now = <frame_system::Pallet<T>>::block_number();
            // Pay down the list from top (oldest) to bottom until we've paid out everyone or run out of available funds
            <WithdrawalRequestQueue<T>>::mutate(|request_vec|-> Result<(),DispatchError> {
                let mut left_overs: Vec<Request<_, _, _>> = Vec::new();
                for request in request_vec.iter() {
                    // Don't pay anyone still timelocked
                    if now < request.block + <T as Config>::UnstakeLockTime::get(){
                        left_overs.push((*request).clone());
                        continue;
                    }
                    // stop paying people if we've run out of free funds.
                    // The assumption is the collators serving these requests will
                    // finish unstaking next round ( next lottery drawing )
                    if request.balance > funds_available_to_withdraw {
                        left_overs.push((*request).clone());
                        continue;
                    }
                    // we know we can pay this out, do it
                    <SumOfDeposits<T>>::mutate(|sum| (*sum).saturating_sub(request.balance));
                    log::debug!("Transferring {:?} to {:?}",request.balance.clone(), request.user.clone());
                    <T as pallet_parachain_staking::Config>::Currency::transfer(
                        &Self::account_id(),
                        &request.user,
                        request.balance,
                        KeepAlive,
                    )?;
                }
                log::debug!("Have {:?} requests left over after transfers",left_overs.len());
                // Update T::WithdrawalRequestQueue if we paid at least one guy
                if left_overs.len() != (*request_vec).len() {
                    request_vec.clear();
                    for c in left_overs {
                        request_vec.push(c);
                    }
                }
                Ok(())
            })?;
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // public getters for lottery state
        /// Returns the block the next drawing will execute, if any
        pub fn next_drawing_at() -> Option<T::BlockNumber> {
            T::Scheduler::next_dispatch_time(Self::lottery_schedule_id()).ok()
        }
        /// funds in the lottery that are not staked or assigned to previous winners ( can be used to pay TX fees )
        pub fn lottery_funds_surplus() -> BalanceOf<T> {
            let non_staked_funds =
                pallet_parachain_staking::Pallet::<T>::get_delegator_stakable_free_balance(
                    &Self::account_id(),
                );
            non_staked_funds.saturating_sub(Self::total_unclaimed_winnings())
        }
        /// funds in the lottery pallet that are not needed/reserved for anything
        pub fn lottery_funds_surplus_idle() -> BalanceOf<T> {
            // let outstanding_withdrawal_requests = <WithdrawalRequestQueue<T>>::get()
                // .iter()
                // .map(|request| request.balance)
                // .reduce(|acc, e| acc + e)
                // .unwrap_or(0u32.into());

            Self::lottery_funds_surplus()
                .saturating_sub(Self::gas_reserve())
                // .saturating_sub(outstanding_withdrawal_requests)
        }
        /// Returns if we're within the pre-drawing time where deposits/withdrawals are frozen
        pub fn not_in_drawing_freezeout() -> bool {
            match Self::next_drawing_at() {
                Some(drawing) => {
                    let now = <frame_system::Pallet<T>>::block_number();
                    now < drawing
                        .saturating_sub(<T as Config>::DrawingFreezeout::get())
                        .into()
                }
                None => {
                    true // can't be frozen if lottery stopped
                }
            }
        }
    }
}
