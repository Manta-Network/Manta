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

//! # Lottery Pallet generating funds with parachain_staking
//! User funds are staked and the lottery draws its prize pool from the staking rewards accrued during the lottery period
//! Funds deposited to the lottery become eligible to win after one drawing.
//! Funds withdrawn from the the lottery are subject to a timelock determined by parachain-staking before they can be claimed.
//!
//! ### Rules
//! A drawing is scheduled to happen every `<LotteryInterval<T>>::get()` blocks.
//! A designated manager can start & stop the drawings as well as rebalance the stake to better collators
//! Winnings are paid out directly to the winner's wallet after each drawing
//! In order to prevent gaming of the lottery winner, no modifications to this pallet are allowed a configurable amount of time before a drawing
//! This is needed e.g. using BABE Randomness, where the randomness will be known a day before the scheduled drawing

use frame_support::{pallet_prelude::*, traits::Randomness, PalletId};
use frame_system::pallet_prelude::*;
use pallet_scheduler::Call as SchedulerCall;
use sp_runtime::{traits::Hash, DispatchResult};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::WeightInfo;
    use frame_support::traits::{Currency, Randomness};
    use manta_primitives::{types::BlockNumber, constants::time::DAYS};
    use pallet_parachain_staking::BalanceOf;

    const TIME_BETWEEN_DRAWINGS: BlockNumber = 1 * manta_primitives::WEEK; // 1 WEEK in seconds

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_scheduler::Config {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;
        // Randomness source to use for determining lottery winner
        type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;
        /// Origin that can manage lottery parameters and start/stop drawings
        type ManageOrigin: EnsureOrigin<Self::Origin>;
        /// Account Identifier from which the internal Pot is generated.
        type LotteryPot: Get<PalletId>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn lottery_interval)]
    type LotteryInterval<T: Config> = StorageValue<_, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_rebalancing)]
    type RebalanceInProgress<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Depending on the randomness source, the winner might be established before the drawing, this prevents modification of the eligible winning set after the winner
    /// has been established but before it is selected by fn draw_lottery()
    #[pallet::storage]
    #[pallet::getter(fn drawing_buffer)]
    type PreDrawingModificationLockBlocks<T: Config> = StorageValue<_, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_drawing)]
    type NextDrawingAt<T: Config> = StorageValue<_, BlockNumber, OptionQuery>;

    /// NOTE: the total pot is the total number of KMA eligible to win in the current drawing cycle
    #[pallet::storage]
    #[pallet::getter(fn total_pot)]
    type TotalPot<T: Config> = StorageValue<_, BalanceOf, ValueQuery>;

    /// NOTE: sum of all user's deposits, to ensure balance never drops below
    #[pallet::storage]
    #[pallet::getter(fn sum_of_deposits)]
    type SumOfDeposits<T: Config> = StorageValue<_, BalanceOf, ValueQuery>;
    /// NOTE: how much KMA to keep in the pallet for gas
    /// This must be initialized at genesis, otherwise the pallet will run out of gas at the first drawing

    #[pallet::storage]
    #[pallet::getter(fn gas_reserve)]
    type GasReserve<T: Config> = StorageValue<_, BalanceOf, ValueQuery>;

    /// This value is imposed by the staking solution used and must be configured >= than what it uses
    #[pallet::storage]
    #[pallet::getter(fn unstake_time)]
    type UnstakeTime<T: Config> = StorageValue<_, BlockNumber, ValueQuery>;

    #[pallet::storage]
    type ActiveBalancePerUser<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AcccountId, T::BalanceOf>;

    #[pallet::storage]
    type RemainingBalanceOnUnstakingCollator<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AcccountId, T::BalanceOf>;

    // enum DepositState {
    //     Onboarding,
    //     Offboarding,
    // }
    struct Request {
        user: T::AccountId,
        block: T::BlockNumber,
        balance: T::BalanceOf
    }
    // #[pallet::storage]
    // type RequestQueue<T: Config> = StorageValue<_, Vec<Request>, ValueQuery>;

    #[pallet::storage]
    type DepositRequestQueue<T: Config> = StorageValue<_, Vec<Request>, ValueQuery>;

    #[pallet::storage]
    type WithdrawalRequestQueue<T: Config> = StorageValue<_, Vec<Request>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub lottery_interval: BlockNumber,
        pub drawing_freezeout: BlockNumber,
        pub unstake_time: BlockNumber,
        /// amount of token to keep in the pot for paying gas fees
        pub gas_reserve: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                lottery_interval: 7 * DAYS,
                drawing_freezeout: 1 * DAYS,
                unstake_time: 7 * DAYS,
                gas_reserve: 100_000 * KMA
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        AddedToLottery(T::AccountId, T::BalanceOf),
        LotteryStarted(T::BlockNumber),
        LotteryStopped(T::BlockNumber),
        LotteryWinner(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        OnlyRootOrigin,
        OnlyRootOrManageOrigin,
        LotteryNotStarted,
        LotteryAlreadyStarted,
        LotteryAlreadyStopped,
        LotteryNotScheduled,
        TooEarlyForDrawing,
        TooCloseToDrawing
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            ensure_signed(origin)?;

            // Ensure we're not too close to the drawing
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!( now < next_drawing() - drawing_buffer(),
                    Error::<T>::TooCloseToDrawing
            );

            // Transfer funds to pot
            Currency::transfer(origin, LotteryPot::get().into_account(), amount)?;

            do_stake(amount)?;

            // TODO: Benchmark this based on length of vector
            DepositRequestQueue::<T>::mutate(|deposit_vec|{
                deposit_vec.push(Request{origin.into_account(),now,amount})
            });

            Self::deposit_event(Event::AddedToLottery(origin, amount));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn request_withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            // Ensure we're not too close to the drawing
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!( now < next_drawing() - drawing_buffer(),
                    Error::<T>::TooCloseToDrawing
            );

            // Ensure user has enough funds active and mark them as offboarding
            ActiveBalancePerUser::<T>::try_mutate(caller, |balance|{
                // Withdraw only what's active
                ensure!(balance >= amount);
                // Mark funds as offboarding
                WithdrawalRequestQueue::<T>::mutate(|withdraw_vec|{
                    withdraw_vec.push(Request{caller,now,amount})
                });
                // store reduced balance
                balance - amount
                T::TotalPot::mutate(|pot|{pot-amount});
            });

            // Revoke delegation of those funds
            // TODO: Once we have multiple collators we need a method to decide from whom to unbond
            pallet_parachain_staking::<T>::delegation_schedule_bond_decrease(
                LotteryPot::get().into_account(),
                some_collator,
                amount,
            );
            // or
            // pallet_parachain_staking::<T>::schedule_revoke_delegation(LotteryPot::into_account(),some_collator);

            // schedule payout after T::ReduceBondDelay expires
            // RAD: What happens if delegation_execute_scheduled_request fails?
            // TODO: pallet_scheduler::<T>::schedule(batch(delegation_execute_scheduled_request(),transfer_to_user))

            Self::deposit_event(Event::scheduled_withdraw(origin, amount));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn cancel_withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            ensure_signed(origin)?;

            // TODO: Ensure we're not too close to the drawing
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!( now + drawing_buffer() < next_drawing(),
                    Error::<T>::TooCloseToDrawing
            );

            // TODO: Mark funds as eligible again -> onboarding
            pallet_parachain_staking::<T>::cancel_delegation_request(
                LotteryPot::get().into_account(),
                some_collator,
                amount,
            );

            // Revoke delegation of those funds
            // TODO: Once we have multiple collators we need a method to decide from whom to unbond
            pallet_parachain_staking::<T>::delegation_schedule_bond_decrease(
                LotteryPot::get().into_account(),
                some_collator,
                amount,
            );
            // or
            // pallet_parachain_staking::<T>::schedule_revoke_delegation(LotteryPot::into_account(),some_collator);

            // schedule payout after T::ReduceBondDelay expires
            // RAD: What happens if delegation_execute_scheduled_request fails?
            // TODO: pallet_scheduler::<T>::schedule(batch(delegation_execute_scheduled_request(),transfer_to_user))

            Self::deposit_event(Event::scheduled_withdraw(origin, amount));
            Ok(())
        }

        /// Rebalances stake by removing stake from overallocated collators and adding to underallocated
        ///
        /// It may be necessary to call this if large amounts of token become unstaked, e.g. due to a collator leaving
        #[pallet::weight(0)]
        pub fn rebalance_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            ensure_root_or_manager(origin)?;

            // withdraw from overallocated collators, wait until funds unlock, re-allocate to underallocated collators
            // RAD: This can run in parallel with a drawing, it will just reduce the staking revenue generated in this drawing by the amount of funds being rebalanced
            // TODO: find some balancing algorithm that does this

            // Self::deposit_event(Event::AddedToLottery(origin,amount));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn start_lottery(origin: OriginFor<T>) -> DispatchResult {
            ensure_root_or_manager(origin)?;
            // TODO: Check that the pallet has enough funds to pay gas fees for a drawing

            let now = <frame_system::Pallet<T>>::block_number();
            let drawing_scheduled_at = now + TIME_BETWEEN_DRAWINGS;
            // Schedule the draw lottery call to autoreschedule periodically, it will fail if ID already exists
            pallet_scheduler::Pallet::<T>::schedule_named(
                origin,
                LotteryPot::get().0,
                drawing_scheduled_at,
                TIME_BETWEEN_DRAWINGS,
                pallet_scheduler::pallet::<T>::Priority::LOWEST_PRIORITY,
                SchedulerCall::call(
                    LotteryPot::get().into_account(),
                    Call::draw_lottery.encode(),
                ),
            )
            .map_err(|_| Error::<T>::LotteryAlreadyStarted)?;

            Self::deposit_event(Event::LotteryStarted(scheduled_time));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn stop_lottery(origin: OriginFor<T>) -> DispatchResult {
            ensure_root_or_manager(origin)?;

            ensure!(
                pallet_scheduler::Pallet::<T>::Lookup::contains_key(LotteryPot::get().0),
                Error::<T>::LotteryNotScheduled
            );

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(maybe_scheduled.is_some(), Error::<T>::LotteryNotScheduled);

            pallet_scheduler::Pallet::<T>::cancel_named(origin, LotteryPot::get().0)
                .map_err(|_| Error::<T>::LotteryAlreadyStopped)?;

            Self::deposit_event(Event::LotteryStopped(now));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn draw_lottery(origin: OriginFor<T>) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure_eq!(origin = LotteryPot::get()); // Allow only this pallet to execute

            let pot_account_id = LotteryPot::get().into_account();

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now >= next_drawing(), Error::<T>::TooEarlyForDrawing);

            let funds_in_pot = <Balances<T> as Currency<_>>::total_balance(pot_account_id);
            let total_deposits = SumOfDeposits::<T>::get();

            // always keep some funds for gas
            let payout = funds_in_pot - total_deposits - gas_reserve;

            if (payout > 0){
                // select winner
                let random = T::Randomness::random(); // established a few hours ago by BABE
                // TODO: Match random number to winner

                // Ensure no deposits are lost
                ensure!(funds_in_pot - payout >= total_deposits);

                // TODO: Transfer winnings to winner ( keep some change for gas? )


                Self::deposit_event(Event::LotteryWinner(winner_account));
            }

            // TODO: Update bookkeeping
            process_requests();
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn process_requests(origin: OriginFor<T>) -> DispatchResult {
            ensure_signed(origin)?;

            update_active_funds();
            schedule_withdrawal_payouts();
            Ok(())
        }
    }

    fn do_stake(amount: BalanceOf<T>) -> DispatchResult {
                   // TODO: choose collator, split amount over multiple?
                   let some_collator: T::AccountId = ();
                   pallet_parachain_staking::<T>::delegate(
                       LotteryPot::get().into_account(),
                       some_collator,
                       amount,
                   )
    }

    fn update_active_funds() -> DispatchResult {
        // TODO: move onboarded funds to active
        // TODO: move cancelled offboarding funds to active
        Ok(())
    }

    /// This fn schedules a single shot payout of all matured withdrawals
    /// It is meant to be executed in the course of a drawing
    fn schedule_withdrawal_payouts(origin: Origin) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        T::WithdrawalRequestQueue::try_mutate(|request_vec|{
        ensure!(request_vec.len() > 0);
        let left_overs;
        for request in reqs{
            if (now < request.block + unstake_time() ) {
                // too early to withdraw this request
                left_overs.push(request);
                continue;
            }
            T::SumOfDeposits::mutate(|sum|{sum-request.amount});

            // TODO: immediately execute unstake
            pallet_parachain_staking::<T>::execute_delegation_request(
                origin,
                LotteryPot::get().into_account_truncating(),
                some_collator
            );

            // TODO: schedule transfer offboarded funds to owner
            let res = T::Currency::transfer(
                &LotteryPot::get().into_account(),
                &request.user,
                request.amount,
                KeepAlive,
            );
        }
        // Update T::WithdrawalRequestQueue if changed
        if left_overs.len() {
            request_vec = left_overs;
            Ok()
        } else {
            Err("no changes")
        }
        });
        Ok(())
    }

    fn ensure_root_or_manager<T: Config>(origin: OriginFor<T>) -> DispatchResult {
        ensure!(
            // TODO: Schedule origin must be council and root, not root only
            frame_system::Pallet::<T>::is_root(origin),
            // frame_system::Pallet::<T>::is_root(origin) || T::ManageOrigin::ensure_origin(origin)
            Error::<T>::OnlyRootOrManageOrigin
        );
        Ok(())
    }
}
