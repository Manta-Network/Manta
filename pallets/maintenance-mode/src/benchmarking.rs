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

//! MaintenanceMode pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as MaintenanceMode;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{EventRecord, RawOrigin};

pub fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    // Benchmark `enter_maintenance_mode` extrinsic:
    enter_maintenance_mode {
    }: enter_maintenance_mode(RawOrigin::Root)
    verify {
        assert_last_event::<T>(
            Event::EnteredMaintenanceMode {}.into()
        );
    }

    // Benchmark `resume_normal_operation` extrinsic:
    resume_normal_operation {
        let origin: T::Origin = T::Origin::from(RawOrigin::Root);
        MaintenanceMode::<T>::enter_maintenance_mode(origin)?;
    }: resume_normal_operation(RawOrigin::Root)
    verify {
        assert_last_event::<T>(
            Event::NormalOperationResumed{}.into()
        );
    }
}

impl_benchmark_test_suite!(
    MaintenanceMode,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
