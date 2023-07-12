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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{
    Call, Config, InherentIncluded, Pallet, RandomnessResult, RandomnessResults, RelayEpoch,
    RequestType,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

benchmarks! {
    // Benchmark for inherent included in every block
    set_babe_randomness_results {
        // set the current relay epoch as 9, `get_epoch_index` configured to return 10
        const BENCHMARKING_OLD_EPOCH: u64 = 9u64;
        RelayEpoch::<T>::put(BENCHMARKING_OLD_EPOCH);
        let benchmarking_babe_output = T::Hash::default();
        let benchmarking_new_epoch = BENCHMARKING_OLD_EPOCH.saturating_add(1u64);
        RandomnessResults::<T>::insert(
            RequestType::BabeEpoch(benchmarking_new_epoch),
            RandomnessResult::new()
        );
    }: _(RawOrigin::None)
    verify {
        // verify randomness result
        assert_eq!(
            RandomnessResults::<T>::get(
                RequestType::BabeEpoch(benchmarking_new_epoch)
            ).unwrap().randomness,
            Some(benchmarking_babe_output)
        );
        assert!(InherentIncluded::<T>::get().is_some());
        assert_eq!(
            RelayEpoch::<T>::get(),
            benchmarking_new_epoch
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::Test;
    use sp_io::TestExternalities;

    pub fn new_test_ext() -> TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        TestExternalities::new(t)
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::benchmarks::tests::new_test_ext(),
    crate::mock::Test
);
