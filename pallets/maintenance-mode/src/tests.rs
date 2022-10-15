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
        events, mock_events, Assets, Call as OuterCall, ExtBuilder, MaintenanceMode,
        Origin, Test, ALICE, BOB,
    },
    Call, Error, Event, ExecutiveHooks,
};
use cumulus_primitives_core::{DmpMessageHandler, ParaId};
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
fn enter_sibling_hack_mode_none_asset() {
    // TODO: affected_assets is none, should disable all asset belong to parachain.
    ExtBuilder::default().build().execute_with(|| {
        assert!(!Pallet::<Test>::hacked_sibling_id(&ParaId::new(1000)));

        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: ParaId::new(1000),
            affected_assets: None,
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));

        assert!(Pallet::<Test>::hacked_sibling_id(&ParaId::new(1000)));
        assert!(!Pallet::<Test>::hacked_sibling_id(&ParaId::new(2000)));

        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: ParaId::new(1000),
            affected_assets: None,
        }
        .into();
        assert_noop!(
            call.dispatch(Origin::root()),
            Error::<Test>::AlreadyInSiblingHackMode
        );
    });
}

#[test]
fn enter_sibling_hack_mode_some_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id: AssetId = 0;
        let sender: AccountId = ALICE;
        let receiver: AccountId = BOB;
        assert_ok!(Assets::force_create(
            Origin::root(),
            asset_id,
            sender.clone(), // owner
            true,
            1
        ));
        assert_ok!(Assets::mint(Origin::signed(sender.clone()), asset_id, sender.clone(), 100));
        assert_eq!(Assets::balance(asset_id, sender.clone()), 100);

        let call: OuterCall = Call::enter_sibling_hack_mode {
            hacked_chain_id: ParaId::new(1000),
            affected_assets: Some(vec![asset_id]),
        }
        .into();
        assert_ok!(call.dispatch(Origin::root()));
        // assert_ok!(call.dispatch(Origin::signed(ALICE)));
        // assert_ok!(call.dispatch(Origin::signed(BOB)));

        assert_noop!(
            Assets::transfer(Origin::signed(sender), asset_id, receiver, 50),
            pallet_assets::Error::<Test>::Frozen
        );
    });
}
