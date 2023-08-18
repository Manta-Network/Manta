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
    mock::{AccountId, Balances, ExtBuilder, NativeBarrier, Runtime, RuntimeOrigin, MOCK_TIME},
    Config, Error,
};
use manta_primitives::types::Balance;

use core::time::Duration;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    traits::{Currency, GenesisBuild, OnInitialize, ReservableCurrency},
};
use frame_system::RawOrigin;

#[test]
fn extrinsics_as_normal_user_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NativeBarrier::set_daily_xcm_limit(RuntimeOrigin::signed(1), Some(10u128)),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            NativeBarrier::set_start_unix_time(RuntimeOrigin::signed(1), Some(Duration::default())),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            NativeBarrier::add_accounts_to_native_barrier(RuntimeOrigin::signed(1), vec![]),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            NativeBarrier::remove_accounts_from_native_barrier(RuntimeOrigin::signed(1), vec![]),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn extrinsics_as_root_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_err!(
            NativeBarrier::add_accounts_to_native_barrier(RawOrigin::Root.into(), vec![]),
            Error::<Runtime>::XcmDailyLimitNotSet
        );
        assert_ok!(NativeBarrier::set_daily_xcm_limit(
            RawOrigin::Root.into(),
            Some(10u128)
        ));
        assert_err!(
            NativeBarrier::add_accounts_to_native_barrier(RawOrigin::Root.into(), vec![]),
            Error::<Runtime>::StartUnixTimeNotSet
        );
        assert_ok!(NativeBarrier::set_start_unix_time(
            RawOrigin::Root.into(),
            Some(Duration::default())
        ));
        assert_ok!(NativeBarrier::add_accounts_to_native_barrier(
            RawOrigin::Root.into(),
            vec![1, 2, 3]
        ));
        assert_ne!(NativeBarrier::get_remaining_xcm_limit(1), None);
        assert_ok!(NativeBarrier::remove_accounts_from_native_barrier(
            RawOrigin::Root.into(),
            vec![1]
        ));
        assert_eq!(NativeBarrier::get_remaining_xcm_limit(1), None);
    });
}

fn set_mock_time(new_time: Duration) {
    MOCK_TIME.with(|time| {
        *time.borrow_mut() = new_time;
    });
}

fn advance_mock_time(delta: Duration) {
    MOCK_TIME.with(|time| {
        *time.borrow_mut() += delta;
    });
}

#[test]
fn start_in_the_past_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        set_mock_time(Duration::default());
        // add balances to 1,2,3
        assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 1, 1000, 0));
        assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 2, 1000, 0));
        assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 3, 1000, 0));

        let daily_limit = 10u128;
        let day_in_secs = 86400;
        assert_ok!(NativeBarrier::set_daily_xcm_limit(
            RawOrigin::Root.into(),
            Some(daily_limit)
        ));
        assert_ok!(NativeBarrier::set_start_unix_time(
            RawOrigin::Root.into(),
            Some(Duration::default())
        ));

        // transfer more than limit should work
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit * 2
        ));

        assert_ok!(NativeBarrier::add_accounts_to_native_barrier(
            RawOrigin::Root.into(),
            vec![1, 2, 3]
        ));

        // transfer under limit should work
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit / 2
        ));
        // transfer more than limit should not work
        assert_err!(
            Balances::transfer(RuntimeOrigin::signed(1), 2, daily_limit / 2 + 1),
            Error::<Runtime>::XcmTransfersLimitExceeded
        );

        // limit should be multiple of daily limit (now - epoch_start)
        // roll one day
        advance_mock_time(Duration::from_secs(day_in_secs));
        NativeBarrier::on_initialize(1);

        // check that limit has been updated
        // transfer more than limit should not work
        assert_err!(
            Balances::transfer(
                RuntimeOrigin::signed(1),
                2,
                daily_limit + daily_limit / 2 + 1
            ),
            Error::<Runtime>::XcmTransfersLimitExceeded
        );
        // transfer under limit should work
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit + daily_limit / 2
        ));

        assert_ok!(NativeBarrier::remove_accounts_from_native_barrier(
            RawOrigin::Root.into(),
            vec![1]
        ));

        // transfer more than limit should work for 1
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            3,
            daily_limit * 5
        ));
        assert_err!(
            Balances::transfer(RuntimeOrigin::signed(2), 2, daily_limit * 2 + 1),
            Error::<Runtime>::XcmTransfersLimitExceeded
        );

        assert_ok!(NativeBarrier::set_daily_xcm_limit(
            RawOrigin::Root.into(),
            None
        ));

        // transfer more than limit should work for all
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            3,
            daily_limit * 5
        ));
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(2),
            3,
            daily_limit * 5
        ));
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(3),
            3,
            daily_limit * 5
        ));
    });
}

#[test]
fn start_in_the_future_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        set_mock_time(Duration::default());
        // add balances to 1,2,3
        assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 1, 1000, 0));
        assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 2, 1000, 0));
        assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 3, 1000, 0));

        // transfer more than limit should work
        assert_ok!(Balances::transfer(RuntimeOrigin::signed(1), 2, 20));

        let daily_limit = 10u128;
        let future_start = 10 * 86400;
        let advance_to = 20 * 86400;
        let future_days = 10;
        assert_ok!(NativeBarrier::set_daily_xcm_limit(
            RawOrigin::Root.into(),
            Some(daily_limit)
        ));
        assert_ok!(NativeBarrier::set_start_unix_time(
            RawOrigin::Root.into(),
            Some(Duration::from_secs(future_start))
        ));

        assert_ok!(NativeBarrier::add_accounts_to_native_barrier(
            RawOrigin::Root.into(),
            vec![1, 2, 3]
        ));

        // transfer more than limit should work
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit + 5
        ));

        // limit should be multiple of daily limit (now - epoch_start)
        // roll one day
        advance_mock_time(Duration::from_secs(advance_to));
        NativeBarrier::on_initialize(1);

        // check that limit has been updated
        // transfer more than limit should not work
        assert_err!(
            Balances::transfer(
                RuntimeOrigin::signed(1),
                2,
                (future_days as u128 + 1) * daily_limit + 1
            ),
            Error::<Runtime>::XcmTransfersLimitExceeded
        );
        // transfer under limit should work
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            (future_days as u128 + 1) * daily_limit
        ));

        assert_ok!(NativeBarrier::remove_accounts_from_native_barrier(
            RawOrigin::Root.into(),
            vec![1]
        ));

        // transfer more than limit should work for 1
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            3,
            (future_days as u128 + 1) * daily_limit + 5
        ));
        assert_err!(
            Balances::transfer(
                RuntimeOrigin::signed(2),
                2,
                (future_days as u128 + 1) * daily_limit + 1
            ),
            Error::<Runtime>::XcmTransfersLimitExceeded
        );

        assert_ok!(NativeBarrier::set_daily_xcm_limit(
            RawOrigin::Root.into(),
            None
        ));

        // transfer more than limit should work for all
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            3,
            (future_days as u128 + 1) * daily_limit * 2
        ));
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(2),
            3,
            (future_days as u128 + 1) * daily_limit * 2
        ));
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(3),
            3,
            (future_days as u128 + 1) * daily_limit * 2
        ));
    });
}

// todo: test that on_inits which don't roll a day will not increase limits.
