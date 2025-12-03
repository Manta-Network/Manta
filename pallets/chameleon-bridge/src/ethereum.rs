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

//! Ethereum Integration
//!
//! This module handles Ethereum-specific logic for the bridge,
//! including event parsing, signature verification, and transaction formatting.

use crate::types::*;
use codec::{Decode, Encode};
use frame_support::traits::Get;
use manta_primitives::types::Balance;
use scale_info::TypeInfo;
use sp_core::{H160, H256, U256};
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::{vec::Vec, collections::btree_map::BTreeMap};

/// Ethereum block number type
pub type EthereumBlockNumber = u64;

/// Ethereum event log entry
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct EthereumEventLog {
    /// Contract address that emitted the event
    pub address: EthereumAddress,
    /// Event topics (indexed parameters)
    pub topics: Vec<H256>,
    /// Event data (non-indexed parameters)
    pub data: Vec<u8>,
    /// Block number where event was emitted
    pub block_number: EthereumBlockNumber,
    /// Transaction hash that generated this event
    pub transaction_hash: EthereumTxHash,
    /// Log index within the transaction
    pub log_index: u32,
}

/// Parsed AssetLocked event from Ethereum bridge contract
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct AssetLockedEvent {
    /// User who locked the assets
    pub user: EthereumAddress,
    /// Asset contract address (or zero for ETH)
    pub asset: EthereumAddress,
    /// Amount locked
    pub amount: Balance,
    /// Chameleon recipient address (as bytes32)
    pub chameleon_address: H256,
    /// Ethereum transaction hash
    pub tx_hash: EthereumTxHash,
    /// Block number
    pub block_number: EthereumBlockNumber,
}

/// Parsed AssetUnlocked event from Ethereum bridge contract
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct AssetUnlockedEvent {
    /// User who received the unlocked assets
    pub user: EthereumAddress,
    /// Asset contract address (or zero for ETH)
    pub asset: EthereumAddress,
    /// Amount unlocked
    pub amount: Balance,
    /// Ethereum transaction hash
    pub tx_hash: EthereumTxHash,
    /// Block number
    pub block_number: EthereumBlockNumber,
}

/// Ethereum bridge contract interface
pub struct EthereumBridgeContract;

impl EthereumBridgeContract {
    /// Chameleon Bridge contract address on Ethereum mainnet
    /// TODO: Replace with actual deployed contract address
    pub const CONTRACT_ADDRESS: EthereumAddress = H160::zero();
    
    /// AssetLocked event signature: AssetLocked(address,address,uint256,bytes32)
    /// keccak256("AssetLocked(address,address,uint256,bytes32)")
    pub const ASSET_LOCKED_SIGNATURE: H256 = H256([
        0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
        0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
        0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
        0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
    ]);
    
    /// AssetUnlocked event signature: AssetUnlocked(address,address,uint256)
    /// keccak256("AssetUnlocked(address,address,uint256)")
    pub const ASSET_UNLOCKED_SIGNATURE: H256 = H256([
        0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81, 0x92,
        0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09, 0x1a,
        0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81, 0x92,
        0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09, 0x1a,
    ]);
    
    /// Parse AssetLocked event from Ethereum log
    pub fn parse_asset_locked_event(log: &EthereumEventLog) -> Result<AssetLockedEvent, BridgeError> {
        // Verify this is the correct event
        if log.topics.is_empty() || log.topics[0] != Self::ASSET_LOCKED_SIGNATURE {
            return Err(BridgeError::InvalidSignature);
        }
        
        // Verify contract address
        if log.address != Self::CONTRACT_ADDRESS {
            return Err(BridgeError::InvalidEthereumAddress);
        }
        
        // Parse topics (indexed parameters)
        if log.topics.len() != 4 {
            return Err(BridgeError::InvalidSignature);
        }
        
        let user = EthereumAddress::from_slice(&log.topics[1].as_bytes()[12..32]);
        let asset = EthereumAddress::from_slice(&log.topics[2].as_bytes()[12..32]);
        let chameleon_address = log.topics[3];
        
        // Parse data (non-indexed parameters)
        if log.data.len() != 32 {
            return Err(BridgeError::InvalidSignature);
        }
        
        let amount = U256::from_big_endian(&log.data[0..32]).low_u128() as Balance;
        
        Ok(AssetLockedEvent {
            user,
            asset,
            amount,
            chameleon_address,
            tx_hash: log.transaction_hash,
            block_number: log.block_number,
        })
    }
    
    /// Parse AssetUnlocked event from Ethereum log
    pub fn parse_asset_unlocked_event(log: &EthereumEventLog) -> Result<AssetUnlockedEvent, BridgeError> {
        // Verify this is the correct event
        if log.topics.is_empty() || log.topics[0] != Self::ASSET_UNLOCKED_SIGNATURE {
            return Err(BridgeError::InvalidSignature);
        }
        
        // Verify contract address
        if log.address != Self::CONTRACT_ADDRESS {
            return Err(BridgeError::InvalidEthereumAddress);
        }
        
        // Parse topics (indexed parameters)
        if log.topics.len() != 3 {
            return Err(BridgeError::InvalidSignature);
        }
        
        let user = EthereumAddress::from_slice(&log.topics[1].as_bytes()[12..32]);
        let asset = EthereumAddress::from_slice(&log.topics[2].as_bytes()[12..32]);
        
        // Parse data (non-indexed parameters)
        if log.data.len() != 32 {
            return Err(BridgeError::InvalidSignature);
        }
        
        let amount = U256::from_big_endian(&log.data[0..32]).low_u128() as Balance;
        
        Ok(AssetUnlockedEvent {
            user,
            asset,
            amount,
            tx_hash: log.transaction_hash,
            block_number: log.block_number,
        })
    }
    
    /// Get bridgeable asset from Ethereum address
    pub fn get_bridgeable_asset(eth_address: EthereumAddress) -> Result<BridgeableAsset, BridgeError> {
        if eth_address == H160::zero() {
            Ok(BridgeableAsset::ETH)
        } else if eth_address == BridgeableAsset::USDC.ethereum_address() {
            Ok(BridgeableAsset::USDC)
        } else if eth_address == BridgeableAsset::USDT.ethereum_address() {
            Ok(BridgeableAsset::USDT)
        } else if eth_address == BridgeableAsset::WBTC.ethereum_address() {
            Ok(BridgeableAsset::WBTC)
        } else {
            Err(BridgeError::UnsupportedAsset)
        }
    }
    
    /// Convert Chameleon address bytes to account ID
    /// This is a simplified conversion - in practice, you'd need proper SS58 decoding
    pub fn chameleon_address_to_account_id<AccountId: Decode>(address_bytes: H256) -> Result<AccountId, BridgeError> {
        // For now, we'll use the first 32 bytes as the account ID
        // In a real implementation, this would properly decode SS58 addresses
        AccountId::decode(&mut &address_bytes.as_bytes()[..])
            .map_err(|_| BridgeError::InvalidEthereumAddress)
    }
}

/// Ethereum signature verification utilities
pub struct EthereumSignature;

impl EthereumSignature {
    /// Verify an Ethereum signature
    /// This is a placeholder - real implementation would use secp256k1
    pub fn verify(
        message: &[u8],
        signature: &[u8],
        signer: &EthereumAddress,
    ) -> bool {
        // TODO: Implement actual ECDSA signature verification
        // For now, return true for testing purposes
        true
    }
    
    /// Recover signer address from signature
    /// This is a placeholder - real implementation would use secp256k1
    pub fn recover_signer(
        message: &[u8],
        signature: &[u8],
    ) -> Result<EthereumAddress, BridgeError> {
        // TODO: Implement actual ECDSA signature recovery
        // For now, return a dummy address
        Ok(H160::zero())
    }
    
    /// Create message hash for withdrawal signing
    pub fn create_withdrawal_message(
        withdrawal_id: WithdrawalId,
        recipient: EthereumAddress,
        asset: EthereumAddress,
        amount: Balance,
    ) -> H256 {
        // Create a deterministic message hash for validators to sign
        // In practice, this would use keccak256 hashing
        let mut message = Vec::new();
        message.extend_from_slice(&withdrawal_id.to_be_bytes());
        message.extend_from_slice(recipient.as_bytes());
        message.extend_from_slice(asset.as_bytes());
        message.extend_from_slice(&amount.to_be_bytes());
        
        // Simple hash for now - replace with keccak256 in production
        use sp_core::hashing::blake2_256;
        blake2_256(&message).into()
    }
}

/// Ethereum transaction builder for withdrawals
pub struct EthereumTransactionBuilder;

impl EthereumTransactionBuilder {
    /// Build unlock transaction data for Ethereum
    pub fn build_unlock_transaction(
        withdrawal: &BridgeWithdrawal<impl Clone>,
        signatures: &[ValidatorSignature],
    ) -> Vec<u8> {
        // Build the transaction data for calling unlockAsset on Ethereum
        // This would encode the function call with parameters
        
        // Function signature: unlockAsset(address,address,uint256,bytes[])
        let function_sig = &[0x12, 0x34, 0x56, 0x78]; // Placeholder
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(function_sig);
        
        // Encode parameters (simplified)
        // In practice, this would use proper ABI encoding
        tx_data.extend_from_slice(withdrawal.eth_destination.as_bytes());
        tx_data.extend_from_slice(&withdrawal.asset.ethereum_address().as_bytes());
        tx_data.extend_from_slice(&withdrawal.amount.to_be_bytes());
        
        // Encode signatures array
        for signature in signatures {
            tx_data.extend_from_slice(signature);
        }
        
        tx_data
    }
    
    /// Estimate gas cost for unlock transaction
    pub fn estimate_gas_cost(
        withdrawal: &BridgeWithdrawal<impl Clone>,
        signature_count: u32,
    ) -> u64 {
        // Base gas cost for unlock transaction
        let base_gas = 100_000u64;
        
        // Additional gas per signature
        let signature_gas = 5_000u64 * signature_count as u64;
        
        // Gas for asset transfer
        let transfer_gas = match withdrawal.asset {
            BridgeableAsset::ETH => 21_000u64,  // ETH transfer
            _ => 65_000u64,  // ERC20 transfer
        };
        
        base_gas + signature_gas + transfer_gas
    }
}

/// Ethereum block confirmation tracker
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct EthereumConfirmationTracker {
    /// Latest Ethereum block number observed
    pub latest_block: EthereumBlockNumber,
    /// Minimum confirmations required
    pub min_confirmations: u32,
    /// Pending deposits awaiting confirmations
    pub pending_deposits: BTreeMap<EthereumTxHash, EthereumBlockNumber>,
}

impl EthereumConfirmationTracker {
    /// Create new confirmation tracker
    pub fn new(min_confirmations: u32) -> Self {
        Self {
            latest_block: 0,
            min_confirmations,
            pending_deposits: BTreeMap::new(),
        }
    }
    
    /// Update latest Ethereum block
    pub fn update_latest_block(&mut self, block_number: EthereumBlockNumber) {
        self.latest_block = block_number;
    }
    
    /// Add deposit for confirmation tracking
    pub fn add_deposit(&mut self, tx_hash: EthereumTxHash, block_number: EthereumBlockNumber) {
        self.pending_deposits.insert(tx_hash, block_number);
    }
    
    /// Check if deposit has enough confirmations
    pub fn is_confirmed(&self, tx_hash: &EthereumTxHash) -> bool {
        if let Some(&deposit_block) = self.pending_deposits.get(tx_hash) {
            let confirmations = self.latest_block.saturating_sub(deposit_block);
            confirmations >= self.min_confirmations as u64
        } else {
            false
        }
    }
    
    /// Get number of confirmations for a deposit
    pub fn get_confirmations(&self, tx_hash: &EthereumTxHash) -> u32 {
        if let Some(&deposit_block) = self.pending_deposits.get(tx_hash) {
            self.latest_block.saturating_sub(deposit_block) as u32
        } else {
            0
        }
    }
    
    /// Remove confirmed deposits
    pub fn cleanup_confirmed_deposits(&mut self) -> Vec<EthereumTxHash> {
        let mut confirmed = Vec::new();
        
        self.pending_deposits.retain(|&tx_hash, deposit_block| {
            let confirmations = self.latest_block.saturating_sub(*deposit_block);
            if confirmations >= self.min_confirmations as u64 {
                confirmed.push(tx_hash);
                false // Remove from pending
            } else {
                true // Keep in pending
            }
        });
        
        confirmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bridgeable_asset_addresses() {
        assert_eq!(BridgeableAsset::ETH.ethereum_address(), H160::zero());
        assert_ne!(BridgeableAsset::USDC.ethereum_address(), H160::zero());
        assert_ne!(BridgeableAsset::USDT.ethereum_address(), H160::zero());
        assert_ne!(BridgeableAsset::WBTC.ethereum_address(), H160::zero());
    }
    
    #[test]
    fn test_wrapped_asset_ids() {
        assert_eq!(BridgeableAsset::ETH.wrapped_asset_id(), 1);
        assert_eq!(BridgeableAsset::USDC.wrapped_asset_id(), 2);
        assert_eq!(BridgeableAsset::USDT.wrapped_asset_id(), 3);
        assert_eq!(BridgeableAsset::WBTC.wrapped_asset_id(), 4);
    }
    
    #[test]
    fn test_asset_decimals() {
        assert_eq!(BridgeableAsset::ETH.decimals(), 18);
        assert_eq!(BridgeableAsset::USDC.decimals(), 6);
        assert_eq!(BridgeableAsset::USDT.decimals(), 6);
        assert_eq!(BridgeableAsset::WBTC.decimals(), 8);
    }
    
    #[test]
    fn test_confirmation_tracker() {
        let mut tracker = EthereumConfirmationTracker::new(12);
        let tx_hash = H256::from_low_u64_be(1);
        
        // Add deposit at block 100
        tracker.add_deposit(tx_hash, 100);
        tracker.update_latest_block(105);
        
        // Should not be confirmed yet (only 5 confirmations)
        assert!(!tracker.is_confirmed(&tx_hash));
        assert_eq!(tracker.get_confirmations(&tx_hash), 5);
        
        // Update to block 112 (12 confirmations)
        tracker.update_latest_block(112);
        assert!(tracker.is_confirmed(&tx_hash));
        assert_eq!(tracker.get_confirmations(&tx_hash), 12);
    }
    
    #[test]
    fn test_get_bridgeable_asset() {
        assert_eq!(
            EthereumBridgeContract::get_bridgeable_asset(H160::zero()).unwrap(),
            BridgeableAsset::ETH
        );
        
        assert_eq!(
            EthereumBridgeContract::get_bridgeable_asset(BridgeableAsset::USDC.ethereum_address()).unwrap(),
            BridgeableAsset::USDC
        );
        
        // Invalid address should return error
        assert_eq!(
            EthereumBridgeContract::get_bridgeable_asset(H160::from_low_u64_be(999)),
            Err(BridgeError::UnsupportedAsset)
        );
    }
}
