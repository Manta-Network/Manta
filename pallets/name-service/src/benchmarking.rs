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
//! NameService pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::{Call, Config, Event, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use pallet::*;
use manta_support::manta_pay::AccountId;
use sp_std::prelude::*;
use sp_std::vec::Vec;
use frame_support::traits::ConstU32;

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    where_clause {  where T: Config,
        T::AccountId: Into<AccountId>,}

    register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let username = "BenchmarkName";
    }: register(
        RawOrigin::Signed(caller.clone()),
        username.as_bytes().to_vec(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T>(Event::NameQueuedForRegister.into());
    }

    accept_register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let username = "".as_bytes().to_vec();
    }: accept_register(
        RawOrigin::Signed(caller.clone()),
        username,
        caller.clone().into()
    ) verify {
        assert_last_event::<T>(Event::NameRegistered.into());
    }

    set_primary_name {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let username = "BenchmarkName";
    }: set_primary_name(
        RawOrigin::Signed(caller.clone()),
        username.as_bytes().to_vec(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T>(Event::NameSetAsPrimary.into());
    }

    cancel_pending_register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let username = "BenchmarkName";
    }: cancel_pending_register(
        RawOrigin::Signed(caller.clone()),
        username.as_bytes().to_vec(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T>(Event::RegisterCanceled.into());
    }

    remove_register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let username = "BenchmarkName";
    }: remove_register(
        RawOrigin::Signed(caller.clone()),
        username.as_bytes().to_vec(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T>(Event::RegisterRemoved.into());
    }
}

impl_benchmark_test_suite!(
    NameService,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
