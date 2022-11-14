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

//! Unit tests for the transaction limit pallet.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use manta_primitives::types::{Balance, DolphinAssetId};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn set_asset_limit_and_unset_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_noop!(
            TransactionLimit::set_asset_limit(Origin::signed(1), 1, 100),
            BadOrigin
        );

        assert_eq!(TransactionLimit::asset_limits(1), 0);
        assert_ok!(TransactionLimit::set_asset_limit(
            RawOrigin::Root.into(),
            1,
            100
        ));
        System::assert_last_event(Event::TransactionLimit(crate::Event::TransactionLimitSet {
            asset_id: 1,
            amount: 100,
        }));
        assert_eq!(TransactionLimit::asset_limits(1), 100);

        assert_ok!(TransactionLimit::set_asset_limit(
            RawOrigin::Root.into(),
            1,
            1000
        ));
        System::assert_last_event(Event::TransactionLimit(crate::Event::TransactionLimitSet {
            asset_id: 1,
            amount: 1000,
        }));
        assert_eq!(TransactionLimit::asset_limits(1), 1000);

        assert_noop!(
            TransactionLimit::unset_asset_limit(Origin::signed(1), 1),
            BadOrigin
        );
        assert_ok!(TransactionLimit::unset_asset_limit(
            RawOrigin::Root.into(),
            1
        ));
        System::assert_last_event(Event::TransactionLimit(
            crate::Event::TransactionLimitUnset { asset_id: 1 },
        ));
        assert_eq!(TransactionLimit::asset_limits(1), 0);
    });
}

#[test]
fn transaction_limitation_validity_check_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 100));

        assert_ok!(TransactionLimit::set_asset_limit(
            RawOrigin::Root.into(),
            1,
            100
        ));
        assert!(<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 0));
        assert!(<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 99));
        assert!(!<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 100));
        assert!(!<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 101));

        assert_ok!(TransactionLimit::set_asset_limit(
            RawOrigin::Root.into(),
            1,
            1000
        ));

        assert!(<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 100));
        assert!(!<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 1000));

        assert_ok!(TransactionLimit::unset_asset_limit(
            RawOrigin::Root.into(),
            1
        ));
        assert!(<TransactionLimit as TransactionLimitation<
            DolphinAssetId,
            Balance,
        >>::ensure_valid(1, 1000));
    });
}
