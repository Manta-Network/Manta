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

const SETCODE_CALL: &<Runtime as frame_system::Config>::Call =
    &mock::Call::System(frame_system::Call::set_code { code: vec![] });

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
        assert_eq!(
            TransactionPause::paused_transactions((
                b"OtherPallet".to_vec(),
                b"pause_transaction".to_vec()
            )),
            Some(())
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
fn pause_unpause_transactions_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            SETCODE_CALL
        ));

        System::set_block_number(1);
        assert_ok!(TransactionPause::pause_transactions(
            RawOrigin::Root.into(),
            vec![(
                b"System".to_vec(),
                vec![b"remark".to_vec(), b"set_code".to_vec()]
            )]
        ));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            Some(())
        );
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"set_code".to_vec())),
            Some(())
        );
        assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(REMARK_CALL));
        assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(SETCODE_CALL));

        assert_noop!(
            TransactionPause::unpause_transactions(
                Origin::signed(1),
                vec![(
                    b"System".to_vec(),
                    vec![b"remark".to_vec(), b"set_code".to_vec()]
                )]
            ),
            BadOrigin
        );
        assert_ok!(TransactionPause::unpause_transactions(
            RawOrigin::Root.into(),
            vec![(
                b"System".to_vec(),
                vec![b"remark".to_vec(), b"set_code".to_vec()]
            )]
        ));
        System::assert_has_event(Event::TransactionPause(crate::Event::TransactionUnpaused(
            b"System".to_vec(),
            b"remark".to_vec(),
        )));
        System::assert_has_event(Event::TransactionPause(crate::Event::TransactionUnpaused(
            b"System".to_vec(),
            b"set_code".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            None
        );
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"set_code".to_vec())),
            None
        );

        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            SETCODE_CALL
        ));
    });
}

#[test]
fn pause_unpause_pallets_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_noop!(
            TransactionPause::pause_pallets(Origin::signed(1), vec![b"Balances".to_vec()]),
            BadOrigin
        );
        assert_noop!(
            TransactionPause::pause_pallets(RawOrigin::Root.into(), vec![b"Balances".to_vec()]),
            Error::<Runtime>::CannotPause
        );

        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            SETCODE_CALL
        ));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            None
        );
        // Although we can pause System in testcase, but BaseCallFilter still works because System is in front of TransactionPause.
        assert_ok!(TransactionPause::pause_pallets(
            RawOrigin::Root.into(),
            vec![b"System".to_vec()]
        ));
        System::assert_last_event(Event::TransactionPause(crate::Event::PalletPaused(
            b"System".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            Some(())
        );
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"set_code".to_vec())),
            Some(())
        );
        assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(REMARK_CALL));
        assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(SETCODE_CALL));

        // unpause pallets
        assert_noop!(
            TransactionPause::unpause_pallets(Origin::signed(1), vec![b"System".to_vec()],),
            BadOrigin
        );
        assert_ok!(TransactionPause::unpause_pallets(
            RawOrigin::Root.into(),
            vec![b"System".to_vec()],
        ));
        System::assert_last_event(Event::TransactionPause(crate::Event::PalletUnpaused(
            b"System".to_vec(),
        )));
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"remark".to_vec())),
            None
        );
        assert_eq!(
            TransactionPause::paused_transactions((b"System".to_vec(), b"set_code".to_vec())),
            None
        );
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            REMARK_CALL
        ));
        assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
            SETCODE_CALL
        ));
    });
}

#[test]
fn pause_pallets_weight_works() {
    ExtBuilder::default().build().execute_with(|| {
        let ps: DispatchResultWithPostInfo =
            TransactionPause::pause_pallets(RawOrigin::Root.into(), vec![b"System".to_vec()]);
        let size: u32 = PausedTransactions::<Runtime>::iter().map(|_x| 1).sum();

        let max_call_len: u32 =
            <<Runtime as Config>::MaxCallNames as sp_runtime::traits::Get<u32>>::get();
        let weight_per_tx: Weight = <Runtime as Config>::WeightInfo::pause_transaction();
        let initial_weight = weight_per_tx.saturating_mul(max_call_len as Weight);

        let ps = ps.unwrap();
        let actual_weight = ps.actual_weight.unwrap();
        assert_eq!(actual_weight, weight_per_tx.saturating_mul(size as Weight));
        assert!(actual_weight < initial_weight);

        let ps2: DispatchResultWithPostInfo =
            TransactionPause::unpause_pallets(RawOrigin::Root.into(), vec![b"System".to_vec()]);
        let ps2 = ps2.unwrap();
        let actual_weight2 = ps2.actual_weight.unwrap();
        assert_eq!(actual_weight, actual_weight2);
    });
}

#[test]
fn paused_transaction_filter_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(!PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
        assert!(!PausedTransactionFilter::<Runtime>::contains(SETCODE_CALL));
        // pause one transaction
        assert_ok!(TransactionPause::pause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        // pause transactions
        assert_ok!(TransactionPause::pause_transactions(
            RawOrigin::Root.into(),
            vec![(b"System".to_vec(), vec![b"set_code".to_vec()])]
        ));
        assert!(PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
        assert!(PausedTransactionFilter::<Runtime>::contains(SETCODE_CALL));

        // unpause one transaction
        assert_ok!(TransactionPause::unpause_transaction(
            RawOrigin::Root.into(),
            b"System".to_vec(),
            b"remark".to_vec()
        ));
        // unpause transactions
        assert_ok!(TransactionPause::unpause_transactions(
            RawOrigin::Root.into(),
            vec![(b"System".to_vec(), vec![b"set_code".to_vec()])]
        ));
        assert!(!PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
        assert!(!PausedTransactionFilter::<Runtime>::contains(SETCODE_CALL));

        // pause pallet
        assert_ok!(TransactionPause::pause_pallets(
            RawOrigin::Root.into(),
            vec![b"System".to_vec()]
        ));
        assert!(PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
        assert!(PausedTransactionFilter::<Runtime>::contains(SETCODE_CALL));

        // unpause pallet
        assert_ok!(TransactionPause::unpause_pallets(
            RawOrigin::Root.into(),
            vec![b"System".to_vec()]
        ));
        assert!(!PausedTransactionFilter::<Runtime>::contains(REMARK_CALL));
        assert!(!PausedTransactionFilter::<Runtime>::contains(SETCODE_CALL));
    });
}
