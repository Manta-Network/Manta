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
    types::{Asset, AssetId, AssetValue},
    Call, Config, Event, Pallet, TransferPost,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use manta_primitives::{
    assets::{AssetConfig, AssetRegistry, FungibleLedger},
    constants::TEST_DEFAULT_ASSET_ED,
};
use scale_codec::Decode;

mod precomputed_coins;

/// Asserts that the last event that has occured is the same as `event`.
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
pub fn init_asset<T>(owner: &T::AccountId, id: AssetId, value: AssetValue)
where
    T: Config,
{
    /* TODO:
    let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistryMetadata::default();
    let storage_metadata: <T::AssetConfig as AssetConfig<T>>::StorageMetadata = metadata.into();
    <T::AssetConfig as AssetConfig<T>>::AssetRegistry::create_asset(
        Pallet::<T>::id_from_field(id).expect("FIXME"),
        storage_metadata,
        TEST_DEFAULT_ASSET_ED,
        true,
    )
    .expect("Unable to create asset.");
    let pallet_account: T::AccountId = Pallet::<T>::account_id();
    <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
        Pallet::<T>::id_from_field(id).expect("FIXME"),
        owner,
        value + TEST_DEFAULT_ASSET_ED,
    )
    .expect("Unable to mint asset to its new owner.");
    <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_minting(
        Pallet::<T>::id_from_field(id).expect("FIXME"),
        &pallet_account,
        TEST_DEFAULT_ASSET_ED,
    )
    .expect("Unable to mint existential deposit to pallet account.");
    */
    todo!()
}

benchmarks! {
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE).unwrap();
        let asset = mint_post.source(0).unwrap();
        init_asset::<T>(&caller, asset.id, asset.value);
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
        for coin in TO_PUBLIC_INPUT {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &**coin).unwrap()
            ).unwrap();
        }
        let reclaim_post = TransferPost::decode(&mut &*TO_PUBLIC).unwrap();
        let asset = reclaim_post.sink(0).unwrap();
        init_asset::<T>(&caller, asset.id, asset.value);
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
        init_asset::<T>(&caller, Pallet::<T>::field_from_id(8), 1_000_000u128);
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
        assert_last_event::<T, _>(Event::PrivateTransfer { origin: caller });
    }

    public_transfer {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        let id = Pallet::<T>::field_from_id(8);
        init_asset::<T>(&caller, id, 1_000_000u128);
        let asset = Asset::new(id, 100);
        let sink =  Pallet::<T>::account_id();
    }: public_transfer (
        RawOrigin::Signed(caller.clone()),
        asset,
        sink.clone()
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::Transfer { asset, source: caller.clone(), sink });
    }
}

// TODO: impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
