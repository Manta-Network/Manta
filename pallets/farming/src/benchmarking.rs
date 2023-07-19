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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg(feature = "runtime-benchmarks")]

use crate::{Pallet as Farming, *};
use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_support::{assert_ok, sp_runtime::traits::UniqueSaturatedFrom};
use frame_system::{Pallet as System, RawOrigin};
use manta_primitives::assets::{AssetConfig, FungibleLedger, TestingDefault};
use pallet_asset_manager::Pallet as AssetManager;

pub const INITIAL_VALUE: u128 = 1_000_000_000_000_000_000_000u128;

benchmarks! {
    where_clause { where T: pallet_assets::Config + pallet_asset_manager::Config, <T as pallet_asset_manager::Config>::Balance: From<u128>, <T as pallet_asset_manager::Config>::AssetId: From<u32> }
    on_initialize {}:{Farming::<T>::on_initialize(T::BlockNumber::from(10u32));}

    create_farming_pool {
        let ksm_asset_id = CurrencyIdOf::<T>::unique_saturated_from(8u128);
        let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
        let tokens_proportion = vec![(ksm_asset_id, Perbill::from_percent(100))];
        let basic_rewards = vec![(ksm_asset_id, token_amount)];
        let gauge_basic_rewards = vec![(ksm_asset_id, token_amount)];
    }: _(RawOrigin::Root,
        tokens_proportion,
        basic_rewards,
        Some((ksm_asset_id, BlockNumberFor::<T>::from(1000u32), gauge_basic_rewards)),
        BalanceOf::<T>::unique_saturated_from(0u128),
        BlockNumberFor::<T>::from(0u32),
        BlockNumberFor::<T>::from(7u32),
        BlockNumberFor::<T>::from(6u32),
        5
    )

    charge {
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::testing_default();
        AssetManager::<T>::do_register_asset(Some(&location), &metadata)?;

        let ksm_asset_id = CurrencyIdOf::<T>::unique_saturated_from(8u128);
        let caller: T::AccountId = whitelisted_caller();
        let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
        let tokens_proportion = vec![(ksm_asset_id, Perbill::from_percent(100))];
        let basic_rewards = vec![(ksm_asset_id, token_amount)];
        let gauge_basic_rewards = vec![(ksm_asset_id, token_amount)];
        assert_ok!(Farming::<T>::create_farming_pool(
            RawOrigin::Root.into(),
            tokens_proportion,
            basic_rewards,
            Some((ksm_asset_id, BlockNumberFor::<T>::from(1000u32), gauge_basic_rewards)),
            BalanceOf::<T>::unique_saturated_from(0u128),
            BlockNumberFor::<T>::from(0u32),
            BlockNumberFor::<T>::from(7u32),
            BlockNumberFor::<T>::from(6u32),
            5,
        ));
        let charge_rewards = vec![(ksm_asset_id, BalanceOf::<T>::unique_saturated_from(300000u128))];

        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
            8.into(),
            &caller,
            INITIAL_VALUE.try_into().unwrap(),
        );
        // assert_ok!(Farming::<T>::charge(RawOrigin::Signed(caller.clone()).into(), 0, charge_rewards));
    }: _(RawOrigin::Signed(caller.clone()), 0, charge_rewards)

    deposit {
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::testing_default();
        AssetManager::<T>::do_register_asset(Some(&location), &metadata)?;

        let ksm_asset_id = CurrencyIdOf::<T>::unique_saturated_from(8u128);
        let caller: T::AccountId = whitelisted_caller();
        let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
        let tokens_proportion = vec![(ksm_asset_id, Perbill::from_percent(100))];
        let basic_rewards = vec![(ksm_asset_id, token_amount)];
        let gauge_basic_rewards = vec![(ksm_asset_id, token_amount)];
        assert_ok!(Farming::<T>::create_farming_pool(
            RawOrigin::Root.into(),
            tokens_proportion,
            basic_rewards,
            Some((ksm_asset_id, BlockNumberFor::<T>::from(1000u32), gauge_basic_rewards)),
            BalanceOf::<T>::unique_saturated_from(0u128),
            BlockNumberFor::<T>::from(0u32),
            BlockNumberFor::<T>::from(7u32),
            BlockNumberFor::<T>::from(6u32),
            5,
        ));
        let charge_rewards = vec![(ksm_asset_id, BalanceOf::<T>::unique_saturated_from(300000u128))];

        // init balance
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
            8.into(),
            &caller,
            INITIAL_VALUE.try_into().unwrap(),
        );
        assert_ok!(Farming::<T>::charge(RawOrigin::Signed(caller.clone()).into(), 0, charge_rewards));
    }: _(RawOrigin::Signed(caller.clone()), 0, token_amount, None)

    withdraw {
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::testing_default();
        AssetManager::<T>::do_register_asset(Some(&location), &metadata)?;

        let ksm_asset_id = CurrencyIdOf::<T>::unique_saturated_from(8u128);
        let caller: T::AccountId = whitelisted_caller();
        let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
        let tokens_proportion = vec![(ksm_asset_id, Perbill::from_percent(100))];
        let basic_rewards = vec![(ksm_asset_id, token_amount)];
        let gauge_basic_rewards = vec![(ksm_asset_id, token_amount)];
        assert_ok!(Farming::<T>::create_farming_pool(
            RawOrigin::Root.into(),
            tokens_proportion,
            basic_rewards,
            Some((ksm_asset_id, BlockNumberFor::<T>::from(1000u32), gauge_basic_rewards)),
            BalanceOf::<T>::unique_saturated_from(0u128),
            BlockNumberFor::<T>::from(0u32),
            BlockNumberFor::<T>::from(7u32),
            BlockNumberFor::<T>::from(6u32),
            5,
        ));

        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
            8.into(),
            &caller,
            INITIAL_VALUE.try_into().unwrap(),
        );
        let charge_rewards = vec![(ksm_asset_id,BalanceOf::<T>::unique_saturated_from(300000u128))];
        assert_ok!(Farming::<T>::charge(RawOrigin::Signed(caller.clone()).into(), 0, charge_rewards));
        assert_ok!(Farming::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), 0, token_amount, None));
    }: _(RawOrigin::Signed(caller.clone()), 0, None)

    claim {
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::testing_default();
        AssetManager::<T>::do_register_asset(Some(&location), &metadata)?;

        let caller: T::AccountId = whitelisted_caller();
        let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
        let ksm_asset_id = CurrencyIdOf::<T>::unique_saturated_from(8u128);
        let tokens_proportion = vec![(ksm_asset_id, Perbill::from_percent(100))];
        let basic_rewards = vec![(ksm_asset_id, token_amount)];
        let gauge_basic_rewards = vec![(ksm_asset_id, token_amount)];
        assert_ok!(Farming::<T>::create_farming_pool(
            RawOrigin::Root.into(),
            tokens_proportion,
            basic_rewards,
            Some((ksm_asset_id, BlockNumberFor::<T>::from(1000u32), gauge_basic_rewards)),
            BalanceOf::<T>::unique_saturated_from(0u128),
            BlockNumberFor::<T>::from(0u32),
            BlockNumberFor::<T>::from(7u32),
            BlockNumberFor::<T>::from(6u32),
            5,
        ));

        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
            8.into(),
            &caller,
            INITIAL_VALUE.try_into().unwrap(),
        );
        let charge_rewards = vec![(ksm_asset_id,BalanceOf::<T>::unique_saturated_from(300000u128))];
        assert_ok!(Farming::<T>::charge(RawOrigin::Signed(caller.clone()).into(), 0, charge_rewards));
        assert_ok!(Farming::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), 0, token_amount, None));
        System::<T>::set_block_number(System::<T>::block_number() + BlockNumberFor::<T>::from(10u32));
        Farming::<T>::on_initialize(BlockNumberFor::<T>::from(0u32));
    }: _(RawOrigin::Signed(caller.clone()), 0)

    gauge_withdraw {
        let location = T::Location::default();
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::testing_default();
        AssetManager::<T>::do_register_asset(Some(&location), &metadata)?;

        let ksm_asset_id = CurrencyIdOf::<T>::unique_saturated_from(8u128);
        let caller: T::AccountId = whitelisted_caller();
        let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
        let tokens_proportion = vec![(ksm_asset_id, Perbill::from_percent(100))];
        let basic_rewards = vec![(ksm_asset_id, token_amount)];
        let gauge_basic_rewards = vec![(ksm_asset_id, token_amount)];
        assert_ok!(Farming::<T>::create_farming_pool(
            RawOrigin::Root.into(),
            tokens_proportion,
            basic_rewards,
            Some((ksm_asset_id, BlockNumberFor::<T>::from(1000u32), gauge_basic_rewards)),
            BalanceOf::<T>::unique_saturated_from(0u128),
            BlockNumberFor::<T>::from(0u32),
            BlockNumberFor::<T>::from(7u32),
            BlockNumberFor::<T>::from(6u32),
            5,
        ));

        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
            8.into(),
            &caller,
            INITIAL_VALUE.try_into().unwrap(),
        );
        let charge_rewards = vec![(ksm_asset_id,BalanceOf::<T>::unique_saturated_from(300000u128))];
        assert_ok!(Farming::<T>::charge(RawOrigin::Signed(caller.clone()).into(), 0, charge_rewards));
        assert_ok!(Farming::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), 0, token_amount, Some((BalanceOf::<T>::unique_saturated_from(100u128), BlockNumberFor::<T>::from(100u32)))));
    }: _(RawOrigin::Signed(caller.clone()), 0)
}
