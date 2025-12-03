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

//! Types for Chameleon MEV Protection Pallet

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec, traits::ConstU32};
use manta_primitives::types::{Hash, Moment};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Encrypted transaction structure
///
/// Contains the encrypted transaction data along with metadata needed
/// for ordering and commitment verification.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct EncryptedTransaction {
    /// Encrypted transaction data
    pub encrypted_data: BoundedVec<u8, ConstU32<1024>>,
    /// Commitment hash (SHA3-256 of original transaction)
    pub commitment: Hash,
    /// Timestamp when submitted (for ordering)
    pub timestamp: Moment,
    /// Block number when submitted
    pub submit_block: u32,
}

/// Threshold decryption share from validator
///
/// Each validator provides a share that can be combined with others
/// to decrypt the transactions in a block.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct DecryptionShare {
    /// Validator providing the share (as bytes for simplicity)
    pub validator: [u8; 32],
    /// Decryption share data
    pub share_data: BoundedVec<u8, ConstU32<1024>>,
    /// Block number this share is for
    pub block_number: u32,
}

/// Block proposal with encrypted transactions
///
/// Contains the ordered list of encrypted transactions for a block
/// along with the commitment to their ordering.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct BlockProposal<T>
where
    T: frame_system::Config,
{
    /// Encrypted transactions in timestamp order
    pub encrypted_txs: Vec<EncryptedTransaction>,
    /// Commitment hash of the ordering
    pub ordering_commitment: Hash,
    /// Proposer validator
    pub proposer: T::AccountId,
    /// Block number
    pub block_number: u32,
}

/// MEV attack types that this pallet prevents
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum MevAttackType {
    /// Front-running: Seeing pending tx and trading ahead
    FrontRunning,
    /// Sandwich attack: Surrounding user trades for profit
    SandwichAttack,
    /// Back-running: Execute trade immediately after victim
    BackRunning,
    /// Time-bandit attack: Reorder transactions within block
    TimeBanditAttack,
}

/// Transaction ordering method
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum OrderingMethod {
    /// First-In-First-Out (timestamp-based)
    FIFO,
    /// Fee-based (traditional, vulnerable to MEV)
    FeeBased,
    /// Random (not deterministic)
    Random,
}

/// Encryption status of a transaction
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum EncryptionStatus {
    /// Transaction is encrypted and hidden
    Encrypted,
    /// Transaction has been decrypted and revealed
    Revealed,
    /// Transaction failed to decrypt
    DecryptionFailed,
}

/// MEV protection statistics
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default, PartialEq, Eq)]
pub struct MevProtectionStats {
    /// Total transactions protected
    pub total_protected_txs: u64,
    /// Prevented front-running attempts
    pub prevented_front_runs: u64,
    /// Prevented sandwich attacks
    pub prevented_sandwiches: u64,
    /// Average encryption time (milliseconds)
    pub avg_encryption_time_ms: u64,
    /// Average decryption time (milliseconds)
    pub avg_decryption_time_ms: u64,
}
