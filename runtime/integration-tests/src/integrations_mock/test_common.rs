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

//! Common Parachain Integration Tests.

#![cfg(test)]
#![allow(clippy::identity_op)] // keep e.g. 1 * DAYS for legibility

use super::{mock::*, *};
use frame_support::{
    assert_err, assert_noop, assert_ok,
    error::BadOrigin,
    traits::{tokens::ExistenceRequirement, Get, PalletInfo, PalletsInfoAccess},
};
use runtime_common::test_helpers::{
    self_reserve_xcm_message_receiver_side, self_reserve_xcm_message_sender_side,
    to_reserve_xcm_message_receiver_side, to_reserve_xcm_message_sender_side,
    ADVERTISED_DEST_WEIGHT,
};

use manta_primitives::{
    assets::{
        AssetConfig, AssetLocation, AssetRegistryMetadata, AssetStorageMetadata, FungibleLedger,
        FungibleLedgerError,
    },
    constants::time::{DAYS, HOURS},
    types::{AccountId, Balance, CalamariAssetId},
};
use session_key_primitives::util::unchecked_account_id;
use xcm::{
    opaque::latest::{
        Junction::{PalletInstance, Parachain},
        Junctions::X2,
        MultiLocation,
    },
    VersionedMultiLocation,
};
use xcm_executor::traits::WeightBounds;

use super::{ALICE, ALICE_SESSION_KEYS};
use sp_core::sr25519;
use sp_runtime::{DispatchError, ModuleError};

mod parachain_staking_tests {
    use super::*;

    #[test]
    fn ensure_block_per_round_and_leave_delays_equal_7days() {
        // NOTE: If you change one, change the other as well
        type LeaveCandidatesDelay =
            <Runtime as pallet_parachain_staking::Config>::LeaveCandidatesDelay;
        type LeaveDelegatorsDelay =
            <Runtime as pallet_parachain_staking::Config>::LeaveDelegatorsDelay;
        type CandidateBondLessDelay =
            <Runtime as pallet_parachain_staking::Config>::CandidateBondLessDelay;
        type DelegationBondLessDelay =
            <Runtime as pallet_parachain_staking::Config>::DelegationBondLessDelay;
        assert_eq!(
            7 * DAYS,
            DefaultBlocksPerRound::get() * LeaveDelayRounds::get()
        );
        assert_eq!(
            7 * DAYS,
            DefaultBlocksPerRound::get() * LeaveCandidatesDelay::get()
        );
        assert_eq!(
            7 * DAYS,
            DefaultBlocksPerRound::get() * LeaveDelegatorsDelay::get()
        );
        assert_eq!(
            7 * DAYS,
            DefaultBlocksPerRound::get() * CandidateBondLessDelay::get()
        );
        assert_eq!(
            7 * DAYS,
            DefaultBlocksPerRound::get() * DelegationBondLessDelay::get()
        );
    }

    #[test]
    fn collator_cant_join_below_standard_bond() {
        ExtBuilder::default()
            .with_balances(vec![
                (ALICE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (CHARLIE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            ])
            .with_collators(vec![(ALICE.clone(), 50)])
            .build()
            .execute_with(|| {
                assert_noop!(
                    ParachainStaking::join_candidates(
                        RuntimeOrigin::signed(BOB.clone()),
                        MIN_BOND_TO_BE_CONSIDERED_COLLATOR - 1,
                        6u32
                    ),
                    pallet_parachain_staking::Error::<Runtime>::CandidateBondBelowMin
                );
            });
    }

    #[test]
    fn collator_can_join_with_min_bond() {
        ExtBuilder::default()
            .with_collators(vec![(ALICE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR)])
            .with_balances(vec![
                (ALICE.clone(), INITIAL_BALANCE),
                (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            ])
            .build()
            .execute_with(|| {
                // Create and bond session keys to Bob's account.
                assert_ok!(Session::set_keys(
                    RuntimeOrigin::signed(BOB.clone()),
                    BOB_SESSION_KEYS.clone(),
                    vec![]
                ));
                assert!(<Session as frame_support::traits::ValidatorRegistration<
                    AccountId,
                >>::is_registered(&BOB));

                assert_ok!(ParachainStaking::join_candidates(
                    RuntimeOrigin::signed(BOB.clone()),
                    <Runtime as pallet_parachain_staking::Config>::MinCandidateStk::get(),
                    3u32
                ));

                // BOB is now a candidate
                assert!(ParachainStaking::candidate_pool().contains(
                    &pallet_parachain_staking::Bond {
                        owner: BOB.clone(),
                        amount: MIN_BOND_TO_BE_CONSIDERED_COLLATOR
                    }
                ));

                // After one round
                run_to_block(
                    <Runtime as pallet_parachain_staking::Config>::DefaultBlocksPerRound::get() + 1,
                );

                // BOB becomes part of the selected candidates set
                assert!(ParachainStaking::selected_candidates().contains(&BOB));
            });
    }

    #[test]
    fn collator_with_large_stake_but_too_low_self_bond_not_selected_for_block_production() {
        ExtBuilder::default()
            .with_balances(vec![
                (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (CHARLIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (DAVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (EVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (FERDIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (USER.clone(), 400_000_000 * UNIT),
            ])
            .with_invulnerables(vec![])
            .with_authorities(vec![
                (ALICE.clone(), ALICE_SESSION_KEYS.clone()),
                (BOB.clone(), BOB_SESSION_KEYS.clone()),
                (CHARLIE.clone(), CHARLIE_SESSION_KEYS.clone()),
                (DAVE.clone(), DAVE_SESSION_KEYS.clone()),
                (EVE.clone(), EVE_SESSION_KEYS.clone()),
                (FERDIE.clone(), FERDIE_SESSION_KEYS.clone()),
            ])
            .build()
            .execute_with(|| {
                initialize_collators_through_whitelist(vec![
                    ALICE.clone(),
                    BOB.clone(),
                    CHARLIE.clone(),
                    DAVE.clone(),
                    EVE.clone(),
                    FERDIE.clone(),
                ]);
                // Increase self-bond for everyone but ALICE
                for collator in vec![
                    BOB.clone(),
                    CHARLIE.clone(),
                    DAVE.clone(),
                    EVE.clone(),
                    FERDIE.clone(),
                ] {
                    assert_ok!(ParachainStaking::candidate_bond_more(
                        RuntimeOrigin::signed(collator.clone()),
                        MIN_BOND_TO_BE_CONSIDERED_COLLATOR - EARLY_COLLATOR_MINIMUM_STAKE
                    ));
                }

                // Delegate a large amount of tokens to EVE and ALICE
                for collator in vec![EVE.clone(), ALICE.clone()] {
                    assert_ok!(ParachainStaking::delegate(
                        RuntimeOrigin::signed(USER.clone()),
                        collator,
                        100_000_000 * UNIT,
                        50,
                        50
                    ));
                }

                // Ensure ALICE is not selected despite having a large total stake through delegation
                // NOTE: Must use 6 or more collators because 5 is the minimum on calamari
                assert!(!ParachainStaking::compute_top_candidates().contains(&ALICE));
                assert!(ParachainStaking::compute_top_candidates().contains(&BOB));
                assert!(ParachainStaking::compute_top_candidates().contains(&CHARLIE));
                assert!(ParachainStaking::compute_top_candidates().contains(&DAVE));
                assert!(ParachainStaking::compute_top_candidates().contains(&EVE));
                assert!(ParachainStaking::compute_top_candidates().contains(&FERDIE));
            });
    }

    #[test]
    fn collator_can_leave_if_below_standard_bond() {
        ExtBuilder::default()
            .with_balances(vec![
                (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (BOB.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (CHARLIE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (DAVE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (EVE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (FERDIE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
            ])
            .with_invulnerables(vec![])
            .with_authorities(vec![
                (ALICE.clone(), ALICE_SESSION_KEYS.clone()),
                (BOB.clone(), BOB_SESSION_KEYS.clone()),
                (CHARLIE.clone(), CHARLIE_SESSION_KEYS.clone()),
                (DAVE.clone(), DAVE_SESSION_KEYS.clone()),
                (EVE.clone(), EVE_SESSION_KEYS.clone()),
                (FERDIE.clone(), FERDIE_SESSION_KEYS.clone()),
            ])
            .build()
            .execute_with(|| {
                initialize_collators_through_whitelist(vec![
                    ALICE.clone(),
                    BOB.clone(),
                    CHARLIE.clone(),
                    DAVE.clone(),
                    EVE.clone(),
                    FERDIE.clone(),
                ]);
                // Attempt to leave as whitelist collator
                assert_ok!(ParachainStaking::schedule_leave_candidates(
                    RuntimeOrigin::signed(FERDIE.clone()),
                    6
                ));
            });
    }

    #[test]
    fn collator_with_400k_not_selected_for_block_production() {
        ExtBuilder::default()
            .with_balances(vec![
                (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (CHARLIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (DAVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (EVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (FERDIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
            ])
            .with_invulnerables(vec![])
            .with_authorities(vec![
                (ALICE.clone(), ALICE_SESSION_KEYS.clone()),
                (BOB.clone(), BOB_SESSION_KEYS.clone()),
                (CHARLIE.clone(), CHARLIE_SESSION_KEYS.clone()),
                (DAVE.clone(), DAVE_SESSION_KEYS.clone()),
                (EVE.clone(), EVE_SESSION_KEYS.clone()),
                (FERDIE.clone(), FERDIE_SESSION_KEYS.clone()),
            ])
            .build()
            .execute_with(|| {
                initialize_collators_through_whitelist(vec![
                    ALICE.clone(),
                    BOB.clone(),
                    CHARLIE.clone(),
                    DAVE.clone(),
                    EVE.clone(),
                    FERDIE.clone(),
                ]);
                // Increase bond for everyone but FERDIE
                for collator in vec![
                    BOB.clone(),
                    CHARLIE.clone(),
                    DAVE.clone(),
                    EVE.clone(),
                    FERDIE.clone(),
                ] {
                    assert_ok!(ParachainStaking::candidate_bond_more(
                        RuntimeOrigin::signed(collator.clone()),
                        MIN_BOND_TO_BE_CONSIDERED_COLLATOR - EARLY_COLLATOR_MINIMUM_STAKE
                    ));
                }

                // Ensure CHARLIE and later are not selected
                // NOTE: Must use 6 or more collators because 5 is the minimum on calamari
                assert!(!ParachainStaking::compute_top_candidates().contains(&ALICE));
                assert!(ParachainStaking::compute_top_candidates().contains(&BOB));
                assert!(ParachainStaking::compute_top_candidates().contains(&CHARLIE));
                assert!(ParachainStaking::compute_top_candidates().contains(&DAVE));
                assert!(ParachainStaking::compute_top_candidates().contains(&EVE));
                assert!(ParachainStaking::compute_top_candidates().contains(&FERDIE));
            });
    }
}

#[test]
fn balances_operations_should_work() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
            (DAVE.clone(), INITIAL_BALANCE),
        ])
        .build()
        .execute_with(|| {
            let transfer_amount = 10 * UNIT;

            // Basic transfer should work
            assert_ok!(Balances::transfer(
                RuntimeOrigin::signed(ALICE.clone()),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                transfer_amount,
            ));
            assert_eq!(
                Balances::free_balance(ALICE.clone()),
                INITIAL_BALANCE - transfer_amount
            );
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                INITIAL_BALANCE + transfer_amount
            );

            // Force transfer some tokens from one account to another with Root
            assert_ok!(Balances::force_transfer(
                root_origin(),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                sp_runtime::MultiAddress::Id(ALICE.clone()),
                transfer_amount,
            ));
            assert_eq!(Balances::free_balance(ALICE.clone()), INITIAL_BALANCE);
            assert_eq!(Balances::free_balance(CHARLIE.clone()), INITIAL_BALANCE);

            // Should not be able to transfer all with this call
            assert_err!(
                Balances::transfer_keep_alive(
                    RuntimeOrigin::signed(ALICE.clone()),
                    sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                    INITIAL_BALANCE,
                ),
                pallet_balances::Error::<Runtime>::KeepAlive
            );

            // Transfer all down to zero
            assert_ok!(Balances::transfer_all(
                RuntimeOrigin::signed(BOB.clone()),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                false
            ));
            assert_eq!(Balances::free_balance(BOB.clone()), 0);
            assert_eq!(Balances::free_balance(CHARLIE.clone()), INITIAL_BALANCE * 2);

            // Transfer all but keep alive with ED
            assert_ok!(Balances::transfer_all(
                RuntimeOrigin::signed(DAVE.clone()),
                sp_runtime::MultiAddress::Id(ALICE.clone()),
                true
            ));
            assert_eq!(
                Balances::free_balance(DAVE.clone()),
                NativeTokenExistentialDeposit::get()
            );

            // Even though keep alive is set to false alice cannot fall below the ED
            // because it has an outstanding consumer reference, from being a collator.
            assert_ok!(Balances::transfer_all(
                RuntimeOrigin::signed(ALICE.clone()),
                sp_runtime::MultiAddress::Id(CHARLIE.clone()),
                false
            ));
            assert_eq!(
                Balances::free_balance(ALICE.clone()),
                NativeTokenExistentialDeposit::get()
            );
        });
}

#[test]
fn root_can_change_default_xcm_vers() {
    ExtBuilder::default().build().execute_with(|| {
        // Root sets the defaultXcm
        assert_ok!(PolkadotXcm::force_default_xcm_version(
            root_origin(),
            Some(2)
        ));
    })
}

#[test]
fn sanity_check_round_duration() {
    assert_eq!(DefaultBlocksPerRound::get(), 6 * HOURS);
}

#[test]
fn concrete_fungible_ledger_transfers_work() {
    ExtBuilder::default()
        .with_balances(vec![
            (ALICE.clone(), INITIAL_BALANCE),
            (BOB.clone(), INITIAL_BALANCE),
            (CHARLIE.clone(), INITIAL_BALANCE),
        ])
        .build()
        .execute_with(|| {
            let transfer_amount = 10 * UNIT;
            let mut current_balance_alice = INITIAL_BALANCE;
            let mut current_balance_charlie = INITIAL_BALANCE;

            // Transfer tests for native assets:

            // Try to transfer more than available
            assert_err!(
                RuntimeConcreteFungibleLedger::transfer(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &ALICE.clone(),
                    &CHARLIE.clone(),
                    INITIAL_BALANCE + 1,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <Runtime as frame_system::Config>::PalletInfo::index::<Balances>()
                        .unwrap() as u8,
                    error: [2, 0, 0, 0],
                    message: Some("InsufficientBalance")
                }))
            );
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                current_balance_charlie
            );

            // Try to transfer and go below existential deposit
            assert_err!(
                RuntimeConcreteFungibleLedger::transfer(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &ALICE.clone(),
                    &CHARLIE.clone(),
                    INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <Runtime as frame_system::Config>::PalletInfo::index::<Balances>()
                        .unwrap() as u8,
                    error: [4, 0, 0, 0],
                    message: Some("KeepAlive")
                }))
            );
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                current_balance_charlie
            );

            // A normal transfer should work
            assert_ok!(RuntimeConcreteFungibleLedger::transfer(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &ALICE.clone(),
                &CHARLIE.clone(),
                transfer_amount,
                ExistenceRequirement::KeepAlive
            ));
            current_balance_alice -= transfer_amount;
            current_balance_charlie += transfer_amount;
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(CHARLIE.clone()),
                current_balance_charlie
            );

            // Should not be able to create new account with lower than ED balance
            let new_account = unchecked_account_id::<sr25519::Public>("NewAccount");
            assert_err!(
                RuntimeConcreteFungibleLedger::transfer(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &ALICE.clone(),
                    &new_account,
                    NativeTokenExistentialDeposit::get() - 1,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <Runtime as frame_system::Config>::PalletInfo::index::<Balances>()
                        .unwrap() as u8,
                    error: [3, 0, 0, 0],
                    message: Some("ExistentialDeposit")
                }))
            );

            // Should be able to create new account with enough balance
            assert_ok!(RuntimeConcreteFungibleLedger::transfer(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &ALICE.clone(),
                &new_account,
                NativeTokenExistentialDeposit::get(),
                ExistenceRequirement::KeepAlive
            ));
            current_balance_alice -= NativeTokenExistentialDeposit::get();
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(new_account),
                NativeTokenExistentialDeposit::get()
            );

            // Transfer all of your balance without dropping below ED should work
            assert_ok!(RuntimeConcreteFungibleLedger::transfer(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &BOB.clone(),
                &ALICE.clone(),
                INITIAL_BALANCE - NativeTokenExistentialDeposit::get(),
                ExistenceRequirement::KeepAlive
            ));
            current_balance_alice += INITIAL_BALANCE - NativeTokenExistentialDeposit::get();
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(
                Balances::free_balance(BOB.clone()),
                NativeTokenExistentialDeposit::get()
            );

            // Transfer the ED should work if AllowDeath is selected
            assert_ok!(RuntimeConcreteFungibleLedger::transfer(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &BOB.clone(),
                &ALICE.clone(),
                NativeTokenExistentialDeposit::get(),
                ExistenceRequirement::AllowDeath
            ));
            current_balance_alice += NativeTokenExistentialDeposit::get();
            assert_eq!(Balances::free_balance(ALICE.clone()), current_balance_alice);
            assert_eq!(Balances::free_balance(BOB.clone()), 0);
            assert!(!frame_system::Account::<Runtime>::contains_key(BOB.clone()));

            // Transfer tests for non-native assets:

            let min_balance = 10u128;
            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Kusama".to_vec(),
                    symbol: b"KSM".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: true,
            };
            let source_location =
                AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);

            // Register and mint for testing.
            let amount = Balance::MAX;
            assert_ok!(RuntimeConcreteFungibleLedger::deposit_minting(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                amount,
            ),);
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                amount
            );

            // Transferring and falling below ED of the asset should not work with KeepAlive.
            assert_err!(
                RuntimeConcreteFungibleLedger::transfer(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &BOB.clone(),
                    amount,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
                        as u8,
                    error: [0, 0, 0, 0],
                    message: Some("BalanceLow")
                }))
            );
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                amount
            );

            assert_err!(
                RuntimeConcreteFungibleLedger::transfer(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &BOB.clone(),
                    min_balance - 1,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Token(
                    sp_runtime::TokenError::BelowMinimum
                ))
            );
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                amount
            );

            // Transferring normal amounts should work.
            assert_ok!(RuntimeConcreteFungibleLedger::transfer(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                &BOB.clone(),
                transfer_amount,
                ExistenceRequirement::KeepAlive
            ),);
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                u128::MAX - transfer_amount
            );
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    BOB.clone()
                ),
                transfer_amount
            );

            // Transferring all of the balance of an account should work.
            assert_ok!(RuntimeConcreteFungibleLedger::transfer(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &BOB.clone(),
                &ALICE.clone(),
                transfer_amount,
                ExistenceRequirement::AllowDeath
            ),);
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    BOB.clone()
                ),
                0
            );

            // Transferring unregistered asset ID should not work.
            assert_err!(
                RuntimeConcreteFungibleLedger::transfer(
                    CalamariAssetId::MAX,
                    &ALICE.clone(),
                    &CHARLIE.clone(),
                    transfer_amount,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::InvalidTransfer(DispatchError::Module(ModuleError {
                    index: <Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
                        as u8,
                    error: [3, 0, 0, 0],
                    message: Some("Unknown")
                }))
            );
        });
}

mod tx_pause_tests {
    use super::*;

    const PALLET_NAME: &[u8] = b"Utility";
    const FUNCTION_NAME: &[u8] = b"batch";

    fn pause_transaction_storage_event_works(pause: bool) {
        if pause {
            System::assert_last_event(RuntimeEvent::TransactionPause(pallet_tx_pause::Event::<
                Runtime,
            >::TransactionPaused(
                PALLET_NAME.to_vec(),
                FUNCTION_NAME.to_vec(),
            )));
            assert_eq!(
                TransactionPause::paused_transactions((
                    PALLET_NAME.to_vec(),
                    FUNCTION_NAME.to_vec(),
                )),
                Some(())
            );
        } else {
            System::assert_has_event(RuntimeEvent::TransactionPause(pallet_tx_pause::Event::<
                Runtime,
            >::TransactionUnpaused(
                PALLET_NAME.to_vec(),
                FUNCTION_NAME.to_vec(),
            )));
            assert_eq!(
                TransactionPause::paused_transactions((
                    PALLET_NAME.to_vec(),
                    FUNCTION_NAME.to_vec(),
                )),
                None
            );
        }
    }
    fn pause_transactions_storage_event_works(pause: bool, pallet: bool) {
        let function_names: Vec<Vec<u8>> = vec![b"batch".to_vec(), b"batch_all".to_vec()];

        if pause {
            if pallet {
                System::assert_last_event(RuntimeEvent::TransactionPause(
                    pallet_tx_pause::Event::<Runtime>::PalletPaused(PALLET_NAME.to_vec()),
                ));
            } else {
                for function_name in function_names.clone() {
                    System::assert_has_event(RuntimeEvent::TransactionPause(
                        pallet_tx_pause::Event::<Runtime>::TransactionPaused(
                            PALLET_NAME.to_vec(),
                            function_name,
                        ),
                    ));
                }
            }
            for function_name in function_names {
                assert_eq!(
                    TransactionPause::paused_transactions((PALLET_NAME.to_vec(), function_name)),
                    Some(())
                );
            }
        } else {
            if pallet {
                System::assert_last_event(RuntimeEvent::TransactionPause(
                    pallet_tx_pause::Event::<Runtime>::PalletUnpaused(PALLET_NAME.to_vec()),
                ));
            } else {
                for function_name in function_names.clone() {
                    System::assert_has_event(RuntimeEvent::TransactionPause(
                        pallet_tx_pause::Event::<Runtime>::TransactionUnpaused(
                            PALLET_NAME.to_vec(),
                            function_name,
                        ),
                    ));
                }
            }
            for function_name in function_names {
                assert_eq!(
                    TransactionPause::paused_transactions((PALLET_NAME.to_vec(), function_name)),
                    None
                );
            }
        }
    }

    #[test]
    fn tx_pause_works() {
        let alice = unchecked_account_id::<sr25519::Public>("Alice");
        ExtBuilder::default().build().execute_with(|| {
            assert_noop!(
                TransactionPause::pause_transaction(
                    RuntimeOrigin::signed(alice),
                    b"Balances".to_vec(),
                    b"transfer".to_vec()
                ),
                BadOrigin
            );
            assert_noop!(
                TransactionPause::pause_pallets(root_origin(), vec![b"Balances".to_vec()]),
                pallet_tx_pause::Error::<Runtime>::CannotPause
            );
            assert_noop!(
                TransactionPause::pause_pallets(root_origin(), vec![b"TransactionPause".to_vec()]),
                pallet_tx_pause::Error::<Runtime>::CannotPause
            );

            let function_names: Vec<Vec<u8>> = vec![b"batch".to_vec(), b"batch_all".to_vec()];

            // pause transaction
            assert_ok!(TransactionPause::pause_transaction(
                root_origin(),
                PALLET_NAME.to_vec(),
                FUNCTION_NAME.to_vec(),
            ));
            pause_transaction_storage_event_works(true);

            // unpause transaction
            assert_ok!(TransactionPause::unpause_transaction(
                root_origin(),
                PALLET_NAME.to_vec(),
                FUNCTION_NAME.to_vec(),
            ));
            pause_transaction_storage_event_works(false);

            // pause transactions
            System::reset_events();
            assert_ok!(TransactionPause::pause_transactions(
                root_origin(),
                vec![(PALLET_NAME.to_vec(), function_names.clone())]
            ));
            pause_transactions_storage_event_works(true, false);

            // unpause transactions
            assert_ok!(TransactionPause::unpause_transactions(
                root_origin(),
                vec![(PALLET_NAME.to_vec(), function_names)]
            ));
            pause_transactions_storage_event_works(false, false);

            // pause pallet
            assert_ok!(TransactionPause::pause_pallets(
                root_origin(),
                vec![PALLET_NAME.to_vec()]
            ));
            pause_transactions_storage_event_works(true, true);

            // unpause pallet
            assert_ok!(TransactionPause::unpause_pallets(
                root_origin(),
                vec![PALLET_NAME.to_vec()]
            ));
            pause_transactions_storage_event_works(false, true);
        });
    }

    #[test]
    fn non_pausable_pallets_exist() {
        let all_pallets = AllPalletsWithSystem::infos();
        let all_pallet_names: Vec<&str> = all_pallets.into_iter().map(|info| info.name).collect();
        for pallet in NonPausablePallets::get() {
            let pallet_str = sp_std::str::from_utf8(&pallet).unwrap();
            assert!(all_pallet_names.contains(&pallet_str), "{pallet_str:?}");
        }
    }
}

#[test]
fn concrete_fungible_ledger_can_deposit_and_mint_works() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.clone(), INITIAL_BALANCE)])
        .build()
        .execute_with(|| {
            // Native asset tests:

            let new_account = unchecked_account_id::<sr25519::Public>("NewAccount");
            assert_err!(
                RuntimeConcreteFungibleLedger::can_deposit(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &new_account,
                    NativeTokenExistentialDeposit::get() - 1,
                    true,
                ),
                FungibleLedgerError::BelowMinimum
            );

            // Non-native asset tests:

            let min_balance = 10u128;
            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Kusama".to_vec(),
                    symbol: b"KSM".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: true,
            };
            let source_location =
                AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);

            assert_err!(
                RuntimeConcreteFungibleLedger::can_deposit(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    0,
                    true,
                ),
                FungibleLedgerError::BelowMinimum
            );
            assert_err!(
                RuntimeConcreteFungibleLedger::can_deposit(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get() + 1,
                    &ALICE.clone(),
                    11,
                    true,
                ),
                FungibleLedgerError::UnknownAsset
            );
            assert_ok!(RuntimeConcreteFungibleLedger::deposit_minting(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                u128::MAX,
            ),);
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                u128::MAX
            );
            assert_err!(
                RuntimeConcreteFungibleLedger::can_deposit(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    1,
                    true,
                ),
                FungibleLedgerError::Overflow
            );

            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Rococo".to_vec(),
                    symbol: b"Roc".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: false,
            };

            let source_location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X2(Parachain(1), PalletInstance(1)),
            )));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);
            assert_err!(
                RuntimeConcreteFungibleLedger::can_deposit(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get() + 1,
                    &XcmFeesAccount::get(),
                    11,
                    true,
                ),
                FungibleLedgerError::CannotCreate
            );
        });
}

// `can_withdraw` uses `reducible_amount` implementation in order to use the `keep_alive` argument.
// Unfortunately that function does not return the reason for failure cases like `can_withdraw`.
// The errors that would've been returned if `can_withdraw` was used instead of `reducible_amount`
// are included as comments on top of each case for more clarity.
#[test]
fn concrete_fungible_ledger_can_withdraw_works() {
    ExtBuilder::default()
        .with_balances(vec![(CHARLIE.clone(), INITIAL_BALANCE)])
        .build()
        .execute_with(|| {
            let existential_deposit = NativeTokenExistentialDeposit::get();

            // Native asset tests:

            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &CHARLIE.clone(),
                    &(INITIAL_BALANCE + 1),
                    ExistenceRequirement::KeepAlive
                ),
                // Underflow
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE - existential_deposit)
            );

            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &CHARLIE.clone(),
                    &INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                // WouldDie
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE - existential_deposit)
            );

            assert_ok!(RuntimeConcreteFungibleLedger::can_withdraw(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                &CHARLIE.clone(),
                &INITIAL_BALANCE,
                ExistenceRequirement::AllowDeath
            ),);

            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get(),
                    &BOB.clone(),
                    &INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                // NoFunds
                FungibleLedgerError::CannotWithdrawMoreThan(0)
            );

            // Non-native asset tests:

            let min_balance = 10u128;
            let asset_metadata = AssetRegistryMetadata {
                metadata: AssetStorageMetadata {
                    name: b"Kusama".to_vec(),
                    symbol: b"KSM".to_vec(),
                    decimals: 12,
                    is_frozen: false,
                },
                min_balance,
                is_sufficient: true,
            };
            let source_location =
                AssetLocation(VersionedMultiLocation::V1(MultiLocation::parent()));
            assert_ok!(AssetManager::register_asset(
                root_origin(),
                source_location,
                asset_metadata
            ),);

            assert_ok!(RuntimeConcreteFungibleLedger::deposit_minting(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                INITIAL_BALANCE,
            ),);
            assert_eq!(
                Assets::balance(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    ALICE.clone()
                ),
                INITIAL_BALANCE
            );

            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &(INITIAL_BALANCE + 1),
                    ExistenceRequirement::AllowDeath
                ),
                // Underflow
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE)
            );

            assert_ok!(RuntimeConcreteFungibleLedger::can_withdraw(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &ALICE.clone(),
                &INITIAL_BALANCE,
                ExistenceRequirement::AllowDeath
            ),);

            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &BOB.clone(),
                    &10u128,
                    ExistenceRequirement::AllowDeath
                ),
                // NoFunds
                FungibleLedgerError::CannotWithdrawMoreThan(0)
            );

            assert_ok!(RuntimeConcreteFungibleLedger::deposit_minting(
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                &BOB.clone(),
                INITIAL_BALANCE,
            ),);
            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &INITIAL_BALANCE,
                    ExistenceRequirement::KeepAlive
                ),
                FungibleLedgerError::CannotWithdrawMoreThan(INITIAL_BALANCE - min_balance)
            );

            assert_ok!(Assets::freeze(
                RuntimeOrigin::signed(AssetManager::account_id()),
                <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                sp_runtime::MultiAddress::Id(ALICE.clone()),
            ));
            assert_err!(
                RuntimeConcreteFungibleLedger::can_withdraw(
                    <RuntimeAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
                    &ALICE.clone(),
                    &10u128,
                    ExistenceRequirement::AllowDeath
                ),
                // Frozen
                FungibleLedgerError::CannotWithdrawMoreThan(0)
            );
        });
}

#[test]
fn test_receiver_side_weights() {
    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(
        &mut self_reserve_xcm_message_receiver_side::<RuntimeCall>(),
    )
    .unwrap();
    assert!(weight <= ADVERTISED_DEST_WEIGHT);

    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(
        &mut to_reserve_xcm_message_receiver_side::<RuntimeCall>(),
    )
    .unwrap();
    assert!(weight <= ADVERTISED_DEST_WEIGHT);
}

#[test]
fn test_sender_side_xcm_weights() {
    let mut msg = self_reserve_xcm_message_sender_side::<RuntimeCall>();
    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
    assert!(weight < ADVERTISED_DEST_WEIGHT);

    let mut msg = to_reserve_xcm_message_sender_side::<RuntimeCall>();
    let weight = <XcmExecutorConfig as xcm_executor::Config>::Weigher::weight(&mut msg).unwrap();
    assert!(weight < ADVERTISED_DEST_WEIGHT);
}

mod governance_tests {
    use super::{super::mock::ExtBuilder, *};
    use frame_support::{codec::Encode, traits::StorePreimage};
    use sp_core::H256;
    use sp_runtime::traits::{BlakeTwo256, Hash};

    enum ExternalType {
        SimpleMajority,
        NegativeTurnoutBias,
        PositiveTurnoutBias,
    }

    fn note_preimage(proposer: &AccountId, proposal_call: &RuntimeCall) -> H256 {
        let preimage = proposal_call.encode();
        let preimage_hash = BlakeTwo256::hash(&preimage[..]);
        assert_ok!(Preimage::note_preimage(
            RuntimeOrigin::signed(proposer.clone()),
            preimage
        ));
        preimage_hash
    }

    fn propose_council_motion(council_motion: &RuntimeCall, proposer: &AccountId) -> H256 {
        let council_motion_len: u32 = council_motion.using_encoded(|p| p.len() as u32);
        assert_ok!(Council::propose(
            RuntimeOrigin::signed(proposer.clone()),
            1,
            Box::new(council_motion.clone()),
            council_motion_len
        ));

        BlakeTwo256::hash_of(&council_motion)
    }

    fn start_external_proposal_governance_assertions(
        proposer: &AccountId,
        external_type: ExternalType,
    ) -> H256 {
        // Setup the preimage and preimage hash
        let runtime_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![0] });
        let preimage_hash = note_preimage(proposer, &runtime_call);
        let proposal = Preimage::bound(runtime_call).unwrap();

        // Setup the Council and Technical Committee
        assert_ok!(Council::set_members(
            root_origin(),
            vec![proposer.clone()],
            None,
            0
        ));
        assert_ok!(TechnicalCommittee::set_members(
            root_origin(),
            vec![proposer.clone()],
            None,
            0
        ));

        // Setup and propose the Council motion for external_propose_default routine
        // No voting required because there's only 1 seat.
        let council_motion = match external_type {
            ExternalType::NegativeTurnoutBias => {
                RuntimeCall::Democracy(pallet_democracy::Call::external_propose_default {
                    proposal,
                })
            }
            ExternalType::SimpleMajority => {
                RuntimeCall::Democracy(pallet_democracy::Call::external_propose_majority {
                    proposal,
                })
            }
            ExternalType::PositiveTurnoutBias => {
                RuntimeCall::Democracy(pallet_democracy::Call::external_propose { proposal })
            }
        };
        let council_motion_hash = propose_council_motion(&council_motion, proposer);

        assert_eq!(
            last_event(),
            RuntimeEvent::Council(pallet_collective::Event::Executed {
                proposal_hash: council_motion_hash,
                result: Ok(())
            })
        );

        preimage_hash
    }

    fn end_governance_assertions(
        referendum_index: u32,
        end_of_referendum: u32,
        enactment_period: u32,
    ) {
        let time_of_enactment = end_of_referendum + enactment_period;
        run_to_block(end_of_referendum - 1);
        assert_eq!(1, Democracy::referendum_count());

        // After the voting period the referendum ends and is scheduled for enactment:
        run_to_block(end_of_referendum);
        assert_eq!(
            last_event(),
            RuntimeEvent::Scheduler(pallet_scheduler::Event::Scheduled {
                when: time_of_enactment,
                index: referendum_index
            })
        );

        // After the enactment period the proposal is dispatched:
        run_to_block(time_of_enactment);
        assert_eq!(
            last_event(),
            RuntimeEvent::Scheduler(pallet_scheduler::Event::Dispatched {
                task: (time_of_enactment, referendum_index),
                id: Some([
                    100, 101, 109, 111, 99, 114, 97, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ]),
                result: Ok(())
            })
        );
    }

    fn assert_proposal_is_filtered(proposer: &AccountId, motion: &RuntimeCall) {
        let council_motion_hash = propose_council_motion(motion, proposer);

        assert_eq!(
            last_event(),
            RuntimeEvent::Council(pallet_collective::Event::Executed {
                proposal_hash: council_motion_hash,
                result: Err(DispatchError::Module(ModuleError {
                    index: 0,
                    error: [5, 0, 0, 0],
                    message: None
                }))
            })
        );
    }

    #[test]
    fn fast_track_available() {
        assert!(<Runtime as pallet_democracy::Config>::InstantAllowed::get());
    }

    #[test]
    fn sanity_check_governance_periods() {
        assert_eq!(LaunchPeriod::get(), 7 * DAYS);
        assert_eq!(VotingPeriod::get(), 7 * DAYS);
        assert_eq!(EnactmentPeriod::get(), DAYS);
    }

    #[test]
    fn slow_simple_majority_governance_works() {
        ExtBuilder::default().build().execute_with(|| {
            let _preimage_hash =
                start_external_proposal_governance_assertions(&ALICE, ExternalType::SimpleMajority);

            let start_of_referendum = LaunchPeriod::get();
            let referendum_index = 0;

            run_to_block(start_of_referendum - 1);
            assert_eq!(0, Democracy::referendum_count());

            // 7 days in the external proposal queue before the referendum starts.
            run_to_block(start_of_referendum);
            assert_eq!(
                last_event(),
                RuntimeEvent::Democracy(pallet_democracy::Event::Started {
                    ref_index: referendum_index,
                    threshold: pallet_democracy::VoteThreshold::SimpleMajority
                })
            );
            // Time to vote for the referendum with some amount
            assert_ok!(Democracy::vote(
                RuntimeOrigin::signed(ALICE.clone()),
                0,
                pallet_democracy::AccountVote::Standard {
                    vote: pallet_democracy::Vote {
                        aye: true,
                        conviction: pallet_democracy::Conviction::None
                    },
                    balance: 10 * UNIT
                }
            ));

            end_governance_assertions(
                referendum_index,
                start_of_referendum + VotingPeriod::get(),
                EnactmentPeriod::get(),
            );
        });
    }

    #[test]
    fn slow_negative_turnout_bias_governance_works() {
        ExtBuilder::default().build().execute_with(|| {
            let _preimage_hash = start_external_proposal_governance_assertions(
                &ALICE,
                ExternalType::NegativeTurnoutBias,
            );

            let start_of_referendum = LaunchPeriod::get();
            let referendum_index = 0;

            run_to_block(start_of_referendum - 1);
            assert_eq!(0, Democracy::referendum_count());

            // 7 days in the external proposal queue before the referendum starts.
            run_to_block(start_of_referendum);
            assert_eq!(1, Democracy::referendum_count());
            assert_eq!(
                last_event(),
                RuntimeEvent::Democracy(pallet_democracy::Event::Started {
                    ref_index: referendum_index,
                    threshold: pallet_democracy::VoteThreshold::SuperMajorityAgainst
                })
            );
            // Time to vote for the referendum with some amount
            assert_ok!(Democracy::vote(
                RuntimeOrigin::signed(ALICE.clone()),
                referendum_index,
                pallet_democracy::AccountVote::Standard {
                    vote: pallet_democracy::Vote {
                        aye: true,
                        conviction: pallet_democracy::Conviction::None
                    },
                    balance: 100 * UNIT
                }
            ));

            end_governance_assertions(
                referendum_index,
                start_of_referendum + VotingPeriod::get(),
                EnactmentPeriod::get(),
            );
        });
    }

    #[test]
    fn fast_track_governance_works() {
        ExtBuilder::default().build().execute_with(|| {
            let preimage_hash = start_external_proposal_governance_assertions(
                &ALICE,
                ExternalType::NegativeTurnoutBias,
            );

            let voting_period = 5;
            let enactment_period = 5;
            let referendum_index = 0;

            // Setup and propose the Technical Committee motion for the fast_track routine
            // No voting required because there's only 1 seat.
            // Voting and delay periods of 5 blocks so this should be enacted on block 11
            let tech_committee_motion =
                RuntimeCall::Democracy(pallet_democracy::Call::fast_track {
                    proposal_hash: preimage_hash,
                    voting_period,
                    delay: enactment_period,
                });
            let tech_committee_motion_len: u32 =
                tech_committee_motion.using_encoded(|p| p.len() as u32);
            let tech_committee_motion_hash = BlakeTwo256::hash_of(&tech_committee_motion);
            assert_ok!(TechnicalCommittee::propose(
                RuntimeOrigin::signed(ALICE.clone()),
                1,
                Box::new(tech_committee_motion),
                tech_committee_motion_len
            ));
            // Make sure the motion was actually executed
            assert_eq!(
                last_event(),
                RuntimeEvent::TechnicalCommittee(pallet_collective::Event::Executed {
                    proposal_hash: tech_committee_motion_hash,
                    result: Ok(())
                })
            );

            // Time to vote for the referendum with some amount
            assert_ok!(Democracy::vote(
                RuntimeOrigin::signed(ALICE.clone()),
                referendum_index,
                pallet_democracy::AccountVote::Standard {
                    vote: pallet_democracy::Vote {
                        aye: true,
                        conviction: pallet_democracy::Conviction::None
                    },
                    balance: 10 * UNIT
                }
            ));

            // No launch period because of the fast track.
            end_governance_assertions(
                referendum_index,
                System::block_number() + voting_period,
                enactment_period,
            );
        });
    }

    #[test]
    fn governance_filters_work() {
        assert!(<Runtime as pallet_democracy::Config>::InstantAllowed::get());

        ExtBuilder::default().build().execute_with(|| {
            // Setup the preimage and preimage hash
            let runtime_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![0] });
            let proposal = Preimage::bound(runtime_call).unwrap();

            // Setup the Council
            assert_ok!(Council::set_members(
                root_origin(),
                vec![ALICE.clone()],
                None,
                0
            ));

            // Public proposals should be filtered out.
            assert_proposal_is_filtered(
                &ALICE,
                &RuntimeCall::Democracy(pallet_democracy::Call::propose {
                    proposal,
                    value: 100 * UNIT,
                }),
            );
        });
    }

    #[test]
    fn slow_positive_turnout_bias_governance_works() {
        ExtBuilder::default().build().execute_with(|| {
            let _preimage_hash = start_external_proposal_governance_assertions(
                &ALICE,
                ExternalType::PositiveTurnoutBias,
            );

            let start_of_referendum = LaunchPeriod::get();
            let referendum_index = 0;

            run_to_block(start_of_referendum - 1);
            assert_eq!(0, Democracy::referendum_count());

            // 7 days in the external proposal queue before the referendum starts.
            run_to_block(start_of_referendum);
            assert_eq!(
                last_event(),
                RuntimeEvent::Democracy(pallet_democracy::Event::Started {
                    ref_index: referendum_index,
                    threshold: pallet_democracy::VoteThreshold::SuperMajorityApprove
                })
            );
            // Time to vote for the referendum with some amount
            assert_ok!(Democracy::vote(
                RuntimeOrigin::signed(ALICE.clone()),
                0,
                pallet_democracy::AccountVote::Standard {
                    vote: pallet_democracy::Vote {
                        aye: true,
                        conviction: pallet_democracy::Conviction::None
                    },
                    balance: 10 * UNIT
                }
            ));

            end_governance_assertions(
                referendum_index,
                start_of_referendum + VotingPeriod::get(),
                EnactmentPeriod::get(),
            );
        });
    }

    #[test]
    fn asset_manager_filters_outgoing_assets_with_council() {
        ExtBuilder::default().build().execute_with(|| {
            // Setup the preimage and preimage hash
            let runtime_call = RuntimeCall::AssetManager(
                pallet_asset_manager::Call::update_outgoing_filtered_assets {
                    filtered_location: MultiLocation::default().into(),
                    should_add: true,
                },
            );

            assert_ok!(Council::set_members(
                root_origin(),
                vec![ALICE.clone()],
                None,
                0
            ));
            let council_motion_hash = propose_council_motion(&runtime_call, &ALICE);

            assert_eq!(
                last_event(),
                RuntimeEvent::Council(pallet_collective::Event::Executed {
                    proposal_hash: council_motion_hash,
                    result: Ok(())
                })
            );
        });
    }
}
