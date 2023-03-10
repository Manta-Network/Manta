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
//! 1. A drawing is scheduled to happen every `<LotteryInterval<T>>::get()` blocks.
//! 2. A designated manager can start & stop the drawings as well as rebalance the stake to better collators
//! 3. Winnings are paid out directly to the winner's wallet after each drawing
//! 4. In order to prevent gaming of the lottery winner, no modifications to this pallet are allowed a configurable amount of time before a drawing
//!     This is needed e.g. using BABE Randomness, where the randomness will be known a day before the scheduled drawing
//! 5. Deposits happen instantly
//! 6. Withdrawals have a up-to 7 day timelock and are paid out automatically (via scheduler) in the first lottery drawing after it expires

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

pub use pallet::*;

// pub use weights::WeightInfo;
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        ensure, log,
        pallet_prelude::*,
        traits::{schedule::LOWEST_PRIORITY, ExistenceRequirement::KeepAlive, *},
        PalletId,
    };
    pub use frame_system::WeightInfo;
    use frame_system::{pallet_prelude::*, RawOrigin};
    use manta_primitives::constants::time::DAYS;
    use sp_core::U256;
    use sp_runtime::{
        traits::{Hash, Saturating},
        DispatchResult,
    };
    use sp_std::prelude::*;

    use sp_runtime::traits::AccountIdConversion;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_scheduler::Config + pallet_parachain_staking::Config
    {
        /// Overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The currency mechanism.
        type Currency: LockableCurrency<Self::AccountId>
            + From<<Self as pallet_parachain_staking::Config>::Currency>;
        // Randomness source to use for determining lottery winner
        type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;
        /// Origin that can manage lottery parameters and start/stop drawings
        type ManageOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
        /// Account Identifier from which the internal Pot is generated.
        type LotteryPot: Get<PalletId>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    // #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    // Configurable (constant) storage items

    #[pallet::storage]
    #[pallet::getter(fn lottery_interval)]
    pub(super) type LotteryInterval<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// Depending on the randomness source, the winner might be established before the drawing, this prevents modification of the eligible winning set after the winner
    /// has been established but before it is selected by fn draw_lottery()
    #[pallet::storage]
    #[pallet::getter(fn drawing_buffer)]
    pub(super) type PreDrawingModificationLockBlocks<T: Config> =
        StorageValue<_, T::BlockNumber, ValueQuery>;

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

    /// This value is imposed by the staking solution used and must be configured >= than what it uses
    #[pallet::storage]
    #[pallet::getter(fn unstake_time)]
    pub(super) type UnstakeTime<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

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
    #[pallet::getter(fn next_drawing)]
    pub(super) type NextDrawingAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_rebalancing)]
    pub(super) type RebalanceInProgress<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    pub(super) type ActiveBalancePerUser<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[derive(Clone, Encode, Decode, TypeInfo)]
    pub(super) struct UnstakingCollator<AccountId, BlockNumber> {
        account: AccountId,
        since: BlockNumber,
    }

    #[pallet::storage]
    pub(super) type UnstakingCollators<T: Config> =
        StorageValue<_, Vec<UnstakingCollator<T::AccountId, T::BlockNumber>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn remaining_unstaking_balance)]
    pub(super) type RemainingUnstakingBalance<T: Config> =
        StorageValue<_, BalanceOf<T>, ValueQuery>;
    // type RemainingBalanceOnUnstakingCollator<T: Config> =
    //     StorageMap<_, Blake2_128Concat, T::AcccountId, T::BalanceOf>;

    #[derive(Clone,Encode, Decode, TypeInfo)]
    pub(super) struct Request<AccountId, BlockNumber, Balance> {
        user: AccountId,
        block: BlockNumber,
        balance: Balance,
    }

    #[pallet::storage]
    pub(super) type WithdrawalRequestQueue<T: Config> =
        StorageValue<_, Vec<Request<T::AccountId, T::BlockNumber, BalanceOf<T>>>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub lottery_interval: T::BlockNumber,
        pub drawing_freezeout: T::BlockNumber,
        pub unstake_time: T::BlockNumber,
        /// amount of token to keep in the pot for paying gas fees
        pub gas_reserve: BalanceOf<T>,
        pub min_deposit: BalanceOf<T>,
        pub min_withdraw: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                lottery_interval: (7 * DAYS).into(),
                drawing_freezeout: (1 * DAYS).into(),
                min_deposit: 1u32.into(),
                min_withdraw: 1u32.into(),
                unstake_time: (7 * DAYS).into(),
                gas_reserve: 10_000u32.into(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        #[inline]
        fn build(&self) {
            LotteryInterval::<T>::set(self.lottery_interval);
            PreDrawingModificationLockBlocks::<T>::set(self.drawing_freezeout);
            UnstakeTime::<T>::set(self.unstake_time);
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
        LotteryWinner(T::AccountId),
        Deposited(T::AccountId, BalanceOf<T>),
        ScheduledWithdraw(T::AccountId, BalanceOf<T>),
        Withdrawn(T::AccountId, BalanceOf<T>),
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
        TooCloseToDrawing,
        PotBalanceTooLow,
        NoWinnerFound,
        DepositBelowMinAmount,
        WithdrawBelowMinAmount,
        WithdrawAboveDeposit,
        WithdrawFailed,
        PalletMisconfigured,
        TODO,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        <<T as pallet_parachain_staking::Config>::Currency as frame_support::traits::Currency<
            <T as frame_system::Config>::AccountId,
        >>::Balance: From<
            <<T as Config>::Currency as frame_support::traits::Currency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
        >,
    {
        #[pallet::weight(0)]
        pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult
        where
            <<T as pallet_parachain_staking::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance: From<<<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance>
        {
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
            <T as Config>::Currency::transfer(
                &caller_account.clone(),
                &Self::account_id(),
                amount,
                KeepAlive,
            )?;

            // Attempt to stake them with some collator
            // TODO: get highest APY collator available
            // TEMP: Get first active collator
            let some_collator = pallet_parachain_staking::Pallet::<T>::selected_candidates()[0].clone(); // no panic, at least one collator must be chosen or the chain is borked
            Self::do_stake(some_collator, amount)?;

            // Add to active funds
            ActiveBalancePerUser::<T>::mutate(caller_account.clone(), |balance| *balance += amount);
            TotalPot::<T>::mutate(|balance| *balance += amount);

            Self::deposit_event(Event::Deposited(caller_account.clone(), amount));
            Ok(())
        }

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

            // Ensure user has enough funds active and mark them as offboarding
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
                TotalPot::<T>::mutate(|pot| *pot -= amount);
                // Ok(())
                Ok::<(), DispatchError>(())
            });

            // Unstaking workflow
            // 1. See if this withdrawal can be serviced with left-over balance from an already unstaking collator, if so deduct remaining balance and schedule the request
            // 2. If it can't, find the collator with the smallest delegation that is able to handle this withdrawal request and fully unstake it
            // 3. Add balance overshoot to "remaining balance" to handle further requests from

            // If the withdrawal fits in the available funds, do nothing else
            RemainingUnstakingBalance::<T>::try_mutate(|remaining| {
                *remaining = (*remaining).saturating_sub(amount);

                let zero: BalanceOf<T> = 0u32.into();
                (*remaining > zero)
                    .then(|| ())
                    .ok_or("not enough left to handle this request from current unstaking funds")
            })
            .or_else(|_| {
                // Withdrawal needs an extra collator to unstake to have enough funds to serve withdrawals, do it

                // If this fails, something weird is going on or the next line needs to be implemented
                // TODO: add some arithmetic to only unstake the diff between needed and remaining instead of using needed
                // TODO: Error handling
                Self::do_unstake_collator(amount, now);

                // TODO: Try mutate again
                RemainingUnstakingBalance::<T>::try_mutate(|remaining| {
                    *remaining = (*remaining).saturating_sub(amount);
                    // remaining -= amount;
                    (*remaining > 0u32.into()).then(|| ()).ok_or(
                        "not enough left to handle this request from current unstaking funds",
                    )
                })
            }); // TODO: Error handling
                // .or_else(|_| Error::<T>::WithdrawFailed.into())?;
                // END UNSTAKING SECTION

            // schedule payout after T::ReduceBondDelay expires
            // RAD: What happens if delegation_execute_scheduled_request fails?
            // TODO: pallet_scheduler::<T>::schedule(batch(delegation_execute_scheduled_request(),transfer_to_user))
            Self::deposit_event(Event::ScheduledWithdraw(caller, amount));
            // Ok(())
            Ok::<(), DispatchError>(())
        }

        /// Rebalances stake by removing stake from overallocated collators and adding to underallocated
        ///
        /// It may be necessary to call this if large amounts of token become unstaked, e.g. due to a collator leaving
        #[pallet::weight(0)]
        pub fn rebalance_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            Self::ensure_root_or_manager(origin)?;

            // withdraw from overallocated collators, wait until funds unlock, re-allocate to underallocated collators
            // RAD: This can run in parallel with a drawing, it will just reduce the staking revenue generated in this drawing by the amount of funds being rebalanced
            // TODO: find some balancing algorithm that does this

            // Self::deposit_event(Event::StartedRebalance(amount));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn start_lottery(origin: OriginFor<T>) -> DispatchResult {
            Self::ensure_root_or_manager(origin)?;
            // TODO: Check that the pallet has enough funds to pay gas fees for at least the first drawing

            let now = <frame_system::Pallet<T>>::block_number();
            let drawing_interval = Self::lottery_interval();
            ensure!(
                drawing_interval > 0u32.into(),
                Error::<T>::PalletMisconfigured
            );
            let drawing_scheduled_at = now + drawing_interval;
            let lottery_drawing_call = Call::draw_lottery{};

            pallet_scheduler::Pallet::<T>::schedule_named(
                origin,
                T::LotteryPot::get().0.to_vec(),
                drawing_scheduled_at,
                Some((drawing_interval, 99999u32)), // XXX: Seems scheduler has no way to schedule infinite amount
                LOWEST_PRIORITY,
                Box::new(lottery_drawing_call),
            )?;

            Self::deposit_event(Event::LotteryStarted(drawing_scheduled_at));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn stop_lottery(origin: OriginFor<T>) -> DispatchResult {
            Self::ensure_root_or_manager(origin.clone())?;

            // TODO
            // ensure!(
            //     <pallet_scheduler::Pallet<T>::Lookup>::contains_key(T::LotteryPot::get().0.to_vec()),
            //     Error::<T>::LotteryNotScheduled
            // );

            let now = <frame_system::Pallet<T>>::block_number();

            pallet_scheduler::Pallet::<T>::cancel_named(origin, T::LotteryPot::get().0.to_vec())
                .map_err(|_| Error::<T>::LotteryAlreadyStopped)?;

            Self::deposit_event(Event::LotteryStopped(now));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn draw_lottery(origin: OriginFor<T>) -> DispatchResult {
            Self::ensure_root_or_manager(origin.clone())?; // Allow only the origin that scheduled the lottery to execute

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now >= Self::next_drawing(), Error::<T>::TooEarlyForDrawing);

            let pot_account_id = Self::account_id();
            let funds_in_pot = <T as Config>::Currency::total_balance(&pot_account_id);
            let total_deposits = SumOfDeposits::<T>::get();

            // always keep some funds for gas
            // TODO: Convert to saturating math
            let payout = funds_in_pot - total_deposits - Self::gas_reserve();

            ensure!(payout > 0u32.into(), Error::<T>::PotBalanceTooLow);
            ensure!(
                funds_in_pot - payout >= total_deposits,
                Error::<T>::PotBalanceTooLow
            );

            // Match random number to winner. We select a winning **balance** and then just add up accounts in the order they're stored until the sum of balance exceeds the winning amount
            // IMPORTANT: This order and active balances must be locked to modification after the random seed is created (relay BABE randomness, 2 epochs ago)
            let random = T::RandomnessSource::random(&[0u8, 1]);

            // Ensure freezeout started before the randomness was known to prevent manipulation
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                random.1
                    > Self::next_drawing()
                        .saturating_sub(Self::drawing_buffer())
                        .into(),
                Error::<T>::PalletMisconfigured
            );

            // TODO: Fix this conversion
            // let random_hash = random.0;
            // let as_number = U256::from_big_endian(random_hash.as_ref());
            // let winning_number = as_number.low_u128();
            // let winning_balance: BalanceOf<T> = winning_number.into() % Self::total_pot().into();
            let winning_balance = 10u32.into();

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

            <T as Config>::Currency::transfer(
                &Self::account_id(),
                &winner.clone().unwrap(),
                payout,
                KeepAlive,
            )?;
            Self::deposit_event(Event::LotteryWinner(winner.unwrap()));

            // TODO: Update bookkeeping
            NextDrawingAt::<T>::set(now + Self::lottery_interval());

            Self::update_active_funds()?;
            Self::finish_unstaking_collators(origin.clone());
            Self::schedule_withdrawal_payouts(origin.clone())?;
            Self::rebalance_remaining_funds(origin.clone());

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get a unique, inaccessible account id from the `PotId`.
        fn account_id() -> T::AccountId {
            T::LotteryPot::get().into_account_truncating()
        }

        fn do_stake(collator: T::AccountId, amount: BalanceOf<T>) -> DispatchResult
        where
            <<T as pallet_parachain_staking::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance: From<<<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance>
        {
            let some_collator = collator;

            // TODO: Calculate these from current values
            const candidate_delegation_count: u32 = 500;
            const delegation_count: u32 = 500;

            pallet_parachain_staking::Pallet::<T>::delegate(
                RawOrigin::Signed(Self::account_id()).into(),
                some_collator,
                amount.into(),
                candidate_delegation_count,
                delegation_count,
            )
            .map_err(|e| {
                log::error!("Could not delegate to collator with error {:?}", e);
                e
            });

            Ok(()) // TODO: Error handling
        }

        fn do_unstake_collator(amount: BalanceOf<T>, now: T::BlockNumber) -> DispatchResult {
            let some_collator = pallet_parachain_staking::Pallet::<T>::selected_candidates()[0].clone(); // no panic, at least one collator must be chosen or the chain is borked

            // TODO: Find the smallest currently active delegation larger than `amount`
            let delegated_amount_to_be_unstaked: BalanceOf<T> = 0u32.into();

            // TODO: If none can be found, find a combination of collators so it can
            // TODO: If it still can't be found, either there's a logic bug or we've been hacked

            // unstake from parachain staking
            // NOTE: All funds that were delegated here no longer produce staking rewards
            pallet_parachain_staking::Pallet::<T>::schedule_revoke_delegation(
                RawOrigin::Signed(Self::account_id()).into(),
                some_collator.clone(),
            )
            .map_err(|_| Error::<T>::TODO)?;
            // TODO: Error handling

            // TODO: Update remaining balance
            RemainingUnstakingBalance::<T>::mutate(|bal| {
                *bal += delegated_amount_to_be_unstaked.into()
            });

            // update unstaking storage
            UnstakingCollators::<T>::mutate(|collators| {
                collators.push(UnstakingCollator {
                    account: some_collator.clone(),
                    since: now,
                })
            });

            // unstake from parachain staking
            // NOTE: All funds that were delegated here no longer produce staking rewards
            pallet_parachain_staking::Pallet::<T>::schedule_revoke_delegation(
                RawOrigin::Signed(Self::account_id()).into(),
                some_collator,
            )
            .map_err(|_| Error::<T>::TODO)?;
            Ok(())
        }

        fn update_active_funds() -> DispatchResult {
            Ok(())
        }

        /// NOTE: This code assumes UnstakingCollators is sorted
        fn finish_unstaking_collators(origin: OriginFor<T>) {
            Self::ensure_root_or_manager(origin);

            let now = <frame_system::Pallet<T>>::block_number();
            let unstaking = UnstakingCollators::<T>::get();
            UnstakingCollators::<T>::try_mutate(|unstaking| {
                let remaining_collators = unstaking.iter().filter_map(|collator| {
                    // only attempt to resolve fully unstaked collators
                    if collator.since + UnstakeTime::<T>::get() > now {
                        return Some(collator.clone());
                    };
                    // There can only be one request per collator and it is always a full revoke_delegation call
                    match pallet_parachain_staking::Pallet::<T>::execute_delegation_request(
                        RawOrigin::Signed(Self::account_id()).into(),
                        Self::account_id(),
                        collator.account.clone(),
                    ){
                        Ok(_) => {
                            // collator was unstaked, remove from unstaking collators
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
                    for c in remaining_collators{
                        unstaking.push(c.clone());
                    }
                    Ok(())
                } else {
                    Err("no change")
                }
            });
        }

        pub fn rebalance_remaining_funds(origin: OriginFor<T>)
            where
            <<T as pallet_parachain_staking::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance: From<<<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance>
        {
            Self::ensure_root_or_manager(origin);

            let pot_account_id = Self::account_id();
            let available_balance = <T as Config>::Currency::free_balance(&pot_account_id);
            let stakable_balance = available_balance - Self::gas_reserve();

            // TODO: Find highest APY for this deposit (possibly balance deposit to multiple collators)
            let some_output: Vec<(T::AccountId, BalanceOf<T>)> = vec![(
                pallet_parachain_staking::Pallet::<T>::selected_candidates()[0].clone(), // no panic, at least one collator must be chosen or the chain is borked
                stakable_balance,
            )];

            // Stake it to one or more collators
            for (collator, balance) in some_output {
                Self::do_stake(collator, balance);
            }
        }

        /// This fn schedules a single shot payout of all matured withdrawals
        /// It is meant to be executed in the course of a drawing
        fn schedule_withdrawal_payouts(origin: OriginFor<T>) -> DispatchResult {
            let some_collator = pallet_parachain_staking::Pallet::<T>::selected_candidates()[0].clone(); // no panic, at least one collator must be chosen or the chain is borked
            let now = <frame_system::Pallet<T>>::block_number();

            <WithdrawalRequestQueue<T>>::try_mutate(|request_vec| {
                ensure!(
                    (*request_vec).is_empty(),
                    Error::<T>::WithdrawBelowMinAmount
                );
                let mut left_overs: Vec<Request<_, _, _>> = Vec::new();
                for request in request_vec.iter() {
                    if now < request.block + <UnstakeTime<T>>::get() {
                        // too early to withdraw this request
                        left_overs.push((*request).clone());
                        continue;
                    }
                    // Pallet::<T>::SumOfDeposits::mutate(|sum| sum - request.amount);
                    <SumOfDeposits<T>>::mutate(|sum| *sum - request.balance);

                    // TODO: immediately execute unstake
                    pallet_parachain_staking::Pallet::<T>::execute_delegation_request(
                        RawOrigin::Signed(Self::account_id()).into(),
                        Self::account_id(),
                        some_collator.clone(),
                    )?;

                    // TODO: schedule transfer offboarded funds to owner
                    <T as Config>::Currency::transfer(
                        &Self::account_id(),
                        &request.user,
                        request.balance,
                        KeepAlive,
                    )?;
                }
                // Update T::WithdrawalRequestQueue if changed
                if left_overs.len() != (*request_vec).len() {
                    request_vec.clear();
                    for c in left_overs{
                        request_vec.push(c);
                    }
                    Ok(())
                } else {
                    Err("no changes")
                }
            });
            Ok(())
        }

        pub(super) fn not_in_drawing_freezeout() -> bool {
            let now = <frame_system::Pallet<T>>::block_number();
            now < (Self::next_drawing()
                .saturating_sub(Self::drawing_buffer())
                .into())
        }

        pub(super) fn ensure_root_or_manager(origin: OriginFor<T>) -> DispatchResult {
            ensure!(
                // TODO: Schedule origin must be council and root, not root only
                frame_system::ensure_root(origin).is_ok(),
                // frame_system::ensure_root(origin) || T::ManageOrigin::ensure_origin(origin)
                Error::<T>::OnlyRootOrManageOrigin
            );
            Ok(())
        }
    }
}
