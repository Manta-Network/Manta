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

use crate::{Call, Config, Event, Pallet, StandardAssetId, TransferPost};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use manta_support::manta_pay::{
    asset_value_decode, asset_value_encode, field_from_id, id_from_field, AccountId, Asset,
};

use manta_primitives::{
    assets::{AssetConfig, FungibleLedger},
    constants::TEST_DEFAULT_ASSET_ED,
    types::Balance,
};
use scale_codec::Decode;

pub const INITIAL_VALUE: u128 = 1_000_000_000_000_000_000_000u128;

const MINTS_OFFSET: usize = 2;
const MINT_SIZE: usize = 553;

const TRANSFERS_OFFSET: usize = 4;
const TRANSFER_SIZE: usize = 1291;

const RECLAIMS_OFFSET: usize = 4;
const RECLAIM_SIZE: usize = 1001;

// 14000 iterations of mint, mint, mint, transfer, mint, mint, reclaim
const TOTAL_ITERATIONS: usize = 14000;

/// Asserts that the last event that has occurred is the same as `event`.
#[inline]
pub fn assert_last_event<T, E>(event: E)
where
    T: Config,
    E: Into<<T as Config>::RuntimeEvent>,
{
    let events = frame_system::Pallet::<T>::events();
    assert_eq!(events[events.len() - 1].event, event.into().into());
}

/// Init assets for manta-pay
#[inline]
pub fn init_asset<T>(owner: &T::AccountId, id: StandardAssetId, value: Balance)
where
    T: Config,
    T::AccountId: From<AccountId> + Into<AccountId>,
{
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
    where_clause {  where T::AccountId: From<AccountId> + Into<AccountId> }
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        let mint_coins = core::include_bytes!("../../../../tests/data/precomputed_mints");
        let mints_start = MINTS_OFFSET + TOTAL_ITERATIONS * MINT_SIZE;
        let to_private_coin = &mint_coins[mints_start..mints_start + MINT_SIZE];
        let mint_post = TransferPost::decode(&mut &*to_private_coin).unwrap();
        let asset = mint_post.source(0).unwrap();
        init_asset::<T>(&caller, id_from_field(asset.id).unwrap(), asset_value_decode(asset.value));
    }: to_private (
        RawOrigin::Signed(caller.clone()),
        mint_post
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPrivate { asset, source: caller });
    }

    to_public {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        let reclaim_coins = core::include_bytes!("../../../../tests/data/precomputed_reclaims");

        let reclaim_start = RECLAIMS_OFFSET + TOTAL_ITERATIONS * (2 * MINT_SIZE + RECLAIM_SIZE);
        let to_public_input = {
            let mint_post1 = &reclaim_coins[reclaim_start..reclaim_start + MINT_SIZE];
            let mint_post2 = &reclaim_coins[reclaim_start + MINT_SIZE..reclaim_start + 2 * MINT_SIZE];

            [mint_post1, mint_post2]
        };

        for coin in to_public_input {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &*coin).unwrap()
            ).unwrap();
        }

        let to_public_post = &reclaim_coins[reclaim_start + 2 * MINT_SIZE..reclaim_start + 2 * MINT_SIZE + RECLAIM_SIZE];
        let reclaim_post = TransferPost::decode(&mut &*to_public_post).unwrap();
        let asset = reclaim_post.sink(0).unwrap();
    }: to_public (
        RawOrigin::Signed(caller.clone()),
        reclaim_post.clone()
    ) verify {
        // FIXME: add balance checking
        assert_last_event::<T, _>(Event::ToPublic { asset, sink: T::AccountId::from(reclaim_post.sink_accounts[0]) });
    }

    private_transfer {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        let private_transfer_coins = core::include_bytes!("../../../../tests/data/precomputed_transfers");
        let transfer_start = TRANSFERS_OFFSET + TOTAL_ITERATIONS * (2 * MINT_SIZE + TRANSFER_SIZE);
        let private_transfer_input = {
            let mint_post1 = &private_transfer_coins[transfer_start..transfer_start + MINT_SIZE];
            let mint_post2 = &private_transfer_coins[transfer_start + MINT_SIZE..transfer_start + 2 * MINT_SIZE];

            [mint_post1, mint_post2]
        };

        for coin in private_transfer_input {
            Pallet::<T>::to_private(
                origin.clone(),
                TransferPost::decode(&mut &*coin).unwrap(),
            ).unwrap();
        }
        let private_transfer_post = &private_transfer_coins[transfer_start + 2 * MINT_SIZE..transfer_start + 2 * MINT_SIZE + TRANSFER_SIZE];
        let private_transfer_post = TransferPost::decode(&mut &*private_transfer_post).unwrap();
    }: private_transfer (
        RawOrigin::Signed(caller.clone()),
        private_transfer_post
    ) verify {
        assert_last_event::<T, _>(Event::PrivateTransfer { origin: Some(caller) });
    }

    public_transfer {
        let caller: T::AccountId = whitelisted_caller();
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
        init_asset::<T>(&caller, <T::AssetConfig as AssetConfig<T>>::StartNonNativeAssetId::get(), INITIAL_VALUE);
        let asset = Asset::new(field_from_id(8u128), asset_value_encode(100));
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

// Unit tests are disabled as they are not compatible with the custom chain-spec logic we have for the benchmarks
// impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
