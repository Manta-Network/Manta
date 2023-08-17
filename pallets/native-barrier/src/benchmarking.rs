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
//! # Native Barrier Pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as NativeBarrier;
use core::time::Duration;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Hooks;
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;

const SEED: u32 = 0;

benchmarks! {
    //TODO: docs
    on_initialize {
        let daily_limit = T::Balance::zero();
        let _ = NativeBarrier::<T>::set_daily_xcm_limit(RawOrigin::Root.into(), Some(daily_limit))?;
        let start_unix_time = Duration::default();
        let _ = NativeBarrier::<T>::set_start_unix_time(RawOrigin::Root.into(), Some(start_unix_time))?;
        let barrier_addresses: Vec<T::AccountId> = vec![
            account("address_0", 0, SEED),
            account("address_1", 0, SEED),
            account("address_2", 0, SEED),
            account("address_3", 0, SEED),
            account("address_4", 0, SEED)
        ];
        let _ = NativeBarrier::<T>::add_accounts_to_native_barrier(RawOrigin::Root.into(), barrier_addresses)?;
    }:{NativeBarrier::<T>::on_initialize(T::BlockNumber::from(1_000_000u32));}

    //
    set_start_unix_time {
        let caller: T::AccountId = whitelisted_caller();
        let start_unix_time = Duration::default();
    }: set_start_unix_time(RawOrigin::Root, Some(start_unix_time))
    verify {
        assert_eq!(NativeBarrier::<T>::get_start_unix_time().unwrap(), start_unix_time);
    }

    //
    set_daily_xcm_limit {
        let caller: T::AccountId = whitelisted_caller();
        let daily_limit = T::Balance::zero();
    }: set_daily_xcm_limit(RawOrigin::Root, Some(daily_limit))
    verify {
        assert_eq!(NativeBarrier::<T>::get_daily_xcm_limit().unwrap(), daily_limit);
    }

    //
    add_accounts_to_native_barrier {
        let caller: T::AccountId = whitelisted_caller();
        let daily_limit = T::Balance::zero();
        let _ = NativeBarrier::<T>::set_daily_xcm_limit(RawOrigin::Root.into(), Some(daily_limit))?;
        let start_unix_time = Duration::default();
        let _ = NativeBarrier::<T>::set_start_unix_time(RawOrigin::Root.into(), Some(start_unix_time));
        let barrier_addresses: Vec<T::AccountId> = vec![
            account("address_0", 0, SEED),
            account("address_1", 0, SEED),
            account("address_2", 0, SEED),
            account("address_3", 0, SEED),
            account("address_4", 0, SEED)
        ];
        let daily_limit = T::Balance::zero();
    }: add_accounts_to_native_barrier(RawOrigin::Root, barrier_addresses)
    verify {
        for (account_id, _) in RemainingXcmLimit::<T>::iter() {
            assert_eq!(RemainingXcmLimit::<T>::get(account_id).unwrap(), daily_limit);
        }
    }

    //
    remove_accounts_from_native_barrier {
        let caller: T::AccountId = whitelisted_caller();
        let daily_limit = T::Balance::zero();
        let _ = NativeBarrier::<T>::set_daily_xcm_limit(RawOrigin::Root.into(), Some(daily_limit))?;
        let start_unix_time = Duration::default();
        let _ = NativeBarrier::<T>::set_start_unix_time(RawOrigin::Root.into(), Some(start_unix_time))?;
        let barrier_addresses: Vec<T::AccountId> = vec![
            account("address_0", 0, SEED),
            account("address_1", 0, SEED),
            account("address_2", 0, SEED),
            account("address_3", 0, SEED),
            account("address_4", 0, SEED)
        ];
        let _ = NativeBarrier::<T>::add_accounts_to_native_barrier(RawOrigin::Root.into(), barrier_addresses.clone())?;
        let remove_addresses = vec![
            barrier_addresses[0].clone(),
            barrier_addresses[2].clone(),
            barrier_addresses[4].clone()
        ];
    }: remove_accounts_from_native_barrier(RawOrigin::Root, remove_addresses)
    verify {
        assert_eq!(None, RemainingXcmLimit::<T>::get(account::<T::AccountId>("address_0", 0, SEED)));
        assert_eq!(None, RemainingXcmLimit::<T>::get(account::<T::AccountId>("address_2", 0, SEED)));
        assert_eq!(None, RemainingXcmLimit::<T>::get(account::<T::AccountId>("address_4", 0, SEED)));
    }

    impl_benchmark_test_suite!(
        NativeBarrier,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
