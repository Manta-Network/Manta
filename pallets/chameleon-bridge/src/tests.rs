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

//! Tests for the Chameleon Bridge pallet

use crate::{
    mock::*,
    BridgeableAsset, DepositStatus, WithdrawalStatus,
    Error, Event,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::Get,
};
use sp_core::{H160, H256};

#[test]
fn add_validator_works() {
    new_test_ext().execute_with(|| {
        // Add validator
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        // Check validator was added
        assert!(ChameleonBridge::is_validator(&1));
        
        // Check event was emitted
        System::assert_last_event(Event::ValidatorAdded { validator: 1 }.into());
    });
}

#[test]
fn add_duplicate_validator_fails() {
    new_test_ext().execute_with(|| {
        // Add validator
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        // Try to add same validator again
        assert_noop!(
            ChameleonBridge::add_validator(
                RuntimeOrigin::root(),
                1
            ),
            Error::<Test>::AlreadyProcessed
        );
    });
}

#[test]
fn remove_validator_works() {
    new_test_ext().execute_with(|| {
        // Add validator first
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        // Remove validator
        assert_ok!(ChameleonBridge::remove_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        // Check validator was removed
        assert!(!ChameleonBridge::is_validator(&1));
        
        // Check event was emitted
        System::assert_last_event(Event::ValidatorRemoved { validator: 1 }.into());
    });
}

#[test]
fn mint_wrapped_asset_works() {
    new_test_ext().execute_with(|| {
        // Add validator
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        let eth_tx_hash = H256::from([1u8; 32]);
        let recipient = 2;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        
        // Mint wrapped asset
        assert_ok!(ChameleonBridge::mint_wrapped_asset(
            RuntimeOrigin::signed(1),
            eth_tx_hash,
            asset.clone(),
            amount,
            recipient
        ));
        
        // Check deposit was recorded
        let deposit = ChameleonBridge::get_pending_deposit(eth_tx_hash).unwrap();
        assert_eq!(deposit.recipient, recipient);
        assert_eq!(deposit.asset, asset);
        assert_eq!(deposit.amount, amount);
        assert_eq!(deposit.status, DepositStatus::Confirmed);
        
        // Check event was emitted
        System::assert_last_event(Event::AssetMinted {
            recipient,
            asset,
            amount,
        }.into());
    });
}

#[test]
fn mint_wrapped_asset_non_validator_fails() {
    new_test_ext().execute_with(|| {
        let eth_tx_hash = H256::from([1u8; 32]);
        let recipient = 2;
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        
        // Try to mint without being validator
        assert_noop!(
            ChameleonBridge::mint_wrapped_asset(
                RuntimeOrigin::signed(1),
                eth_tx_hash,
                asset,
                amount,
                recipient
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn initiate_withdrawal_works() {
    new_test_ext().execute_with(|| {
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        let eth_destination = H160::from([1u8; 20]);
        
        // Initiate withdrawal
        assert_ok!(ChameleonBridge::initiate_withdrawal(
            RuntimeOrigin::signed(1),
            asset.clone(),
            amount,
            eth_destination
        ));
        
        // Check withdrawal was recorded
        let withdrawal = ChameleonBridge::get_pending_withdrawal(0).unwrap();
        assert_eq!(withdrawal.from, 1);
        assert_eq!(withdrawal.asset, asset);
        assert_eq!(withdrawal.amount, amount);
        assert_eq!(withdrawal.eth_destination, eth_destination);
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        assert_eq!(withdrawal.signature_count, 0);
        
        // Check event was emitted
        System::assert_last_event(Event::WithdrawalInitiated {
            withdrawal_id: 0,
            from: 1,
            eth_destination,
            asset,
            amount,
        }.into());
    });
}

#[test]
fn sign_withdrawal_works() {
    new_test_ext().execute_with(|| {
        // Add validator
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        let eth_destination = H160::from([1u8; 20]);
        
        // Initiate withdrawal
        assert_ok!(ChameleonBridge::initiate_withdrawal(
            RuntimeOrigin::signed(2),
            asset,
            amount,
            eth_destination
        ));
        
        let signature = vec![1, 2, 3, 4]; // Mock signature
        
        // Sign withdrawal
        assert_ok!(ChameleonBridge::sign_withdrawal(
            RuntimeOrigin::signed(1),
            0,
            signature.clone()
        ));
        
        // Check signature was recorded
        let stored_signature = ChameleonBridge::get_withdrawal_signature(0, &1).unwrap();
        assert_eq!(stored_signature, signature);
        
        // Check signature count increased
        assert_eq!(ChameleonBridge::count_withdrawal_signatures(0), 1);
        
        // Check event was emitted
        System::assert_last_event(Event::WithdrawalSigned {
            withdrawal_id: 0,
            validator: 1,
        }.into());
    });
}

#[test]
fn sign_withdrawal_non_validator_fails() {
    new_test_ext().execute_with(|| {
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        let eth_destination = H160::from([1u8; 20]);
        
        // Initiate withdrawal
        assert_ok!(ChameleonBridge::initiate_withdrawal(
            RuntimeOrigin::signed(1),
            asset,
            amount,
            eth_destination
        ));
        
        let signature = vec![1, 2, 3, 4]; // Mock signature
        
        // Try to sign without being validator
        assert_noop!(
            ChameleonBridge::sign_withdrawal(
                RuntimeOrigin::signed(2),
                0,
                signature
            ),
            Error::<Test>::NotValidator
        );
    });
}

#[test]
fn pause_and_resume_bridge_works() {
    new_test_ext().execute_with(|| {
        // Pause bridge
        assert_ok!(ChameleonBridge::pause_bridge(RuntimeOrigin::root()));
        assert!(ChameleonBridge::is_paused());
        System::assert_last_event(Event::BridgePaused.into());
        
        // Resume bridge
        assert_ok!(ChameleonBridge::resume_bridge(RuntimeOrigin::root()));
        assert!(!ChameleonBridge::is_paused());
        System::assert_last_event(Event::BridgeResumed.into());
    });
}

#[test]
fn operations_fail_when_paused() {
    new_test_ext().execute_with(|| {
        // Add validator
        assert_ok!(ChameleonBridge::add_validator(
            RuntimeOrigin::root(),
            1
        ));
        
        // Pause bridge
        assert_ok!(ChameleonBridge::pause_bridge(RuntimeOrigin::root()));
        
        let eth_tx_hash = H256::from([1u8; 32]);
        let asset = BridgeableAsset::ETH;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH
        let eth_destination = H160::from([1u8; 20]);
        
        // Try to mint when paused
        assert_noop!(
            ChameleonBridge::mint_wrapped_asset(
                RuntimeOrigin::signed(1),
                eth_tx_hash,
                asset.clone(),
                amount,
                2
            ),
            Error::<Test>::BridgePaused
        );
        
        // Try to initiate withdrawal when paused
        assert_noop!(
            ChameleonBridge::initiate_withdrawal(
                RuntimeOrigin::signed(1),
                asset,
                amount,
                eth_destination
            ),
            Error::<Test>::BridgePaused
        );
    });
}

#[test]
fn calculate_fee_works() {
    new_test_ext().execute_with(|| {
        let amount = 1_000_000u128; // 1 USDC
        
        // Shield fee (0.02%)
        let shield_fee = ChameleonBridge::calculate_fee(amount, true);
        assert_eq!(shield_fee, 200u128); // 0.02% of 1,000,000
        
        // Unshield fee (0.05%)
        let unshield_fee = ChameleonBridge::calculate_fee(amount, false);
        assert_eq!(unshield_fee, 500u128); // 0.05% of 1,000,000
    });
}

#[test]
fn min_amounts_are_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(ChameleonBridge::get_min_amount(&BridgeableAsset::ETH), 1_000_000_000_000_000u128);
        assert_eq!(ChameleonBridge::get_min_amount(&BridgeableAsset::USDC), 1_000_000u128);
        assert_eq!(ChameleonBridge::get_min_amount(&BridgeableAsset::USDT), 1_000_000u128);
        assert_eq!(ChameleonBridge::get_min_amount(&BridgeableAsset::WBTC), 10_000u128);
    });
}

#[test]
fn asset_validation_works() {
    new_test_ext().execute_with(|| {
        assert!(ChameleonBridge::is_valid_asset(&BridgeableAsset::ETH));
        assert!(ChameleonBridge::is_valid_asset(&BridgeableAsset::USDC));
        assert!(ChameleonBridge::is_valid_asset(&BridgeableAsset::USDT));
        assert!(ChameleonBridge::is_valid_asset(&BridgeableAsset::WBTC));
    });
}