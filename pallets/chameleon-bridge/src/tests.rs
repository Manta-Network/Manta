// Copyright 2020-2024 Manta Network.
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

//! Tests for the Chameleon Bridge Pallet

use crate::{
    mock::*,
    types::*,
    Error, Event,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Get, OnFinalize, OnInitialize},
};
use sp_core::{H160, H256};
use sp_runtime::traits::Zero;

/// Run to block n
fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            ChameleonBridge::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        ChameleonBridge::on_initialize(System::block_number());
    }
}

/// Create a test deposit
fn create_test_deposit() -> BridgeDeposit<AccountId> {
    BridgeDeposit {
        eth_tx_hash: H256::from_low_u64_be(1),
        eth_address: H160::from_low_u64_be(1),
        recipient: 1,
        asset: BridgeableAsset::ETH,
        amount: 1_000_000_000_000_000_000, // 1 ETH
        eth_block_number: 100,
        confirmations_required: 12,
        confirmations: 0,
        status: DepositStatus::Pending,
        validator_confirmations: vec![],
        reported_at: 1,
    }
}

/// Create a test withdrawal
fn create_test_withdrawal() -> BridgeWithdrawal<AccountId> {
    BridgeWithdrawal {
        withdrawal_id: 1,
        from: 1,
        eth_destination: H160::from_low_u64_be(1),
        asset: BridgeableAsset::ETH,
        amount: 1_000_000_000_000_000_000, // 1 ETH
        signatures: vec![],
        signatures_required: 5,
        status: WithdrawalStatus::Pending,
        initiated_at: 1,
        fee_paid: 100_000_000_000_000_000, // 0.1 CHML
    }
}

#[test]
fn report_lock_event_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let tx_hash = H256::from_low_u64_be(1);
        let eth_address = H160::from_low_u64_be(1);
        let recipient = 1;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000; // 1 ETH
        
        // Add validator first
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1,
            H160::from_low_u64_be(1)
        ));
        
        // Report lock event
        assert_ok!(ChameleonBridge::report_lock_event(
            RuntimeOrigin::signed(1),
            tx_hash,
            eth_address,
            recipient,
            asset.clone(),
            amount
        ));
        
        // Check that deposit was created
        let deposit = ChameleonBridge::pending_deposits(tx_hash).unwrap();
        assert_eq!(deposit.eth_address, eth_address);
        assert_eq!(deposit.recipient, recipient);
        assert_eq!(deposit.asset, asset);
        assert_eq!(deposit.amount, amount);
        assert_eq!(deposit.status, DepositStatus::Pending);
        
        // Check event was emitted
        System::assert_has_event(
            Event::LockEventReported {
                tx_hash,
                validator: 1,
                recipient,
                asset,
                amount,
            }.into()
        );
    });
}

#[test]
fn report_lock_event_fails_for_non_validator() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let tx_hash = H256::from_low_u64_be(1);
        let eth_address = H160::from_low_u64_be(1);
        let recipient = 1;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000;
        
        // Try to report without being a validator
        assert_noop!(
            ChameleonBridge::report_lock_event(
                RuntimeOrigin::signed(1),
                tx_hash,
                eth_address,
                recipient,
                asset,
                amount
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn burn_for_unlock_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let asset_id = BridgeableAsset::ETH.wrapped_asset_id();
        let amount = 1_000_000_000_000_000_000; // 1 wETH
        let eth_destination = H160::from_low_u64_be(1);
        
        // First mint some wrapped tokens to the user
        assert_ok!(Assets::force_create(
            RuntimeOrigin::root(),
            asset_id.into(),
            1, // owner
            true, // is_sufficient
            1 // min_balance
        ));
        
        assert_ok!(Assets::mint(
            RuntimeOrigin::signed(1),
            asset_id.into(),
            1, // beneficiary
            amount
        ));
        
        // Burn for unlock
        assert_ok!(ChameleonBridge::burn_for_unlock(
            RuntimeOrigin::signed(1),
            BridgeableAsset::ETH,
            amount,
            eth_destination
        ));
        
        // Check that withdrawal was created
        let withdrawal = ChameleonBridge::pending_withdrawals(1).unwrap();
        assert_eq!(withdrawal.from, 1);
        assert_eq!(withdrawal.asset, BridgeableAsset::ETH);
        assert_eq!(withdrawal.amount, amount);
        assert_eq!(withdrawal.eth_destination, eth_destination);
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        
        // Check that tokens were burned
        assert_eq!(Assets::balance(asset_id, &1), 0);
        
        // Check event was emitted
        System::assert_has_event(
            Event::WithdrawalInitiated {
                withdrawal_id: 1,
                from: 1,
                asset: BridgeableAsset::ETH,
                amount,
                eth_destination,
                fee: withdrawal.fee_paid,
            }.into()
        );
    });
}

#[test]
fn burn_for_unlock_fails_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let amount = 1_000_000_000_000_000_000; // 1 wETH
        let eth_destination = H160::from_low_u64_be(1);
        
        // Try to burn without having tokens
        assert_noop!(
            ChameleonBridge::burn_for_unlock(
                RuntimeOrigin::signed(1),
                BridgeableAsset::ETH,
                amount,
                eth_destination
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn sign_withdrawal_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        // Create a withdrawal
        let mut withdrawal = create_test_withdrawal();
        ChameleonBridge::insert_pending_withdrawal(1, withdrawal.clone());
        
        // Add validators
        for i in 1..=9 {
            assert_ok!(ChameleonBridge::add_validator(
                RuntimeOrigin::root(),
                i,
                H160::from_low_u64_be(i)
            ));
        }
        
        // Sign withdrawal with 5 validators (threshold)
        for i in 1..=5 {
            assert_ok!(ChameleonBridge::sign_withdrawal(
                RuntimeOrigin::signed(i),
                1,
                vec![0u8; 65] // dummy signature
            ));
        }
        
        // Check that withdrawal is ready to execute
        let updated_withdrawal = ChameleonBridge::pending_withdrawals(1).unwrap();
        assert_eq!(updated_withdrawal.status, WithdrawalStatus::ReadyToExecute);
        assert_eq!(updated_withdrawal.signatures.len(), 5);
    });
}

#[test]
fn sign_withdrawal_fails_for_non_validator() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        // Create a withdrawal
        let withdrawal = create_test_withdrawal();
        ChameleonBridge::insert_pending_withdrawal(1, withdrawal);
        
        // Try to sign without being a validator
        assert_noop!(
            ChameleonBridge::sign_withdrawal(
                RuntimeOrigin::signed(1),
                1,
                vec![0u8; 65]
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn add_validator_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let account = 1;
        let eth_address = H160::from_low_u64_be(1);
        
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            account,
            eth_address
        ));
        
        // Check validator was added
        let validator = ChameleonBridge::bridge_validators(account).unwrap();
        assert_eq!(validator.account, account);
        assert_eq!(validator.eth_address, eth_address);
        assert!(validator.is_active);
        
        // Check reputation was initialized
        let reputation = ChameleonBridge::validator_reputations(account).unwrap();
        assert_eq!(reputation.score, 50); // Default score
        assert!(reputation.is_trusted);
        
        // Check event was emitted
        System::assert_has_event(
            Event::ValidatorAdded {
                account,
                eth_address,
            }.into()
        );
    });
}

#[test]
fn remove_validator_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let account = 1;
        let eth_address = H160::from_low_u64_be(1);
        
        // Add validator first
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            account,
            eth_address
        ));
        
        // Remove validator
        assert_ok!(ChameleonBridge::remove_validator(
            RuntimeOrigin::root(),
            account
        ));
        
        // Check validator was deactivated
        let validator = ChameleonBridge::bridge_validators(account).unwrap();
        assert!(!validator.is_active);
        
        // Check event was emitted
        System::assert_has_event(
            Event::ValidatorRemoved {
                account,
            }.into()
        );
    });
}

#[test]
fn pause_and_resume_bridge_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        // Pause bridge
        assert_ok!(ChameleonBridge::pause_bridge(RuntimeOrigin::root()));
        
        let config = ChameleonBridge::bridge_config();
        assert!(config.is_paused);
        
        // Check event was emitted
        System::assert_has_event(Event::BridgePaused.into());
        
        // Resume bridge
        assert_ok!(ChameleonBridge::resume_bridge(RuntimeOrigin::root()));
        
        let config = ChameleonBridge::bridge_config();
        assert!(!config.is_paused);
        
        // Check event was emitted
        System::assert_has_event(Event::BridgeResumed.into());
    });
}

#[test]
fn bridge_operations_fail_when_paused() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        // Pause bridge
        assert_ok!(ChameleonBridge::pause_bridge(RuntimeOrigin::root()));
        
        // Add validator first (this should still work)
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1,
            H160::from_low_u64_be(1)
        ));
        
        // Try to report lock event (should fail)
        assert_noop!(
            ChameleonBridge::report_lock_event(
                RuntimeOrigin::signed(1),
                H256::from_low_u64_be(1),
                H160::from_low_u64_be(1),
                1,
                BridgeableAsset::ETH,
                1_000_000_000_000_000_000
            ),
            Error::<Test>::BridgePaused
        );
        
        // Try to burn for unlock (should fail)
        assert_noop!(
            ChameleonBridge::burn_for_unlock(
                RuntimeOrigin::signed(1),
                BridgeableAsset::ETH,
                1_000_000_000_000_000_000,
                H160::from_low_u64_be(1)
            ),
            Error::<Test>::BridgePaused
        );
    });
}

#[test]
fn fee_calculation_works() {
    new_test_ext().execute_with(|| {
        use crate::types::{DefaultBridgeFeeCalculator, BridgeFeeCalculator};
        
        let amount = 1_000_000_000_000_000_000; // 1 ETH equivalent in CHML
        
        // Test shielding fee
        let shielding_fee = DefaultBridgeFeeCalculator::calculate_shielding_fee(amount);
        assert!(shielding_fee.total_fee > 0);
        assert_eq!(shielding_fee.bridge_ops_fee + shielding_fee.treasury_fee, shielding_fee.total_fee);
        assert!(shielding_fee.bridge_ops_fee > shielding_fee.treasury_fee); // 70% vs 30%
        
        // Test unshielding fee
        let unshielding_fee = DefaultBridgeFeeCalculator::calculate_unshielding_fee(amount);
        assert!(unshielding_fee.total_fee > shielding_fee.total_fee); // Higher fee for unshielding
        assert_eq!(unshielding_fee.bridge_ops_fee + unshielding_fee.treasury_fee, unshielding_fee.total_fee);
        
        // Test minimum fee
        let small_amount = 1000; // Very small amount
        let min_shielding_fee = DefaultBridgeFeeCalculator::calculate_shielding_fee(small_amount);
        assert_eq!(min_shielding_fee.total_fee, 100_000_000_000_000_000); // Should be minimum fee
    });
}

#[test]
fn deposit_confirmation_threshold_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let tx_hash = H256::from_low_u64_be(1);
        let eth_address = H160::from_low_u64_be(1);
        let recipient = 1;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000;
        
        // Add 9 validators
        for i in 1..=9 {
            assert_ok!(ChameleonBridge::add_validator(
                RuntimeOrigin::root(),
                i,
                H160::from_low_u64_be(i)
            ));
        }
        
        // Report lock event with 4 validators (below threshold)
        for i in 1..=4 {
            assert_ok!(ChameleonBridge::report_lock_event(
                RuntimeOrigin::signed(i),
                tx_hash,
                eth_address,
                recipient,
                asset.clone(),
                amount
            ));
        }
        
        // Should still be pending
        let deposit = ChameleonBridge::pending_deposits(tx_hash).unwrap();
        assert_eq!(deposit.status, DepositStatus::Pending);
        assert_eq!(deposit.validator_confirmations.len(), 4);
        
        // Report with 5th validator (reaches threshold)
        assert_ok!(ChameleonBridge::report_lock_event(
            RuntimeOrigin::signed(5),
            tx_hash,
            eth_address,
            recipient,
            asset.clone(),
            amount
        ));
        
        // Should now be confirmed
        let deposit = ChameleonBridge::pending_deposits(tx_hash).unwrap();
        assert_eq!(deposit.status, DepositStatus::Confirmed);
        assert_eq!(deposit.validator_confirmations.len(), 5);
    });
}

#[test]
fn duplicate_validator_confirmation_ignored() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        let tx_hash = H256::from_low_u64_be(1);
        let eth_address = H160::from_low_u64_be(1);
        let recipient = 1;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000;
        
        // Add validator
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1,
            H160::from_low_u64_be(1)
        ));
        
        // Report lock event twice from same validator
        assert_ok!(ChameleonBridge::report_lock_event(
            RuntimeOrigin::signed(1),
            tx_hash,
            eth_address,
            recipient,
            asset.clone(),
            amount
        ));
        
        assert_ok!(ChameleonBridge::report_lock_event(
            RuntimeOrigin::signed(1),
            tx_hash,
            eth_address,
            recipient,
            asset.clone(),
            amount
        ));
        
        // Should only have one confirmation
        let deposit = ChameleonBridge::pending_deposits(tx_hash).unwrap();
        assert_eq!(deposit.validator_confirmations.len(), 1);
    });
}

#[test]
fn withdrawal_signature_threshold_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        
        // Create a withdrawal
        let withdrawal = create_test_withdrawal();
        ChameleonBridge::insert_pending_withdrawal(1, withdrawal);
        
        // Add 9 validators
        for i in 1..=9 {
            assert_ok!(ChameleonBridge::add_validator(
                RuntimeOrigin::root(),
                i,
                H160::from_low_u64_be(i)
            ));
        }
        
        // Sign with 4 validators (below threshold)
        for i in 1..=4 {
            assert_ok!(ChameleonBridge::sign_withdrawal(
                RuntimeOrigin::signed(i),
                1,
                vec![0u8; 65]
            ));
        }
        
        // Should still be pending
        let withdrawal = ChameleonBridge::pending_withdrawals(1).unwrap();
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        assert_eq!(withdrawal.signatures.len(), 4);
        
        // Sign with 5th validator (reaches threshold)
        assert_ok!(ChameleonBridge::sign_withdrawal(
            RuntimeOrigin::signed(5),
            1,
            vec![0u8; 65]
        ));
        
        // Should now be ready to execute
        let withdrawal = ChameleonBridge::pending_withdrawals(1).unwrap();
        assert_eq!(withdrawal.status, WithdrawalStatus::ReadyToExecute);
        assert_eq!(withdrawal.signatures.len(), 5);
    });
}
