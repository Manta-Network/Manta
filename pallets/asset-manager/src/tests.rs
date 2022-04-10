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
//
// The pallet-tx-pause pallet is forked from Acala's transaction-pause module https://github.com/AcalaNetwork/Acala/tree/master/modules/transaction-pause
// The original license is the following - SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

//! unit tests for asset-manager

use crate::{
	self as asset_manager, AssetIdLocation, AssetIdMetadata, LocationAssetId, UnitsPerSecond,
};
use asset_manager::mock::*;
use frame_support::{assert_noop, assert_ok};
use manta_primitives::assets::{
	AssetConfig, AssetLocation, AssetRegistrarMetadata, FungibleLedger,
};
use sp_runtime::traits::BadOrigin;
use xcm::{latest::prelude::*, VersionedMultiLocation};

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);

#[test]
fn basic_setup_should_work() {
	new_test_ext().execute_with(|| {
		let asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
		let asset_location = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetLocation::get();
		let asset_metadata = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetMetadata::get();
		assert_eq!(
			AssetIdLocation::<Runtime>::get(asset_id),
			Some(asset_location.clone())
		);
		assert_eq!(
			AssetIdMetadata::<Runtime>::get(asset_id),
			Some(asset_metadata)
		);
		assert_eq!(
			LocationAssetId::<Runtime>::get(asset_location),
			Some(asset_id)
		);
	});
}

#[test]
fn wrong_modifier_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		let asset_metadata = AssetRegistrarMetadata {
			name: b"Kusama".to_vec(),
			symbol: b"KSM".to_vec(),
			decimals: 12,
			min_balance: 1u128,
			evm_address: None,
			is_frozen: false,
			is_sufficient: true,
		};
		let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
		assert_noop!(
			AssetManager::register_asset(
				Origin::signed([0u8; 32].into()),
				source_location.clone(),
				asset_metadata.clone()
			),
			BadOrigin
		);
		assert_noop!(
			AssetManager::update_asset_location(
				Origin::signed([2u8; 32].into()),
				0,
				source_location
			),
			BadOrigin
		);
		assert_noop!(
			AssetManager::update_asset_metadata(
				Origin::signed([3u8; 32].into()),
				0,
				asset_metadata
			),
			BadOrigin
		);
		assert_noop!(
			AssetManager::set_units_per_second(Origin::signed([4u8; 32].into()), 0, 0),
			BadOrigin
		);
	})
}

#[test]
fn register_asset_should_work() {
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let new_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X2(Parachain(1), PalletInstance(PALLET_BALANCES_INDEX)),
	)));
	new_test_ext().execute_with(|| {
		let mut counter: u32 =
			<MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
		// Register relay chain native token
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_eq!(
			AssetIdLocation::<Runtime>::get(counter),
			Some(source_location.clone())
		);
		counter += 1;
		// Register twice will fail
		assert_noop!(
			AssetManager::register_asset(Origin::root(), source_location, asset_metadata.clone()),
			crate::Error::<Runtime>::LocationAlreadyExists
		);
		// Register a new asset
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			new_location.clone(),
			asset_metadata.clone()
		));
		assert_eq!(AssetIdLocation::<Runtime>::get(counter), Some(new_location));
	})
}

#[test]
fn update_asset() {
	let asset_metadata = AssetRegistrarMetadata {
		name: b"Kusama".to_vec(),
		symbol: b"KSM".to_vec(),
		decimals: 12,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	let mut new_metadata = asset_metadata.clone();
	new_metadata.is_frozen = true;
	let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
	let new_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
		1,
		X2(Parachain(1), PalletInstance(PALLET_BALANCES_INDEX)),
	)));
	new_test_ext().execute_with(|| {
		// Register relay chain native token
		let asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_eq!(
			AssetIdLocation::<Runtime>::get(asset_id),
			Some(source_location.clone())
		);
		// Update the asset metadata
		assert_ok!(AssetManager::update_asset_metadata(
			Origin::root(),
			asset_id,
			new_metadata.clone()
		));
		// Update the asset location
		assert_ok!(AssetManager::update_asset_location(
			Origin::root(),
			asset_id,
			new_location.clone()
		));
		// Update asset units per seconds
		assert_ok!(AssetManager::set_units_per_second(
			Origin::root(),
			asset_id,
			125u128
		));
		assert_eq!(UnitsPerSecond::<Runtime>::get(asset_id), Some(125));
		// Update a non-exist asset should fail
		assert_noop!(
			AssetManager::update_asset_location(Origin::root(), 2, new_location.clone()),
			crate::Error::<Runtime>::UpdateNonExistAsset
		);
		assert_noop!(
			AssetManager::update_asset_metadata(Origin::root(), 2, new_metadata.clone()),
			crate::Error::<Runtime>::UpdateNonExistAsset
		);
		// Update an asset to an existing location will fail
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			source_location.clone(),
			asset_metadata.clone()
		));
		assert_noop!(
			AssetManager::update_asset_location(Origin::root(), 1, new_location),
			crate::Error::<Runtime>::LocationAlreadyExists
		);
	})
}

#[test]
fn mint_asset() {
	new_test_ext().execute_with(|| {
		// mint native asset
		let native_asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
		assert_ok!(
			<MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::mint(
				native_asset_id,
				&ALICE,
				1000_000
			)
		);

		// mint non-native asset
		let non_native_asset_id =
			<MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
		let asset_metadata = AssetRegistrarMetadata {
			name: b"Kusama".to_vec(),
			symbol: b"KSM".to_vec(),
			decimals: 12,
			min_balance: 1u128,
			evm_address: None,
			is_frozen: false,
			is_sufficient: true,
		};
		let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			source_location,
			asset_metadata
		));
		assert_ok!(
			<MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::mint(
				non_native_asset_id,
				&ALICE,
				1000_000
			)
		);
	});
}
