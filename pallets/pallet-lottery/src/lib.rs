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
//! * [`Call::draw_lottery`]: Immediately executes a lottery drawing ( can be called manually even if lottery is stopped )
//! * [`Call::process_matured_withdrawals`]: Immediately transfer funds of all matured withdrawals to their respective owner's wallets
//! * [`Call::liquidate_lottery`]: Unstakes all lottery funds and schedules [`Call::process_matured_withdrawals`] after the timelock period
//! * [`Call::rebalance_stake`]: Immediately unstakes overweight collators (with low APY) for later restaking into underweight collators (with high APY)
//!
//! ### Important state queries callable via RPC
//! * [`Pallet::next_drawing_at`]: Block number where the next drawing will happen
//! * [`Pallet::not_in_drawing_freezeout`]: False if deposits/withdrawals are currently frozen
//! * [`Pallet::current_prize_pool`]: Token amount currently in the pallet the winner would get if the drawing was now
//! Call these from a frontend as e.g.
//! ```bash
//!    curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"lottery_next_drawing_at","params": []}'
//!    curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"lottery_current_prize_pool","params": []}'
//!    curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"lottery_not_in_drawing_freezeout","params": []}'
//! ```
//!
//! Please refer to [`Pallet`] for more documentation on each function.
//! Furthermore, the storage items containing all relevant information about lottery state can be queried via e.g. the [polkadot.js API](https://polkadot.js.org/docs/api)

#![cfg_attr(not(feature = "std"), no_std)]

mod staking;

#[cfg(feature = "rpc")]
pub mod rpc;
pub mod runtime;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    pub use ::function_name::named;
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
    use pallet_parachain_staking::BalanceOf;
    use sp_arithmetic::traits::SaturatedConversion;
    use sp_core::U256;
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedAdd, CheckedSub, Dispatchable, Saturating, Zero},
        ArithmeticError, DispatchResult,
    };
    use sp_std::prelude::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    pub type CallOf<T> = <T as Config>::RuntimeCall;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_parachain_staking::Config {
        /// The aggregated `RuntimeCall` type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + From<Call<Self>>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
            > + frame_support::traits::EstimateCallFee<Call<Self>, BalanceOf<Self>>;
        /// Origin that can manage lottery parameters and start/stop drawings
        type ManageOrigin: EnsureOrigin<Self::RuntimeOrigin>;
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

    /// sum of all user's deposits, to ensure balance never drops below
    /// Incremented on [`Call::deposit`]
    /// Decremented on withdrawal to user wallet in [`Call::process_matured_withdrawals`]
    #[pallet::storage]
    #[pallet::getter(fn sum_of_deposits)]
    pub(super) type SumOfDeposits<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Total number of token eligible to win in the current drawing cycle
    /// Incremented on [`Call::deposit`]
    /// Decremented on [`Call::request_withdraw`]
    #[pallet::storage]
    #[pallet::getter(fn total_pot)]
    pub(super) type TotalPot<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_users)]
    pub(super) type TotalUsers<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_rebalancing)]
    pub(super) type RebalanceInProgress<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_balance_per_user)]
    pub(super) type ActiveBalancePerUser<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn unclaimed_winnings_by_account)]
    pub(super) type UnclaimedWinningsByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    /// Free balance in the pallet that belongs to a previous lottery winner
    /// Incremented on winner election in the course of a drawing
    /// Decremented on transfer of winnings to ower wallet in [`Call::claim_my_winnings`]
    #[pallet::storage]
    #[pallet::getter(fn total_unclaimed_winnings)]
    pub(super) type TotalUnclaimedWinnings<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Free balance in the pallet that was unstaked from a collator and is needed for future withdrawal requests
    /// Incremented on successful unstaking of a collator
    /// Decremented on transfer of funds to withdrawer and on restaking of funds a collator
    #[pallet::storage]
    #[pallet::getter(fn unlocked_unstaking_funds)]
    pub(super) type UnlockedUnstakingFunds<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[derive(Clone, Encode, Decode, TypeInfo)]
    pub(super) struct UnstakingCollator<AccountId, BlockNumber> {
        pub account: AccountId,
        pub since: BlockNumber,
    }

    #[pallet::storage]
    pub(super) type UnstakingCollators<T: Config> =
        StorageValue<_, Vec<UnstakingCollator<T::AccountId, T::BlockNumber>>, ValueQuery>;

    /// This is balance unstaked from a collator that is not needed to service user's withdrawal requests
    /// Incremented on initiation of a collator unstake in [`Call::request_withdraw`]
    /// Decremented on [`Call::request_withdraw`] (no collator unstake) and [`Call::rebalance_stake`] (restaking of surplus funds)
    #[pallet::storage]
    #[pallet::getter(fn surplus_unstaking_balance)]
    pub(super) type SurplusUnstakingBalance<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
    pub struct Request<AccountId, BlockNumber, Balance> {
        pub user: AccountId,
        pub block: BlockNumber,
        pub balance: Balance,
    }

    #[pallet::storage]
    #[pallet::getter(fn withdrawal_request_queue)]
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
        LotteryStarted,
        LotteryStopped,
        LotteryWinner {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        Deposited {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        ScheduledWithdraw {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        Withdrawn {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        Claimed {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Lottery has not been started
        LotteryNotStarted,
        /// Lottery has already been started
        LotteryIsRunning,
        /// Pre-drawing freeze in effect, can't modify balances
        TooCloseToDrawing,
        /// FATAL: Assigning/Transferring winning claims
        /// would **remove** user deposited funds from pallet
        PotBalanceTooLow,
        /// FATAL: Can't stake the requested amount with available funds
        PotBalanceTooLowToStake,
        /// Pallet balance is lower than the needed gas fee buffer
        PotBalanceBelowGasReserve,
        /// Pallet balance is too low to submit a needed transaction
        PotBalanceTooLowToPayTxFee,
        /// No funds eligible to win
        NobodyPlaying,
        /// No funds to win in pool
        NothingToWin,
        /// Fatal: No winner could be selected
        NoWinnerFound,
        /// Deposit amount is below minimum amount
        DepositBelowMinAmount,
        /// Requested Withdrawal amount is below minimum
        WithdrawBelowMinAmount,
        /// Requested to withdraw more than you deposited
        WithdrawAboveDeposit,
        /// No deposits found for this account
        NoDepositForAccount,
        /// Fatal: No collators found to assign this deposit to
        NoCollatorForDeposit,
        /// Fatal: No collators found to withdraw from
        NoCollatorForWithdrawal,
        /// Fatal: A calculation that must not be negative would underflow
        ArithmeticUnderflow,
        /// Fatal: A calculation that would overflow
        ArithmeticOverflow,
        /// Fatal: Pallet configuration violates sanity checks
        PalletMisconfigured,
        /// Fatal: Could not schedule lottery drawings
        CouldNotSchedule,
        /// Fatal: Functionality not yet supported
        NotImplemented,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Allows any user to deposit tokens into the lottery
        ///
        /// # Arguments
        ///
        /// * `amount` - The amount of tokens to be deposited.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::deposit(Pallet::<T>::total_users(), pallet_parachain_staking::Pallet::<T>::selected_candidates().len() as u32))]
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
            ensure! { // Sanity check: make sure we dont accept deposits that will fail in staking
                Self::min_deposit() >= <T as pallet_parachain_staking::Config>::MinDelegation::get(),
                Error::<T>::PalletMisconfigured
            };

            // Transfer funds to pot
            <T as pallet_parachain_staking::Config>::Currency::transfer(
                &caller_account,
                &Self::account_id(),
                amount,
                KeepAlive,
            )?;

            // Attempt to stake them
            let collator_balance_pairs = Self::calculate_deposit_distribution(amount);
            ensure!(
                !collator_balance_pairs.is_empty(),
                Error::<T>::NoCollatorForDeposit
            );
            for (some_collator, balance) in collator_balance_pairs {
                // TODO: What if the `balance` is below `MinDelegation`a on a new collator? this will fail
                Self::do_stake_one_collator(some_collator, balance)?;
            }

            // Add to active funds
            ActiveBalancePerUser::<T>::mutate(caller_account.clone(), |balance| *balance += amount);
            TotalPot::<T>::mutate(|balance| *balance += amount);
            TotalUsers::<T>::mutate(|users| *users += 1);
            SumOfDeposits::<T>::mutate(|balance| *balance += amount);
            Self::deposit_event(Event::Deposited {
                account: caller_account,
                amount,
            });
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
        /// The withdrawal is paid from [`SurplusUnstakingBalance`]
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
        /// * The user has no or not enough active funds
        /// * There are any arithmetic underflows
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::request_withdraw(Pallet::<T>::total_users(), pallet_parachain_staking::Pallet::<T>::selected_candidates().len() as u32))]
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
            // Ensure user has enough funds active and mark them as offboarding (remove from `ActiveBalancePerUser`)
            ActiveBalancePerUser::<T>::try_mutate_exists(caller.clone(), |maybe_balance| {
                match maybe_balance {
                    None => Err(Error::<T>::NoDepositForAccount),
                    Some(balance) => {
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
                        *maybe_balance = match balance
                            .checked_sub(&amount)
                            .ok_or(Error::<T>::ArithmeticUnderflow)?
                        {
                            new_balance if new_balance.is_zero() => {
                                // remove user if this was his last remaining funds
                                TotalUsers::<T>::try_mutate(|users| {
                                    *users = (*users)
                                        .checked_sub(1u32)
                                        .ok_or(Error::<T>::ArithmeticUnderflow)?;
                                    Ok(())
                                })?;
                                None
                            }
                            new_balance => Some(new_balance),
                        };
                        TotalPot::<T>::try_mutate(|pot| {
                            *pot = (*pot)
                                .checked_sub(&amount)
                                .ok_or(Error::<T>::ArithmeticUnderflow)?;
                            Ok(())
                        })?;
                        Ok(())
                    }
                }
            })?;

            // Unstaking workflow
            // 1. See if this withdrawal can be serviced with left-over balance from an already unstaking collator, if so deduct remaining balance and schedule the request
            // 2. If it can't, find the collator with the smallest delegation that is able to handle this withdrawal request and fully unstake it
            // 3. Add balance overshoot to "remaining balance" to handle further requests from

            // If the withdrawal fits in the currently unstaking funds, do nothing else
            SurplusUnstakingBalance::<T>::try_mutate(|remaining_balance| {
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
                let reserve = SurplusUnstakingBalance::<T>::get();
                let mut remaining_to_withdraw = amount - reserve;

                // unstake collators as necessary. This updates `SurplusUnstakingBalance`
                for collator_to_unstake in Self::calculate_withdrawal_distribution(remaining_to_withdraw){
                    let our_stake = StakedCollators::<T>::get(collator_to_unstake.clone());
                    remaining_to_withdraw = remaining_to_withdraw.saturating_sub(our_stake);
                    // The following call updates `SurplusUnstakingBalance` with newly unstaked funds
                    Self::do_unstake_collator(now,collator_to_unstake)?;
                }
                if !remaining_to_withdraw.is_zero() {
                    return Err("FATAL: Didn't unstake the full requested balance (or more)");
                }
                SurplusUnstakingBalance::<T>::try_mutate(|remaining_balance| {
                    match (*remaining_balance).checked_sub(&amount){
                        Some(subtracted) => {
                            *remaining_balance = subtracted;
                            Ok(())
                        }
                        _ => {
                            Err("not enough unstaking balance to handle request after unstaking additional collators")
                        }
                    }
                })
            })?;
            // END UNSTAKING SECTION
            Self::deposit_event(Event::ScheduledWithdraw {
                account: caller,
                amount,
            });
            Ok(())
        }

        /// Allows the caller to transfer any of the account's previously unclaimed winnings to his their wallet
        ///
        /// # Errors
        ///
        /// CannotLookup: The caller has no unclaimed winnings.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::claim_my_winnings(pallet_parachain_staking::Pallet::<T>::selected_candidates().len() as u32))]
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
                        // Sanity check: Never pay out funds that would draw on user deposits
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
                    )?; // NOTE: If the transfer fails, the TXN get rolled back and the winnings stay in the map for claiming later
                    Self::deposit_event(Event::Claimed {
                        account: caller,
                        amount: winnings,
                    });
                    Ok(())
                }
                None => Err(DispatchError::CannotLookup),
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
        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        pub fn rebalance_stake(origin: OriginFor<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            Err(crate::pallet::DispatchError::Other(
                Error::<T>::NotImplemented.into(),
            ))

            // withdraw from overallocated collators, wait until funds unlock, re-allocate to underallocated collators
            // TODO: find some balancing algorithm that does this or just reuse the one we use for deposits

            // Self::deposit_event(Event::StartedRebalance(amount));
            // Ok(())
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
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::start_lottery())]
        pub fn start_lottery(origin: OriginFor<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            ensure!(
                Self::next_drawing_at().is_none(),
                Error::<T>::LotteryIsRunning
            );
            // Pallet has enough funds to pay gas fees for at least the first drawing
            ensure!(
                Self::surplus_funds() >= Self::gas_reserve(),
                Error::<T>::PotBalanceBelowGasReserve
            );
            // NOTE: If more than gas_reserve is in the pallet, the full excess will be paid out to the winner of the next drawing! This is intended to dope the winning balance with extra rewards

            let drawing_interval = <T as Config>::DrawingInterval::get();
            ensure!(
                drawing_interval > 0u32.into(),
                Error::<T>::PalletMisconfigured
            );
            let lottery_drawing_call: CallOf<T> = Call::draw_lottery {}.into();
            T::Scheduler::schedule_named(
                Self::lottery_schedule_id(),
                DispatchTime::After(drawing_interval),
                Some((drawing_interval, u32::MAX)), // XXX: Seems scheduler has no way to schedule infinite amount
                LOWEST_PRIORITY,
                frame_support::dispatch::RawOrigin::Root.into(),
                MaybeHashed::Value(lottery_drawing_call),
            )
            .map_err(|_| Error::<T>::CouldNotSchedule)?;

            Self::deposit_event(Event::LotteryStarted);
            Ok(())
        }

        /// Stops the ongoing lottery and cancels the scheduled and any future drawings.
        ///
        /// This function cancels the scheduled drawing. Does not prevent users from interacting with the pallet
        ///
        /// Can only be called by the account set as [`Config::ManageOrigin`]
        ///
        /// # Errors
        ///
        /// * BadOrigin: Caller is not manager
        /// * LotteryNotStarted: Nothing to stop
        ///
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::stop_lottery())]
        pub fn stop_lottery(origin: OriginFor<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            T::Scheduler::cancel_named(Self::lottery_schedule_id())
                .map_err(|_| Error::<T>::LotteryNotStarted)?;
            Self::deposit_event(Event::LotteryStopped);
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
        /// * PotBalanceBelowGasReserve: The balance of the pot is below the gas reserve so no winner will be paid out
        ///
        /// ## Fatal
        /// * ArithmeticError::Underflow: An underflow occurred when calculating the payout.
        /// * PotBalanceTooLow: The balance of the pot is too low.
        /// * NoWinnerFound: Nobody was selected as winner
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::draw_lottery(Pallet::<T>::total_users(), pallet_parachain_staking::Pallet::<T>::selected_candidates().len() as u32))]
        pub fn draw_lottery(origin: OriginFor<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            let now = <frame_system::Pallet<T>>::block_number();
            log::trace!("Drawing lottery called at block {:?}", now.clone());

            let total_funds_in_pallet =
                <T as pallet_parachain_staking::Config>::Currency::total_balance(
                    &Self::account_id(),
                );
            // all surplus tokens accrued at this point can be paid out to the winner
            let winning_claim = Self::current_prize_pool();
            let participating_funds = Self::total_pot();
            log::debug!(
                "drawing: total funds: {:?}, participating funds: {:?}, surplus funds/winner payout: {:?}",
                total_funds_in_pallet.clone(),
                participating_funds.clone(),
                winning_claim.clone()
            );
            // If there's nothing to win or nobody is playing we skip the drawing logic
            if !winning_claim.is_zero() && !participating_funds.is_zero() {
                ensure!(
                    // Sanity check: Prevent allocating funds as winnings to a user that would have to be paid from user deposits
                    Self::sum_of_deposits()                                 // all users' deposits (staked and unstaking)
                        .saturating_add(Self::total_unclaimed_winnings())   // all prior winnings
                        .saturating_add(winning_claim)                      // and the current winner's new claim
                        <= total_funds_in_pallet, // don't exceed funds in the pallet
                    Error::<T>::PotBalanceTooLow
                );
                Self::select_winner(winning_claim)?;
            } else {
                log::debug!(
                    "drawing: skipped due to zero winning claim {:?} or participating funds {:?}",
                    winning_claim,
                    participating_funds,
                );
            }
            // unstake, pay out tokens due for withdrawals and restake excess funds
            // At this point, all excess funds except for `gas_reserve` have been reserved for the current winner
            Self::process_matured_withdrawals(origin)?;
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
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::process_matured_withdrawals())]
        pub fn process_matured_withdrawals(origin: OriginFor<T>) -> DispatchResult {
            log::trace!("process_matured_withdrawals");
            T::ManageOrigin::ensure_origin(origin.clone())?;
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
        #[pallet::call_index(8)]
        #[pallet::weight(0)]
        pub fn liquidate_lottery(origin: OriginFor<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;

            ensure!(
                Self::next_drawing_at().is_none(),
                Error::<T>::LotteryIsRunning
            );

            Err(crate::pallet::DispatchError::Other(
                Error::<T>::NotImplemented.into(),
            ))

            // TODO: Unstake all collators, schedule return of all user deposits
            // for collator in collators_we_staked_to {
            //     do_unstake(collator);
            // }
            // TODO: Lock everything until this process is finished

            // TODO: return user deposits and pay out winnings, deposit event

            // Ok(())
        }
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::set_min_deposit())]
        pub fn set_min_deposit(origin: OriginFor<T>, min_deposit: BalanceOf<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            ensure!(
                min_deposit >= Self::min_withdraw(),
                Error::<T>::PalletMisconfigured
            );
            MinDeposit::<T>::set(min_deposit);
            Ok(())
        }
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::set_min_withdraw())]
        pub fn set_min_withdraw(
            origin: OriginFor<T>,
            min_withdraw: BalanceOf<T>,
        ) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            MinWithdraw::<T>::set(min_withdraw);
            Ok(())
        }
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::set_gas_reserve())]
        pub fn set_gas_reserve(origin: OriginFor<T>, gas_reserve: BalanceOf<T>) -> DispatchResult {
            T::ManageOrigin::ensure_origin(origin.clone())?;
            GasReserve::<T>::set(gas_reserve);
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
        fn select_winning_balance(
            max_winning_balance: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, Error<T>> {
            const MAX_NUMBER_OF_RESAMPLES: u8 = 3;
            let mut winning_number = 0; // XXX: This shouldn't need initialization but the compiler doesn't get it
            for n in 0u8..MAX_NUMBER_OF_RESAMPLES {
                let random: (T::Hash, BlockNumberFor<T>);
                #[cfg(feature = "runtime-benchmarks")]
                {
                    use rand::{RngCore, SeedableRng};
                    use sp_runtime::traits::Hash;
                    // XXX: Benchmarking randomness changes per block instead of per epoch
                    let mut rng = rand::rngs::StdRng::seed_from_u64(
                        <frame_system::Pallet<T>>::block_number()
                            .try_into()
                            .unwrap_or(n as u64),
                    );
                    let mut rnd = [0u8; 32];
                    rng.fill_bytes(&mut rnd);
                    let randomness = T::Hashing::hash(&rnd);
                    random = (randomness, <T as frame_system::Config>::BlockNumber::zero());
                    log::debug!("select-winner using randomness {:?}", random);
                }
                #[cfg(not(feature = "runtime-benchmarks"))]
                {
                    random = T::RandomnessSource::random(&[n; 1]);
                    log::debug!("select-winner using randomness {:?}", random);
                    // TODO: The following check needs a change to pallet randomness but is static,
                    //       so this can be done manually on deployment of the pallet
                    // ensure!(
                    //     random.1 = randomness_established_at_block
                    //         .saturating_add(<T as Config>::DrawingFreezeout::get())
                    //         < <frame_system::Pallet<T>>::block_number(),
                    //     Error::<T>::PalletMisconfigured
                    // );
                }
                let random_hash = random.0;
                let as_number = U256::from_big_endian(random_hash.as_ref());
                winning_number = as_number.low_u128();
                // naive application of the modulo operation can bias the result, reject and resample if the number is larger than the maximum divisor of user array length in the u128 number range
                debug_assert_eq!(
                    core::mem::size_of::<BalanceOf<T>>(),
                    core::mem::size_of::<u128>()
                );
                let number_to_start_rejecting_at: u128 = u128::max_value().saturating_sub(
                    u128::max_value() % max_winning_balance.saturated_into::<u128>(),
                );
                debug_assert!((number_to_start_rejecting_at
                    % max_winning_balance.saturated_into::<u128>())
                .is_zero());
                if winning_number.saturated_into::<u128>() < number_to_start_rejecting_at {
                    break;
                } else {
                    // sample must be rejected because it can't be safely modulo'd, retry with high u128
                    winning_number = (as_number >> 128).low_u128();
                    let number_to_start_rejecting_at: u128 = u128::max_value().saturating_sub(
                        u128::max_value() % max_winning_balance.saturated_into::<u128>(),
                    );
                    debug_assert!(
                        number_to_start_rejecting_at
                            <= max_winning_balance.saturated_into::<u128>()
                    );
                    if winning_number.saturated_into::<u128>() < number_to_start_rejecting_at {
                        break;
                    }
                    if n + 1 < MAX_NUMBER_OF_RESAMPLES {
                        // if still not good, we need to re-request randomness with a changed nonce
                        continue;
                    } else {
                        // we loop this up to 3 times (yielding 6 possible values) before giving up, printing a warning and accepting that the result was biased
                        log::warn!("No unbiased random samples found after {:?} retries. using {:?} which will be subject to modulo bias",(n+1)*2,winning_number.clone());
                        break;
                    }
                }
            }
            // no risk of modulo bias here unless we ran out of retries above
            let winning_balance: BalanceOf<T> = BalanceOf::<T>::try_from(winning_number)
                .map_err(|_| Error::<T>::ArithmeticOverflow)?
                % max_winning_balance;
            log::debug!(
                "winning_number: {:?}, winning balance: {:?}",
                winning_number,
                winning_balance
            );
            Ok(winning_balance)
        }
        fn select_winner(payout_for_winner: BalanceOf<T>) -> DispatchResult {
            if payout_for_winner.is_zero() {
                return Err(Error::<T>::NothingToWin.into());
            }
            let participating_funds = Self::total_pot();
            if participating_funds.is_zero() {
                return Err(Error::<T>::NobodyPlaying.into());
            }
            // Match random number to winner. We select a winning **balance** and then just add up accounts in the order they're stored until the sum of balance exceeds the winning amount
            // IMPORTANT: This order and active balances must be locked to modification after the random seed is created (relay BABE randomness, 2 epochs ago)
            let winning_balance = Self::select_winning_balance(participating_funds)?;
            let mut maybe_winner: Option<T::AccountId> = None;
            let mut count: BalanceOf<T> = 0u32.into();
            for (account, balance) in ActiveBalancePerUser::<T>::iter() {
                count += balance;
                if count >= winning_balance {
                    maybe_winner = Some(account);
                    break;
                }
            }
            // Should be impossible: If no winner was selected, return Error
            ensure!(maybe_winner.is_some(), Error::<T>::NoWinnerFound);
            let winner = maybe_winner.expect("we checked a winner exists before. qed");
            // Allow winner to manually claim their winnings later
            UnclaimedWinningsByAccount::<T>::mutate(winner.clone(), |maybe_balance| {
                *maybe_balance = Some(
                    maybe_balance
                        .unwrap_or_else(|| 0u32.into())
                        .saturating_add(payout_for_winner),
                );
            });
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
            Self::deposit_event(Event::LotteryWinner {
                account: winner,
                amount: payout_for_winner,
            });
            Ok(())
        }

        /// Unstake any collators we can unstake
        /// This is infallible, if any step fails we just leave the collator in the request queue
        fn finish_unstaking_collators() {
            let now = <frame_system::Pallet<T>>::block_number();
            let mut unstaking = UnstakingCollators::<T>::get();
            let original_len = unstaking.len();
            if unstaking.is_empty() {
                return;
            };
            // Unstake what we can (false), leave the rest (true)
            unstaking.retain(|collator|{
                    // Leave collators that are not finished unstaking alone
                    if collator.since + <T as Config>::UnstakeLockTime::get() > now {
                        return true;
                    };
                    // Recover funds locked in the collator
                    // There can only be one request per collator and it is always a full revoke_delegation call
                    let delegation_requests_against_this_collator = pallet_parachain_staking::Pallet::<T>::delegation_scheduled_requests(collator.account.clone());
                    let balance_to_unstake = match delegation_requests_against_this_collator.iter().find(|request|request.delegator == Self::account_id()){
                        Some(our_request) if matches!(our_request.action, pallet_parachain_staking::DelegationAction::Revoke(_)) => {
                            if T::BlockNumber::from(our_request.when_executable) > now {
                                log::error!("Collator {:?} finished lottery unstaking timelock but not the pallet_parachain_staking one. leaving in queue", collator.account.clone());
                                return true;
                            };
                            our_request.action.amount()
                        }
                        _ => {
                                log::error!( "Expected revoke_delegation request not found on collator {:?}. Leaving in withdraw queue", collator.account.clone() );
                                return true;
                            }
                    };
                    // Ensure the pallet has enough gas to pay for this. Should never run out as long as its's called from `draw_lottery`
                    let fee_estimate : BalanceOf<T> = T::EstimateCallFee::estimate_call_fee(&pallet_parachain_staking::Call::execute_delegation_request { delegator: Self::account_id() , candidate: collator.account.clone()  }, None::<u64>.into());
                    if Self::surplus_funds() <= fee_estimate{
                        log::warn!("could not finish unstaking delegation because the pallet is out of funds to pay TX fees. Skipping");
                        return true;
                    };
                    match pallet_parachain_staking::Pallet::<T>::execute_delegation_request(
                        RawOrigin::Signed(Self::account_id()).into(),
                        Self::account_id(),
                        collator.account.clone(),
                    ){
                        Err(e) => {
                            log::error!("Collator finished unstaking timelock but could not be removed with error {:?}",e);
                            true
                        },
                        Ok(_) => {
                            // collator was unstaked, its funds are now "free balance", we track it so it won't be given to the next winner
                            log::debug!("Unstaked {:?} from collator {:?}",balance_to_unstake,collator.account.clone());
                            <UnlockedUnstakingFunds<T>>::mutate(|unlocked| *unlocked = (*unlocked).saturating_add(balance_to_unstake));
                            <StakedCollators<T>>::remove(collator.account.clone());
                            // don't retain this collator in the unstaking collators vec
                            false
                        },
                    }
            });
            if original_len != unstaking.len() {
                log::debug!(
                    "Finished unstaking {:?} out of {:?} requests",
                    original_len - unstaking.len(),
                    original_len
                );
                UnstakingCollators::<T>::put(unstaking);
            }
        }

        #[named]
        fn do_rebalance_remaining_funds() -> DispatchResult {
            log::trace!(function_name!());
            // NOTE: This fn assumes `finish_unstaking_collators` and `process_outstanding_withdrawals`
            // were previously called and all unlockable funds are claimed

            // Only restake what
            // - isn't needed to service the still outstanding withdrawal requests
            // - is funds that were previously unstaked
            // - is surplus funds (we may have some from `finish_unstaking_collators`)
            // NOTE: Funds tracked in `surplus_unstaking_balance` might still be partially stake locked
            let outstanding_balance_to_withdraw = <WithdrawalRequestQueue<T>>::get()
                .iter()
                .map(|request| request.balance)
                .reduce(|acc, balance| acc + balance)
                .unwrap_or_else(|| 0u32.into());
            let restakable_balance =
                Self::unlocked_unstaking_funds().saturating_sub(outstanding_balance_to_withdraw);
            if restakable_balance < Self::min_deposit() {
                log::debug!(
                    "Restakable balance of {:?} is below staking minimum of {:?}. Not restaking",
                    restakable_balance,
                    Self::min_deposit()
                );
                return Ok(());
            }
            let collator_balance_pairs = Self::calculate_deposit_distribution(restakable_balance);
            if collator_balance_pairs.is_empty() {
                log::debug!(
                    "No collators for redepositing available (likely all currently unstaking)"
                );
                return Ok(());
            }
            for (collator, amount_to_stake) in collator_balance_pairs {
                Self::do_stake_one_collator(collator.clone(), amount_to_stake)?;
                log::debug!(
                    "Rebalanced {:?} to collator {:?}",
                    amount_to_stake,
                    collator
                );
            }
            SurplusUnstakingBalance::<T>::try_mutate(|bal| -> DispatchResult {
                *bal = (*bal)
                    .checked_sub(&restakable_balance)
                    .ok_or(Error::<T>::ArithmeticUnderflow)?;
                Ok(())
            })?;
            UnlockedUnstakingFunds::<T>::try_mutate(|unlocked| -> DispatchResult {
                *unlocked = (*unlocked)
                    .checked_sub(&restakable_balance)
                    .ok_or(Error::<T>::ArithmeticUnderflow)?;
                Ok(())
            })?;
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
            let now = <frame_system::Pallet<T>>::block_number();
            log::debug!(
                "Serving withdrawals from unlocked unstaking funds of {:?}",
                Self::unlocked_unstaking_funds()
            );
            // Pay down the list from top (oldest) to bottom until we've paid out everyone or run out of available funds
            <WithdrawalRequestQueue<T>>::mutate(|request_vec| -> Result<(), DispatchError> {
                let mut left_overs: Vec<Request<_, _, _>> = Vec::new();
                for request in request_vec.iter() {
                    let funds_available_to_withdraw = Self::unlocked_unstaking_funds();
                    // Don't pay anyone unless we have surplus funds
                    if funds_available_to_withdraw.is_zero() {
                        left_overs.push((*request).clone());
                        continue;
                    }
                    // Don't pay anyone still timelocked
                    if request.block + <T as Config>::UnstakeLockTime::get() > now {
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
                    <SumOfDeposits<T>>::mutate(|sum| *sum = (*sum).saturating_sub(request.balance));
                    log::debug!(
                        "Transferring {:?} to {:?}",
                        request.balance.clone(),
                        request.user.clone()
                    );
                    <T as pallet_parachain_staking::Config>::Currency::transfer(
                        &Self::account_id(),
                        &request.user,
                        request.balance,
                        KeepAlive,
                    )?;
                    <UnlockedUnstakingFunds<T>>::try_mutate(|funds| -> DispatchResult {
                        *funds = (*funds)
                            .checked_sub(&request.balance)
                            .ok_or(Error::<T>::ArithmeticUnderflow)?;
                        Ok(())
                    })?;
                    Self::deposit_event(Event::Withdrawn {
                        account: request.user.clone(),
                        amount: request.balance,
                    });
                }
                log::debug!(
                    "Have {:?} requests, {:?} free unstaking and {:?} surplus funds left over after transfers",
                    left_overs.len(),
                    Self::unlocked_unstaking_funds(),
                    Self::surplus_funds()
                );
                // Update T::WithdrawalRequestQueue by mutating `request_vec` if we paid at least one guy
                if left_overs.len() != (*request_vec).len() {
                    request_vec.clear();
                    request_vec.append(&mut left_overs);
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
        /// funds in the lottery that are not staked, unstaked-pending-restaking or assigned to previous winners ( can be used to pay TX fees )
        pub(crate) fn surplus_funds() -> BalanceOf<T> {
            // Returns all funds the pallet holds that are not locked in staking
            // Notably excludes `ActiveBalancePerUser` and the still locked part of `SurplusUnstakingBalance`
            let non_staked_funds =
                pallet_parachain_staking::Pallet::<T>::get_delegator_stakable_free_balance(
                    &Self::account_id(),
                );
            // unclaimed winnings are unlocked balance sitting in the pallet until a user claims, ensure we don't touch these
            let unclaimed = Self::total_unclaimed_winnings();
            // Parts of `SurplusUnstakingBalance` become unlocked in `finish_unstaking_collators` once out of staking timelock
            // It is possible they are only partially restaked in the same TX, the other part staying unlocked,
            // waiting to serve a pending withdrawal in the next cycle.
            // These free funds must not be touched until then, so we don't consider this balance a surplus
            let unlocked = Self::unlocked_unstaking_funds();

            non_staked_funds
                .saturating_sub(unclaimed)
                .saturating_sub(unlocked)
        }
        /// funds in the lottery pallet that are not needed/reserved for anything and can be paid to the next winner
        pub fn current_prize_pool() -> BalanceOf<T> {
            // Ensure we keep a gas reserve from the staking rewards to be able to pay tx fees for staking/unstaking and withdrawals
            Self::surplus_funds().saturating_sub(Self::gas_reserve())
        }
        /// Returns if we're within the pre-drawing time where deposits/withdrawals are frozen
        pub fn not_in_drawing_freezeout() -> bool {
            match Self::next_drawing_at() {
                None => {
                    true // can't be frozen if lottery stopped
                }
                Some(drawing) => {
                    let now = <frame_system::Pallet<T>>::block_number();
                    now < drawing.saturating_sub(<T as Config>::DrawingFreezeout::get())
                }
            }
        }
    }
}
