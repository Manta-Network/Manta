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
use frame_system::RawOrigin;
use manta_support::manta_pay::AccountId;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

#[inline]
pub fn assert_last_event<T, E>(event: E)
where
    T: Config,
    E: Into<<T as Config>::RuntimeEvent>,
{
    let events = frame_system::Pallet::<T>::events();
    assert_eq!(events[events.len() - 1].event, event.into().into());
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
        username.clone(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T, _>(Event::NameQueuedForRegister {
            hash_username: T::Hashing::hash_of(&username),
            hash_owner: T::Hashing::hash_of(&caller.into()),
        });
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
        username.clone(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T, _>(Event::NameRegistered {
            username,
            owner: caller.into(),
        });
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
        username.clone(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T, _>(Event::NameSetAsPrimary {
            owner: caller.into(),
            username,
        });
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
        username.clone(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T, _>(Event::RegisterCanceled{
            hash_username: T::Hashing::hash_of(&username),
            hash_owner: T::Hashing::hash_of(&caller.into()),
        });
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
        username.clone(),
        caller.clone().into()
    ) verify {
        assert_last_event::<T, _>(Event::RegisterRemoved {
            username,
            owner: caller.into(),
        });
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
