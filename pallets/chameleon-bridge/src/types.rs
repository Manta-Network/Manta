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

//! Bridge Types
//!
//! This module defines the core types used by the Chameleon Bridge pallet
//! for cross-chain asset transfers between Ethereum and Chameleon Network.

use codec::{Decode, Encode};
use manta_primitives::types::{Balance, BlockNumber};
use scale_info::TypeInfo;
use sp_core::{H160, H256};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// Ethereum address type (20 bytes)
pub type EthereumAddress = H160;

/// Ethereum transaction hash type (32 bytes)
pub type EthereumTxHash = H256;

/// Validator signature type
pub type ValidatorSignature = Vec<u8>;

/// Withdrawal ID type
pub type WithdrawalId = u64;

/// Asset ID type for wrapped tokens
pub type AssetId = u32;

/// Supported bridgeable assets between Ethereum and Chameleon
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum BridgeableAsset {
    /// Ethereum (ETH) -> Wrapped Ethereum (wETH)
    ETH,
    /// USD Coin (USDC) -> Chameleon USDC (cUSDC)
    USDC,
    /// Tether USD (USDT) -> Chameleon USDT (cUSDT)
    USDT,
    /// Wrapped Bitcoin (WBTC) -> Chameleon WBTC (cWBTC)
    WBTC,
}

impl BridgeableAsset {
    /// Get the Ethereum contract address for this asset
    pub fn ethereum_address(&self) -> EthereumAddress {
        match self {
            BridgeableAsset::ETH => H160::zero(), // ETH uses zero address convention
            BridgeableAsset::USDC => {
                // USDC contract address: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&[
                    0xA0, 0xb8, 0x69, 0x91, 0xc6, 0x21, 0x8b, 0x36, 0xc1, 0xd1,
                    0x9D, 0x4a, 0x2e, 0x9E, 0xb0, 0xcE, 0x36, 0x06, 0xeB, 0x48
                ]);
                H160::from(bytes)
            },
            BridgeableAsset::USDT => {
                // USDT contract address: 0xdAC17F958D2ee523a2206206994597C13D831ec7
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&[
                    0xdA, 0xC1, 0x7F, 0x95, 0x8D, 0x2e, 0xe5, 0x23, 0xa2, 0x20,
                    0x62, 0x06, 0x99, 0x45, 0x97, 0xC1, 0x3D, 0x83, 0x1e, 0xc7
                ]);
                H160::from(bytes)
            },
            BridgeableAsset::WBTC => {
                // WBTC contract address: 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&[
                    0x22, 0x60, 0xFA, 0xC5, 0xE5, 0x54, 0x2a, 0x77, 0x3A, 0xa4,
                    0x4f, 0xBC, 0xfe, 0xDf, 0x7C, 0x19, 0x3b, 0xc2, 0xC5, 0x99
                ]);
                H160::from(bytes)
            },
        }
    }

    /// Get the wrapped asset ID on Chameleon for this asset
    pub fn wrapped_asset_id(&self) -> AssetId {
        match self {
            BridgeableAsset::ETH => 1,  // wETH
            BridgeableAsset::USDC => 2, // cUSDC
            BridgeableAsset::USDT => 3, // cUSDT
            BridgeableAsset::WBTC => 4, // cWBTC
        }
    }

    /// Get the symbol for the wrapped asset
    pub fn wrapped_symbol(&self) -> &'static str {
        match self {
            BridgeableAsset::ETH => "wETH",
            BridgeableAsset::USDC => "cUSDC",
            BridgeableAsset::USDT => "cUSDT",
            BridgeableAsset::WBTC => "cWBTC",
        }
    }

    /// Get the name for the wrapped asset
    pub fn wrapped_name(&self) -> &'static str {
        match self {
            BridgeableAsset::ETH => "Wrapped Ethereum",
            BridgeableAsset::USDC => "Chameleon USD Coin",
            BridgeableAsset::USDT => "Chameleon Tether USD",
            BridgeableAsset::WBTC => "Chameleon Wrapped Bitcoin",
        }
    }

    /// Get the decimals for this asset
    pub fn decimals(&self) -> u8 {
        match self {
            BridgeableAsset::ETH => 18,
            BridgeableAsset::USDC => 6,
            BridgeableAsset::USDT => 6,
            BridgeableAsset::WBTC => 8,
        }
    }
}

/// Status of a bridge deposit (Ethereum -> Chameleon)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum DepositStatus {
    /// Deposit is pending confirmation
    Pending,
    /// Deposit has sufficient confirmations
    Confirmed,
    /// Wrapped tokens have been minted
    Minted,
    /// Deposit failed (invalid or disputed)
    Failed,
}

/// Status of a bridge withdrawal (Chameleon -> Ethereum)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum WithdrawalStatus {
    /// Withdrawal is pending validator signatures
    Pending,
    /// Withdrawal has enough signatures and is ready to execute
    ReadyToExecute,
    /// Withdrawal has been executed on Ethereum
    Executed,
    /// Withdrawal was cancelled or failed
    Cancelled,
}

/// Bridge deposit request from Ethereum to Chameleon
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BridgeDeposit<AccountId> {
    /// Ethereum transaction hash that locked the assets
    pub eth_tx_hash: EthereumTxHash,
    /// Ethereum address that locked the assets
    pub eth_address: EthereumAddress,
    /// Recipient account on Chameleon
    pub recipient: AccountId,
    /// Asset being bridged
    pub asset: BridgeableAsset,
    /// Amount deposited (in asset's native decimals)
    pub amount: Balance,
    /// Ethereum block number when deposit occurred
    pub eth_block_number: u64,
    /// Number of confirmations required
    pub confirmations_required: u32,
    /// Current number of confirmations
    pub confirmations: u32,
    /// Current status of the deposit
    pub status: DepositStatus,
    /// Validators who have confirmed this deposit
    pub validator_confirmations: Vec<AccountId>,
    /// Block number when deposit was first reported
    pub reported_at: BlockNumber,
}

/// Validator signature for a withdrawal
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ValidatorWithdrawalSignature<AccountId> {
    /// Validator who provided the signature
    pub validator: AccountId,
    /// The signature bytes
    pub signature: ValidatorSignature,
    /// Block number when signature was provided
    pub signed_at: BlockNumber,
}

/// Bridge withdrawal request from Chameleon to Ethereum
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BridgeWithdrawal<AccountId> {
    /// Unique withdrawal identifier
    pub withdrawal_id: WithdrawalId,
    /// Account initiating the withdrawal
    pub from: AccountId,
    /// Destination Ethereum address
    pub eth_destination: EthereumAddress,
    /// Asset being withdrawn
    pub asset: BridgeableAsset,
    /// Amount to withdraw (in asset's native decimals)
    pub amount: Balance,
    /// Validator signatures collected
    pub signatures: Vec<ValidatorWithdrawalSignature<AccountId>>,
    /// Number of signatures required (5-of-9)
    pub signatures_required: u32,
    /// Current status of the withdrawal
    pub status: WithdrawalStatus,
    /// Block number when withdrawal was initiated
    pub initiated_at: BlockNumber,
    /// Fee paid for this withdrawal
    pub fee_paid: Balance,
}

/// Bridge validator information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BridgeValidator<AccountId> {
    /// Validator account ID
    pub account: AccountId,
    /// Ethereum address for signing transactions
    pub eth_address: EthereumAddress,
    /// Whether the validator is active
    pub is_active: bool,
    /// Block number when validator was added
    pub added_at: BlockNumber,
}

/// Bridge configuration parameters
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BridgeConfig {
    /// Minimum number of Ethereum confirmations required
    pub min_confirmations: u32,
    /// Number of validator signatures required for withdrawals
    pub signature_threshold: u32,
    /// Maximum amount that can be bridged in a single transaction
    pub max_single_deposit: Balance,
    /// Maximum total amount that can be bridged per day
    pub daily_limit: Balance,
    /// Whether the bridge is currently paused
    pub is_paused: bool,
    /// Challenge period for fraud proofs (in blocks)
    pub challenge_period: BlockNumber,
}

/// Fee calculation result
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BridgeFee {
    /// Total fee amount in CHML
    pub total_fee: Balance,
    /// Amount going to bridge operations reserve
    pub bridge_ops_fee: Balance,
    /// Amount going to treasury
    pub treasury_fee: Balance,
}

/// Bridge statistics
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct BridgeStats {
    /// Total number of deposits processed
    pub total_deposits: u64,
    /// Total number of withdrawals processed
    pub total_withdrawals: u64,
    /// Total value bridged (in CHML equivalent)
    pub total_volume: Balance,
    /// Total fees collected (in CHML)
    pub total_fees: Balance,
    /// Number of active validators
    pub active_validators: u32,
}

/// Error types for bridge operations
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum BridgeError {
    /// Asset is not supported for bridging
    UnsupportedAsset,
    /// Deposit not found
    DepositNotFound,
    /// Withdrawal not found
    WithdrawalNotFound,
    /// Insufficient confirmations
    InsufficientConfirmations,
    /// Invalid validator
    InvalidValidator,
    /// Duplicate signature
    DuplicateSignature,
    /// Bridge is paused
    BridgePaused,
    /// Amount exceeds limits
    AmountExceedsLimit,
    /// Invalid Ethereum address
    InvalidEthereumAddress,
    /// Insufficient balance
    InsufficientBalance,
    /// Invalid signature
    InvalidSignature,
    /// Deposit already processed
    DepositAlreadyProcessed,
    /// Withdrawal already executed
    WithdrawalAlreadyExecuted,
}

/// Trait for bridge fee calculation
pub trait BridgeFeeCalculator<T: frame_system::Config> {
    /// Calculate shielding fee (Ethereum -> Chameleon)
    fn calculate_shielding_fee(amount: Balance) -> BridgeFee;
    
    /// Calculate unshielding fee (Chameleon -> Ethereum)
    fn calculate_unshielding_fee(amount: Balance) -> BridgeFee;
}

/// Default implementation of bridge fee calculator
pub struct DefaultBridgeFeeCalculator;

impl<T: frame_system::Config> BridgeFeeCalculator<T> for DefaultBridgeFeeCalculator {
    fn calculate_shielding_fee(amount: Balance) -> BridgeFee {
        use manta_primitives::chameleon_constants::fees::*;
        use sp_runtime::traits::Saturating;
        
        // Calculate percentage fee
        let percentage_fee = SHIELDING_FEE_PERCENT.mul_floor(amount);
        
        // Use the higher of percentage fee or minimum fee
        let total_fee = percentage_fee.max(SHIELDING_MIN_FEE);
        
        // EVM bridges: 70% Bridge Ops Reserve, 30% Treasury
        let bridge_ops_fee = total_fee.saturating_mul(70) / 100;
        let treasury_fee = total_fee.saturating_sub(bridge_ops_fee);
        
        BridgeFee {
            total_fee,
            bridge_ops_fee,
            treasury_fee,
        }
    }
    
    fn calculate_unshielding_fee(amount: Balance) -> BridgeFee {
        use manta_primitives::chameleon_constants::fees::*;
        use sp_runtime::traits::Saturating;
        
        // Calculate percentage fee
        let percentage_fee = UNSHIELDING_FEE_PERCENT.mul_floor(amount);
        
        // Use the higher of percentage fee or minimum fee
        let total_fee = percentage_fee.max(UNSHIELDING_MIN_FEE);
        
        // EVM bridges: 70% Bridge Ops Reserve, 30% Treasury
        let bridge_ops_fee = total_fee.saturating_mul(70) / 100;
        let treasury_fee = total_fee.saturating_sub(bridge_ops_fee);
        
        BridgeFee {
            total_fee,
            bridge_ops_fee,
            treasury_fee,
        }
    }
}
