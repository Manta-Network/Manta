// Copyright 2020-2022 Manta Network.
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
//
//! Unit tests for the Name Service.

#![cfg(test)]

use super::*;
use crate::mock::{NameService, Runtime, RuntimeOrigin as MockOrigin, *};
use frame_support::{assert_noop, assert_ok};

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);

/// Initializes a test by funding accounts.
#[inline]
fn initialize_test() {
    assert_ok!(Balances::set_balance(
        MockOrigin::root(),
        ALICE,
        1_000_000_000_000_000,
        0
    ));
    assert_ok!(Balances::set_balance(
        MockOrigin::root(),
        BOB,
        1_000_000_000_000_000,
        0
    ));
}

#[test]
fn register_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
        System::set_block_number(5);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
    });
}

#[test]
fn re_register_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert!(crate::PendingRegister::<Runtime>::contains_key(
            <Runtime as frame_system::Config>::Hashing::hash_of(&"test".as_bytes().to_vec())
        ));
        System::set_block_number(5);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert!(crate::UsernameRecords::<Runtime>::contains_key("test".as_bytes().to_vec()));
        assert_ok!(
            NameService::remove_register(
                MockOrigin::signed(ALICE),
                "test".as_bytes().to_vec(),
                ALICE.into()
            )
        );
        assert!(!crate::UsernameRecords::<Runtime>::contains_key("test".as_bytes().to_vec()));
        assert!(!crate::PendingRegister::<Runtime>::contains_key(
            <Runtime as frame_system::Config>::Hashing::hash_of(&"test".as_bytes().to_vec())
        ));

        // test registering again
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert!(crate::PendingRegister::<Runtime>::contains_key(
            <Runtime as frame_system::Config>::Hashing::hash_of(&"test".as_bytes().to_vec())
        ));
        System::set_block_number(10);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert!(crate::UsernameRecords::<Runtime>::contains_key("test".as_bytes().to_vec()));
    });
}

#[test]
fn set_primary_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
        System::set_block_number(5);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
        assert_ok!(NameService::set_primary_name(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
    });
}

#[test]
fn cancel_register_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
        assert_ok!(NameService::cancel_pending_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
    });
}

#[test]
fn remove_register_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
        System::set_block_number(5);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
        assert_ok!(NameService::remove_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into()
        ));
    });
}

#[test]
fn register_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert_noop!(
            NameService::register(
                MockOrigin::signed(ALICE),
                "test".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::AlreadyPendingRegister
        );
        assert_noop!(
            NameService::register(
                MockOrigin::signed(ALICE),
                "!#".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::InvalidUsernameFormat
        );
        assert_noop!(
            NameService::accept_register(
                MockOrigin::signed(ALICE),
                "test".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::RegisterTimeNotReached
        );
    });
}

#[test]
fn register_time_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert_noop!(
            NameService::accept_register(
                MockOrigin::signed(ALICE),
                "test".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::RegisterTimeNotReached
        );
    });
}

#[test]
fn register_accept_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        System::set_block_number(5);
        assert_noop!(
            NameService::accept_register(
                MockOrigin::signed(BOB),
                "test".as_bytes().to_vec(),
                BOB.into()
            ),
            Error::<Runtime>::NotOwned
        );
    });
}

#[test]
fn set_primary_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        System::set_block_number(5);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert_noop!(
            NameService::set_primary_name(
                MockOrigin::signed(BOB),
                "test".as_bytes().to_vec(),
                BOB.into()
            ),
            Error::<Runtime>::NotOwned
        );
        assert_noop!(
            NameService::set_primary_name(
                MockOrigin::signed(ALICE),
                "testtest".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::NotRegistered
        );
    });
}

#[test]
fn cancel_register_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert_noop!(
            NameService::cancel_pending_register(
                MockOrigin::signed(BOB),
                "test".as_bytes().to_vec(),
                BOB.into()
            ),
            Error::<Runtime>::NotOwned
        );
        assert_noop!(
            NameService::cancel_pending_register(
                MockOrigin::signed(ALICE),
                "testtest".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::UsernameNotFound
        );
    });
}

#[test]
fn remove_register_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_test();
        assert_ok!(NameService::register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        System::set_block_number(5);
        assert_ok!(NameService::accept_register(
            MockOrigin::signed(ALICE),
            "test".as_bytes().to_vec(),
            ALICE.into(),
        ));
        assert_noop!(
            NameService::remove_register(
                MockOrigin::signed(ALICE),
                "testtest".as_bytes().to_vec(),
                ALICE.into()
            ),
            Error::<Runtime>::NotRegistered
        );
        assert_noop!(
            NameService::remove_register(
                MockOrigin::signed(BOB),
                "test".as_bytes().to_vec(),
                BOB.into()
            ),
            Error::<Runtime>::NotOwned
        );
    });
}
