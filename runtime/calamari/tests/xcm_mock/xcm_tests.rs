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

//! Simulation Tests for XCM

#![cfg(test)]

use codec::Encode;
use frame_support::{
    assert_err, assert_noop, assert_ok, weights::constants::WEIGHT_PER_SECOND, WeakBoundedVec,
};
use manta_primitives::assets::AssetLocation;
use xcm::{latest::prelude::*, v2::Response, VersionedMultiLocation, WrapVersion};
use xcm_executor::traits::{Convert, WeightBounds};
use xcm_simulator::TestExt;

use super::{
    super::*,
    parachain::{
        create_asset_location, create_asset_metadata, register_assets_on_parachain, AssetManager,
        ParaTokenPerSecond, XcmExecutorConfig as ParaXcmExecutorConfig, PALLET_ASSET_INDEX,
    },
    *,
};

// `reserved_transfer_asset` contains the following 4 instructions
//  1. ReserveAssetDeposited(assets.clone()),
//  2. ClearOrigin,
//  3. BuyExecution { fees, weight_limit: Limited(0) },
//  4. DepositAsset { assets: Wild(All), max_assets, beneficiary },
//  each instruction's weight is 1_000, thus, the total weight is 4_000
const RESERVE_TRANSFER_WEIGHT_ON_RELAY: u64 = 4_000;

fn calculate_fee(units_per_seconds: u128, weight: u64) -> u128 {
    units_per_seconds * (weight as u128) / (WEIGHT_PER_SECOND as u128)
}

fn self_reserve_xtokens_weight_on_receiver() -> u64 {
    let mut msg = self_reserve_xcm_message_receiver_side();
    <ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap()
}

fn non_self_reserve_xtokens_weight_on_receiver() -> u64 {
    let mut msg = to_reserve_xcm_message_receiver_side();
    <ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap()
}

// Helper function for forming buy execution message
fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
    BuyExecution {
        fees: fees.into(),
        weight_limit: Unlimited,
    }
}

#[test]
fn dmp() {
    MockNet::reset();

    let remark = parachain::Call::System(
        frame_system::Call::<parachain::Runtime>::remark_with_event {
            remark: vec![1, 2, 3],
        },
    );
    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::send_xcm(
            Here,
            Parachain(1),
            Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: INITIAL_BALANCE as u64,
                call: remark.encode().into(),
            }]),
        ));
    });

    ParaA::execute_with(|| {
        use parachain::{Event, System};
        assert!(System::events()
            .iter()
            .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
    });
}

#[test]
fn dmp_transact_from_parent_should_pass_barrier() {
    MockNet::reset();

    let remark = parachain::Call::System(
        frame_system::Call::<parachain::Runtime>::remark_with_event {
            remark: vec![1, 2, 3],
        },
    );

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::send_xcm(
            Here,
            Parachain(1),
            Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: INITIAL_BALANCE as u64,
                call: remark.encode().into(),
            }]),
        ));
    });
    ParaA::execute_with(|| {
        use parachain::{Event, System};
        assert!(System::events()
            .iter()
            .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
    });
}

#[test]
fn ump() {
    MockNet::reset();

    let remark = relay_chain::Call::System(
        frame_system::Call::<relay_chain::Runtime>::remark_with_event {
            remark: vec![1, 2, 3],
        },
    );
    ParaA::execute_with(|| {
        assert_ok!(ParachainPalletXcm::send_xcm(
            Here,
            Parent,
            Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: INITIAL_BALANCE as u64,
                call: remark.encode().into(),
            }]),
        ));
    });

    Relay::execute_with(|| {
        use relay_chain::{Event, System};
        assert!(System::events()
            .iter()
            .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
    });
}

#[test]
fn xcmp_transact_from_sibling_parachain_blocked_by_barrier() {
    MockNet::reset();

    let remark = parachain::Call::System(
        frame_system::Call::<parachain::Runtime>::remark_with_event {
            remark: vec![1, 2, 3],
        },
    );
    ParaA::execute_with(|| {
        assert_ok!(ParachainPalletXcm::send_xcm(
            Here,
            (Parent, Parachain(2)),
            Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: INITIAL_BALANCE as u64,
                call: remark.encode().into(),
            }]),
        ));
    });

    // The `AllowUnpaidExecutionFrom<ParentLocation>` barrier implementation
    // only allows Transact instructions sent by the relay chain's governance
    ParaB::execute_with(|| {
        use parachain::{Event, System};
        assert!(!System::events()
            .iter()
            .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
    });
}

#[test]
fn reserve_transfer_relaychain_to_parachain_a_then_back() {
    MockNet::reset();

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let relay_asset_id = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(0u128),
        None,
    );

    let amount = 123;
    let weight_at_most = 40000;

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(PARA_A_ID)).into().into()),
            Box::new(
                X1(AccountId32 {
                    network: Any,
                    id: ALICE.into()
                })
                .into()
                .into()
            ),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            parachain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount
        );
    });

    ParaA::execute_with(|| {
        // free execution, full amount received
        assert_eq!(
            pallet_assets::Pallet::<parachain::Runtime>::balance(relay_asset_id, &ALICE),
            amount
        );
    });

    // Checking the balance of relay chain before sending token back
    let mut balance_before_sending = 0;
    Relay::execute_with(|| {
        balance_before_sending = RelayBalances::free_balance(&ALICE);
    });

    let dest = MultiLocation {
        parents: 1,
        interior: X1(AccountId32 {
            network: NetworkId::Any,
            id: ALICE.into(),
        }),
    };

    ParaA::execute_with(|| {
        // free execution, full amount received
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(relay_asset_id),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            weight_at_most
        ));
    });

    ParaA::execute_with(|| {
        // free execution, this will drain the parachain asset account
        assert_eq!(parachain::Assets::balance(relay_asset_id, &ALICE), 0);
    });

    Relay::execute_with(|| {
        // free execution, full amount received
        assert_eq!(
            RelayBalances::free_balance(&ALICE),
            balance_before_sending + amount
        );
    });
}

#[test]
fn send_para_a_native_asset_to_para_b() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = INITIAL_BALANCE;

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, false);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, false);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Transfer ParaA balance to B
    // Also tests that a sender can send all of their balance
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            ADVERTISED_DEST_WEIGHT
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        );
        assert!(!frame_system::Account::<parachain::Runtime>::contains_key(
            ALICE
        ));
    });

    // Make sure B received the token
    ParaB::execute_with(|| {
        // free execution, full amount received
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), amount);
    });
}

#[test]
fn send_para_a_native_asset_to_para_b_barriers_should_work() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 10000000000000u128;
    let units_per_sec = 125000000000;

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, false);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, false);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // AllowTopLevelPaidExecutionFrom<Everything> should fail because weight is not enough
    let weight = self_reserve_xtokens_weight_on_receiver() - 1;
    assert!(weight <= ADVERTISED_DEST_WEIGHT);
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            weight
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    // The `AllowTopLevelPaidExecutionFrom<Everything>` barrier implementation
    // should not let the transfer through
    ParaB::execute_with(|| {
        use parachain::{Event, System};
        assert!(System::events().iter().any(|r| matches!(
            r.event,
            Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
                message_hash: Some(_),
                error: xcm_simulator::XcmError::Barrier,
                weight: _
            })
        )));
    });

    // Make sure B didn't receive the token
    ParaB::execute_with(|| {
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });
}

#[test]
fn send_insufficient_asset_from_para_a_to_para_b() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 8888888u128;
    let units_per_second_at_b = 1_250_000u128;
    let fee_at_b = calculate_fee(
        units_per_second_at_b,
        self_reserve_xtokens_weight_on_receiver(),
    );

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, false);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, false);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_second_at_b),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second_at_b),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    assert!(amount >= fee_at_b);
    // Transfer ParaA balance to B
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            ADVERTISED_DEST_WEIGHT
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    ParaB::execute_with(|| {
        // The total supply should not include the paid fee,
        // because the XcmFeesAccount had 0 providers with is_sufficient set to false,
        // so the mint_into() operation for the refund amount failed.
        assert_eq!(
            parachain::Assets::total_supply(a_asset_id_on_b),
            amount - fee_at_b
        );
        assert_eq!(
            parachain::Assets::balance(a_asset_id_on_b, &ALICE),
            amount - fee_at_b
        );
    });

    // Setting the balance will in effect create the account
    // incrementing its providers counter to from 0 to 1
    ParaB::execute_with(|| {
        assert_ok!(pallet_balances::Pallet::<parachain::Runtime>::set_balance(
            parachain::Origin::root(),
            parachain::AssetManager::account_id(),
            1000000000000000,
            1000000000000000
        ));
    });

    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            ADVERTISED_DEST_WEIGHT
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount * 2
        )
    });

    ParaB::execute_with(|| {
        // This time we expect the total supply to be the full amount
        // as the refund will be deposited to the XcmFeesAccount
        assert_eq!(
            parachain::Assets::total_supply(a_asset_id_on_b),
            (amount - fee_at_b) + amount
        );
        assert_eq!(
            parachain::Assets::balance(a_asset_id_on_b, &ALICE),
            (amount - fee_at_b) * 2
        );
    });
}

#[test]
fn send_para_a_native_asset_to_para_b_must_fail_cases() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 1u128;
    let units_per_sec = 125000000000;

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, false);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, false);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // High amount should fail on the sender side
    let weight = self_reserve_xtokens_weight_on_receiver() * 100_000_000;
    ParaA::execute_with(|| {
        assert_err!(
            parachain::XTokens::transfer(
                parachain::Origin::signed(ALICE),
                parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                amount + INITIAL_BALANCE,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                weight
            ),
            orml_xtokens::Error::<parachain::Runtime>::XcmExecutionFailed
        );
        assert_eq!(parachain::Balances::free_balance(&ALICE), INITIAL_BALANCE);
    });

    // Low amount for the required weight results in TooExpensive error on the receiver side
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            weight
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    ParaB::execute_with(|| {
        use parachain::{Event, System};

        assert!(System::events().iter().any(|r| {
            matches!(
                r.event,
                Event::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
                    message_hash: Some(_),
                    error: xcm_simulator::XcmError::TooExpensive,
                    weight: _
                })
            )
        }));
    });

    // Make sure B didn't receive the token
    ParaB::execute_with(|| {
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });
}

#[test]
fn register_insufficient_with_zero_min_balance_should_fail() {
    MockNet::reset();

    let a_source_location = create_asset_location(1, PARA_A_ID);
    let a_asset_metadata_on_b =
        create_asset_metadata("ParaAToken", "ParaA", 18, 0, None, false, false);

    ParaB::execute_with(|| {
        assert_err!(
            AssetManager::register_asset(
                parachain::Origin::root(),
                a_source_location.clone(),
                a_asset_metadata_on_b.clone()
            ),
            pallet_asset_manager::Error::<parachain::Runtime>::ErrorCreatingAsset
        );
    });
}

#[test]
fn send_para_a_custom_asset_to_para_b() {
    let amount = 321;

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_a_doge_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X3(
            Parachain(PARA_A_ID),
            PalletInstance(PALLET_ASSET_INDEX),
            GeneralIndex(0_u128),
        ),
    )));

    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_a_doge_asset_metadata =
        create_asset_metadata("ParaADogeToken", "ParaADoge", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, false);

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        Some((ALICE, 1, true, false)),
    );
    let doge_currency_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_doge_location,
        &para_a_doge_asset_metadata,
        Some(0u128),
        Some((ALICE, 1, true, false)),
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let doge_currency_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_doge_location,
        &para_a_doge_asset_metadata,
        Some(0u128),
        None,
    );

    let alice_on_b = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    ParaA::execute_with(|| {
        // Force customized asset balance for Alice
        assert_ok!(parachain::Assets::mint(
            parachain::Origin::signed(ALICE),
            doge_currency_id_on_a,
            ALICE,
            INITIAL_BALANCE
        ));
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(doge_currency_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_b)),
            ADVERTISED_DEST_WEIGHT
        ));
        assert_eq!(
            parachain::Assets::balance(doge_currency_id_on_a, &ALICE),
            INITIAL_BALANCE - amount
        );
    });

    // Make sure B received the token
    ParaB::execute_with(|| {
        // free execution, full amount received
        assert_eq!(
            parachain::Assets::balance(doge_currency_id_on_b, &ALICE),
            amount
        );
    });
}

#[test]
fn send_para_a_native_asset_para_b_and_then_send_back() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 5000000u128;
    let weight = non_self_reserve_xtokens_weight_on_receiver();
    let fee_on_b_when_send_back = calculate_fee(ParaTokenPerSecond::get().1, weight);
    assert!(fee_on_b_when_send_back < amount);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );

    let alice_on_b = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_b)),
            weight
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    // Make sure B received the token
    ParaB::execute_with(|| {
        // free execution, full amount received
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), amount);
    });

    let alice_on_a = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_A_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Send wrapped a back to a
    ParaB::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_b),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_a)),
            weight
        ));
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });

    // make sure that a received the token
    ParaA::execute_with(|| {
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - fee_on_b_when_send_back
        )
    });
}

#[test]
fn send_para_a_native_asset_from_para_b_to_para_c() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);
    let para_c_source_location = create_asset_location(1, PARA_C_ID);

    let amount = 8888888u128;
    let weight = non_self_reserve_xtokens_weight_on_receiver();
    let fee_at_reserve = calculate_fee(ParaTokenPerSecond::get().1, weight);
    assert!(amount >= fee_at_reserve * 2_u128);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, false);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, false);
    let para_c_asset_metadata =
        create_asset_metadata("ParaCToken", "ParaC", 18, 1, None, false, false);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &para_c_source_location,
        &para_c_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaC>(
        &para_c_source_location,
        &para_c_asset_metadata,
        Some(0u128),
        None,
    );
    let a_asset_id_on_c = register_assets_on_parachain::<ParaC>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );

    // A send B some token
    let alice_on_b = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_b.clone())),
            weight
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    ParaB::execute_with(|| {
        // free execution, full amount received
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), amount);
    });

    // B send C para A asset
    let alice_on_c = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_C_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    ParaB::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_b),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_c)),
            weight,
        ));
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });

    // Make sure C received the token
    ParaC::execute_with(|| {
        // free execution, full amount received
        assert_eq!(
            parachain::Assets::balance(a_asset_id_on_c, &ALICE),
            amount - fee_at_reserve
        );
    });
}

#[test]
fn receive_relay_asset_with_trader_on_parachain() {
    MockNet::reset();

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    let amount = 666u128;
    // We charge 10^9 as units per second on ParaA
    let units_per_second = 1_000_000_000u128;
    let fee = calculate_fee(units_per_second, RESERVE_TRANSFER_WEIGHT_ON_RELAY);
    assert!(fee > 0);

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second),
        None,
    );
    let relay_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_second),
        None,
    );

    let dest: MultiLocation = AccountId32 {
        network: Any,
        id: ALICE.into(),
    }
    .into();

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(1)).into().into()),
            Box::new(VersionedMultiLocation::V1(dest)),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            relay_chain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount
        );
    });

    ParaA::execute_with(|| {
        // ALICE gets amount - fee
        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_a, &ALICE),
            amount - fee
        );
        // Fee sink gets fee
        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_a, AssetManager::account_id()),
            fee
        );
    });
}

#[test]
fn send_para_a_asset_to_para_b_with_trader_and_fee() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 222u128;
    let units_per_second = 1_250_000u128;
    let fee = calculate_fee(units_per_second, self_reserve_xtokens_weight_on_receiver());

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_second),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer_with_fee(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            fee,
            Box::new(VersionedMultiLocation::V1(dest)),
            ADVERTISED_DEST_WEIGHT,
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount - fee
        )
    });

    ParaB::execute_with(|| {
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), amount);
    });
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c_with_trader() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);
    let para_c_source_location = create_asset_location(1, PARA_C_ID);

    let mut amount = 8888888u128;
    let units_per_second = 1_250_000u128;
    let fee_at_b = calculate_fee(units_per_second, self_reserve_xtokens_weight_on_receiver());
    let fee_at_a = calculate_fee(
        ParaTokenPerSecond::get().1,
        non_self_reserve_xtokens_weight_on_receiver(),
    );

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);
    let para_c_asset_metadata =
        create_asset_metadata("ParaCToken", "ParaC", 18, 1, None, false, true);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &para_c_source_location,
        &para_c_asset_metadata,
        Some(units_per_second),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second),
        None,
    );

    let _ = register_assets_on_parachain::<ParaC>(
        &para_c_source_location,
        &para_c_asset_metadata,
        Some(units_per_second),
        None,
    );
    let a_asset_id_on_c = register_assets_on_parachain::<ParaC>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second),
        None,
    );

    // A send B some token
    let alice_on_b = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(2),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    assert!(amount >= fee_at_b);
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_b.clone())),
            ADVERTISED_DEST_WEIGHT
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    ParaB::execute_with(|| {
        assert_eq!(parachain::Assets::total_supply(a_asset_id_on_b), amount);
        amount -= fee_at_b;
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), amount);
    });

    // B send C para A asset
    let alice_on_c = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(3),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    assert!(amount >= fee_at_b + fee_at_a);
    ParaB::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_b),
            amount,
            Box::new(VersionedMultiLocation::V1(alice_on_c)),
            ADVERTISED_DEST_WEIGHT
        ));
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });

    // Make sure C received the token
    ParaC::execute_with(|| {
        amount = amount - fee_at_b - fee_at_a;
        assert_eq!(parachain::Assets::balance(a_asset_id_on_c, &ALICE), amount);
    });
}

#[test]
fn receive_relay_asset_on_parachain_with_insufficient_fee_payment_should_fail() {
    MockNet::reset();

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    let amount = 20u128;
    // We charge 2 x 10^10 as units per second on ParaA
    let units_per_second = 20_000_000_000u128;
    let fee = calculate_fee(units_per_second, RESERVE_TRANSFER_WEIGHT_ON_RELAY);
    assert!(fee > amount);

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second),
        None,
    );
    let relay_asset_id = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_second),
        None,
    );

    let dest: MultiLocation = AccountId32 {
        network: Any,
        id: ALICE.into(),
    }
    .into();

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(1)).into().into()),
            Box::new(VersionedMultiLocation::V1(dest)),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            relay_chain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount
        );
    });

    ParaA::execute_with(|| {
        // ALICE gets nothing
        assert_eq!(parachain::Assets::balance(relay_asset_id, &ALICE), 0);
        // Asset manager gets nothing, all balance stuck
        assert_eq!(
            parachain::Assets::balance(relay_asset_id, AssetManager::account_id()),
            0
        );
    });
}

#[test]
fn receive_relay_should_fail_without_specifying_units_per_second() {
    MockNet::reset();

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    let amount = 333u128;

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        None,
        None,
    );
    let relay_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        None,
        None,
    );

    let dest: MultiLocation = AccountId32 {
        network: Any,
        id: ALICE.into(),
    }
    .into();

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(1)).into().into()),
            Box::new(VersionedMultiLocation::V1(dest)),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            relay_chain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount
        );
    });

    ParaA::execute_with(|| {
        // ALICE gets nothing
        assert_eq!(parachain::Assets::balance(relay_asset_id_on_a, &ALICE), 0);
        // Asset manager gets nothing, all balance stuck
        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_a, AssetManager::account_id()),
            0
        );
    });
}

#[test]
fn send_para_a_asset_to_para_b_with_insufficient_fee() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 15u128;
    let units_per_second = 20_000_000u128;
    let dest_weight = 800_000u64;
    let fee = calculate_fee(units_per_second, dest_weight);
    assert!(fee > amount);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_second),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_second),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(2),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Transfer ParaA balance to B
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            dest_weight,
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    // Alice on B should receive nothing since the fee is insufficient
    ParaB::execute_with(|| {
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });
}

#[test]
fn send_para_a_asset_to_para_b_without_specifying_units_per_second() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let amount = 567u128;
    let dest_weight = 800_000u64;

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        None,
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Transfer ParaA balance to B
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            dest_weight,
        ));
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - amount
        )
    });

    // Alice on B should receive nothing since we didn't specify the unit per second
    ParaB::execute_with(|| {
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 0);
    });
}

#[test]
fn receive_insufficient_relay_asset_on_parachain() {
    MockNet::reset();

    let new_account = [5u8; 32];

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, false);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    let amount = 123u128;
    let units_per_sec = 0u128;

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let relay_asset_id = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let dest: MultiLocation = AccountId32 {
        network: Any,
        id: new_account,
    }
    .into();

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(1)).into().into()),
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            relay_chain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount
        );
    });

    // parachain should not have received assets
    ParaA::execute_with(|| {
        assert_eq!(
            parachain::Assets::balance(relay_asset_id, &new_account.into()),
            0
        );
    });

    let fresh_account_amount = 100;
    // Send native token to fresh_account
    ParaA::execute_with(|| {
        assert_ok!(parachain::Balances::transfer(
            parachain::Origin::signed(ALICE),
            new_account.into(),
            fresh_account_amount
        ));
    });

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(1)).into().into()),
            Box::new(VersionedMultiLocation::V1(dest)),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            relay_chain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount + amount
        );
    });

    // parachain should not have received assets
    ParaA::execute_with(|| {
        assert_eq!(
            parachain::Balances::free_balance(&new_account.into()),
            fresh_account_amount
        );
    });
}

#[test]
fn receive_sufficient_relay_asset_on_parachain() {
    MockNet::reset();

    let new_account = [5u8; 32];

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    let amount = 123u128;
    let units_per_sec = 0;

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let relay_asset_id = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let dest: MultiLocation = AccountId32 {
        network: Any,
        id: new_account,
    }
    .into();

    Relay::execute_with(|| {
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(X1(Parachain(1)).into().into()),
            Box::new(VersionedMultiLocation::V1(dest)),
            Box::new((Here, amount).into()),
            0,
        ));
        assert_eq!(
            relay_chain::Balances::free_balance(&para_account_id(1)),
            INITIAL_BALANCE + amount
        );
    });

    // parachain should have received assets
    ParaA::execute_with(|| {
        assert_eq!(
            parachain::Assets::balance(relay_asset_id, &new_account.into()),
            amount
        );
    });
}

/// Scenario:
/// A parachain transfers funds on the relay chain to another parachain account.
///
/// Asserts that the parachain accounts are updated as expected.
#[test]
fn withdraw_and_deposit() {
    MockNet::reset();

    let send_amount = 10;

    ParaA::execute_with(|| {
        let message = Xcm(vec![
            WithdrawAsset((Here, send_amount).into()),
            buy_execution((Here, send_amount)),
            DepositAsset {
                assets: All.into(),
                max_assets: 1,
                beneficiary: Parachain(2).into(),
            },
        ]);
        // Send withdraw and deposit
        assert_ok!(ParachainPalletXcm::send_xcm(Here, Parent, message));
    });

    Relay::execute_with(|| {
        assert_eq!(
            relay_chain::Balances::free_balance(para_account_id(1)),
            INITIAL_BALANCE - send_amount
        );
        assert_eq!(
            relay_chain::Balances::free_balance(para_account_id(2)),
            send_amount
        );
    });
}

/// Scenario:
/// A parachain wants to be notified that a transfer worked correctly.
/// It sends a `QueryHolding` after the deposit to get notified on success.
///
/// Asserts that the balances are updated correctly and the expected XCM is sent.
#[test]
fn query_holding() {
    MockNet::reset();

    let send_amount = 10;
    let query_id_set = 1234;

    // Send a message which fully succeeds on the relay chain
    ParaA::execute_with(|| {
        let message = Xcm(vec![
            WithdrawAsset((Here, send_amount).into()),
            buy_execution((Here, send_amount)),
            DepositAsset {
                assets: All.into(),
                max_assets: 1,
                beneficiary: Parachain(2).into(),
            },
            QueryHolding {
                query_id: query_id_set,
                dest: Parachain(1).into(),
                assets: All.into(),
                max_response_weight: 1_000_000_000,
            },
        ]);
        // Send withdraw and deposit with query holding
        assert_ok!(ParachainPalletXcm::send_xcm(Here, Parent, message,));
    });

    // Check that transfer was executed
    Relay::execute_with(|| {
        // Withdraw executed
        assert_eq!(
            relay_chain::Balances::free_balance(para_account_id(1)),
            INITIAL_BALANCE - send_amount
        );
        // Deposit executed
        assert_eq!(
            relay_chain::Balances::free_balance(para_account_id(2)),
            send_amount
        );
    });

    // Check that QueryResponse message was received
    // AllowKnownQueryResponses<PolkadotXcm> barrier impl should have let it through:
    ParaA::execute_with(|| {
        assert_eq!(
            parachain::MsgQueue::received_dmp(),
            vec![Xcm(vec![QueryResponse {
                query_id: query_id_set,
                response: Response::Assets(MultiAssets::new()),
                max_weight: 1_000_000_000,
            }])],
        );
    });
}

#[test]
fn test_versioning_on_runtime_upgrade_with_relay() {
    MockNet::reset();

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let para_a_source_location = create_asset_location(1, PARA_A_ID);

    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let para_a_asset_metadata = create_asset_metadata("ParaA", "ParaA", 12, 1, None, false, true);

    // register relay asset in parachain A (XCM version 1)
    ParaA::execute_with(|| {
        // SelfReserve
        parachain::set_current_xcm_version(1);
    });
    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(0u128),
        None,
    );

    let response = Response::Version(2);

    // This is irrelevant, nothing will be done with this message,
    // but we need to pass a message as an argument to trigger the storage change
    let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
        query_id: 0,
        response,
        max_weight: 0,
    }]);

    let dest: MultiLocation = AccountId32 {
        network: Any,
        id: ALICE.into(),
    }
    .into();

    Relay::execute_with(|| {
        // This sets the default version, for not known destinations
        assert_ok!(RelayChainPalletXcm::force_default_xcm_version(
            relay_chain::Origin::root(),
            Some(2)
        ));

        // Wrap version, which sets VersionedStorage
        // This is necessary because the mock router does not use wrap_version, but
        // this is not necessary in prod.
        // more specifically, this will trigger `note_unknown_version` to put the
        // version to `VersionDiscoveryQueue` on relay-chain's pallet-xcm
        assert_ok!(<RelayChainPalletXcm as WrapVersion>::wrap_version(
            &Parachain(PARA_A_ID).into(),
            mock_message
        ));

        // Transfer assets. Since it is an unknown destination, it will query for version
        assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
            relay_chain::Origin::signed(ALICE),
            Box::new(Parachain(PARA_A_ID).into().into()),
            Box::new(VersionedMultiLocation::V1(dest)),
            Box::new((Here, 123).into()),
            0,
        ));

        // Let's advance the relay. This should trigger the subscription message
        relay_chain::relay_roll_to(2);

        // queries should have been updated
        assert!(RelayChainPalletXcm::query(0).is_some());
    });

    let expected_supported_version: relay_chain::Event =
        pallet_xcm::Event::SupportedVersionChanged(
            MultiLocation {
                parents: 0,
                interior: X1(Parachain(PARA_A_ID)),
            },
            1,
        )
        .into();

    Relay::execute_with(|| {
        // Assert that the events vector contains the version change
        assert!(relay_chain::relay_events().contains(&expected_supported_version));
    });

    let expected_version_notified: parachain::Event = pallet_xcm::Event::VersionChangeNotified(
        MultiLocation {
            parents: 1,
            interior: Here,
        },
        2,
    )
    .into();

    // ParaA changes version to 2, and calls on_runtime_upgrade. This should notify the targets
    // of the new version change
    ParaA::execute_with(|| {
        // Set version
        parachain::set_current_xcm_version(2);
        // Do runtime upgrade
        parachain::on_runtime_upgrade();
        // Initialize block, to call on_initialize and notify targets
        parachain::para_roll_to(2);
        // Expect the event in the parachain
        assert!(parachain::para_events().contains(&expected_version_notified));
    });

    // This event should have been seen in the relay
    let expected_supported_version_2: relay_chain::Event =
        pallet_xcm::Event::SupportedVersionChanged(
            MultiLocation {
                parents: 0,
                interior: X1(Parachain(PARA_A_ID)),
            },
            2,
        )
        .into();

    Relay::execute_with(|| {
        // Assert that the events vector contains the new version change
        assert!(relay_chain::relay_events().contains(&expected_supported_version_2));
    });
}

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_para_b() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);
    let response = Response::Version(2);

    // This is irrelevant, nothing will be done with this message,
    // but we need to pass a message as an argument to trigger the storage change
    let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
        query_id: 0,
        response,
        max_weight: 0,
    }]);

    ParaA::execute_with(|| {
        // advertised version
        parachain::set_current_xcm_version(2);
    });

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    ParaB::execute_with(|| {
        // advertised version
        parachain::set_current_xcm_version(0);
    });

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let a_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );

    ParaA::execute_with(|| {
        // This sets the default version, for not known destinations
        assert_ok!(ParachainPalletXcm::force_default_xcm_version(
            parachain::Origin::root(),
            Some(2)
        ));
        // Wrap version, which sets VersionedStorage
        assert_ok!(<ParachainPalletXcm as WrapVersion>::wrap_version(
            &MultiLocation::new(1, X1(Parachain(2))),
            mock_message
        ));

        parachain::para_roll_to(2);

        // queries should have been updated
        assert!(ParachainPalletXcm::query(0).is_some());
    });

    let expected_supported_version: parachain::Event = pallet_xcm::Event::SupportedVersionChanged(
        MultiLocation {
            parents: 1,
            interior: X1(Parachain(PARA_B_ID)),
        },
        0,
    )
    .into();

    ParaA::execute_with(|| {
        // Assert that the events vector contains the version change
        assert!(parachain::para_events().contains(&expected_supported_version));
    });

    // Let's ensure talking in v0 works
    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(PARA_B_ID),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    ParaA::execute_with(|| {
        // free execution, full amount received
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            100,
            Box::new(VersionedMultiLocation::V1(dest)),
            ADVERTISED_DEST_WEIGHT
        ));
        // free execution, full amount received
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            INITIAL_BALANCE - 100
        );
    });

    ParaB::execute_with(|| {
        // free execution, full amount received
        assert_eq!(parachain::Assets::balance(a_asset_id_on_b, &ALICE), 100);
    });

    let expected_version_notified: parachain::Event = pallet_xcm::Event::VersionChangeNotified(
        MultiLocation {
            parents: 1,
            interior: X1(Parachain(PARA_A_ID)),
        },
        2,
    )
    .into();

    // ParaB changes version to 2, and calls on_runtime_upgrade. This should notify the targets
    // of the new version change
    ParaB::execute_with(|| {
        // Set version
        parachain::set_current_xcm_version(2);
        // Do runtime upgrade
        parachain::on_runtime_upgrade();
        // Initialize block, to call on_initialize and notify targets
        parachain::para_roll_to(2);
        // Expect the event in the parachain
        assert!(parachain::para_events().contains(&expected_version_notified));
    });

    // This event should have been seen in para A
    let expected_supported_version_2: parachain::Event =
        pallet_xcm::Event::SupportedVersionChanged(
            MultiLocation {
                parents: 1,
                interior: X1(Parachain(PARA_B_ID)),
            },
            2,
        )
        .into();

    // Para A should have received the version change
    ParaA::execute_with(|| {
        // Assert that the events vector contains the new version change
        assert!(parachain::para_events().contains(&expected_supported_version_2));
    });
}

#[test]
fn filtered_multilocation_should_not_work() {
    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = create_asset_location(1, PARA_B_ID);
    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(4), // set para id as 4
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Sending xcm to parachain 5 should not work
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer(
                parachain::Origin::signed(ALICE),
                parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                100,
                Box::new(VersionedMultiLocation::V1(dest)),
                80
            ),
            orml_xtokens::Error::<parachain::Runtime>::NotSupportedMultiLocation,
        );
    });

    let x3_dest = MultiLocation {
        parents: 1,
        interior: X3(
            Parachain(2),
            PalletInstance(PALLET_ASSET_INDEX),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };
    // We don't support X3 or more longer Junctions.
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer(
                parachain::Origin::signed(ALICE),
                parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                100,
                Box::new(VersionedMultiLocation::V1(x3_dest)),
                80
            ),
            orml_xtokens::Error::<parachain::Runtime>::NotSupportedMultiLocation,
        );
    });

    let parents_as_2_relay_dest = MultiLocation {
        parents: 2,
        interior: X1(AccountId32 {
            network: NetworkId::Any,
            id: ALICE.into(),
        }),
    };
    // relaychain dest with wrong parents should not work.
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer(
                parachain::Origin::signed(ALICE),
                parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                100,
                Box::new(VersionedMultiLocation::V1(parents_as_2_relay_dest)),
                80
            ),
            orml_xtokens::Error::<parachain::Runtime>::NotSupportedMultiLocation,
        );
    });

    let parents_as_2_dest = MultiLocation {
        parents: 2,
        interior: X2(
            Parachain(2),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };
    // Wrong parents should not work.
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer(
                parachain::Origin::signed(ALICE),
                parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                100,
                Box::new(VersionedMultiLocation::V1(parents_as_2_dest)),
                80
            ),
            orml_xtokens::Error::<parachain::Runtime>::NotSupportedMultiLocation,
        );
    });

    let here_dest = MultiLocation {
        parents: 1,
        interior: Here,
    };
    // The destination with no receiver should not work.
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer(
                parachain::Origin::signed(ALICE),
                parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                100,
                Box::new(VersionedMultiLocation::V1(here_dest)),
                80
            ),
            orml_xtokens::Error::<parachain::Runtime>::NotSupportedMultiLocation,
        );
    });

    // Correct relaychain location should work, (1, Here)
    let relay_dest = MultiLocation {
        parents: 1,
        interior: X1(AccountId32 {
            network: NetworkId::Any,
            id: ALICE.into(),
        }),
    };
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            100,
            Box::new(VersionedMultiLocation::V1(relay_dest)),
            80
        ));
    });

    // Correct sibling location should work
    let sibling_chain_dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(2),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            parachain::Origin::signed(ALICE),
            parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
            100,
            Box::new(VersionedMultiLocation::V1(sibling_chain_dest)),
            80
        ));
    });
}

#[test]
fn less_than_min_xcm_fee_should_not_work() {
    MockNet::reset();

    let para_a_source_location = create_asset_location(1, PARA_A_ID);
    let para_b_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X2(
            Parachain(PARA_B_ID),
            GeneralKey(WeakBoundedVec::force_from(b"ParaBToken".to_vec(), None)),
        ),
    )));
    let para_b_as_reserve_chain = create_asset_location(1, PARA_B_ID);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let relay_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(0u128),
        None,
    );
    let b_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );

    let _ = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(0u128),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(0u128),
        None,
    );

    // Initialize some tokens for alice
    assert_ok!(ParaA::execute_with(|| {
        parachain::Assets::mint(
            parachain::Origin::signed(parachain::AssetManager::account_id()),
            b_asset_id_on_a,
            ALICE,
            1000,
        )
    }));
    assert_ok!(ParaA::execute_with(|| {
        parachain::Assets::mint(
            parachain::Origin::signed(parachain::AssetManager::account_id()),
            relay_asset_id_on_a,
            ALICE,
            1000,
        )
    }));

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(2),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    let amount = 450;
    let fee_amount: u128 = 200;
    // Minimum xcm execution fee paid on destination chain.
    // Current only support `ToReserve` with relay-chain asset as fee. other case
    // like `NonReserve` or `SelfReserve` with relay-chain fee is not support.
    // And our `MaxAssetsForTransfer` for xtokens is 1,
    // so `transfer_multicurrencies` is not supported on calamari.
    // If min-xcm-fee is not set, no one can pay xcm fee(MinXcmFeeNotDefined).
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (
                        parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                        amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                        fee_amount
                    )
                ],
                1,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                40,
            ),
            orml_xtokens::Error::<parachain::Runtime>::MinXcmFeeNotDefined
        );
    });

    // set min xcm fee on ParaA
    let min_xcm_fee = 40;
    ParaA::execute_with(|| {
        assert_ok!(AssetManager::set_min_xcm_fee(
            parachain::Origin::root(),
            para_b_as_reserve_chain,
            min_xcm_fee,
        ));
    });

    // fee is bigger than min-xcm-fee should work(39 < 40).
    ParaA::execute_with(|| {
        assert_noop!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (
                        parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                        amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                        39
                    )
                ],
                1,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                40,
            ),
            orml_xtokens::Error::<parachain::Runtime>::FeeNotEnough
        );
    });

    // fee is bigger than min-xcm-fee should work
    ParaA::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer_multicurrencies(
            Some(ALICE).into(),
            vec![
                (
                    parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                    amount
                ),
                (
                    parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                    fee_amount
                )
            ],
            1,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            40,
        ));
    });
}

#[test]
fn transfer_multicurrencies_should_work_scenarios() {
    MockNet::reset();

    let para_a_id = 1;
    let para_b_id = 2;
    let para_a_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X1(Parachain(para_a_id)),
    )));
    let para_b_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
        1,
        X1(Parachain(para_b_id)),
    )));
    let units_per_sec = 0;

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);

    let _ = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let relay_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let b_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let b_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let relay_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let relay_asset_amount_minted_on_a = 10000000;
    // Initialize some relay chain tokens for alice
    assert_ok!(ParaA::execute_with(|| {
        parachain::Assets::mint(
            parachain::Origin::signed(parachain::AssetManager::account_id()),
            relay_asset_id_on_a,
            ALICE,
            relay_asset_amount_minted_on_a,
        )
    }));

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(para_a_id),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Send some ParaB tokens from Alice on B to Alice on A
    let amount_to_a = 10000000;
    ParaB::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            Some(ALICE).into(),
            parachain::CurrencyId::MantaCurrency(b_asset_id_on_b),
            amount_to_a,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            ADVERTISED_DEST_WEIGHT,
        ));
    });

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(para_b_id),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    let amount_back_to_b = 100;
    let fee_amount: u128 = 50;
    let min_xcm_fee = 10;
    // Send some ParaB tokens from Alice on A back to Alice on B
    ParaA::execute_with(|| {
        assert_ok!(AssetManager::set_min_xcm_fee(
            parachain::Origin::root(),
            para_b_source_location,
            min_xcm_fee,
        ));

        assert_ok!(parachain::XTokens::transfer_multicurrencies(
            Some(ALICE).into(),
            vec![
                (
                    parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                    amount_back_to_b
                ),
                (
                    parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                    fee_amount
                )
            ],
            1,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            ADVERTISED_DEST_WEIGHT,
        ));

        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_a, &ALICE),
            relay_asset_amount_minted_on_a - fee_amount
        );
        assert_eq!(
            // Notice that total supply between the two chains is:
            // `(relay_asset_amount_minted_on_a - fee_amount) + (fee_amount - min_xcm_fee)`
            // `relay_asset_amount_minted_on_a - fee_amount` is still on ParaA
            // `fee_amount - min_xcm_fee` is on ParaB. (min_xcm_fee is subtracted in the ORML code)
            // The total comes out to `relay_asset_amount_minted_on_a - min_xcm_fee`, meaning one `min_xcm_fee` is destroyed
            // This is a design choice by ORML to make these kinds of transfers possible.
            // Practically some of tokens held as reserve on the reserve chain, will become unwithdrawable.
            parachain::Assets::total_supply(relay_asset_id_on_a),
            relay_asset_amount_minted_on_a - fee_amount
        );

        assert_eq!(
            parachain::Assets::balance(b_asset_id_on_a, &ALICE),
            amount_to_a - amount_back_to_b
        );
    });

    Relay::execute_with(|| {
        let para_a_sovereign_on_relay = para_account_id(1);
        let para_b_sovereign_on_relay = para_account_id(2);
        assert_eq!(
            relay_chain::Balances::free_balance(&para_a_sovereign_on_relay),
            INITIAL_BALANCE - (fee_amount - min_xcm_fee)
        );
        assert_eq!(
            relay_chain::Balances::free_balance(&para_b_sovereign_on_relay),
            // 0 because transfer_multicurrencies uses a Teleport for this case
            0
        );
    });

    ParaB::execute_with(|| {
        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            // because transfer_multicurrencies uses a Teleport for this case
            // we no longer expect the amount_back_to_b to be in the sovereign account
            INITIAL_BALANCE - amount_to_a
        );

        // Parachain A sovereign account on Parachain B should receive: 0
        // because transfer_multicurrencies uses Teleport in this case
        let para_a_sovereign_on_para_b = parachain::LocationToAccountId::convert_ref(
            MultiLocation::new(1, X1(Parachain(para_a_id))),
        )
        .unwrap();
        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_b, &para_a_sovereign_on_para_b),
            0
        );

        assert_eq!(
            // min_xcm_fee will be just enough to pay for execution so
            // no change from BuyExecution will be deposited in Alice's account.
            parachain::Assets::balance(relay_asset_id_on_b, &ALICE),
            0
        );
        assert_eq!(
            parachain::Assets::total_supply(relay_asset_id_on_b),
            // Since all of the fee is used the total supply should be 0
            0
        );
    });
}

/// Checks only must-fail cases related to transfer_multicurrencies
/// First part is for testing cases on the sender side.
/// Second part is for testing cases on the receiver side.
#[test]
fn transfer_multicurrencies_should_fail_scenarios() {
    MockNet::reset();

    let para_a_id = 1;
    let para_b_id = 2;
    let para_c_id = 3;
    let para_a_source_location = create_asset_location(1, para_a_id);
    let para_b_source_location = create_asset_location(1, para_b_id);
    let para_c_source_location = create_asset_location(1, para_c_id);

    let para_a_asset_metadata =
        create_asset_metadata("ParaAToken", "ParaA", 18, 1, None, false, true);
    let para_b_asset_metadata =
        create_asset_metadata("ParaBToken", "ParaB", 18, 1, None, false, true);
    let para_c_asset_metadata =
        create_asset_metadata("ParaCToken", "ParaC", 18, 1, None, false, true);

    let relay_source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
    let relay_asset_metadata = create_asset_metadata("Kusama", "KSM", 12, 1, None, false, true);
    let units_per_sec = 0;

    let a_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let relay_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let b_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let c_asset_id_on_a = register_assets_on_parachain::<ParaA>(
        &para_c_source_location,
        &para_c_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let b_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &para_b_source_location,
        &para_b_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let _ = register_assets_on_parachain::<ParaB>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let relay_asset_id_on_b = register_assets_on_parachain::<ParaB>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let c_asset_id_on_c = register_assets_on_parachain::<ParaC>(
        &para_c_source_location,
        &para_c_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let _ = register_assets_on_parachain::<ParaC>(
        &para_a_source_location,
        &para_a_asset_metadata,
        Some(units_per_sec),
        None,
    );
    let _ = register_assets_on_parachain::<ParaC>(
        &relay_source_location,
        &relay_asset_metadata,
        Some(units_per_sec),
        None,
    );

    let amount = 1000;
    assert_ok!(ParaA::execute_with(|| {
        parachain::Assets::mint(
            parachain::Origin::signed(parachain::AssetManager::account_id()),
            relay_asset_id_on_a,
            ALICE,
            amount,
        )
    }));

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(para_a_id),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };
    ParaC::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            Some(ALICE).into(),
            parachain::CurrencyId::MantaCurrency(c_asset_id_on_c),
            amount,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            ADVERTISED_DEST_WEIGHT,
        ));
    });
    ParaB::execute_with(|| {
        assert_ok!(parachain::XTokens::transfer(
            Some(ALICE).into(),
            parachain::CurrencyId::MantaCurrency(b_asset_id_on_b),
            amount,
            Box::new(VersionedMultiLocation::V1(dest)),
            ADVERTISED_DEST_WEIGHT,
        ));
    });

    let dest = MultiLocation {
        parents: 1,
        interior: X2(
            Parachain(para_b_id),
            AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            },
        ),
    };

    // Sender Side Tests:

    let fee_amount: u128 = 50;
    let min_xcm_fee = 10;
    ParaA::execute_with(|| {
        assert_ok!(AssetManager::set_min_xcm_fee(
            parachain::Origin::root(),
            para_b_source_location.clone(),
            min_xcm_fee,
        ));
        assert_ok!(AssetManager::set_min_xcm_fee(
            parachain::Origin::root(),
            para_c_source_location,
            min_xcm_fee,
        ));

        assert_err!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (
                        parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                        amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(c_asset_id_on_a),
                        fee_amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                        fee_amount
                    )
                ],
                2,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                ADVERTISED_DEST_WEIGHT,
            ),
            // Assets and fee must have the same reserve
            orml_xtokens::Error::<parachain::Runtime>::DistinctReserveForAssetAndFee
        );

        assert_err!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (
                        parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                        amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                        fee_amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(c_asset_id_on_a),
                        fee_amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(a_asset_id_on_a),
                        fee_amount
                    )
                ],
                2,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                ADVERTISED_DEST_WEIGHT,
            ),
            // MaxAssetsForTransfer is set to 3 in the mock
            orml_xtokens::Error::<parachain::Runtime>::TooManyAssetsBeingSent
        );

        assert_err!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (
                        parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                        amount
                    ),
                    (
                        parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                        fee_amount
                    )
                ],
                2,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                ADVERTISED_DEST_WEIGHT,
            ),
            orml_xtokens::Error::<parachain::Runtime>::AssetIndexNonExistent
        );

        assert_err!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (
                        parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                        amount
                    ),
                    (parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a), 0)
                ],
                1,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                ADVERTISED_DEST_WEIGHT,
            ),
            // 0 fees should not work
            orml_xtokens::Error::<parachain::Runtime>::ZeroAmount
        );

        assert_err!(
            parachain::XTokens::transfer_multicurrencies(
                Some(ALICE).into(),
                vec![
                    (parachain::CurrencyId::MantaCurrency(b_asset_id_on_a), 0),
                    (
                        parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                        fee_amount
                    )
                ],
                1,
                Box::new(VersionedMultiLocation::V1(dest.clone())),
                ADVERTISED_DEST_WEIGHT,
            ),
            // 0 assets should not work
            orml_xtokens::Error::<parachain::Runtime>::ZeroAmount
        );
    });

    // Receiver Side Tests:

    let amount_back_to_b = 100;
    let fee_amount: u128 = 50;
    let min_xcm_fee = 40;
    // Setup to succeed on the sender side, but fail on the receiver side due to not enough fees.
    ParaA::execute_with(|| {
        assert_ok!(AssetManager::set_min_xcm_fee(
            parachain::Origin::root(),
            para_b_source_location,
            min_xcm_fee,
        ));

        assert_eq!(parachain::Assets::balance(b_asset_id_on_a, &ALICE), amount);
        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_a, &ALICE),
            amount
        );

        assert_ok!(parachain::XTokens::transfer_multicurrencies(
            Some(ALICE).into(),
            vec![
                (
                    parachain::CurrencyId::MantaCurrency(b_asset_id_on_a),
                    amount_back_to_b
                ),
                (
                    parachain::CurrencyId::MantaCurrency(relay_asset_id_on_a),
                    fee_amount
                )
            ],
            1,
            Box::new(VersionedMultiLocation::V1(dest.clone())),
            ADVERTISED_DEST_WEIGHT,
        ));

        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_a, &ALICE),
            amount - fee_amount
        );

        assert_eq!(
            parachain::Assets::balance(b_asset_id_on_a, &ALICE),
            amount - amount_back_to_b
        );
    });

    Relay::execute_with(|| {
        let para_a_sovereign_on_relay = para_account_id(1);
        let para_b_sovereign_on_relay = para_account_id(2);
        assert_eq!(
            relay_chain::Balances::free_balance(&para_a_sovereign_on_relay),
            INITIAL_BALANCE - (fee_amount - min_xcm_fee)
        );
        assert_eq!(
            relay_chain::Balances::free_balance(&para_b_sovereign_on_relay),
            // 0 because transfer_multicurrencies uses a Teleport for this case
            0
        );
    });

    ParaB::execute_with(|| {
        // Parachain A sovereign account on Parachain B should receive: 0
        // because transfer_multicurrencies uses Teleport in this case
        let para_a_sovereign_on_para_b = parachain::LocationToAccountId::convert_ref(
            MultiLocation::new(1, X1(Parachain(para_a_id))),
        )
        .unwrap();
        assert_eq!(
            parachain::Assets::balance(relay_asset_id_on_b, &para_a_sovereign_on_para_b),
            0
        );

        assert_eq!(
            // The BuyExecution failed so no change to deposit in Alice's account.
            parachain::Assets::balance(relay_asset_id_on_b, &ALICE),
            0
        );
        // Since BuyExecution failed so no change to deposit in Alice's account
        // the total_supply will remain 0.
        assert_eq!(parachain::Assets::total_supply(relay_asset_id_on_b), 0);

        assert_eq!(
            parachain::Balances::free_balance(&ALICE),
            // Did not receive amount_back_to_b, because there was not enough of the relay fee
            INITIAL_BALANCE - amount
        );
    });
}

#[test]
fn test_receiver_side_weight() {
    let weight = <ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(
        &mut self_reserve_xcm_message_receiver_side::<parachain::Call>(),
    )
    .unwrap();
    assert!(weight <= ADVERTISED_DEST_WEIGHT);

    let weight = <ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(
        &mut to_reserve_xcm_message_receiver_side::<parachain::Call>(),
    )
    .unwrap();
    assert!(weight <= ADVERTISED_DEST_WEIGHT);
}

#[test]
fn test_sender_side_xcm_weight() {
    let mut msg = self_reserve_xcm_message_sender_side::<parachain::Call>();
    let weight =
        <ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
    assert!(weight < ADVERTISED_DEST_WEIGHT);

    let mut msg = to_reserve_xcm_message_sender_side::<parachain::Call>();
    let weight =
        <ParaXcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
    assert!(weight < ADVERTISED_DEST_WEIGHT);
}
