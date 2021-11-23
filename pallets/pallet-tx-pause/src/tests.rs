// Copyright 2020-2021 Manta Network.
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

const BALANCE_TRANSFER: &<Runtime as frame_system::Config>::Call =
	&mock::Call::Balances(pallet_balances::Call::transfer {
		dest: ALICE,
		value: 10,
	});

#[test]
fn pause_transaction_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
			BALANCE_TRANSFER
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
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			None
		);
		assert_ok!(TransactionPause::pause_transaction(
			RawOrigin::Root.into(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		System::assert_last_event(Event::TransactionPause(crate::Event::TransactionPaused(
			b"Balances".to_vec(),
			b"transfer".to_vec(),
		)));
		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			Some(())
		);
		assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(BALANCE_TRANSFER));

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
				b"TransactionPause".to_vec(),
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
fn unpause_transaction_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
			BALANCE_TRANSFER
		));

		System::set_block_number(1);

		assert_ok!(TransactionPause::pause_transaction(
			RawOrigin::Root.into(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			Some(())
		);

		assert!(!<Runtime as frame_system::Config>::BaseCallFilter::contains(BALANCE_TRANSFER));

		assert_noop!(
			TransactionPause::unpause_transaction(
				Origin::signed(1),
				b"Balances".to_vec(),
				b"transfer".to_vec()
			),
			BadOrigin
		);

		assert_ok!(TransactionPause::unpause_transaction(
			RawOrigin::Root.into(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		System::assert_last_event(Event::TransactionPause(crate::Event::TransactionUnpaused(
			b"Balances".to_vec(),
			b"transfer".to_vec(),
		)));
		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			None
		);

		assert!(<Runtime as frame_system::Config>::BaseCallFilter::contains(
			BALANCE_TRANSFER
		));
	});
}

#[test]
fn paused_transaction_filter_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!PausedTransactionFilter::<Runtime>::contains(
			BALANCE_TRANSFER
		));
		assert_ok!(TransactionPause::pause_transaction(
			RawOrigin::Root.into(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert!(PausedTransactionFilter::<Runtime>::contains(
			BALANCE_TRANSFER
		));
		assert_ok!(TransactionPause::unpause_transaction(
			RawOrigin::Root.into(),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert!(!PausedTransactionFilter::<Runtime>::contains(
			BALANCE_TRANSFER
		));
	});
}
