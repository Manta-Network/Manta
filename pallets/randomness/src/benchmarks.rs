// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{
    Call, Config, InherentIncluded, Pallet, RandomnessResult, RandomnessResults, RelayEpoch,
    RequestType,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
    dispatch::DispatchResult,
    traits::{Currency, Get, OnInitialize},
};
use frame_system::RawOrigin;
use nimbus_primitives::{digests::CompatibleDigestItem as NimbusDigest, NimbusId};
use parity_scale_codec::{alloc::string::ToString, Decode};
use scale_info::prelude::string::String;
use sp_core::{
    crypto::{ByteArray, UncheckedFrom},
    sr25519, H160, H256,
};
use sp_runtime::traits::One;
use sp_std::{mem::size_of, vec};

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
