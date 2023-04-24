// 202
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

//! # Staking Pallet Unit Tests
//! The unit tests are organized by the call they test. The order matches the order
//! of the calls in the `lib.rs`.
//! 1. Root
//! 2. Monetary Governance
//! 3. Public (Collator, Nominator)
//! 4. Miscellaneous Property-Based Tests
use crate::{
    assert_eq_events, assert_eq_last_events, assert_event_emitted, assert_last_event,
    assert_tail_eq,
    mock::{
        roll_one_block, roll_to, roll_to_round_begin, roll_to_round_end, AccountId, Balance,
        Balances, CollatorSelection, Event as MetaEvent, ExtBuilder, Lottery, Origin,
        ParachainStaking, Test,
    },
    Error, Event,
};

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use pallet_parachain_staking::{
    AtStake, Bond, CollatorStatus, DelegatorStatus, InflationInfo, Range, DELEGATOR_LOCK_ID,
};
use session_key_primitives::util::unchecked_account_id;
use sp_core::sr25519::Public;
use sp_runtime::{traits::Zero, DispatchError, ModuleError, Perbill, Percent};

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
        .with_funded_lottery_account(HIGH_BALANCE.clone())
        .build()
        .execute_with(|| {
            assert_ok!(Lottery::start_lottery(RawOrigin::Root.into()));
        });
}
#[test]
fn depositing_and_withdrawing_should_work() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), HIGH_BALANCE.clone()),
            (BOB.clone(), HIGH_BALANCE),
        ])
        .with_candidates(vec![(BOB.clone(), balance)])
        .with_funded_lottery_account(HIGH_BALANCE.clone())
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_ok!(Lottery::deposit(
                Origin::signed(ALICE.clone()),
                balance.clone()
            ));
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), balance);
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(ALICE.clone()),
                balance.clone()
            ));
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), 0);
        });
}
#[test]
fn depositing_and_withdrawing_leaves_correct_balance_with_user() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), HIGH_BALANCE),
            (BOB.clone(), HIGH_BALANCE),
        ])
        .with_candidates(vec![(BOB.clone(), balance)])
        .with_funded_lottery_account(HIGH_BALANCE.clone())
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            let alice_starting_balance = Balances::free_balance(ALICE.clone());
            assert_ok!(Lottery::deposit(Origin::signed(ALICE.clone()), balance));
            assert!(Balances::free_balance(ALICE.clone()) <= alice_starting_balance - balance);
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), balance);
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(ALICE.clone()),
                balance.clone()
            ));
            assert_eq!(Lottery::sum_of_deposits(), balance);
            assert_eq!(Lottery::total_pot(), 0);
            let alice_balance_after_request = Balances::free_balance(ALICE.clone());
            assert!(alice_balance_after_request <= alice_starting_balance - balance);
            roll_to_round_begin(3);
            assert_ok!(Lottery::process_matured_withdrawals(RawOrigin::Root.into()));
            assert_eq!(Lottery::sum_of_deposits(), 0);
            assert_eq!(
                Balances::free_balance(ALICE.clone()),
                alice_balance_after_request + balance
            );
        });
}
#[test]
fn double_processing_withdrawals_does_not_double_pay() {
    let balance = 500_000_000 * UNIT;
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), HIGH_BALANCE),
            (BOB.clone(), HIGH_BALANCE),
        ])
        .with_candidates(vec![(BOB.clone(), balance)])
        .with_funded_lottery_account(HIGH_BALANCE.clone())
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance);
            assert_ok!(Lottery::deposit(Origin::signed(ALICE.clone()), balance));
            assert_ok!(Lottery::request_withdraw(
                Origin::signed(ALICE.clone()),
                balance.clone()
            ));
            let alice_balance_after_request = Balances::free_balance(ALICE.clone());
            roll_to_round_begin(3);
            assert_ok!(Lottery::process_matured_withdrawals(RawOrigin::Root.into()));
            assert_ok!(Lottery::process_matured_withdrawals(RawOrigin::Root.into()));
            assert_eq!(
                Balances::free_balance(ALICE.clone()),
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
            (ALICE.clone(), HIGH_BALANCE),
            (BOB.clone(), HIGH_BALANCE),
            (CHARLIE.clone(), HIGH_BALANCE),
        ])
        .with_candidates(vec![
            (ALICE.clone(), balance4),
            (BOB.clone(), balance5),
            (CHARLIE.clone(), balance6),
        ])
        .build()
        .execute_with(|| {
            assert!(HIGH_BALANCE > balance6 + balance5 + balance4);
            assert_eq!(
                ParachainStaking::candidate_info(ALICE.clone())
                    .unwrap()
                    .total_counted,
                balance4
            );
            assert_ok!(Lottery::deposit(Origin::signed(ALICE.clone()), balance6));
            // Median = 50k, ALICE is the only underallocated, gets all tokend
            assert_eq!(
                ParachainStaking::candidate_info(ALICE.clone())
                    .unwrap()
                    .total_counted,
                balance4 + balance6
            );
            assert_ok!(Lottery::deposit(Origin::signed(ALICE.clone()), balance5));
            //  Median = 60k, BOB is the only underallocated, gets all token
            assert_eq!(
                ParachainStaking::candidate_info(BOB.clone())
                    .unwrap()
                    .total_counted,
                balance5 + balance5
            );
            assert_ok!(Lottery::deposit(Origin::signed(ALICE.clone()), balance4));
            // Median = 100k CHARLIE is the only underallocated, gets all token
            assert_eq!(
                ParachainStaking::candidate_info(CHARLIE.clone())
                    .unwrap()
                    .total_counted,
                balance6 + balance4
            );
            // Now all 3 tie at 100k, there is no underallocation, deposit is given randomly
            assert_ok!(Lottery::deposit(Origin::signed(ALICE.clone()), balance6));
            assert!(
                (ParachainStaking::candidate_info(ALICE.clone())
                    .unwrap()
                    .total_counted
                    == balance4 + balance6 + balance6)
                    || (ParachainStaking::candidate_info(BOB.clone())
                        .unwrap()
                        .total_counted
                        == balance5 + balance5 + balance6)
                    || (ParachainStaking::candidate_info(CHARLIE.clone())
                        .unwrap()
                        .total_counted
                        == balance6 + balance4 + balance6),
            );
        });
}
