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

//! unit tests for asset-manager

use crate::{
    mock::*, AssetIdLocation, AssetIdMetadata, AssetIdPairToLp, Error, LocationAssetId,
    LpToAssetIdPair, NextAssetId, UnitsPerSecond,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{fungibles::InspectMetadata, Contains},
    WeakBoundedVec,
};
use manta_primitives::{
    assets::{AssetConfig, AssetLocation, AssetRegistryMetadata, FungibleLedger},
    types::Balance,
};
use orml_traits::GetByKey;
use sp_core::Get;
use sp_runtime::{traits::BadOrigin, ArithmeticError};
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
        let asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, false, true);
        let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
        assert_noop!(
            AssetManager::register_asset(
                RuntimeOrigin::signed([0u8; 32].into()),
                source_location.clone(),
                asset_metadata.clone()
            ),
            BadOrigin
        );
        assert_noop!(
            AssetManager::update_asset_location(
                RuntimeOrigin::signed([2u8; 32].into()),
                0,
                source_location
            ),
            BadOrigin
        );
        assert_noop!(
            AssetManager::update_asset_metadata(
                RuntimeOrigin::signed([3u8; 32].into()),
                0,
                asset_metadata.metadata
            ),
            BadOrigin
        );
        assert_noop!(
            AssetManager::set_units_per_second(RuntimeOrigin::signed([4u8; 32].into()), 0, 0),
            BadOrigin
        );
    })
}

#[test]
fn register_asset_should_work() {
    let para_id = 1;
    let asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, false, true);
    let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let new_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(Parachain(para_id), PalletInstance(PALLET_BALANCES_INDEX)),
    )));
    new_test_ext().execute_with(|| {
        let mut counter = <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
        // Register relay chain native token
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
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
            AssetManager::register_asset(
                RuntimeOrigin::root(),
                source_location,
                asset_metadata.clone()
            ),
            Error::<Runtime>::LocationAlreadyExists
        );
        // Register a new asset
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
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
    let asset_metadata =
        create_asset_metadata("Kusama", "KSM", original_decimals, 1u128, false, false);
    let mut new_metadata = asset_metadata.clone();
    let new_name = b"NotKusama".to_vec();
    let new_symbol = b"NotKSM".to_vec();
    let new_decimals = original_decimals + 1;
    new_metadata.metadata.name = new_name.clone();
    new_metadata.metadata.symbol = new_symbol.clone();
    new_metadata.metadata.decimals = new_decimals;
    let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let new_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(Parachain(para_id), PalletInstance(PALLET_BALANCES_INDEX)),
    )));
    new_test_ext().execute_with(|| {
        // Register relay chain native token
        let asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
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
                RuntimeOrigin::root(),
                native_asset_id,
                new_metadata.metadata.clone(),
            ),
            Error::<Runtime>::CannotUpdateNativeAssetMetadata
        );
        assert_ok!(AssetManager::update_asset_metadata(
            RuntimeOrigin::root(),
            asset_id,
            new_metadata.metadata.clone(),
        ),);
        assert_eq!(Assets::name(&asset_id), new_name);
        assert_eq!(Assets::symbol(&asset_id), new_symbol);
        assert_eq!(Assets::decimals(&asset_id), new_decimals);
        // Update the asset location
        assert_ok!(AssetManager::update_asset_location(
            RuntimeOrigin::root(),
            asset_id,
            new_location.clone()
        ));
        // Update asset units per seconds
        assert_ok!(AssetManager::set_units_per_second(
            RuntimeOrigin::root(),
            asset_id,
            125u128
        ));
        assert_eq!(UnitsPerSecond::<Runtime>::get(asset_id), Some(125));
        let next_asset_id = asset_id + 1;
        // Update a non-exist asset should fail
        assert_noop!(
            AssetManager::update_asset_location(
                RuntimeOrigin::root(),
                next_asset_id,
                new_location.clone()
            ),
            Error::<Runtime>::UpdateNonExistentAsset
        );
        assert_noop!(
            AssetManager::update_asset_metadata(
                RuntimeOrigin::root(),
                next_asset_id,
                new_metadata.metadata.clone()
            ),
            Error::<Runtime>::UpdateNonExistentAsset
        );
        // Re-registering the original location and metadata should work,
        // as we modified the previous asset.
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
            source_location.clone(),
            asset_metadata.clone()
        ));
        // But updating the asset to an existing location will fail.
        assert_noop!(
            AssetManager::update_asset_location(RuntimeOrigin::root(), next_asset_id, new_location),
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
            RuntimeOrigin::root(),
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
        let manta_asset_metadata = create_asset_metadata("Manta", "MANTA", 18, 1u128, false, false);
        let mut manta_native_location = AssetLocation(VersionedMultiLocation::V1(
            MultiLocation::new(1, X1(Parachain(manta_para_id))),
        ));

        // registering manta native asset should work.
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
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
            create_asset_metadata("Manta", "eMANTA", 18, 1u128, false, false);
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
            RuntimeOrigin::root(),
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
            RuntimeOrigin::root(),
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
            RuntimeOrigin::root(),
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
            <MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::deposit_minting(
                native_asset_id,
                &ALICE,
                1_000_000
            )
        );

        // mint non-native asset
        let non_native_asset_id =
            <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get();
        let asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, false, true);
        let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
            source_location,
            asset_metadata
        ));
        assert_ok!(
            <MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::deposit_minting(
                non_native_asset_id,
                &ALICE,
                1_000_000
            )
        );
    });
}

#[test]
fn filter_asset_location_should_work() {
    let kusama_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1u128, false, false);
    let kusama_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));

    let para_id = 2015;
    let manta_asset_metadata = create_asset_metadata("Manta", "MANTA", 18, 1u128, false, false);
    let manta_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X1(Parachain(para_id)),
    )));
    new_test_ext().execute_with(|| {
        // Register relay chain native token
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
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
            RuntimeOrigin::root(),
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
    let manta_asset_metadata = create_asset_metadata("Manta", "MANTA", 18, 1u128, false, false);
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
            RuntimeOrigin::root(),
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
                RuntimeOrigin::signed([2u8; 32].into()),
                manta_location.clone(),
                min_xcm_fee,
            ),
            BadOrigin
        );

        // only sudo can set it.
        assert_ok!(AssetManager::set_min_xcm_fee(
            RuntimeOrigin::root(),
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

fn create_asset_and_location(token: &str) -> (AssetRegistryMetadata<Balance>, AssetLocation) {
    let manta_asset_metadata = create_asset_metadata(token, token, 12, 1u128, false, false);
    let manta_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(
            Parachain(2015),
            GeneralKey(WeakBoundedVec::force_from(token.as_bytes().to_vec(), None)),
        ),
    )));
    (manta_asset_metadata, manta_location)
}

#[test]
fn register_lp_asset_should_work() {
    new_test_ext().execute_with(|| {
        // Register first non native token.
        let (manta_asset_metadata8, manta_location8) = create_asset_and_location("Asset8");
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
            manta_location8.clone(),
            manta_asset_metadata8.clone()
        ));
        assert_eq!(AssetIdLocation::<Runtime>::get(8), Some(manta_location8));

        assert_noop!(
            AssetManager::register_lp_asset(
                RuntimeOrigin::root(),
                8,
                8,
                manta_asset_metadata8.clone()
            ),
            Error::<Runtime>::AssetIdNotDifferent
        );
        assert_noop!(
            AssetManager::register_lp_asset(RuntimeOrigin::root(), 8, 9, manta_asset_metadata8),
            Error::<Runtime>::AssetIdNotExist
        );

        let (manta_asset_metadata9, manta_location9) = create_asset_and_location("Asset9");
        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
            manta_location9.clone(),
            manta_asset_metadata9
        ));
        assert_eq!(AssetIdLocation::<Runtime>::get(9), Some(manta_location9));

        let manta_asset_metadata10 = create_asset_metadata("LP10", "LP10", 12, 1u128, false, false);
        assert_ok!(AssetManager::register_lp_asset(
            RuntimeOrigin::root(),
            8,
            9,
            manta_asset_metadata10.clone()
        ),);
        assert_eq!(AssetIdPairToLp::<Runtime>::get((8, 9)), Some(10));
        assert_eq!(LpToAssetIdPair::<Runtime>::get(10), Some((8, 9)));
        assert_eq!(
            AssetIdMetadata::<Runtime>::get(10),
            Some(manta_asset_metadata10)
        );

        let (manta_asset_metadata11, manta_location11) = create_asset_and_location("Asset11");
        assert_noop!(
            AssetManager::register_lp_asset(
                RuntimeOrigin::root(),
                8,
                9,
                manta_asset_metadata11.clone()
            ),
            Error::<Runtime>::AssetAlreadyRegistered
        );

        assert_ok!(AssetManager::register_asset(
            RuntimeOrigin::root(),
            manta_location11,
            manta_asset_metadata11
        ));
        let manta_asset_metadata12 = create_asset_metadata("LP12", "LP12", 12, 1u128, false, false);
        // sort asset by order
        assert_ok!(AssetManager::register_lp_asset(
            RuntimeOrigin::root(),
            11,
            8,
            manta_asset_metadata12
        ),);
        assert_eq!(AssetIdPairToLp::<Runtime>::get((8, 11)), Some(12));
        assert_eq!(AssetIdPairToLp::<Runtime>::get((11, 8)), None);
        assert_eq!(LpToAssetIdPair::<Runtime>::get(12), Some((8, 11)));

        let manta_asset_metadata13 = create_asset_metadata("LP13", "LP13", 12, 1u128, false, false);
        assert_noop!(
            AssetManager::register_lp_asset(RuntimeOrigin::root(), 12, 8, manta_asset_metadata13),
            Error::<Runtime>::AssetIdNotExist
        );
    });
}

#[test]
fn permissionless_edge_cases() {
    new_test_ext().execute_with(|| {
        let native_asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
        assert_ok!(
            <MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::deposit_minting(
                native_asset_id,
                &ALICE,
                1_000_000
            )
        );

        // cannot create asset with zero supply
        assert_noop!(
            AssetManager::permissionless_register_asset(
                RuntimeOrigin::signed(ALICE),
                "dog token".as_bytes().to_vec().try_into().unwrap(),
                "dog".as_bytes().to_vec().try_into().unwrap(),
                0_u8,
                0_u128,
            ),
            Error::<Runtime>::TotalSupplyTooLow
        );

        // cannot create asset with zero supply
        assert_noop!(
            AssetManager::permissionless_register_asset(
                RuntimeOrigin::signed(ALICE),
                "dog token".as_bytes().to_vec().try_into().unwrap(),
                "dog".as_bytes().to_vec().try_into().unwrap(),
                1_u8,
                0_u128,
            ),
            Error::<Runtime>::TotalSupplyTooLow
        );

        // cannot create asset with zero decimals
        assert_noop!(
            AssetManager::permissionless_register_asset(
                RuntimeOrigin::signed(ALICE),
                "dog token".as_bytes().to_vec().try_into().unwrap(),
                "dog".as_bytes().to_vec().try_into().unwrap(),
                0_u8,
                1_u128,
            ),
            Error::<Runtime>::DecimalIsZero
        );

        // overflows when decimal is too high
        assert_noop!(
            AssetManager::permissionless_register_asset(
                RuntimeOrigin::signed(ALICE),
                "dog token".as_bytes().to_vec().try_into().unwrap(),
                "dog".as_bytes().to_vec().try_into().unwrap(),
                u8::MAX,
                1_000_000_000_000_000_000_u128,
            ),
            ArithmeticError::Overflow
        );

        // cannot create asset with too small total supply
        assert_noop!(
            AssetManager::permissionless_register_asset(
                RuntimeOrigin::signed(ALICE),
                "dog token".as_bytes().to_vec().try_into().unwrap(),
                "dog".as_bytes().to_vec().try_into().unwrap(),
                12,
                1,
            ),
            Error::<Runtime>::TotalSupplyTooLow,
        );

        // really small total supply is one
        let asset_id: u128 = <Runtime as crate::pallet::Config>::PermissionlessStartId::get();
        assert_ok!(AssetManager::permissionless_register_asset(
            RuntimeOrigin::signed(ALICE),
            "dog token".as_bytes().to_vec().try_into().unwrap(),
            "dog".as_bytes().to_vec().try_into().unwrap(),
            1,
            1_000,
        ));

        let metadata = AssetIdMetadata::<Runtime>::get(asset_id).unwrap();
        assert_eq!(metadata.min_balance, 1);
    });
}

#[test]
fn permissionless_register_asset_works() {
    new_test_ext().execute_with(|| {
        let amount = 1_000_000;
        let registry_cost: Balance =
            <Runtime as crate::pallet::Config>::PermissionlessAssetRegistryCost::get();
        let native_asset_id = <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get();
        assert_ok!(
            <MantaAssetConfig as AssetConfig<Runtime>>::FungibleLedger::deposit_minting(
                native_asset_id,
                &ALICE,
                amount,
            )
        );

        let asset_id = <Runtime as crate::pallet::Config>::PermissionlessStartId::get();
        assert_ok!(AssetManager::permissionless_register_asset(
            RuntimeOrigin::signed(ALICE),
            "dog token".as_bytes().to_vec().try_into().unwrap(),
            "dog".as_bytes().to_vec().try_into().unwrap(),
            12,
            1_000_000_000_000_000,
        ));

        // asset created gives alice the token
        assert_eq!(Assets::balance(asset_id, &ALICE), 1_000_000_000_000_000);
        // cost native token
        assert_eq!(Balances::free_balance(&ALICE), amount - registry_cost);

        let metadata = AssetIdMetadata::<Runtime>::get(asset_id).unwrap();
        assert!(metadata.is_sufficient);
        assert_eq!(metadata.min_balance, 100_000);
        assert!(!metadata.metadata.is_frozen);
        assert_eq!(metadata.metadata.decimals, 12);
        assert_eq!(metadata.metadata.name, "dog token".as_bytes().to_vec());
        assert_eq!(metadata.metadata.symbol, "dog".as_bytes().to_vec());

        // Max balance works
        assert_ok!(AssetManager::permissionless_register_asset(
            RuntimeOrigin::signed(ALICE),
            "dog token".as_bytes().to_vec().try_into().unwrap(),
            "dog".as_bytes().to_vec().try_into().unwrap(),
            6,
            u128::MAX,
        ));
    });
}

#[test]
fn counters_test() {
    new_test_ext().execute_with(|| {
        let last_id: Balance = <Runtime as crate::pallet::Config>::PermissionlessStartId::get();
        let last_id_minus_one = last_id - 1;
        let last_id_plus_one = last_id + 1;
        NextAssetId::<Runtime>::put(last_id_minus_one);

        assert_eq!(
            AssetManager::next_asset_id_and_increment().unwrap(),
            last_id_minus_one
        );
        assert_noop!(
            AssetManager::next_asset_id_and_increment(),
            Error::<Runtime>::AssetIdOverflow
        );

        assert_eq!(
            AssetManager::next_permissionless_asset_id_and_increment().unwrap(),
            last_id
        );
        assert_eq!(
            AssetManager::next_permissionless_asset_id_and_increment().unwrap(),
            last_id_plus_one
        );
    });
}
