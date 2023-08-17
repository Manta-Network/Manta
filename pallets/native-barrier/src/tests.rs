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
    mock::{AccountId, ExtBuilder, NativeBarrier, Runtime, RuntimeOrigin},
    Config, Error,
};
use manta_primitives::types::Balance;

use core::time::Duration;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
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

// TODO:
// set start in the past and checks
// set start in the future and checks
// checks should be for balances, xtokens and mantapay
