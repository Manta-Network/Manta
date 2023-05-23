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

//! TransactionPause pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TransactionPause;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{EventRecord, RawOrigin};

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    // Benchmark `pause_transaction` extrinsic:
    pause_transaction {
        let pallet_name = b"System".to_vec();
        let function_name =  b"remark".to_vec();
    }: pause_transaction(RawOrigin::Root, pallet_name.clone(), function_name.clone())
    verify {
        assert_last_event::<T>(
            Event::TransactionPaused(pallet_name.clone(), function_name).into()
        );
    }

    // Benchmark `unpause_transaction` extrinsic:
    unpause_transaction {
        let origin: T::RuntimeOrigin = T::RuntimeOrigin::from(RawOrigin::Root);
        let pallet_name = b"System".to_vec();
        let function_name =  b"remark".to_vec();
        TransactionPause::<T>::pause_transaction(origin, pallet_name.clone(), function_name.clone())?;
    }: unpause_transaction(RawOrigin::Root, pallet_name.clone(), function_name.clone())
    verify {
        assert_last_event::<T>(
            Event::TransactionUnpaused(pallet_name, function_name).into()
        );
    }
}

impl_benchmark_test_suite!(
    TransactionPause,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
