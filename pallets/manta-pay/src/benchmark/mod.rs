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

use crate::{
    benchmark::precomputed_coins::{
        PRIVATE_TRANSFER, PRIVATE_TRANSFER_INPUT, TO_PRIVATE, TO_PUBLIC, TO_PUBLIC_INPUT,
    },
    types::{asset_value_decode, asset_value_encode, Asset},
    Call, Config, Event, Pallet, StandardAssetId, TransferPost,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;

use manta_primitives::{
    assets::{AssetConfig, AssetRegistry, FungibleLedger, TestingDefault},
    constants::TEST_DEFAULT_ASSET_ED,
    types::Balance,
};
use scale_codec::Decode;

mod precomputed_coins;

pub const INITIAL_VALUE: u128 = 1_000_000_000_000_000_000_000u128;

/// Asserts that the last event that has occurred is the same as `event`.
#[inline]
pub fn assert_last_event<T, E>(event: E)
where
    T: Config,
    E: Into<<T as Config>::Event>,
{
    let events = frame_system::Pallet::<T>::events();
    assert_eq!(events[events.len() - 1].event, event.into().into());
}

/// Init assets for manta-pay
#[inline]
pub fn init_asset<T>(owner: &T::AccountId, id: StandardAssetId, value: Balance)
where
    T: Config,
{
    let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::testing_default();
    let storage_metadata: <T::AssetConfig as AssetConfig<T>>::StorageMetadata = metadata.into();
    <T::AssetConfig as AssetConfig<T>>::AssetRegistry::create_asset(
        id,
        storage_metadata,
        TEST_DEFAULT_ASSET_ED,
        true,
    )
    .expect("Unable to create asset.");
    let pallet_account: T::AccountId = Pallet::<T>::account_id();
    <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
        id,
        owner,
        value + TEST_DEFAULT_ASSET_ED,
    )
    .expect("Unable to mint asset to its new owner.");
    <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
        id,
        &pallet_account,
        TEST_DEFAULT_ASSET_ED,
    )
    .expect("Unable to mint existential deposit to pallet account.");
}
benchmarks! {
    where_clause {  where sp_runtime::AccountId32: From<<T as frame_system::Config>::AccountId>  }

    to_private_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &caller, INITIAL_VALUE, true);
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &Pallet::<T>::account_id(), INITIAL_VALUE, true);
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE[0 as usize]).unwrap();
        let asset = mint_post.source(0).unwrap();
    }: {
        Pallet::<T>::to_private(origin.clone(), mint_post).unwrap();
    } verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPrivate { asset, source: caller });
    }

    to_private_non_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE[1usize]).unwrap();
        let asset = mint_post.source(0).unwrap();
        init_asset::<T>(&caller, Pallet::<T>::id_from_field(asset.id).unwrap(), asset_value_decode(asset.value));
    }: {
        Pallet::<T>::to_private(origin.clone(), mint_post).unwrap();
    } verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPrivate { asset, source: caller });
    }

    to_public_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &caller, INITIAL_VALUE, true);
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &Pallet::<T>::account_id(), INITIAL_VALUE, true);
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*TO_PUBLIC_INPUT[0_usize]).unwrap()
        ).unwrap();
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*TO_PUBLIC_INPUT[1_usize]).unwrap()
        ).unwrap();
        let reclaim_post = TransferPost::decode(&mut &*TO_PUBLIC[0 as usize]).unwrap();
        let asset = reclaim_post.sink(0).unwrap();
    }: {
        Pallet::<T>::to_public(origin.clone(), reclaim_post).unwrap();
    } verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPublic { asset, sink: caller });
    }

    to_public_non_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*TO_PUBLIC_INPUT[2_usize]).unwrap()
        ).unwrap();
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*TO_PUBLIC_INPUT[3_usize]).unwrap()
        ).unwrap();
        let reclaim_post = TransferPost::decode(&mut &*TO_PUBLIC[1 as usize]).unwrap();
        let asset = reclaim_post.sink(0).unwrap();
    }: {
        Pallet::<T>::to_public(origin.clone(), reclaim_post).unwrap();
    } verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPublic { asset, sink: caller });
    }

    private_transfer_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &caller, INITIAL_VALUE, true);
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &Pallet::<T>::account_id(), INITIAL_VALUE, true);
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*PRIVATE_TRANSFER_INPUT[0_usize]).unwrap()
        ).unwrap();
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*PRIVATE_TRANSFER_INPUT[1_usize]).unwrap()
        ).unwrap();
        let private_transfer_post = TransferPost::decode(&mut &*PRIVATE_TRANSFER[0 as usize]).unwrap();
    }: {
        Pallet::<T>::private_transfer(origin.clone(), private_transfer_post).unwrap();
    } verify {
        assert_last_event::<T, _>(Event::PrivateTransfer { origin: Some(caller) });
    }

    private_transfer_non_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*PRIVATE_TRANSFER_INPUT[2_usize]).unwrap()
        ).unwrap();
        Pallet::<T>::to_private(
            origin.clone(),
            TransferPost::decode(&mut &*PRIVATE_TRANSFER_INPUT[3_usize]).unwrap()
        ).unwrap();
        let private_transfer_post = TransferPost::decode(&mut &*PRIVATE_TRANSFER[1 as usize]).unwrap();
    }: {
        Pallet::<T>::private_transfer(origin.clone(), private_transfer_post).unwrap();
    } verify {
        assert_last_event::<T, _>(Event::PrivateTransfer { origin: Some(caller) });
    }

    public_transfer_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &caller, INITIAL_VALUE, true);
        let _ = <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting_with_check(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get(), &Pallet::<T>::account_id(), INITIAL_VALUE, true);
        let asset = Asset::new(Pallet::<T>::field_from_id(<T::AssetConfig as AssetConfig<T>>::NativeAssetId::get()), asset_value_encode(100));
        let sink = Pallet::<T>::account_id();
    }: public_transfer (
        RawOrigin::Signed(caller.clone()),
        asset,
        sink.clone()
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::Transfer { asset, source: caller.clone(), sink });
    }

    public_transfer_non_native {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        let asset = Asset::new(Pallet::<T>::field_from_id(<T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get()), asset_value_encode(100));
        let sink = Pallet::<T>::account_id();
    }: public_transfer (
        RawOrigin::Signed(caller.clone()),
        asset,
        sink.clone()
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::Transfer { asset, source: caller.clone(), sink });
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
