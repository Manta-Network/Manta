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
use manta_primitives::{
    assets::{AssetConfig, UnitsPerSecond},
    util::num::CheckedIncrement,
};
use xcm::latest::prelude::*;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    where_clause { where T::Location: From<MultiLocation>, <T as Config>::Balance: From<u32> }

    register_asset {
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
    }: _(RawOrigin::Root, location.clone(), metadata)
    verify {
        assert_eq!(Pallet::<T>::asset_id_location(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()), Some(location));
    }

    set_units_per_second {
        let mut next = <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get();
        for i in 0..1000 {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            Pallet::<T>::set_units_per_second(RawOrigin::Root.into(), next, 0)?;
            next.checked_increment().unwrap();
        }
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
        let amount = 10;
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata)?;
    }: _(RawOrigin::Root, next, amount)
    verify {
        assert_eq!(Pallet::<T>::units_per_second(&next), Some(amount));
    }

    update_asset_location {
        let mut next = <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get();
        for i in 0..1000 {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            next.checked_increment().unwrap();
        }
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata)?;
        let new_location = T::Location::from(MultiLocation::new(0, X1(Parachain(1000))));
    }: _(RawOrigin::Root, next, new_location.clone())
    verify {
        assert_eq!(Pallet::<T>::asset_id_location(next), Some(new_location));
    }

    update_asset_metadata {
        let mut next = <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get();
        for i in 0..1000 {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            next.checked_increment().unwrap();
        }
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata.clone())?;
    }: _(RawOrigin::Root, next, metadata.clone())
    verify {
        assert_last_event::<T>(Event::AssetMetadataUpdated { asset_id: next, metadata }.into());
    }

    mint_asset {
        let mut next = <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get();
        for i in 0..1000 {
            let location = T::Location::from(MultiLocation::new(0, X1(Parachain(i))));
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            next.checked_increment().unwrap();
        }
        let beneficiary: T::AccountId = whitelisted_caller();
        let amount = 100;
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata)?;
    }: _(RawOrigin::Root, next, beneficiary.clone(), <T as Config>::Balance::from(amount) )
    verify {
        assert_last_event::<T>(Event::AssetMinted { asset_id: next, beneficiary, amount: <T as Config>::Balance::from(amount) }.into());
    }

    set_min_xcm_fee {
        let mut next = <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get();
        for i in 0..1000 {

            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();

            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            Pallet::<T>::set_units_per_second(RawOrigin::Root.into(), next, 0)?;
            next.checked_increment().unwrap();
        }

        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
        let min_xcm_fee = 10;
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata)?;

    }: _(RawOrigin::Root, location.clone(), min_xcm_fee)
    verify {
        assert_eq!(Pallet::<T>::get_min_xcm_fee(location), Some(min_xcm_fee));
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
