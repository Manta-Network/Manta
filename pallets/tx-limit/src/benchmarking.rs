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

//! TransactionLimit pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TransactionLimit;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{EventRecord, RawOrigin};

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    // Benchmark `set_asset_limit` extrinsic:
    set_asset_limit {
        let asset_id: T::AssetId = T::AssetId::default();
        let amount: T::Balance = <T as Config>::Balance::from(100_u32);
    }: set_asset_limit(RawOrigin::Root, asset_id, amount.clone())
    verify {
        assert_last_event::<T>(
            Event::TransactionLimitSet {
                asset_id,
                amount
            }.into()
        );
    }

    // Benchmark `unset_asset_limit` extrinsic:
    unset_asset_limit {
        let origin: T::Origin = T::Origin::from(RawOrigin::Root);
        let asset_id: T::AssetId = T::AssetId::default();
        let amount: T::Balance = <T as Config>::Balance::from(100_u32);

        TransactionLimit::<T>::set_asset_limit(origin, asset_id, amount)?;
    }: unset_asset_limit(RawOrigin::Root, asset_id)
    verify {
        assert_last_event::<T>(
            Event::TransactionLimitUnset { asset_id }.into()
        );
    }
}

impl_benchmark_test_suite!(
    TransactionLimit,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
