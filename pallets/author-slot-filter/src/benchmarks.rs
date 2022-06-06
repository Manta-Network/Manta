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

use crate::num::NonZeroU32;
use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

benchmarks! {
	set_eligible {
		let count = NonZeroU32::new_unchecked(34);
	}: _(RawOrigin::Root, count.clone())
	verify {
		assert_eq!(Pallet::<T>::eligible_count(), count);
	}
}

#[cfg(test)]
mod tests {
	use crate::tests::Test;
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
	crate::tests::Test
);
