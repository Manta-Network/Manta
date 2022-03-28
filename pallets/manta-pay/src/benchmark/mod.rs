// Copyright 2019-2022 Manta Network.
// This file is part of pallet-manta-pay.
//
// pallet-manta-pay is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pallet-manta-pay is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pallet-manta-pay.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	benchmark::precomputed_coins::{
		MINT, PRIVATE_TRANSFER, PRIVATE_TRANSFER_INPUT, RECLAIM, RECLAIM_INPUT,
	},
	Asset, Call, Config, Event, Pallet, TransferPost,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use manta_primitives::{
	assets::{AssetConfig, AssetRegistrar, FungibleLedger},
	types::{AssetId, Balance},
	constants::DEFAULT_ASSET_ED,
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
	let metadata = <T::AssetConfig as AssetConfig>::AssetRegistrarMetadata::default();
	let storage_metadata: <T::AssetConfig as AssetConfig>::StorageMetadata = metadata.into();
	<T::AssetConfig as AssetConfig>::AssetRegistrar::create_asset(id, DEFAULT_ASSET_ED, storage_metadata, true)
		.unwrap();
	let pallet_account: T::AccountId = Pallet::<T>::account_id();
	T::FungibleLedger::mint(id, &owner, value + DEFAULT_ASSET_ED).unwrap();
	T::FungibleLedger::mint(id, &pallet_account, DEFAULT_ASSET_ED).unwrap();
}

benchmarks! {
	to_private {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_asset::<T>(&caller, 2u32, 1_000_000u128);
		let mint_post = TransferPost::decode(&mut &*MINT).unwrap();
		let asset = Asset::new(mint_post.asset_id.unwrap(), mint_post.sources[0]);
	}: to_private (
		RawOrigin::Signed(caller.clone()),
		mint_post
	) verify {
		assert_last_event::<T, _>(Event::ToPrivate { asset, source: caller.clone() });
		// FIXME: add balance checking
		// assert_eq!(Balances::<T>::get(caller, asset.id), 1_000_000 - asset.value);
	}

	private_transfer {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_asset::<T>(&caller, 2u32, 1_000_000u128);
		for coin in PRIVATE_TRANSFER_INPUT {
			Pallet::<T>::to_private(origin.clone(), TransferPost::decode(&mut &**coin).unwrap()).unwrap();
		}
		let private_transfer_post = TransferPost::decode(&mut &*PRIVATE_TRANSFER).unwrap();
	}: private_transfer (
		RawOrigin::Signed(caller.clone()),
		private_transfer_post
	) verify {
		assert_last_event::<T, _>(Event::PrivateTransfer { origin: caller });
	}

	to_public {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_asset::<T>(&caller, 2u32, 1_000_000u128);
		for coin in RECLAIM_INPUT {
			Pallet::<T>::to_private(origin.clone(), TransferPost::decode(&mut &**coin).unwrap()).unwrap();
		}
		let reclaim_post = TransferPost::decode(&mut &*RECLAIM).unwrap();
	}: to_public (
		RawOrigin::Signed(caller.clone()),
		reclaim_post
	) verify {
		assert_last_event::<T, _>(Event::ToPublic { asset: Asset::new(2, 10_000), sink: caller });
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
