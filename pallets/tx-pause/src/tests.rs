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
// The pallet-tx-pause pallet is forked from Acala's transaction-pause module https://github.com/AcalaNetwork/Acala/tree/master/modules/transaction-pause
// The original license is the following - SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

//! Unit tests for the transaction pause pallet.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

const REMARK_CALL: &<Runtime as frame_system::Config>::Call =
    &mock::Call::System(frame_system::Call::remark { remark: vec![] });

#[test]
fn pause_transaction_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));

        System::set_block_number(1);
        assert_noop!(
            TransactionPause::pause_transaction(
                Origin::signed(1),
                b"Balances".to_vec(),
                b"transfer".to_vec()
            ),
            BadOrigin
        );

        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            None
        );
        assert_ok!(TransactionPause::pause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        System::assert_last_event(Event::TransactionPause(crate::Event::TransactionPaused(
            b"System".to_vec(),
            b"remark".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            Some(())
        );
        assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(REMARK_CALL));

        assert_eq!(
            TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
            None
        );
        assert_noop!(
            TransactionPause::pause_transaction(
                RawOrigin::Root.into(),
                b"Balances".to_vec(),
                b"transfer".to_vec()
            ),
            Error::<Runtime>::CannotPause
        );
        assert_noop!(
            TransactionPause::pause_transaction(
                RawOrigin::Root.into(),
                b"TransactionPause".to_vec(),
                b"pause_transaction".to_vec()
            ),
            Error::<Runtime>::CannotPause
        );
        assert_noop!(
            TransactionPause::pause_transaction(
                RawOrigin::Root.into(),
                b"Democracy".to_vec(),
                b"some_other_call".to_vec()
            ),
            Error::<Runtime>::CannotPause
        );
        assert_ok!(TransactionPause::pause_transaction(
            RawOrigin::Root.into(),
            b"OtherPallet".to_vec(),
            b"pause_transaction".to_vec()
        ));
    });
}

#[test]
fn pause_pallets_unpause_transaction_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_noop!(
            TransactionPause::pause_pallets(Origin::signed(1), vec![b"Balances".to_vec()]),
            BadOrigin
        );

        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"txp_pause_all".to_vec())),
            None
        );
        assert_noop!(
            TransactionPause::pause_pallets(RawOrigin::Root.into(), vec![b"Balances".to_vec()]),
            Error::<Runtime>::CannotPause
        );
        // Although we can pause System in testcase, but BaseCallFilter still works because System is in front of TransactionPause.
        assert_ok!(TransactionPause::pause_pallets(
            RawOrigin::Root.into(),
            vec![b"System".to_vec()]
        ));
        System::assert_last_event(Event::TransactionPause(crate::Event::TransactionPaused(
            b"System".to_vec(),
            b"txp_pause_all".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"txp_pause_all".to_vec())),
            Some(())
        );

        assert_noop!(
            TransactionPause::unpause_transaction(
                Origin::signed(1),
                b"System".to_vec(),
                b"txp_pause_all".to_vec()
            ),
            BadOrigin
        );
        assert_ok!(TransactionPause::unpause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"txp_pause_all".to_vec()
        ));
        System::assert_last_event(Event::TransactionPause(crate::Event::TransactionUnpaused(
            b"System".to_vec(),
            b"txp_pause_all".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"txp_pause_all".to_vec())),
            None
        );
    });
}

#[test]
fn unpause_transaction_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));

        System::set_block_number(1);

        assert_ok!(TransactionPause::pause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            Some(())
        );

        assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(REMARK_CALL));

        assert_noop!(
            TransactionPause::unpause_transaction(
                Origin::signed(1),
                b"System".to_vec(),
                b"remark".to_vec()
            ),
            BadOrigin
        );

        assert_ok!(TransactionPause::unpause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        System::assert_last_event(Event::TransactionPause(crate::Event::TransactionUnpaused(
            b"System".to_vec(),
            b"remark".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            None
        );

        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));
    });
}

#[test]
fn paused_transaction_filter_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(!PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
        assert_ok!(TransactionPause::pause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        assert!(PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));

        assert_ok!(TransactionPause::unpause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        assert!(!PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));

        assert_ok!(TransactionPause::pause_pallets(
            RawOrigin::Root.into(),
            vec![b"System".to_vec()]
        ));
        // The real call on storage is `PAUSE_ALL_PALLET_CALLS`, but the `contains` will ignore the function_name.
        // As we can't mock a `PAUSE_ALL_PALLET_CALLS` call, so we use `REMARK_CALL`.
        assert!(PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
    });
}
