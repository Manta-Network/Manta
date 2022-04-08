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

#![cfg(feature = "runtime-benchmarks")]

use crate::{Call, Config, Event, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::{EventRecord, RawOrigin};
use xcm::latest::prelude::*;

use manta_primitives::assets::{AssetConfig, UnitsToWeightRatio};

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {
	where_clause { where <T::AssetConfig as AssetConfig<T>>::AssetLocation: From<MultiLocation> }
	register_asset {
		let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
		let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

	}: _(RawOrigin::Root, location.clone(), metadata.clone())
	verify {
		assert_eq!(Pallet::<T>::asset_id_location(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()), Some(location));
	}

	set_units_per_second {
		let x in (<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get())..(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get() + 50);
		for i in <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()..x {

			let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
			let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::from(location.clone());
			let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

			Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
			Pallet::<T>::set_units_per_second(RawOrigin::Root.into(), i, 0)?;
		}

		// does not really matter what we register, as long as it is different than the previous
		let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
		let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;

	}: _(RawOrigin::Root, x, 10)
	verify {
		assert_eq!(Pallet::<T>::get_units_per_second(x), Some(10));
	}

	update_asset_location {
		let x in (<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get())..(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get() + 50);
		for i in <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()..x {

			let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
			let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::from(location.clone());
			let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

			Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
		}

		// does not really matter what we register, as long as it is different than the previous
		let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
		let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
		let new_location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::from(MultiLocation::new(0, X1(Parachain(x))));
	}: _(RawOrigin::Root, x, new_location.clone())
	verify {
		assert_eq!(Pallet::<T>::asset_id_location(x), Some(new_location));
	}

	update_asset_metadata {
		let x in (<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get())..(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get() + 50);
		for i in <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()..x {

			let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
			let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::from(location.clone());
			let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

			Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
		}

		// does not really matter what we register, as long as it is different than the previous
		let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
		let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
	}: _(RawOrigin::Root, x, metadata.clone())
	verify {
		assert_last_event::<T>(Event::AssetMetadataUpdated { asset_id: x, metadata }.into());
	}

	mint_asset {
		let x in (<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get())..(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get() + 50);
		for i in <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()..x {

			let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::from(MultiLocation::new(0, X1(Parachain(i))));
			let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();

			Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
		}
		let beneficiary: T::AccountId = whitelisted_caller();
		let amount = 100;
		// does not really matter what we register, as long as it is different than the previous
		let location = <T::AssetConfig as AssetConfig<T>>::AssetLocation::default();
		let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
	}: _(RawOrigin::Root, x, beneficiary.clone(), amount)
	verify {
		assert_last_event::<T>(Event::AssetMinted { asset_id: x, beneficiary, amount }.into());
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
