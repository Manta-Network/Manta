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

//! Calamari Parachain Integration Tests.

#![cfg(test)]
#![allow(clippy::identity_op)] // keep e.g. 1 * DAYS for legibility

use super::{mock::*, *};
use frame_support::{
    assert_err, assert_noop, assert_ok,
    traits::{tokens::ExistenceRequirement, Get, PalletInfo},
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

// currently, we ignore all parachain staking tests in integration tests
mod parachain_staking_tests {
    use super::*;

    #[test]
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
    fn collator_with_large_stake_but_too_low_self_bond_not_selected_for_block_production() {
        ExtBuilder::default()
            .with_balances(vec![
                (ALICE.clone(), EARLY_COLLATOR_MINIMUM_STAKE + 100),
                (BOB.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (CHARLIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (DAVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (EVE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (FERDIE.clone(), MIN_BOND_TO_BE_CONSIDERED_COLLATOR + 100),
                (USER.clone(), 400_000_000 * KMA),
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
                        100_000_000 * KMA,
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
    #[ignore]
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
    #[ignore]
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
            let transfer_amount = 10 * KMA;

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
            let transfer_amount = 10 * KMA;
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
