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

use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use xcm::{latest::prelude::*, VersionedMultiLocation};

use manta_primitives::{
	assets::{
		AssetConfig, AssetIdLocationGetter, AssetLocation, AssetMetadata, AssetRegistrar,
		FungibleLedger, UnitsToWeightRatio,
	},
	types::{AssetId, Balance},
};

benchmarks! {
	// register_asset {
	// 	let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
	// 	let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

	// }: _(RawOrigin::Root, location.clone(), metadata.clone())
	// verify {
	// 	//assert_eq!(Pallet::<T>::asset_id_type(asset_id), Some(asset_type));
	// }

	set_units_per_second {
		// We make it dependent on the number of existing assets already
		//let x in 5..100;
		for i in 1..100 {
			log::info!("\n i is: {:?} \n", i);

			let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
			log::info!("\n {:?} \n", location.clone());
			let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::from(location.clone());
			let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

			Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
			Pallet::<T>::set_units_per_second(RawOrigin::Root.into(), i, 0)?;
		}

		// does not really matter what we register, as long as it is different than the previous
		let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
		let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;

	}: _(RawOrigin::Root, 100, 0)
	verify {
		// assert!(Pallet::<T>::supported_fee_payment_assets().contains(&asset_type));
		// assert_eq!(Pallet::<T>::asset_type_units_per_second(asset_type), Some(1));
	}
	// update_asset_location
	// update_asset_metadata
	// mint_asset
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
