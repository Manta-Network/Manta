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
	Asset, Call, Config, Event, Pallet as MantaPay, TransferPost,
};
use frame_support::traits::OriginTrait;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use manta_primitives::assets::AssetConfig;
use frame_system::RawOrigin;
use scale_codec::Decode;
use sp_runtime::traits::StaticLookup;

mod precomputed_coins;

pub const ED: u128 = 1u128;
pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);

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
pub fn init_asset<T>(owner: &T::AccountId, id: T::AssetId, value: T::Balance)
where 
	T: AssetConfig,
{
	
} 

benchmarks! {
	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_native_asset::<T>(&caller, AssetId(0), AssetValue(1_000));
		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup = T::Lookup::unlookup(recipient.clone());
		let asset = Asset::new(0, 10);
	}: transfer (
		RawOrigin::Signed(caller.clone()),
		recipient_lookup,
		asset
	) verify {
		assert_last_event::<T, _>(Event::Transfer { asset, source: caller, sink: recipient.clone() });
		// FIXME: add balance checking
		// assert_eq!(Balances::<T>::get(recipient, asset.id), asset.value);
	}

	mint {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_native_asset::<T>(&caller, AssetId(0), AssetValue(1_000_000));
		let mint_post = TransferPost::decode(&mut &*MINT).unwrap();
		let asset = Asset::new(mint_post.asset_id.unwrap(), mint_post.sources[0]);
	}: mint (
		RawOrigin::Signed(caller.clone()),
		mint_post
	) verify {
		assert_last_event::<T, _>(Event::Mint { asset, source: caller.clone() });
		// FIXME: add balance checking
		// assert_eq!(Balances::<T>::get(caller, asset.id), 1_000_000 - asset.value);
	}

	private_transfer {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_native_asset::<T>(&caller, AssetId(0), AssetValue(1_000_000));
		for coin in PRIVATE_TRANSFER_INPUT {
			MantaPay::<T>::mint(origin.clone(), TransferPost::decode(&mut &**coin).unwrap()).unwrap();
		}
		let private_transfer_post = TransferPost::decode(&mut &*PRIVATE_TRANSFER).unwrap();
	}: private_transfer (
		RawOrigin::Signed(caller.clone()),
		private_transfer_post
	) verify {
		assert_last_event::<T, _>(Event::PrivateTransfer { origin: caller });
	}

	reclaim {
		let caller: T::AccountId = whitelisted_caller();
		let origin = T::Origin::from(RawOrigin::Signed(caller.clone()));
		init_native_asset::<T>(&caller, AssetId(0), AssetValue(1_000_000));
		for coin in RECLAIM_INPUT {
			MantaPay::<T>::mint(origin.clone(), TransferPost::decode(&mut &**coin).unwrap()).unwrap();
		}
		let reclaim_post = TransferPost::decode(&mut &*RECLAIM).unwrap();
	}: reclaim (
		RawOrigin::Signed(caller.clone()),
		reclaim_post
	) verify {
		assert_last_event::<T, _>(Event::Reclaim { asset: Asset::new(0, 10_000), sink: caller });
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
