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

use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::{EventRecord, RawOrigin};
use manta_primitives::{
    assets::{AssetConfig, AssetRegistryMetadata, FungibleLedger, TestingDefault, UnitsPerSecond},
    types::Balance,
};
use xcm::latest::prelude::*;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    where_clause { where T::Location: From<MultiLocation>, <T as Config>::AssetId: From<u32> }

    register_asset {
        let location = T::Location::default();
        let metadata = AssetRegistryMetadata::<Balance>::testing_default();
    }: _(RawOrigin::Root, location.clone(), metadata)
    verify {
        assert_eq!(Pallet::<T>::asset_id_location(crate::NextAssetId::<T>::get() - 1.into()), Some(location));
    }

    set_units_per_second {
        let assets_count = 1000;
        for i in 8..assets_count + 8 {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            Pallet::<T>::set_units_per_second(RawOrigin::Root.into(), <T as Config>::AssetId::from(i), 0)?;
        }
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = AssetRegistryMetadata::<Balance>::testing_default();
        let amount = 10;
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata)?;
        let some_valid_asset_id = <T as Config>::AssetId::from(assets_count + 8);
    }: _(RawOrigin::Root, some_valid_asset_id, amount)
    verify {
        assert_eq!(Pallet::<T>::units_per_second(&some_valid_asset_id), Some(amount));
    }

    update_asset_location {
        let assets_count = 1000;
        for i in 0..assets_count {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
        }
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = AssetRegistryMetadata::<Balance>::testing_default();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata)?;
        let new_location = T::Location::from(MultiLocation::new(0, X1(Parachain(1000))));
        let some_valid_asset_id = <T as Config>::AssetId::from(assets_count);
    }: _(RawOrigin::Root, some_valid_asset_id, new_location.clone())
    verify {
        assert_eq!(Pallet::<T>::asset_id_location(some_valid_asset_id), Some(new_location));
    }

    update_asset_metadata {
        let assets_count = 1000;
        for i in 0..assets_count {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
        }
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = AssetRegistryMetadata::<Balance>::testing_default();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata.clone())?;
        let some_valid_asset_id = <T as Config>::AssetId::from(assets_count);
    }: _(RawOrigin::Root, some_valid_asset_id, metadata.metadata.clone())
    verify {
        assert_last_event::<T>(crate::Event::AssetMetadataUpdated { asset_id: some_valid_asset_id, metadata }.into());
    }

    mint_asset {
        let assets_count = 1000;
        for i in 0..assets_count {
            let location = T::Location::from(MultiLocation::new(0, X1(Parachain(i))));
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
        }
        let beneficiary: T::AccountId = whitelisted_caller();
        let amount = 100;
        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = AssetRegistryMetadata::<Balance>::testing_default();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location, metadata)?;
        let some_valid_asset_id = <T as Config>::AssetId::from(assets_count);
    }: _(RawOrigin::Root, <T as Config>::AssetId::from(assets_count), beneficiary.clone(), amount )
    verify {
        assert_last_event::<T>(crate::Event::AssetMinted { asset_id: some_valid_asset_id, beneficiary, amount }.into());
    }

    set_min_xcm_fee {
        let assets_count = 1000;
        for i in 8..assets_count + 8 {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();

            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
            Pallet::<T>::set_units_per_second(RawOrigin::Root.into(), <T as Config>::AssetId::from(i), 0)?;
        }

        // does not really matter what we register, as long as it is different than the previous
        let location = T::Location::default();
        let metadata = AssetRegistryMetadata::<Balance>::testing_default();
        let min_xcm_fee = 10;
        Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata)?;

    }: _(RawOrigin::Root, location.clone(), min_xcm_fee)
    verify {
        assert_eq!(Pallet::<T>::get_min_xcm_fee(location), Some(min_xcm_fee));
    }

    update_outgoing_filtered_assets {
        let assets_count = 1000;
        for i in 0..assets_count {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
        }
        let location: MultiLocation = MultiLocation::new(0, X1(Parachain(1)));
    }: _(RawOrigin::Root, location.clone().into(), true)
    verify {
        assert_last_event::<T>(crate::Event::AssetLocationFilteredForOutgoingTransfers { filtered_location: location.into() }.into());
    }

    register_lp_asset {
        let current_asset_id = crate::NextAssetId::<T>::get();
        let assets_count = 10;
        for i in 8..assets_count {
            let location: MultiLocation = MultiLocation::new(0, X1(Parachain(i)));
            let location = T::Location::from(location.clone());
            let metadata = AssetRegistryMetadata::<Balance>::testing_default();
            Pallet::<T>::register_asset(RawOrigin::Root.into(), location.clone(), metadata.clone())?;
        }
        let lp_metadata = AssetRegistryMetadata::<Balance>::testing_default();
    }: _(RawOrigin::Root, current_asset_id, current_asset_id + 1.into(), lp_metadata)
    verify {
        assert_eq!(Pallet::<T>::asset_id_pair_to_lp((current_asset_id, current_asset_id + 1.into())), Some(current_asset_id + 2.into()));
        assert_eq!(Pallet::<T>::lp_to_asset_id_pair(current_asset_id + 2.into()), Some((current_asset_id, current_asset_id + 1.into())));
    }

    permissionless_register_asset {
        let caller = whitelisted_caller();
        let native_asset_id = <T::AssetConfig as AssetConfig<T>>::NativeAssetId::get();
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
            native_asset_id,
            &caller,
            1_000_000_000_000_000_000_000u128
        );
    }: _(RawOrigin::Signed(caller), vec![].try_into().unwrap(), vec![].try_into().unwrap(), 12, 1_000_000_000_000_000)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
