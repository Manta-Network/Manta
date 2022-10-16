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

//! Unit tests for the maintenance mode pallet.

use super::*;
use crate::{
    mock::{
        events, mock_events, AssetOwner, Assets, Balances, Call as OuterCall, ExtBuilder,
        MaintenanceMode, Origin, Test, ALICE, BOB,
    },
    Call, Error, Event, ExecutiveHooks,
};
use cumulus_primitives_core::DmpMessageHandler;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::Dispatchable,
    traits::{OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade},
};
use manta_primitives::types::{AccountId, AssetId};

#[test]
fn can_remark_during_normal_operation() {
    ExtBuilder::default().build().execute_with(|| {
        let call: OuterCall = frame_system::Call::remark { remark: vec![] }.into();
        assert_ok!(call.dispatch(Origin::signed(ALICE)));
    })
}

#[test]
fn cannot_remark_during_maintenance_mode() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            let call: OuterCall = frame_system::Call::remark { remark: vec![] }.into();
            assert_noop!(
                call.dispatch(Origin::signed(ALICE)),
                frame_system::Error::<Test>::CallFiltered
            );
        })
}

#[test]
fn can_balances_during_maintenance_mode() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .with_balances(vec![(BOB, 1000)])
        .build()
        .execute_with(|| {
            let call: OuterCall = pallet_balances::Call::transfer {
                dest: ALICE,
                value: 100,
            }
            .into();
            assert_ok!(call.dispatch(Origin::signed(BOB)));
            assert_eq!(900, Balances::free_balance(BOB));
            assert_eq!(100, Balances::free_balance(ALICE));
        })
}

#[test]
fn can_enter_maintenance_mode() {
    ExtBuilder::default().build().execute_with(|| {
        let call: OuterCall = Call::enter_maintenance_mode {}.into();
        assert_ok!(call.dispatch(Origin::root()));

        assert_eq!(events(), vec![Event::EnteredMaintenanceMode,]);
    })
}

#[test]
fn cannot_enter_maintenance_mode_from_wrong_origin() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            let call: OuterCall = Call::enter_maintenance_mode {}.into();
            assert_noop!(
                call.dispatch(Origin::signed(ALICE)),
                frame_system::Error::<Test>::CallFiltered
            );
        })
}

#[test]
fn cannot_enter_maintenance_mode_when_already_in_it() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            let call: OuterCall = Call::enter_maintenance_mode {}.into();
            assert_noop!(
                call.dispatch(Origin::root()),
                Error::<Test>::AlreadyInMaintenanceMode
            );
        })
}

#[test]
fn can_resume_normal_operation() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            let call: OuterCall = Call::resume_normal_operation {}.into();
            assert_ok!(call.dispatch(Origin::root()));

            assert_eq!(events(), vec![Event::NormalOperationResumed,]);
        })
}

#[test]
fn cannot_resume_normal_operation_from_wrong_origin() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            let call: OuterCall = Call::resume_normal_operation {}.into();
            assert_noop!(
                call.dispatch(Origin::signed(ALICE)),
                frame_system::Error::<Test>::CallFiltered
            );
        })
}

#[test]
fn cannot_resume_normal_operation_while_already_operating_normally() {
    ExtBuilder::default().build().execute_with(|| {
        let call: OuterCall = Call::resume_normal_operation {}.into();
        assert_noop!(
            call.dispatch(Origin::root()),
            Error::<Test>::NotInMaintenanceMode
        );
    })
}

#[test]
fn normal_dmp_in_non_maintenance() {
    ExtBuilder::default()
        .with_maintenance_mode(false)
        .build()
        .execute_with(|| {
            assert_eq!(
                MaintenanceMode::handle_dmp_messages(vec![].into_iter(), 1),
                0
            );
        })
}

#[test]
fn maintenance_dmp_in_maintenance() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            assert_eq!(
                MaintenanceMode::handle_dmp_messages(vec![].into_iter(), 1),
                1
            );
        })
}

#[test]
fn normal_hooks_in_non_maintenance() {
    ExtBuilder::default()
        .with_maintenance_mode(false)
        .build()
        .execute_with(|| {
            assert_eq!(ExecutiveHooks::<Test>::on_idle(0, 0), 0);
            assert_eq!(ExecutiveHooks::<Test>::on_initialize(0), 0);
            assert_eq!(ExecutiveHooks::<Test>::on_runtime_upgrade(), 0);
            ExecutiveHooks::<Test>::on_finalize(0);
            ExecutiveHooks::<Test>::offchain_worker(0);

            assert_eq!(
                mock_events(),
                [
                    crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnIdle,
                    crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnInitialize,
                    crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnRuntimeUpgrade,
                    crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnFinalize,
                    crate::mock::mock_pallet_maintenance_hooks::Event::NormalOffchainWorker
                ]
            );
        })
}

#[test]
fn maintenance_hooks_in_maintenance() {
    ExtBuilder::default()
        .with_maintenance_mode(true)
        .build()
        .execute_with(|| {
            assert_eq!(ExecutiveHooks::<Test>::on_idle(0, 0), 1);
            assert_eq!(ExecutiveHooks::<Test>::on_initialize(0), 1);
            assert_eq!(ExecutiveHooks::<Test>::on_runtime_upgrade(), 1);

            ExecutiveHooks::<Test>::on_finalize(0);
            ExecutiveHooks::<Test>::offchain_worker(0);
            assert_eq!(
                mock_events(),
                [
                    crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnIdle,
                    crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnInitialize,
                    crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnRuntimeUpgrade,
                    crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnFinalize,
                    crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOffchainWorker
                ]
            );
        })
}

#[test]
fn sibling_enter_maintenance_and_resume_normal_works() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id: AssetId = 0;
        assert_ok!(Assets::force_create(
            Origin::root(),
            asset_id,
            AssetOwner::get(),
            true,
            1
        ));
        let empty: Vec<AssetId> = vec![];

        assert_eq!(Pallet::<Test>::hacked_sibling_id(&1000), empty);
        assert_eq!(Pallet::<Test>::hacked_sibling_id(&2000), empty);

        // error case: hacked chain:2000 dont have asset registered, enter failed
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 2000,
            affected_assets: vec![0],
        }
        .into();
        assert_noop!(
            call.dispatch(Origin::root()),
            Error::<Test>::NoAssetRegistForParachain
        );

        // error case: hacked chain:2000 not enter before, resume failed
        let call: OuterCall = Call::resume_sibling_normal_mode {
            normal_chain_id: 2000,
            affected_assets: vec![0],
        }
        .into();
        assert_noop!(call.dispatch(Origin::root()), Error::<Test>::AssetNotMarkedAsHack);

        // ok case: hacked chain has registered asset, enter success
        let affected_assets: Vec<AssetId> = vec![0];
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: affected_assets.clone(),
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));
        assert_eq!(
            events(),
            vec![Event::EnteredSiblingHackMode {
                id: 1000,
                affected_assets: affected_assets.clone()
            }]
        );
        assert_eq!(Pallet::<Test>::hacked_sibling_id(&1000), affected_assets.clone());

        // error case: duplicate enter with existed asset failed
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: affected_assets.clone(),
        }
        .into();
        assert_noop!(
            call.dispatch(Origin::root()),
            Error::<Test>::AssetAlreadyMarkedAsHack
        );

        // ok case: hacked chain enter before, resume success
        let call: OuterCall = Call::resume_sibling_normal_mode {
            normal_chain_id: 1000,
            affected_assets: affected_assets.clone(),
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));
        assert_eq!(
            events(),
            vec![Event::ResumedSiblingNormalMode {
                id: 1000,
                affected_assets: affected_assets.clone()
            }]
        );
        assert_eq!(Pallet::<Test>::hacked_sibling_id(&1000), empty);

        // error case: duplicate resume failed because asset not exist anymore
        let call: OuterCall = Call::resume_sibling_normal_mode {
            normal_chain_id: 1000,
            affected_assets: affected_assets.clone()
        }
        .into();
        assert_noop!(call.dispatch(Origin::root()), Error::<Test>::AssetNotMarkedAsHack);

        for i in 1..5 {
            assert_ok!(Assets::force_create(
                Origin::root(),
                i,
                AssetOwner::get(),
                true,
                1
            ));
        }
        // ok case: more than one asset marked as hack mode
        let affected_assets: Vec<AssetId> = vec![0, 1];
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: affected_assets.clone(),
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));
        assert_eq!(
            events(),
            vec![Event::EnteredSiblingHackMode {
                id: 1000,
                affected_assets: affected_assets.clone()
            }]
        );
        assert_eq!(Pallet::<Test>::hacked_sibling_id(&1000), affected_assets.clone());

        // error case: duplicate enter with existed asset failed
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: affected_assets.clone(),
        }
            .into();
        assert_noop!(
            call.dispatch(Origin::root()),
            Error::<Test>::AssetAlreadyMarkedAsHack
        );

        // error case: duplicate enter with non-existed asset but not in asset manager failed
        let affected_assets2: Vec<AssetId> = vec![2];
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: affected_assets2.clone(),
        }
            .into();
        assert_noop!(
            call.dispatch(Origin::root()),
            Error::<Test>::NoAssetRegistForParachain
        );
    });
}

#[test]
fn enter_sibling_hack_mode_asset_owner_not_matched() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id: AssetId = 0;
        let receiver: AccountId = BOB;

        // `AssetsFreezer` use `ALICE` as asset owner in mock.
        // But gere we use `Bob` as asset owner, so freeze asset will failed.
        assert_ok!(Assets::force_create(Origin::root(), asset_id, BOB, true, 1));
        assert_ok!(Assets::mint(
            Origin::signed(BOB),
            asset_id,
            receiver.clone(),
            100
        ));
        assert_eq!(Assets::balance(asset_id, receiver.clone()), 100);

        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: vec![asset_id],
        }
        .into();
        assert_noop!(
            call.dispatch(Origin::root()),
            pallet_assets::Error::<Test>::NoPermission
        );
    });
}

#[test]
fn enter_resume_sibling_hack_mode_some_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id: AssetId = 0;
        let receiver1: AccountId = BOB;
        let receiver2: AccountId = ALICE;

        // create asset
        assert_ok!(Assets::force_create(
            Origin::root(),
            asset_id,
            AssetOwner::get(),
            true,
            1
        ));
        assert_ok!(Assets::mint(
            Origin::signed(AssetOwner::get()),
            asset_id,
            receiver1.clone(),
            100
        ));
        assert_eq!(Assets::balance(asset_id, receiver1.clone()), 100);

        // enter sibling hack mode will freeze parachain asset
        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: 1000,
            affected_assets: vec![asset_id],
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));

        assert_noop!(
            Assets::transfer(
                Origin::signed(receiver1.clone()),
                asset_id,
                receiver2.clone(),
                50
            ),
            pallet_assets::Error::<Test>::Frozen
        );

        // resume sibling normal mode will unfreeze parachain asset
        let call: OuterCall = Call::resume_sibling_normal_mode {
            normal_chain_id: 1000,
            affected_assets: vec![asset_id],
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));
        assert_ok!(Assets::transfer(
            Origin::signed(receiver1.clone()),
            asset_id.clone(),
            receiver2.clone(),
            50
        ));
        assert_eq!(Assets::balance(asset_id, receiver1), 50);
        assert_eq!(Assets::balance(asset_id, receiver2), 50);
    });
}
