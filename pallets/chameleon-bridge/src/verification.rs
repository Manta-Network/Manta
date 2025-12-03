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

//! Transaction Verification
//!
//! This module handles verification of bridge transactions,
//! including multi-signature validation and fraud proof mechanisms.

use crate::types::{*, FraudProof, FraudType};
use codec::{Decode, Encode};
use manta_primitives::types::{Balance, BlockNumber};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{traits::Saturating, RuntimeDebug};
use sp_std::vec::Vec;

/// Multi-signature verification result
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MultiSigVerificationResult {
    /// Verification successful
    Success,
    /// Insufficient signatures
    InsufficientSignatures,
    /// Invalid signature detected
    InvalidSignature,
    /// Duplicate signature detected
    DuplicateSignature,
    /// Unknown validator
    UnknownValidator,
}

// Fraud proof types moved to types.rs

/// Multi-signature validator for bridge operations
pub struct MultiSigValidator<T: frame_system::Config> {
    _phantom: sp_std::marker::PhantomData<T>,
}

impl<T: frame_system::Config> MultiSigValidator<T> {
    /// Verify multi-signature for withdrawal
    pub fn verify_withdrawal_signatures<AccountId: Clone + PartialEq>(
        withdrawal: &BridgeWithdrawal<AccountId>,
        validators: &[BridgeValidator<AccountId>],
        threshold: u32,
    ) -> MultiSigVerificationResult {
        // Check if we have enough signatures
        if withdrawal.signatures.len() < threshold as usize {
            return MultiSigVerificationResult::InsufficientSignatures;
        }
        
        let mut verified_validators = Vec::new();
        
        // Verify each signature
        for sig in &withdrawal.signatures {
            // Find the validator
            let validator = match validators.iter().find(|v| v.account == sig.validator) {
                Some(v) => v,
                None => return MultiSigVerificationResult::UnknownValidator,
            };
            
            // Check if validator is active
            if !validator.is_active {
                return MultiSigVerificationResult::UnknownValidator;
            }
            
            // Check for duplicate signatures
            if verified_validators.contains(&sig.validator) {
                return MultiSigVerificationResult::DuplicateSignature;
            }
            
            // Verify the signature
            let message = Self::create_withdrawal_message(withdrawal);
            if !Self::verify_signature(&message, &sig.signature, &validator.eth_address) {
                return MultiSigVerificationResult::InvalidSignature;
            }
            
            verified_validators.push(sig.validator.clone());
        }
        
        // Check if we have enough valid signatures
        if verified_validators.len() >= threshold as usize {
            MultiSigVerificationResult::Success
        } else {
            MultiSigVerificationResult::InsufficientSignatures
        }
    }
    
    /// Create message to be signed for withdrawal
    fn create_withdrawal_message<AccountId>(
        withdrawal: &BridgeWithdrawal<AccountId>,
    ) -> Vec<u8> {
        let mut message = Vec::new();
        
        // Include withdrawal ID
        message.extend_from_slice(&withdrawal.withdrawal_id.to_be_bytes());
        
        // Include destination address
        message.extend_from_slice(withdrawal.eth_destination.as_bytes());
        
        // Include asset address
        message.extend_from_slice(withdrawal.asset.ethereum_address().as_bytes());
        
        // Include amount
        message.extend_from_slice(&withdrawal.amount.to_be_bytes());
        
        message
    }
    
    /// Verify a single signature (placeholder implementation)
    fn verify_signature(
        message: &[u8],
        signature: &[u8],
        signer: &EthereumAddress,
    ) -> bool {
        // TODO: Implement actual ECDSA signature verification
        // This would use secp256k1 to verify the signature
        // For now, return true for testing
        true
    }
    
    /// Verify deposit confirmation from validators
    pub fn verify_deposit_confirmations<AccountId: Clone + PartialEq>(
        deposit: &BridgeDeposit<AccountId>,
        validators: &[BridgeValidator<AccountId>],
        threshold: u32,
    ) -> bool {
        // Check if we have enough validator confirmations
        if deposit.validator_confirmations.len() < threshold as usize {
            return false;
        }
        
        // Verify all confirming validators are active
        let active_validators: Vec<_> = validators
            .iter()
            .filter(|v| v.is_active)
            .map(|v| &v.account)
            .collect();
        
        let mut confirmed_by_active = 0;
        for confirming_validator in &deposit.validator_confirmations {
            if active_validators.contains(confirming_validator) {
                confirmed_by_active += 1;
            }
        }
        
        confirmed_by_active >= threshold
    }
}

/// Fraud proof validator
pub struct FraudProofValidator<T: frame_system::Config> {
    _phantom: sp_std::marker::PhantomData<T>,
}

impl<T: frame_system::Config> FraudProofValidator<T> {
    /// Validate a fraud proof submission
    pub fn validate_fraud_proof<AccountId>(
        proof: &FraudProof<AccountId>,
        challenge_period: BlockNumber,
        current_block: BlockNumber,
    ) -> Result<(), BridgeError> {
        // Check if we're still within the challenge period
        let blocks_since_submission = current_block.saturating_sub(proof.submitted_at);
        if blocks_since_submission > challenge_period {
            return Err(BridgeError::InvalidSignature); // Challenge period expired
        }
        
        // Validate the fraud proof based on type
        match proof.fraud_type {
            FraudType::InvalidEthereumTx => {
                Self::validate_invalid_ethereum_tx_proof(proof)
            },
            FraudType::DoubleSpending => {
                Self::validate_double_spending_proof(proof)
            },
            FraudType::MaliciousValidator => {
                Self::validate_malicious_validator_proof(proof)
            },
            FraudType::InvalidSignature => {
                Self::validate_invalid_signature_proof(proof)
            },
            FraudType::UnauthorizedWithdrawal => {
                Self::validate_unauthorized_withdrawal_proof(proof)
            },
        }
    }
    
    /// Validate proof of invalid Ethereum transaction
    fn validate_invalid_ethereum_tx_proof<AccountId>(
        proof: &FraudProof<AccountId>,
    ) -> Result<(), BridgeError> {
        // TODO: Implement validation logic
        // This would verify that the claimed Ethereum transaction
        // either doesn't exist or has different data than claimed
        Ok(())
    }
    
    /// Validate proof of double spending
    fn validate_double_spending_proof<AccountId>(
        proof: &FraudProof<AccountId>,
    ) -> Result<(), BridgeError> {
        // TODO: Implement validation logic
        // This would check if the same Ethereum transaction
        // was used to mint tokens multiple times
        Ok(())
    }
    
    /// Validate proof of malicious validator behavior
    fn validate_malicious_validator_proof<AccountId>(
        proof: &FraudProof<AccountId>,
    ) -> Result<(), BridgeError> {
        // TODO: Implement validation logic
        // This would verify evidence of validator misbehavior
        Ok(())
    }
    
    /// Validate proof of invalid signature
    fn validate_invalid_signature_proof<AccountId>(
        proof: &FraudProof<AccountId>,
    ) -> Result<(), BridgeError> {
        // TODO: Implement validation logic
        // This would verify that a signature is invalid
        Ok(())
    }
    
    /// Validate proof of unauthorized withdrawal
    fn validate_unauthorized_withdrawal_proof<AccountId>(
        proof: &FraudProof<AccountId>,
    ) -> Result<(), BridgeError> {
        // TODO: Implement validation logic
        // This would verify that a withdrawal was not properly authorized
        Ok(())
    }
    
    /// Calculate reward for successful fraud proof
    pub fn calculate_fraud_proof_reward(
        fraud_type: &FraudType,
        transaction_amount: Balance,
    ) -> Balance {
        use manta_primitives::chameleon_constants::fees::UNSHIELDING_MIN_FEE;
        
        // Base reward is minimum unshielding fee
        let base_reward = UNSHIELDING_MIN_FEE;
        
        // Additional reward based on fraud type and transaction amount
        let additional_reward = match fraud_type {
            FraudType::InvalidEthereumTx => transaction_amount / 1000, // 0.1%
            FraudType::DoubleSpending => transaction_amount / 100,     // 1%
            FraudType::MaliciousValidator => transaction_amount / 200, // 0.5%
            FraudType::InvalidSignature => base_reward,               // Just base reward
            FraudType::UnauthorizedWithdrawal => transaction_amount / 50, // 2%
        };
        
        base_reward.saturating_add(additional_reward)
    }
}

/// Transaction verification utilities
pub struct TransactionVerifier;

impl TransactionVerifier {
    /// Verify Ethereum transaction exists and has correct data
    pub fn verify_ethereum_transaction(
        tx_hash: &H256,
        expected_data: &[u8],
    ) -> Result<bool, BridgeError> {
        // TODO: Implement actual Ethereum transaction verification
        // This would query an Ethereum node to verify the transaction
        // For now, return true for testing
        Ok(true)
    }
    
    /// Verify transaction hasn't been processed before
    pub fn verify_no_double_processing<AccountId>(
        tx_hash: &H256,
        processed_deposits: &Vec<H256>,
    ) -> bool {
        !processed_deposits.contains(tx_hash)
    }
    
    /// Verify withdrawal amount doesn't exceed limits
    pub fn verify_withdrawal_limits(
        amount: Balance,
        asset: &BridgeableAsset,
        daily_withdrawn: Balance,
        config: &BridgeConfig,
    ) -> Result<(), BridgeError> {
        // Check single transaction limit
        if amount > config.max_single_deposit {
            return Err(BridgeError::AmountExceedsLimit);
        }
        
        // Check daily limit
        if daily_withdrawn.saturating_add(amount) > config.daily_limit {
            return Err(BridgeError::AmountExceedsLimit);
        }
        
        Ok(())
    }
    
    /// Verify deposit amount is reasonable (not dust, not excessive)
    pub fn verify_deposit_amount(
        amount: Balance,
        asset: &BridgeableAsset,
    ) -> Result<(), BridgeError> {
        // Minimum deposit amounts (to prevent dust)
        let min_amount = match asset {
            BridgeableAsset::ETH => 1_000_000_000_000_000, // 0.001 ETH
            BridgeableAsset::USDC => 1_000_000,             // 1 USDC
            BridgeableAsset::USDT => 1_000_000,             // 1 USDT
            BridgeableAsset::WBTC => 1000,                  // 0.00001 WBTC
        };
        
        if amount < min_amount {
            return Err(BridgeError::AmountExceedsLimit);
        }
        
        // Maximum single deposit (to prevent excessive risk)
        let max_amount = match asset {
            BridgeableAsset::ETH => 1000 * 1_000_000_000_000_000_000, // 1000 ETH
            BridgeableAsset::USDC => 1_000_000 * 1_000_000,           // 1M USDC
            BridgeableAsset::USDT => 1_000_000 * 1_000_000,           // 1M USDT
            BridgeableAsset::WBTC => 100 * 100_000_000,               // 100 WBTC
        };
        
        if amount > max_amount {
            return Err(BridgeError::AmountExceedsLimit);
        }
        
        Ok(())
    }
}

// ValidatorReputation moved to types.rs

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::H256;
    
    #[test]
    fn test_validator_reputation() {
        let mut reputation = ValidatorReputation::default();
        
        // New validator should have neutral score
        assert_eq!(reputation.score, 50);
        assert!(reputation.is_trusted);
        
        // After correct validations, score should improve
        for _ in 0..10 {
            reputation.update_after_validation(true);
        }
        assert_eq!(reputation.score, 100);
        assert!(reputation.is_trusted);
        
        // After incorrect validation, score should decrease
        reputation.update_after_validation(false);
        assert_eq!(reputation.score, 90); // 10/11 = ~90%
        assert!(reputation.is_trusted);
        
        // After offline incident, score should decrease further
        reputation.update_after_offline();
        assert_eq!(reputation.score, 85); // 90 - 5 penalty
        assert!(reputation.is_trusted);
    }
    
    #[test]
    fn test_deposit_amount_validation() {
        // Valid amounts should pass
        assert!(TransactionVerifier::verify_deposit_amount(
            1_000_000_000_000_000_000, // 1 ETH
            &BridgeableAsset::ETH
        ).is_ok());
        
        // Too small amounts should fail
        assert!(TransactionVerifier::verify_deposit_amount(
            100, // 0.0000000000000001 ETH
            &BridgeableAsset::ETH
        ).is_err());
        
        // Too large amounts should fail
        assert!(TransactionVerifier::verify_deposit_amount(
            10000 * 1_000_000_000_000_000_000, // 10000 ETH
            &BridgeableAsset::ETH
        ).is_err());
    }
    
    #[test]
    fn test_fraud_proof_reward_calculation() {
        let amount = 1_000_000_000_000_000_000; // 1 ETH equivalent
        
        let double_spend_reward = FraudProofValidator::<()>::calculate_fraud_proof_reward(
            &FraudType::DoubleSpending,
            amount
        );
        
        let invalid_tx_reward = FraudProofValidator::<()>::calculate_fraud_proof_reward(
            &FraudType::InvalidEthereumTx,
            amount
        );
        
        // Double spending should have higher reward than invalid tx
        assert!(double_spend_reward > invalid_tx_reward);
    }
}
