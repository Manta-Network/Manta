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
        PRIVATE_TRANSFER, PRIVATE_TRANSFER_INPUT, TO_PUBLIC, TO_PUBLIC_INPUT,
    },
    Asset, Call, Config, Event, Pallet, Shards, TransferPost,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, Vec};
use frame_system::RawOrigin;
use manta_primitives::{
    assets::{AssetConfig, AssetRegistrar, FungibleLedger},
    constants::DEFAULT_ASSET_ED,
    types::{AssetId, Balance},
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
pub fn init_asset<T>(owner: &T::AccountId, id: AssetId, value: Balance)
where
    T: Config,
{
    let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistrarMetadata::default();
    let storage_metadata: <T::AssetConfig as AssetConfig<T>>::StorageMetadata = metadata.into();
    <T::AssetConfig as AssetConfig<T>>::AssetRegistrar::create_asset(
        id,
        DEFAULT_ASSET_ED,
        storage_metadata,
        true,
    )
    .expect("Unable to create asset.");
    let pallet_account: T::AccountId = Pallet::<T>::account_id();
    <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_can_mint(
        id,
        owner,
        value + DEFAULT_ASSET_ED,
    )
    .expect("Unable to mint asset to its new owner.");
    <T::AssetConfig as AssetConfig<T>>::FungibleLedger::deposit_can_mint(
        id,
        &pallet_account,
        DEFAULT_ASSET_ED,
    )
    .expect("Unable to mint existential deposit to pallet account.");
}

//pub const COINS_SIZE: usize = 699; // 2 v0
pub const COINS_SIZE: usize = 87250004; // 250k v0
                                        // pub const COINS_SIZE: usize = 977; // 2 v1
                                        // pub const COINS_SIZE: usize = 4880002; // 10k v1
                                        //  pub const COINS_SIZE: usize = 48802; // 100 v1
                                        // pub const COINS_SIZE: usize = 488002; // 1000 v1
pub const COINS: &'static [u8; COINS_SIZE] = include_bytes!("./precomputed_mints");
pub const V0_SIZE: usize = 349;
pub const V1_SIZE: usize = 488;
pub const COINS_COUNT: usize = 250_000;
pub const OFFSET: usize = 4;

benchmarks! {
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
        for i in 8 .. 4000 {
            init_asset::<T>(&caller, i, 1_000_000_000_000_000_000_000_000_000_000u128);
        }

        // for i in 0 .. COINS_COUNT - 1 {
        //     let coin: TransferPost = TransferPost::decode(&mut &*MINT[i]).unwrap();
        //     Pallet::<T>::to_private(
        //         origin.clone(),
        //         coin
        //     ).unwrap();
        // }
        // let mint_post = TransferPost::decode(&mut &*MINT[COINS_COUNT - 1]).unwrap();

        for i in 0 .. COINS_COUNT - 1 {
            let start = OFFSET + i * V0_SIZE;
            let end = start + V0_SIZE;
            let coin: TransferPost = TransferPost::decode(&mut &COINS[start..end]).unwrap();
            Pallet::<T>::to_private(
                origin.clone(),
                coin
            ).unwrap();
        }
        let start = 2 + (COINS_COUNT - 1) * V0_SIZE;
        let end = start + V0_SIZE;
        let mint_post = TransferPost::decode(&mut &COINS[start..end]).unwrap();

        let asset = Asset::new(mint_post.asset_id.unwrap(), mint_post.sources[0]);
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
        for i in 8 .. 4000 {
            init_asset::<T>(&caller, i, 1_000_000_000_000_000_000_000_000_000_000u128);
        }

        for coin in TO_PUBLIC_INPUT {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &**coin).unwrap(),
            ).unwrap();
        }

        // for coin in MINT {
        //     Pallet::<T>::to_private(
        //         origin.clone(),
        //         TransferPost::decode(&mut &**coin).unwrap(),
        //     ).unwrap();
        // }

        for i in 0 .. COINS_COUNT {
            let start = OFFSET + i * V0_SIZE;
            let end = start + V0_SIZE;
            let coin: TransferPost = TransferPost::decode(&mut &COINS[start..end]).unwrap();
            Pallet::<T>::to_private(
                origin.clone(),
                coin
            ).unwrap();
        }

        let reclaim_post = TransferPost::decode(&mut &*TO_PUBLIC).unwrap();
        let asset = Asset::new(reclaim_post.asset_id.unwrap(), reclaim_post.sinks[0]);
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
        for i in 8 .. 4000 {
            init_asset::<T>(&caller, i, 1_000_000_000_000_000_000_000_000_000_000u128);
        }

        for coin in PRIVATE_TRANSFER_INPUT {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &**coin).unwrap(),
            ).unwrap();
        }

        // for coin in MINT {
        //     Pallet::<T>::to_private(
        //         origin.clone(),
        //         TransferPost::decode(&mut &**coin).unwrap(),
        //     ).unwrap();
        // }

        for i in 0 .. COINS_COUNT {
            let start = OFFSET    + i * V0_SIZE;
            let end = start + V0_SIZE;
            let coin: TransferPost = TransferPost::decode(&mut &COINS[start..end]).unwrap();
            Pallet::<T>::to_private(
                origin.clone(),
                coin
            ).unwrap();
        }

        let private_transfer_post = TransferPost::decode(&mut &*PRIVATE_TRANSFER).unwrap();
    }: private_transfer (
        RawOrigin::Signed(caller.clone()),
        private_transfer_post
    ) verify {
        assert_last_event::<T, _>(Event::PrivateTransfer { origin: caller });
    }

    // public_transfer {
    //     let caller: T::AccountId = whitelisted_caller();
    //     let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
    //     init_asset::<T>(&caller, 8u32, 1_000_000u128);
    //     let asset = Asset::new(8, 100);
    //     let sink = Pallet::<T>::account_id();
    // }: public_transfer (
    //     RawOrigin::Signed(caller.clone()),
    //     asset,
    //     sink.clone()
    // ) verify {
    //     // FIXME: add balance checking
    //     assert_last_event::<T, _>(Event::Transfer { asset, source: caller.clone(), sink, });
    // }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
