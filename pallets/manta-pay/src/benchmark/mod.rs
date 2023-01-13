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
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE).unwrap();
        let asset = mint_post.source(0).unwrap();
        init_asset::<T>(&caller, Pallet::<T>::id_from_field(asset.id).unwrap(), asset_value_decode(asset.value));
    }: to_private (
        RawOrigin::Signed(caller.clone()),
        mint_post
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPrivate { asset, source: caller });
    }

    to_public {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        for coin in TO_PUBLIC_INPUT {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &**coin).unwrap()
            ).unwrap();
        }
        let reclaim_post = TransferPost::decode(&mut &*TO_PUBLIC).unwrap();
        let asset = reclaim_post.sink(0).unwrap();
    }: to_public (
        RawOrigin::Signed(caller.clone()),
        reclaim_post
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPublic { asset, sink: caller });
    }

    private_transfer {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        for coin in PRIVATE_TRANSFER_INPUT {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &**coin).unwrap(),
            ).unwrap();
        }
        let private_transfer_post = TransferPost::decode(&mut &*PRIVATE_TRANSFER).unwrap();
    }: private_transfer (
        RawOrigin::Signed(caller.clone()),
        private_transfer_post
    ) verify {
        assert_last_event::<T, _>(Event::PrivateTransfer { origin: Some(caller) });
    }

    public_transfer {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        let asset = Asset::new(Pallet::<T>::field_from_id(8u128), asset_value_encode(100));
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
