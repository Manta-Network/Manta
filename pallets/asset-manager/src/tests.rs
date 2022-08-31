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

//! unit tests for asset-manager

use crate::{
    self as asset_manager, AssetIdLocation, AssetIdMetadata, Error, LocationAssetId, UnitsPerSecond,
};
use asset_manager::mock::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{fungibles::InspectMetadata, Contains},
    WeakBoundedVec,
};
use manta_primitives::assets::{AssetConfig, AssetLocation, FungibleLedger};
use orml_traits::GetByKey;
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
        let asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, None, false, true);
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
    let para_id = 1;
    let asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, None, false, true);
    let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let new_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(Parachain(para_id), PalletInstance(PALLET_BALANCES_INDEX)),
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
        // relaychain has no para id.
        assert!(!crate::AllowedDestParaIds::<Runtime>::contains_key(para_id));
        counter += 1;
        // Register twice will fail
        assert_noop!(
            AssetManager::register_asset(Origin::root(), source_location, asset_metadata.clone()),
            Error::<Runtime>::LocationAlreadyExists
        );
        // Register a new asset
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            new_location.clone(),
            asset_metadata.clone()
        ));
        assert_eq!(AssetIdLocation::<Runtime>::get(counter), Some(new_location));
        // check para ids
        assert!(crate::AllowedDestParaIds::<Runtime>::contains_key(para_id));
    })
}

#[test]
fn update_asset() {
    let para_id = 1;
    let original_decimals = 12;
    let asset_metadata = create_asset_metadata(
        "Kusama",
        "KSM",
        original_decimals,
        1u128,
        None,
        false,
        false,
    );
    let mut new_metadata = asset_metadata.clone();
    let new_name = b"NotKusama".to_vec();
    let new_symbol = b"NotKSM".to_vec();
    let new_decimals = original_decimals + 1;
    new_metadata.name = new_name.clone();
    new_metadata.symbol = new_symbol.clone();
    new_metadata.decimals = new_decimals;
    let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let new_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(Parachain(para_id), PalletInstance(PALLET_BALANCES_INDEX)),
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
        // Cannot update asset 1. Will be reserved for the native asset.
        let native_asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
        assert_noop!(
            AssetManager::update_asset_metadata(
                Origin::root(),
                native_asset_id,
                new_metadata.clone(),
            ),
            Error::<Runtime>::CannotUpdateNativeAssetMetadata
        );
        assert_ok!(AssetManager::update_asset_metadata(
            Origin::root(),
            asset_id,
            new_metadata.clone(),
        ),);
        assert_eq!(Assets::name(&asset_id), new_name);
        assert_eq!(Assets::symbol(&asset_id), new_symbol);
        assert_eq!(Assets::decimals(&asset_id), new_decimals);
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
        let next_asset_id = asset_id + 1;
        // Update a non-exist asset should fail
        assert_noop!(
            AssetManager::update_asset_location(
                Origin::root(),
                next_asset_id,
                new_location.clone()
            ),
            Error::<Runtime>::UpdateNonExistAsset
        );
        assert_noop!(
            AssetManager::update_asset_metadata(
                Origin::root(),
                next_asset_id,
                new_metadata.clone()
            ),
            Error::<Runtime>::UpdateNonExistAsset
        );
        // Re-registering the original location and metadata should work,
        // as we modified the previous asset.
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            source_location.clone(),
            asset_metadata.clone()
        ));
        // But updating the asset to an existing location will fail.
        assert_noop!(
            AssetManager::update_asset_location(Origin::root(), next_asset_id, new_location),
            Error::<Runtime>::LocationAlreadyExists
        );

        // If the existing asset location has been changed para id, the old para id should be
        // deleted from `AllowedDestParaIds` and new one should be inserted.
        let new_para_id = para_id + 1;
        let new_location_2 = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
            1,
            X2(
                Parachain(new_para_id),
                PalletInstance(PALLET_BALANCES_INDEX),
            ),
        )));
        assert!(crate::AllowedDestParaIds::<Runtime>::contains_key(para_id));

        assert_ok!(AssetManager::update_asset_location(
            Origin::root(),
            asset_id,
            new_location_2,
        ));
        // Old para id should be deleted.
        assert!(!crate::AllowedDestParaIds::<Runtime>::contains_key(para_id));
        assert!(crate::AllowedDestParaIds::<Runtime>::contains_key(
            new_para_id
        ));
    })
}

#[test]
fn check_para_id_info_when_update_asset_location() {
    new_test_ext().execute_with(|| {
        let manta_para_id = 2015;
        let manta_asset_metadata =
            create_asset_metadata("Manta", "MANTA", 18, 1u128, None, false, false);
        let mut manta_native_location = AssetLocation(VersionedMultiLocation::V1(
            MultiLocation::new(1, X1(Parachain(manta_para_id))),
        ));

        // registering manta native asset should work.
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            manta_native_location,
            manta_asset_metadata
        ));
        let manta_asset_id = crate::NextAssetId::<Runtime>::get() - 1;
        // check para id
        assert!(crate::AllowedDestParaIds::<Runtime>::contains_key(
            manta_para_id
        ));
        assert_eq!(
            crate::AllowedDestParaIds::<Runtime>::get(manta_para_id),
            Some(1)
        );

        // create a non manta asset.
        let manta_non_native_asset_metadata =
            create_asset_metadata("Manta", "eMANTA", 18, 1u128, None, false, false);
        let mut manta_non_native_location =
            AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X2(
                    Parachain(manta_para_id),
                    GeneralKey(WeakBoundedVec::force_from(b"eMANTA".to_vec(), None)),
                ),
            )));
        // registering manta non native asset should work.
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            manta_non_native_location,
            manta_non_native_asset_metadata
        ));
        let manta_non_native_asset_id = crate::NextAssetId::<Runtime>::get() - 1;
        // ParaId=manta_para_id should have 2 assets.
        assert_eq!(
            crate::AllowedDestParaIds::<Runtime>::get(manta_para_id),
            Some(2)
        );

        // Update new para id for manta native location
        let new_para_id = manta_para_id + 1;
        manta_native_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
            1,
            X2(
                Parachain(new_para_id),
                GeneralKey(WeakBoundedVec::force_from(b"eMANTA".to_vec(), None)),
            ),
        )));
        assert_ok!(AssetManager::update_asset_location(
            Origin::root(),
            manta_asset_id,
            manta_native_location,
        ));
        // ParaId=manta_para_id should have 1 asset.
        assert_eq!(
            crate::AllowedDestParaIds::<Runtime>::get(manta_para_id),
            Some(1)
        );
        // ParaId=new_para_id should have 1 asset.
        assert_eq!(
            crate::AllowedDestParaIds::<Runtime>::get(new_para_id),
            Some(1)
        );

        // Update para id for manta_non_native_location
        let new_para_id_again = new_para_id + 1;
        manta_non_native_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
            1,
            X2(
                Parachain(new_para_id_again),
                GeneralKey(WeakBoundedVec::force_from(b"eMANTA".to_vec(), None)),
            ),
        )));
        assert_ok!(AssetManager::update_asset_location(
            Origin::root(),
            manta_non_native_asset_id,
            manta_non_native_location,
        ));
        // ParaId=manta_para_id should deleted.
        assert!(!crate::AllowedDestParaIds::<Runtime>::contains_key(
            manta_para_id
        ));
        // ParaId=new_para_id_again should have 1 asset.
        assert_eq!(
            crate::AllowedDestParaIds::<Runtime>::get(new_para_id_again),
            Some(1)
        );
        // ParaId=new_para_id should have 1 asset.
        assert_eq!(
            crate::AllowedDestParaIds::<Runtime>::get(new_para_id),
            Some(1)
        );
    });
}

#[test]
fn mint_asset() {
    new_test_ext().execute_with(|| {
        // mint native asset
        let native_asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
        assert_ok!(
            <MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::deposit_can_mint(
                native_asset_id,
                &ALICE,
                1_000_000
            )
        );

        // mint non-native asset
        let non_native_asset_id =
            <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
        let asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, None, false, true);
        let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            source_location,
            asset_metadata
        ));
        assert_ok!(
            <MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::deposit_can_mint(
                non_native_asset_id,
                &ALICE,
                1_000_000
            )
        );
    });
}

#[test]
fn filter_asset_location_should_work() {
    let kusama_asset_metadata =
        create_asset_metadata("Kusama", "KSM", 12, 1u128, None, false, false);
    let kusama_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));

    let para_id = 2015;
    let manta_asset_metadata =
        create_asset_metadata("Manta", "MANTA", 18, 1u128, None, false, false);
    let manta_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X1(Parachain(para_id)),
    )));
    new_test_ext().execute_with(|| {
        // Register relay chain native token
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            kusama_location.clone(),
            kusama_asset_metadata.clone()
        ));
        let kusama_asset_id = crate::NextAssetId::<Runtime>::get() - 1;
        assert_eq!(
            AssetIdLocation::<Runtime>::get(kusama_asset_id),
            Some(kusama_location.clone())
        );

        // Register manta para chain native token
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            manta_location.clone(),
            manta_asset_metadata.clone()
        ));

        let manta_asset_id = crate::NextAssetId::<Runtime>::get() - 1;
        assert_eq!(
            AssetIdLocation::<Runtime>::get(manta_asset_id),
            Some(manta_location.clone())
        );

        // correct location should work
        let relay_dest = MultiLocation {
            parents: 1,
            interior: X1(AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            }),
        };
        let para_dest = MultiLocation {
            parents: 1,
            interior: X2(
                Parachain(para_id),
                AccountId32 {
                    network: NetworkId::Any,
                    id: ALICE.into(),
                },
            ),
        };
        assert!(crate::Pallet::<Runtime>::contains(&para_dest));
        assert!(crate::Pallet::<Runtime>::contains(&relay_dest));

        // wrong location should be filtered
        let wrong_relay_dest = MultiLocation {
            parents: 1,
            interior: Here,
        };
        let wrong_para_dest = MultiLocation {
            parents: 1,
            interior: X2(
                Parachain(para_id + 1),
                AccountId32 {
                    network: NetworkId::Any,
                    id: ALICE.into(),
                },
            ),
        };
        assert!(!crate::Pallet::<Runtime>::contains(&wrong_relay_dest));
        assert!(!crate::Pallet::<Runtime>::contains(&wrong_para_dest));

        // AccountKey20 based location should work
        let eve = [1u8; 20]; // evm based account
        let para_dest_with_evm_account = MultiLocation {
            parents: 1,
            interior: X2(
                Parachain(para_id),
                AccountKey20 {
                    network: NetworkId::Any,
                    key: eve,
                },
            ),
        };
        assert!(crate::Pallet::<Runtime>::contains(
            &para_dest_with_evm_account
        ));
    })
}

#[test]
fn set_min_xcm_fee_should_work() {
    let manta_asset_metadata =
        create_asset_metadata("Manta", "MANTA", 18, 1u128, None, false, false);
    let manta_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(
            Parachain(2015),
            GeneralKey(WeakBoundedVec::force_from(b"MANTA".to_vec(), None)),
        ),
    )));
    new_test_ext().execute_with(|| {
        // Register a non native token.
        assert_ok!(AssetManager::register_asset(
            Origin::root(),
            manta_location.clone(),
            manta_asset_metadata.clone()
        ));

        let manta_asset_id = crate::NextAssetId::<Runtime>::get() - 1;
        assert_eq!(
            AssetIdLocation::<Runtime>::get(manta_asset_id),
            Some(manta_location.clone())
        );

        let min_xcm_fee = 100;
        // normal account cannot set min xcm fee.
        assert_noop!(
            AssetManager::set_min_xcm_fee(
                Origin::signed([2u8; 32].into()),
                manta_location.clone(),
                min_xcm_fee,
            ),
            BadOrigin
        );

        // only sudo can set it.
        assert_ok!(AssetManager::set_min_xcm_fee(
            Origin::root(),
            manta_location.clone(),
            min_xcm_fee,
        ));
        assert_eq!(
            crate::MinXcmFee::<Runtime>::get(&manta_location),
            Some(min_xcm_fee)
        );

        // u128::MAX will be returned if min-xcm-fee is not set,
        // that means your crosschain transaction will fail due to no one can pay u128::MAX.
        let calamari_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
            1,
            X2(
                Parachain(2084),
                GeneralKey(WeakBoundedVec::force_from(b"KMA".to_vec(), None)),
            ),
        )));

        assert_eq!(
            crate::Pallet::<Runtime>::get(
                &Into::<Option<MultiLocation>>::into(calamari_location).unwrap()
            ),
            None
        );
    })
}
