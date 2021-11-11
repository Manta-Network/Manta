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

//! TransactionPause pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

use crate::Pallet as TransactionPause;

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {

	// Benchmark `pause_transaction` extrinsic:
	pause_transaction {
		let pallet_name = b"Balances".to_vec();
		let function_name =  b"transfer".to_vec();
		let caller: T::AccountId = whitelisted_caller();
	}: pause_transaction(RawOrigin::Root, pallet_name.clone(), function_name.clone())
	verify {
		assert_last_event::<T>(
			Event::TransactionPaused(pallet_name.clone(), function_name.clone()).into()
		);
	}

	// Benchmark `unpause_transaction` extrinsic:
	unpause_transaction {
		let caller: T::AccountId = whitelisted_caller();
		let origin: T::Origin = T::Origin::from(RawOrigin::Root);
		let pallet_name = b"Balances".to_vec();
		let function_name =  b"transfer".to_vec();

		TransactionPause::<T>::pause_transaction(origin.clone(), pallet_name.clone(), function_name.clone())?;

	}: unpause_transaction(RawOrigin::Root, pallet_name.clone(), function_name.clone())
	verify {
		assert_last_event::<T>(
			Event::TransactionUnpaused(pallet_name.clone(), function_name.clone()).into()
		);
	}
}

impl_benchmark_test_suite!(
	TransactionPause,
	crate::tests_composite::ExtBuilder::default().build(),
	crate::tests_composite::Test,
);
