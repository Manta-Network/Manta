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

use crate::{
    assert_last_event,
    mock::{
        roll_one_block, roll_to, roll_to_round_begin, AccountId, Balance, Balances, ExtBuilder,
        Lottery, ParachainStaking, RuntimeOrigin as Origin, System, Test,
    },
    Config, Error,
};

use frame_support::{assert_noop, assert_ok, traits::Currency};
use frame_system::RawOrigin;

lazy_static::lazy_static! {
    pub(crate) static ref ALICE: AccountId = 1;
    pub(crate) static ref BOB: AccountId = 2;
    pub(crate) static ref CHARLIE: AccountId =3;
    pub(crate) static ref DAVE: AccountId =4;
    pub(crate) static ref EVE: AccountId =5;
}

const UNIT: Balance = 1_000_000_000_000;
const HIGH_BALANCE: Balance = 1_000_000_000 * UNIT;

#[test]
fn call_manager_extrinsics_as_normal_user_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Lottery::start_lottery(Origin::signed(1)), // Somebody who is not T::ManageOrigin
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            Lottery::stop_lottery(Origin::signed(1)), // Somebody who is not T::ManageOrigin
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            Lottery::draw_lottery(Origin::signed(1)), // Somebody who is not T::ManageOrigin
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            Lottery::process_matured_withdrawals(Origin::signed(1)), // Somebody who is not T::ManageOrigin
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            Lottery::liquidate_lottery(Origin::signed(1)), // Somebody who is not T::ManageOrigin
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            Lottery::rebalance_stake(Origin::signed(1)), // Somebody who is not T::ManageOrigin
            sp_runtime::DispatchError::BadOrigin
        );
    });
}
#[test]
fn starting_lottery_without_gas_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Lottery::start_lottery(RawOrigin::Root.into()),
            Error::<Test>::PotBalanceBelowGasReserve
        );
    });
}
#[test]
fn starting_funded_lottery_should_work() {
    ExtBuilder::default()
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert_ok!(Lottery::start_lottery(RawOrigin::Root.into()));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::LotteryStarted
            ));
            assert_noop!(
                Lottery::start_lottery(RawOrigin::Root.into()),
                Error::<Test>::LotteryIsRunning
            ); // Ensure doublestarting fails
        });
}
#[test]
fn restarting_funded_lottery_should_work() {
    ExtBuilder::default()
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert_ok!(Lottery::start_lottery(RawOrigin::Root.into()));
            assert_ok!(Lottery::stop_lottery(RawOrigin::Root.into()));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::LotteryStopped
            ));
            assert_noop!(
                Lottery::stop_lottery(RawOrigin::Root.into()),
                Error::<Test>::LotteryNotStarted
            ); // Ensure doublestopping fails
            assert_ok!(Lottery::start_lottery(RawOrigin::Root.into()));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::LotteryStarted
            ));
        });
}

#[test]
fn depositing_and_withdrawing_in_freezeout_should_not_work() {
    let balance = 300_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), balance);
            assert_ok!(Lottery::start_lottery(RawOrigin::Root.into(),));
            assert!(Lottery::not_in_drawing_freezeout());
            roll_to(
                Lottery::next_drawing_at().unwrap() - <Test as Config>::DrawingFreezeout::get(),
            );
            assert!(!Lottery::not_in_drawing_freezeout());
            assert_noop!(
                Lottery::deposit(Origin::signed(*ALICE), balance),
                Error::<Test>::TooCloseToDrawing
            );
            assert_noop!(
                Lottery::request_withdraw(Origin::signed(*ALICE), balance),
                Error::<Test>::TooCloseToDrawing
            );
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), balance);
        });
}
#[test]
fn depositing_and_withdrawing_should_work() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::Deposited {
                    account: *ALICE,
                    amount: balance
                }
            ));
            assert_eq!(Lottery::active_balance_per_user(*ALICE), balance);
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), balance);
            assert_ok!(Lottery::request_withdraw(Origin::signed(*ALICE), balance));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::ScheduledWithdraw {
                    account: *ALICE,
                    amount: balance
                }
            ));
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::active_balance_per_user(*ALICE), 0);
            assert_eq!(Lottery::total_pot(), 0);
        });
}
#[test]
fn depositing_and_withdrawing_leaves_correct_balance_with_user() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            let alice_starting_balance = Balances::free_balance(*ALICE);
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert!(Balances::free_balance(*ALICE) <= alice_starting_balance - balance);
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), balance);
            assert_ok!(Lottery::request_withdraw(Origin::signed(*ALICE), balance));
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), 0);
            let alice_balance_after_request = Balances::free_balance(*ALICE);
            assert!(alice_balance_after_request <= alice_starting_balance - balance);
            roll_to_round_begin(3);
            assert_ok!(Lottery::process_matured_withdrawals(RawOrigin::Root.into()));
            assert_eq!(Lottery::sum_of_deposits(), 0);
            assert_eq!(
                Balances::free_balance(*ALICE),
                alice_balance_after_request + balance
            );
        });
}
#[test]
fn double_processing_withdrawals_does_not_double_pay() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_ok!(Lottery::request_withdraw(Origin::signed(*ALICE), balance));
            let alice_balance_after_request = Balances::free_balance(*ALICE);
            roll_to_round_begin(3);
            assert_ok!(Lottery::process_matured_withdrawals(RawOrigin::Root.into()));
            assert_ok!(Lottery::process_matured_withdrawals(RawOrigin::Root.into()));
            assert_eq!(
                Balances::free_balance(*ALICE),
                alice_balance_after_request + balance
            );
        });
}
#[test]
fn staking_to_one_underallocated_collator_works() {
    let balance4 = 40_000_000 * UNIT;
    let balance5 = 50_000_000 * UNIT;
    let balance6 = 60_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![
            (*ALICE, HIGH_BALANCE),
            (*BOB, HIGH_BALANCE),
            (*CHARLIE, HIGH_BALANCE),
        ])
        .with_candidates(vec![
            (*ALICE, balance4),
            (*BOB, balance5),
            (*CHARLIE, balance6),
        ])
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance6 + balance5 + balance4);
            assert_eq!(
                ParachainStaking::candidate_info(*ALICE)
                    .unwrap()
                    .total_counted,
                balance4
            );
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance6));
            // Median = 50k, ALICE is the only underallocated, gets all tokend
            assert_eq!(
                ParachainStaking::candidate_info(*ALICE)
                    .unwrap()
                    .total_counted,
                balance4 + balance6
            );
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance5));
            //  Median = 60k, BOB is the only underallocated, gets all token
            assert_eq!(
                ParachainStaking::candidate_info(*BOB)
                    .unwrap()
                    .total_counted,
                balance5 + balance5
            );
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance4));
            // Median = 100k CHARLIE is the only underallocated, gets all token
            assert_eq!(
                ParachainStaking::candidate_info(*CHARLIE)
                    .unwrap()
                    .total_counted,
                balance6 + balance4
            );
            // Now all 3 tie at 100k, there is no underallocation, deposit is given randomly
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance6));
            assert!(
                (ParachainStaking::candidate_info(*ALICE)
                    .unwrap()
                    .total_counted
                    == balance4 + balance6 + balance6)
                    || (ParachainStaking::candidate_info(*BOB)
                        .unwrap()
                        .total_counted
                        == balance5 + balance5 + balance6)
                    || (ParachainStaking::candidate_info(*CHARLIE)
                        .unwrap()
                        .total_counted
                        == balance6 + balance4 + balance6),
            );
        });
}

#[test]
fn unstaking_works_with_0_collators_left() {
    let balance = 50_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*ALICE, balance), (*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_eq!(
                ParachainStaking::candidate_info(*ALICE)
                    .unwrap()
                    .total_counted,
                balance
            );
            assert_eq!(crate::StakedCollators::<Test>::iter().count(), 0);
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(crate::StakedCollators::<Test>::iter().count(), 2);
            assert_eq!(Balances::free_balance(*ALICE), HIGH_BALANCE - 2 * balance);
            assert_eq!(
                ParachainStaking::candidate_info(*ALICE)
                    .unwrap()
                    .total_counted,
                balance * 2
            );
            assert_eq!(
                ParachainStaking::candidate_info(*BOB)
                    .unwrap()
                    .total_counted,
                balance * 2
            );
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                balance * 2
            ));
            assert_eq!(crate::StakedCollators::<Test>::iter().count(), 0);
            assert_ok!(Lottery::start_lottery(RawOrigin::Root.into()));
            roll_to_round_begin(3);
            // by now the withdrawal should have happened by way of lottery drawing
            assert_eq!(Balances::free_balance(*ALICE), HIGH_BALANCE);
        });
}

#[test]
fn winner_distribution_should_be_equal_with_equal_deposits() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![
            (*ALICE, HIGH_BALANCE),
            (*BOB, HIGH_BALANCE),
        ])
        .with_candidates(vec![(*BOB, balance)])
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            <Test as pallet_parachain_staking::Config>::Currency::make_free_balance_be(&Lottery::account_id(), Lottery::gas_reserve());
            // Deposit 50 users with equal deposits
            const WINNING_AMT: u32 = 1;
            const NUMBER_OF_DRAWINGS: u32 = 10_000;
            const NUMBER_OF_USERS: u32 = 50;
            const USER_SEED: u32 = 696_969;
            let min_delegator_bond =
                <<Test as pallet_parachain_staking::Config>::MinDelegatorStk as frame_support::traits::Get<pallet_parachain_staking::BalanceOf<Test>>>::get();
            let deposit_amount: pallet_parachain_staking::BalanceOf<Test> = min_delegator_bond * 10_000u128;
            for user in 0..NUMBER_OF_USERS {
                System::set_block_number(user);
                let (depositor, _) =
                    crate::mock::from_bench::create_funded_user::<Test>("depositor", USER_SEED - 1 - user, deposit_amount);
                assert_ok!(Lottery::deposit(
                    RawOrigin::Signed(depositor).into(),
                    deposit_amount
                ));
            }
            // loop 10000 times, starting from block 5000 to hopefully be outside of drawing freezeout
            for x in 5_000..5_000+NUMBER_OF_DRAWINGS {
                // advance block number to reseed RNG
                System::set_block_number(NUMBER_OF_USERS + x);
                // simulate accrued staking rewards
                assert_ok!(Balances::mutate_account(
                    &Lottery::account_id(),
                    |acc| {
                        acc.free = acc.free.saturating_add(WINNING_AMT.into());
                    }
                ));
                // draw lottery
                assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            }
            assert_eq!(
                Lottery::total_unclaimed_winnings(),
                NUMBER_OF_DRAWINGS as u128 * WINNING_AMT as u128
            );

            // ensure every user has won > 190 times ( 200 optimum = NUMBER_OF_DRAWINGS/NUMBER_OF_USERS )
            let mut winners = vec![];
            for (user, user_winnings) in crate::UnclaimedWinningsByAccount::<Test>::iter() {
                winners.push((user,user_winnings));
                assert!(user_winnings >= (WINNING_AMT as f32 * NUMBER_OF_DRAWINGS as f32 / NUMBER_OF_USERS as f32 * 0.80) as u128);
            }
            log::error!("{:?}",winners);
            assert_eq!(
                winners.len() as u32,
                NUMBER_OF_USERS
            );
        });
}

#[test]
fn depsiting_to_new_collator_multiple_times_in_the_same_block_should_work() {
    let balance = 50_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(balance, Lottery::staked_collators(*BOB));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::Deposited {
                    account: *ALICE,
                    amount: balance
                }
            ));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(2 * balance, Lottery::staked_collators(*BOB));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::Deposited {
                    account: *ALICE,
                    amount: balance
                }
            ));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(3 * balance, Lottery::staked_collators(*BOB));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::Deposited {
                    account: *ALICE,
                    amount: balance
                }
            ));
            assert_eq!(Lottery::sum_of_deposits(), 3 * balance);
        });
}

#[test]
fn deposit_withdraw_deposit_works() {
    let balance = 50_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(HIGH_BALANCE)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(balance, Lottery::staked_collators(*BOB));
            assert_last_event!(crate::mock::RuntimeEvent::Lottery(
                crate::Event::Deposited {
                    account: *ALICE,
                    amount: balance
                }
            ));
            assert_ok!(Lottery::request_withdraw(Origin::signed(*ALICE), balance));
            assert_eq!(0, Lottery::staked_collators(*BOB));
            // join a new collator because BOB is now ineligible to receive deposits
            let (new_collator, _) = crate::mock::from_bench::create_funded_user::<Test>(
                "collator",
                0xDEADBEEF,
                HIGH_BALANCE,
            );
            assert_ok!(ParachainStaking::join_candidates(
                Origin::signed(new_collator),
                balance,
                10
            ));
            assert_eq!(2, ParachainStaking::candidate_pool().len());
            roll_to_round_begin(2);
            assert_eq!(new_collator, ParachainStaking::selected_candidates()[1]);
            // pretend the collator got some rewards
            pallet_parachain_staking::AwardedPts::<Test>::insert(1, new_collator, 20);
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_eq!(balance, Lottery::staked_collators(new_collator));
        });
}

#[test]
fn withdraw_partial_deposit_works() {
    let balance = 500_000_000 * UNIT;
    let half_balance = 250_000_000 * UNIT;
    let quarter_balance = 125_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(balance)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            let alice_post_deposit_balance = Balances::free_balance(*ALICE);
            assert_eq!(balance, Lottery::staked_collators(*BOB));
            assert_eq!(balance, Lottery::total_pot());
            assert_eq!(balance, Lottery::sum_of_deposits());
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                half_balance
            ));
            roll_one_block();
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                quarter_balance
            ));
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_eq!(quarter_balance, Lottery::surplus_unstaking_balance());
            roll_to_round_begin(3);
            pallet_parachain_staking::AwardedPts::<Test>::insert(2, *BOB, 20);
            // funds should be unlocked now and BOB is finished unstaking, so it's eligible for redepositing
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            assert_eq!(
                alice_post_deposit_balance + half_balance + quarter_balance,
                Balances::free_balance(*ALICE)
            );
            assert_eq!(quarter_balance, Lottery::staked_collators(*BOB));
            assert_eq!(0, Lottery::surplus_unstaking_balance());
            assert_eq!(0, Lottery::unlocked_unstaking_funds());
            assert!(Lottery::withdrawal_request_queue().is_empty());
            assert!(crate::UnstakingCollators::<Test>::get().is_empty());
        });
}

#[test]
fn multiround_withdraw_partial_deposit_works() {
    let balance = 500_000_000 * UNIT;
    let half_balance = 250_000_000 * UNIT;
    let quarter_balance = 125_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![(*ALICE, HIGH_BALANCE), (*BOB, HIGH_BALANCE)])
        .with_candidates(vec![(*BOB, balance)])
        .with_funded_lottery_account(balance)
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            // one round to unstake
            assert!(
                <Test as pallet_parachain_staking::Config>::LeaveCandidatesDelay::get() == 1u32
            );
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            let alice_post_deposit_balance = Balances::free_balance(*ALICE);
            assert_eq!(balance, Lottery::staked_collators(*BOB));
            assert_eq!(balance, Lottery::total_pot());
            assert_eq!(balance, Lottery::sum_of_deposits());
            roll_one_block(); // ensure this unlocks *after* round 3 start
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                half_balance
            ));
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_eq!(half_balance, Lottery::surplus_unstaking_balance());

            // withdrawing funds are still locked
            roll_to_round_begin(2);
            roll_one_block(); // ensure this unlocks *after* round 3 start
            pallet_parachain_staking::AwardedPts::<Test>::insert(1, *BOB, 20);
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                quarter_balance
            ));
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            assert_eq!(0, Lottery::staked_collators(*BOB));
            assert_eq!(
                half_balance - quarter_balance,
                Lottery::surplus_unstaking_balance()
            );
            assert_eq!(0, Lottery::unlocked_unstaking_funds());

            // collator becomes unstaked on draw_lottery, must keep quarter for withdrawal, can restake other quarter
            roll_to_round_begin(3);
            pallet_parachain_staking::AwardedPts::<Test>::insert(2, *BOB, 20);
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            assert_eq!(quarter_balance, Lottery::staked_collators(*BOB));
            assert_eq!(quarter_balance, Lottery::unlocked_unstaking_funds());
            assert_eq!(0, Lottery::surplus_unstaking_balance());
            assert_eq!(
                alice_post_deposit_balance + half_balance,
                Balances::free_balance(*ALICE)
            );
            assert_eq!(1, Lottery::withdrawal_request_queue().len());
            assert!(crate::UnstakingCollators::<Test>::get().is_empty());

            roll_to_round_begin(4);
            pallet_parachain_staking::AwardedPts::<Test>::insert(3, *BOB, 20);
            // second withdrawal can be paid out
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            assert_eq!(
                alice_post_deposit_balance + half_balance + quarter_balance,
                Balances::free_balance(*ALICE)
            );
            assert_eq!(quarter_balance, Lottery::staked_collators(*BOB));
            assert_eq!(0, Lottery::surplus_unstaking_balance());
            assert_eq!(0, Lottery::unlocked_unstaking_funds());
            assert!(Lottery::withdrawal_request_queue().is_empty());
        });
}

#[test]
fn multiround_withdraw_partial_deposit_works2() {
    let reserve = 10_000 * UNIT;
    let balance = 500_000_000 * UNIT;
    let quarter_balance = 125_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![
            (*ALICE, HIGH_BALANCE),
            (*BOB, HIGH_BALANCE),
            (*CHARLIE, HIGH_BALANCE),
        ])
        .with_candidates(vec![(*BOB, balance), (*CHARLIE, balance)])
        .with_funded_lottery_account(reserve) // minimally fund lottery
        .build()
        .execute_with(|| {
            assert_eq!(reserve, Lottery::gas_reserve()); // XXX: Cant use getter in the ExtBuilder
            assert_ok!(Lottery::deposit(Origin::signed(*ALICE), balance));
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                quarter_balance
            ));
            roll_to_round_begin(2);
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            roll_one_block();
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                quarter_balance
            ));
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(*ALICE),
                quarter_balance
            ));
            roll_to_round_begin(3);
            pallet_parachain_staking::AwardedPts::<Test>::insert(3, *BOB, 20);
            pallet_parachain_staking::AwardedPts::<Test>::insert(3, *CHARLIE, 20);
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            assert_eq!(2, Lottery::withdrawal_request_queue().len());
            roll_to_round_begin(4);
            pallet_parachain_staking::AwardedPts::<Test>::insert(4, *BOB, 20);
            pallet_parachain_staking::AwardedPts::<Test>::insert(4, *CHARLIE, 20);
            assert_ok!(Lottery::draw_lottery(RawOrigin::Root.into()));
            assert_eq!(0, Lottery::unlocked_unstaking_funds());
            assert!(Lottery::withdrawal_request_queue().is_empty());
        });
}
