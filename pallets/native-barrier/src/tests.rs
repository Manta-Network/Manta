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
    mock::{Balances, ExtBuilder, NativeBarrier, Runtime, RuntimeOrigin, MOCK_TIME},
    Error,
};

use core::time::Duration;
use frame_support::{assert_err, assert_noop, assert_ok, traits::OnInitialize};
use frame_system::RawOrigin;

#[test]
fn extrinsics_as_normal_user_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NativeBarrier::initialize_native_barrier(
                RuntimeOrigin::signed(1),
                Some((10u128, Default::default())),
            ),
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
            Error::<Runtime>::NativeBarrierNotInitialized
        );
        assert_err!(
            NativeBarrier::add_accounts_to_native_barrier(RawOrigin::Root.into(), vec![]),
            Error::<Runtime>::NativeBarrierNotInitialized
        );
        assert_ok!(NativeBarrier::initialize_native_barrier(
            RawOrigin::Root.into(),
            Some((10, Duration::default())),
        ));
        assert_ok!(NativeBarrier::add_accounts_to_native_barrier(
            RawOrigin::Root.into(),
            vec![1, 2, 3]
        ));
        assert_ne!(NativeBarrier::get_remaining_limit(1), None);
        assert_ok!(NativeBarrier::remove_accounts_from_native_barrier(
            RawOrigin::Root.into(),
            vec![1]
        ));
        assert_eq!(NativeBarrier::get_remaining_limit(1), None);
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

fn initialize_test_environment() {
    // add balances to 1,2,3
    assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 1, 1000, 0));
    assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 2, 1000, 0));
    assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 3, 1000, 0));

    // transfer more than limit should work
    assert_ok!(Balances::transfer(RuntimeOrigin::signed(1), 2, 20));
}

fn initialize_native_barrier(daily_limit: u128, start_unix_time: Duration) {
    set_mock_time(Duration::default());

    // add balances to 1,2,3
    assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 1, 1000, 0));
    assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 2, 1000, 0));
    assert_ok!(Balances::set_balance(RawOrigin::Root.into(), 3, 1000, 0));

    assert_ok!(NativeBarrier::initialize_native_barrier(
        RawOrigin::Root.into(),
        Some((daily_limit, start_unix_time)),
    ));
    assert_ok!(NativeBarrier::add_accounts_to_native_barrier(
        RawOrigin::Root.into(),
        vec![1, 2, 3]
    ));
}

#[test]
fn start_in_the_past_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test_environment();

        let daily_limit = 10u128;
        let day_in_secs = 86400;

        initialize_native_barrier(daily_limit, Duration::default());
        advance_mock_time(Duration::from_secs(day_in_secs));

        // transfer more than 1 limit should not work until next block
        assert_eq!(NativeBarrier::get_remaining_limit(1), Some(daily_limit));
        assert_err!(
            Balances::transfer(RuntimeOrigin::signed(1), 2, daily_limit + 1),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );

        NativeBarrier::on_initialize(1);

        // transfer under limit should now work
        assert_eq!(NativeBarrier::get_remaining_limit(1), Some(2 * daily_limit));
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit / 2
        ));

        // transfer more than limit should not work
        assert_eq!(
            NativeBarrier::get_remaining_limit(1),
            Some(daily_limit + daily_limit / 2)
        );
        assert_err!(
            Balances::transfer(RuntimeOrigin::signed(1), 2, 2 * daily_limit + 1),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );
        assert_err!(
            Balances::transfer_keep_alive(RuntimeOrigin::signed(1), 2, 2 * daily_limit + 1),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );
        assert_err!(
            Balances::transfer_all(RuntimeOrigin::signed(1), 2, false),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );

        // limit should be multiple of daily limit (now - epoch_start)
        // roll one day
        advance_mock_time(Duration::from_secs(day_in_secs));
        NativeBarrier::on_initialize(1);

        // check that limit has been updated
        // transfer more than limit should not work
        assert_eq!(
            NativeBarrier::get_remaining_limit(1),
            Some(2 * daily_limit + daily_limit / 2)
        );
        assert_err!(
            Balances::transfer(
                RuntimeOrigin::signed(1),
                2,
                2 * daily_limit + daily_limit / 2 + 1
            ),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );
        assert_err!(
            Balances::transfer_keep_alive(
                RuntimeOrigin::signed(1),
                2,
                2 * daily_limit + daily_limit / 2 + 1
            ),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );
        assert_err!(
            Balances::transfer_all(RuntimeOrigin::signed(1), 2, false),
            Error::<Runtime>::NativeBarrierLimitExceeded
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
        assert_eq!(NativeBarrier::get_remaining_limit(2), Some(3 * daily_limit));
        assert_err!(
            Balances::transfer(RuntimeOrigin::signed(2), 2, 3 * daily_limit + 1),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );

        assert_ok!(NativeBarrier::initialize_native_barrier(
            RawOrigin::Root.into(),
            None,
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
        initialize_test_environment();

        let daily_limit = 10u128;
        let advance_10 = 10 * 86400;
        let future_days = 10;

        // transfer more than limit should work
        assert_eq!(NativeBarrier::get_remaining_limit(1), None);
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit + 5
        ));

        initialize_native_barrier(daily_limit, Duration::from_secs(advance_10));

        // has 1 daily limit but can over-spend it as the barrier is in the future
        assert_eq!(NativeBarrier::get_remaining_limit(1), Some(daily_limit));
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            daily_limit * 2
        ),);
        // The limit should not have been updated
        assert_eq!(NativeBarrier::get_remaining_limit(1), Some(daily_limit));

        advance_mock_time(Duration::from_secs(advance_10 * 2));
        NativeBarrier::on_initialize(1);

        // check that limit has been updated
        // transfer more than limit should not work
        assert_eq!(
            NativeBarrier::get_remaining_limit(1),
            Some(daily_limit + 10 * daily_limit)
        );
        assert_err!(
            Balances::transfer(
                RuntimeOrigin::signed(1),
                2,
                daily_limit + future_days as u128 * daily_limit + 1
            ),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );
        assert_err!(
            Balances::transfer_keep_alive(
                RuntimeOrigin::signed(1),
                2,
                daily_limit + future_days as u128 * daily_limit + 1
            ),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );
        assert_err!(
            Balances::transfer_all(RuntimeOrigin::signed(1), 2, false),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );

        // transfer under limit should work
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            2,
            future_days as u128 * daily_limit
        ));

        assert_ok!(NativeBarrier::remove_accounts_from_native_barrier(
            RawOrigin::Root.into(),
            vec![1]
        ));

        // transfer more than limit should work for 1
        assert_ok!(Balances::transfer(
            RuntimeOrigin::signed(1),
            3,
            future_days as u128 * daily_limit + 5
        ));
        assert_err!(
            Balances::transfer(
                RuntimeOrigin::signed(2),
                2,
                daily_limit + future_days as u128 * daily_limit + 1
            ),
            Error::<Runtime>::NativeBarrierLimitExceeded
        );

        assert_ok!(NativeBarrier::initialize_native_barrier(
            RawOrigin::Root.into(),
            None,
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

fn loop_on_init_and_assert_accounts(should_be: Option<u128>) {
    for _ in 0..1000 {
        NativeBarrier::on_initialize(1);
    }

    // check that limit has not changed
    for k in 1..4 {
        let limit = NativeBarrier::get_remaining_limit(k);
        assert_eq!(limit, should_be);
    }
}

#[test]
fn limits_should_grow_appropriately() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test_environment();
        loop_on_init_and_assert_accounts(None);
        assert_eq!(NativeBarrier::get_last_day_processed(), None);

        let daily_limit = 10u128;
        let advance_10 = 10 * 86400;

        initialize_native_barrier(daily_limit, Duration::from_secs(advance_10));
        loop_on_init_and_assert_accounts(Some(daily_limit));
        assert_eq!(NativeBarrier::get_last_day_processed(), Some(0));

        // roll 20 days (10 days after start)
        advance_mock_time(Duration::from_secs(advance_10 * 2));
        loop_on_init_and_assert_accounts(Some(daily_limit + daily_limit * 10));
        assert_eq!(NativeBarrier::get_last_day_processed(), Some(10));

        // roll another 10 days
        advance_mock_time(Duration::from_secs(advance_10));
        loop_on_init_and_assert_accounts(Some(daily_limit + daily_limit * 20));
        assert_eq!(NativeBarrier::get_last_day_processed(), Some(20));
    });
}
