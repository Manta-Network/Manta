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

use crate::{Call, Config, Event, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::{EventRecord, RawOrigin};
use manta_support::manta_pay::AccountId;
use sp_std::prelude::*;

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    where_clause {  where T: Config,
        T::AccountId: From<AccountId> + Into<AccountId>,
    }

    register {
        let caller: T::AccountId = whitelisted_caller();
        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::RegisterPrice::get() * factor.into());
        let origin = RawOrigin::Signed(caller.clone());
        let username = "test".as_bytes().to_vec();
    }: register(
        origin,
        username,
        caller.into()
    ) verify {
        assert_last_event::<T>(Event::NameQueuedForRegister.into());
    }

    accept_register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());
        let username = "test".as_bytes().to_vec();

        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::RegisterPrice::get() * factor.into());

        Pallet::<T>::register(origin.clone().into(), username.clone(), caller.clone().into())?;
        // move blocknumber forward so pending register is available to move to records
        let new_block: T::BlockNumber = 10u32.into();
        frame_system::Pallet::<T>::set_block_number(new_block);

    }: accept_register(
        origin,
        username,
        caller.clone().into()
    ) verify {
        assert_last_event::<T>(Event::NameRegistered.into());
    }

    set_primary_name {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());
        let username = "test".as_bytes().to_vec();

        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::RegisterPrice::get() * factor.into());

        Pallet::<T>::register(origin.clone().into(), username.clone(), caller.clone().into())?;
        // move blocknumber forward so pending register is available to move to records
        let new_block: T::BlockNumber = 10u32.into();
        frame_system::Pallet::<T>::set_block_number(new_block);
        Pallet::<T>::accept_register(origin.clone().into(), username.clone(), caller.clone().into())?;

    }: set_primary_name(
        origin,
        username,
        caller.into()
    ) verify {
        assert_last_event::<T>(Event::NameSetAsPrimary.into());
    }

    cancel_pending_register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());
        let username = "test".as_bytes().to_vec();

        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::RegisterPrice::get() * factor.into());

        Pallet::<T>::register(origin.clone().into(), username.clone(), caller.clone().into())?;
        // move blocknumber forward so pending register is available to move to records
        let new_block: T::BlockNumber = 10u32.into();
        frame_system::Pallet::<T>::set_block_number(new_block);

    }: cancel_pending_register(
        origin,
        username,
        caller.into()
    ) verify {
        assert_last_event::<T>(Event::RegisterCanceled.into());
    }

    remove_register {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());
        let username = "test".as_bytes().to_vec();

        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::RegisterPrice::get() * factor.into());

        Pallet::<T>::register(origin.clone().into(), username.clone(), caller.clone().into())?;
        // move blocknumber forward so pending register is available to move to records
        let new_block: T::BlockNumber = 10u32.into();
        frame_system::Pallet::<T>::set_block_number(new_block);
        Pallet::<T>::accept_register(origin.clone().into(), username.clone(), caller.clone().into())?;

    }: remove_register(
        origin,
        username,
        caller.into()
    ) verify {
        assert_last_event::<T>(Event::RegisterRemoved.into());
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
