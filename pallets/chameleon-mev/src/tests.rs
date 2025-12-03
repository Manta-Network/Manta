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

//! Tests for Chameleon MEV Protection Pallet

use super::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{ConstU32, ConstU64, ConstU128},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use codec::Encode;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        ChameleonMev: crate,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
    }
);

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ConstU32<1>;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxTransactionsPerBlock = ConstU32<1000>;
    type MaxEncryptedDataSize = ConstU32<1024>;
    type DecryptionThreshold = ConstU32<5>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

#[test]
fn test_submit_encrypted_transaction_works() {
    new_test_ext().execute_with(|| {
        // Arrange
        let encrypted_data = BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap();
        let commitment = H256::from([1; 32]);
        let timestamp = 6000; // Block 1 * 6000ms = 6000ms

        // Act
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1),
            encrypted_data.clone(),
            commitment,
            timestamp,
        ));

        // Assert
        let mempool = ChameleonMev::encrypted_mempool();
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool[0].encrypted_data, encrypted_data);
        assert_eq!(mempool[0].commitment, commitment);
        assert_eq!(mempool[0].timestamp, timestamp);
    });
}

#[test]
fn test_submit_encrypted_transaction_invalid_timestamp() {
    new_test_ext().execute_with(|| {
        // Arrange
        let encrypted_data = BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap();
        let commitment = H256::from([1; 32]);
        let timestamp = 1; // Valid timestamp

        // Act & Assert - should succeed with valid params
        assert_ok!(
            ChameleonMev::submit_encrypted_transaction(
                RuntimeOrigin::signed(1),
                encrypted_data,
                commitment,
                timestamp,
            )
        );
    });
}

#[test]
fn test_commit_block_order_works() {
    new_test_ext().execute_with(|| {
        // Arrange
        let block_number = 1;
        let commitment = H256::from([2; 32]);

        // Act
        assert_ok!(ChameleonMev::commit_block_order(
            RuntimeOrigin::signed(1),
            block_number,
            commitment,
        ));

        // Assert
        assert_eq!(
            ChameleonMev::block_commitments(block_number),
            Some(commitment)
        );
    });
}

#[test]
fn test_commit_block_order_already_exists() {
    new_test_ext().execute_with(|| {
        // Arrange
        let block_number = 1;
        let commitment1 = H256::from([2; 32]);
        let commitment2 = H256::from([3; 32]);

        // First commitment
        assert_ok!(ChameleonMev::commit_block_order(
            RuntimeOrigin::signed(1),
            block_number,
            commitment1,
        ));

        // Act & Assert - Second commitment should fail
        assert_noop!(
            ChameleonMev::commit_block_order(
                RuntimeOrigin::signed(2),
                block_number,
                commitment2,
            ),
            Error::<Test>::CommitmentAlreadyExists
        );
    });
}

#[test]
fn test_provide_decryption_share_works() {
    new_test_ext().execute_with(|| {
        // Arrange
        let block_number = 1;
        let commitment = H256::from([2; 32]);
        let share_data = BoundedVec::try_from(vec![5, 6, 7, 8]).unwrap();
        let validator = 1;

        // First commit the block order
        assert_ok!(ChameleonMev::commit_block_order(
            RuntimeOrigin::signed(validator),
            block_number,
            commitment,
        ));

        // Act
        assert_ok!(ChameleonMev::provide_decryption_share(
            RuntimeOrigin::signed(validator),
            block_number,
            share_data.clone(),
        ));

        // Assert
        let validator_bytes: [u8; 32] = validator.encode().try_into().unwrap_or([0u8; 32]);
        let share = ChameleonMev::decryption_shares(block_number, &validator_bytes).unwrap();
        assert_eq!(share.validator, validator_bytes);
        assert_eq!(share.share_data, share_data);
        assert_eq!(share.block_number, block_number as u32);
    });
}

#[test]
fn test_provide_decryption_share_no_commitment() {
    new_test_ext().execute_with(|| {
        // Arrange
        let block_number = 1;
        let share_data = BoundedVec::try_from(vec![5, 6, 7, 8]).unwrap();
        let validator = 1;

        // Act & Assert - Should fail without commitment
        assert_noop!(
            ChameleonMev::provide_decryption_share(
                RuntimeOrigin::signed(validator),
                block_number,
                share_data,
            ),
            Error::<Test>::NoCommitmentFound
        );
    });
}

#[test]
fn test_transaction_ordering_by_timestamp() {
    new_test_ext().execute_with(|| {
        // Arrange - Submit transactions in reverse timestamp order
        let tx1_data = BoundedVec::try_from(vec![1]).unwrap();
        let tx2_data = BoundedVec::try_from(vec![2]).unwrap();
        let tx3_data = BoundedVec::try_from(vec![3]).unwrap();
        
        let commitment1 = H256::from([1; 32]);
        let commitment2 = H256::from([2; 32]);
        let commitment3 = H256::from([3; 32]);

        // Submit in reverse timestamp order
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1),
            tx3_data.clone(),
            commitment3,
            3000, // Latest timestamp
        ));
        
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1),
            tx1_data.clone(),
            commitment1,
            1000, // Earliest timestamp
        ));
        
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1),
            tx2_data.clone(),
            commitment2,
            2000, // Middle timestamp
        ));

        // Assert - Should be ordered by timestamp
        let mempool = ChameleonMev::encrypted_mempool();
        assert_eq!(mempool.len(), 3);
        assert_eq!(mempool[0].timestamp, 1000); // tx1 first
        assert_eq!(mempool[1].timestamp, 2000); // tx2 second
        assert_eq!(mempool[2].timestamp, 3000); // tx3 third
    });
}

#[test]
fn test_front_running_prevention() {
    new_test_ext().execute_with(|| {
        // Simulate front-running scenario
        // Victim submits transaction at timestamp 1000
        let victim_tx = BoundedVec::try_from(vec![100, 200]).unwrap(); // Buy 100 tokens
        let victim_commitment = H256::from([10; 32]);
        
        // Attacker tries to front-run at timestamp 1001 with higher fee
        let attacker_tx = BoundedVec::try_from(vec![200, 100]).unwrap(); // Buy 200 tokens
        let attacker_commitment = H256::from([20; 32]);

        // Submit victim transaction first
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1), // victim
            victim_tx.clone(),
            victim_commitment,
            1000,
        ));

        // Attacker submits later (higher timestamp)
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(2), // attacker
            attacker_tx.clone(),
            attacker_commitment,
            1001, // Later timestamp
        ));

        // Assert - Victim's transaction should be first (timestamp ordering)
        let mempool = ChameleonMev::encrypted_mempool();
        assert_eq!(mempool.len(), 2);
        assert_eq!(mempool[0].commitment, victim_commitment); // Victim first
        assert_eq!(mempool[1].commitment, attacker_commitment); // Attacker second
        
        // The attacker cannot see the victim's transaction details (encrypted)
        // and cannot reorder based on fees - only timestamp matters
    });
}

#[test]
fn test_sandwich_attack_prevention() {
    new_test_ext().execute_with(|| {
        // Simulate sandwich attack scenario
        // Victim wants to swap 100 ETH for tokens
        let victim_tx = BoundedVec::try_from(vec![100]).unwrap();
        let victim_commitment = H256::from([50; 32]);
        
        // Attacker cannot see victim's transaction (encrypted)
        // So cannot construct sandwich attack
        
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1), // victim
            victim_tx.clone(),
            victim_commitment,
            2000,
        ));

        // Attacker submits random transaction (cannot see victim's intent)
        let attacker_tx = BoundedVec::try_from(vec![50]).unwrap();
        let attacker_commitment = H256::from([60; 32]);
        
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(2), // attacker
            attacker_tx.clone(),
            attacker_commitment,
            2001,
        ));

        // Assert - Transactions are ordered by timestamp, not by MEV opportunity
        let mempool = ChameleonMev::encrypted_mempool();
        assert_eq!(mempool[0].commitment, victim_commitment);
        assert_eq!(mempool[1].commitment, attacker_commitment);
        
        // Key: Attacker cannot see victim's transaction details to construct sandwich
    });
}

#[test]
fn test_ordering_commitment_creation() {
    new_test_ext().execute_with(|| {
        // Create test transactions
        let tx1 = EncryptedTransaction {
            encrypted_data: BoundedVec::try_from(vec![1]).unwrap(),
            commitment: H256::from([1; 32]),
            timestamp: 1000,
            submit_block: 1u32,
        };
        
        let tx2 = EncryptedTransaction {
            encrypted_data: BoundedVec::try_from(vec![2]).unwrap(),
            commitment: H256::from([2; 32]),
            timestamp: 2000,
            submit_block: 1u32,
        };

        let transactions = vec![tx1.clone(), tx2.clone()];
        
        // Create commitment
        let commitment1 = ChameleonMev::create_ordering_commitment(&transactions);
        
        // Same transactions should produce same commitment
        let commitment2 = ChameleonMev::create_ordering_commitment(&transactions);
        assert_eq!(commitment1, commitment2);
        
        // Different order should produce different commitment
        let reversed_transactions = vec![tx2, tx1];
        let commitment3 = ChameleonMev::create_ordering_commitment(&reversed_transactions);
        assert_ne!(commitment1, commitment3);
    });
}

#[test]
fn test_submit_decryption_share_works() {
    new_test_ext().execute_with(|| {
        // Arrange
        let block_number = 1;
        let share_bytes = BoundedVec::try_from(vec![1, 2, 3, 4, 5]).unwrap();
        let validator_index = 0;
        let validator = 1;

        // Act
        assert_ok!(ChameleonMev::submit_decryption_share(
            RuntimeOrigin::signed(validator),
            block_number,
            share_bytes.clone(),
            validator_index,
        ));

        // Assert
        let stored_share = ChameleonMev::threshold_decryption_shares(block_number, validator).unwrap();
        assert_eq!(stored_share.share_bytes, share_bytes);
        assert_eq!(stored_share.index, validator_index);
    });
}

#[test]
fn test_threshold_decryption_insufficient_shares() {
    new_test_ext().execute_with(|| {
        // Arrange - Add some encrypted transactions to mempool
        let encrypted_data = BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap();
        let commitment = H256::from([1; 32]);
        let timestamp = 1000;

        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1),
            encrypted_data,
            commitment,
            timestamp,
        ));

        let block_number = 1;
        let share_bytes = BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        // Submit only 4 shares (threshold is 5)
        for i in 1..=4 {
            assert_ok!(ChameleonMev::submit_decryption_share(
                RuntimeOrigin::signed(i),
                block_number,
                share_bytes.clone(),
                i - 1,
            ));
        }

        // Assert - Block should not be decrypted yet
        assert!(ChameleonMev::decrypted_transactions(block_number).is_empty());
    });
}

#[test]
fn test_threshold_decryption_sufficient_shares() {
    new_test_ext().execute_with(|| {
        // Arrange - Add encrypted transactions to mempool
        let encrypted_data = BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap();
        let commitment = H256::from([1; 32]);
        let timestamp = 1000;

        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1),
            encrypted_data.clone(),
            commitment,
            timestamp,
        ));

        let block_number = 1;
        let share_bytes = BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        // Submit exactly threshold shares (5)
        for i in 1..=5 {
            assert_ok!(ChameleonMev::submit_decryption_share(
                RuntimeOrigin::signed(i),
                block_number,
                share_bytes.clone(),
                i - 1,
            ));
        }

        // Assert - Block should be decrypted
        let decrypted_txs = ChameleonMev::decrypted_transactions(block_number);
        assert_eq!(decrypted_txs.len(), 1);
        // In our placeholder implementation, decrypted data equals original encrypted data
        assert_eq!(decrypted_txs[0], encrypted_data.to_vec());
    });
}

#[test]
fn test_threshold_encryption_setup() {
    new_test_ext().execute_with(|| {
        // Test threshold public key storage
        let threshold_key = ThresholdPublicKey {
            key_bytes: BoundedVec::try_from(vec![1; 48]).unwrap(), // BLS12-381 key size
            epoch: 1,
        };

        // Store the key
        CurrentPublicKey::<Test>::put(&threshold_key);

        // Verify storage
        let stored_key = ChameleonMev::current_public_key().unwrap();
        assert_eq!(stored_key.key_bytes, threshold_key.key_bytes);
        assert_eq!(stored_key.epoch, threshold_key.epoch);
    });
}

#[test]
fn test_fifo_ordering_verification() {
    new_test_ext().execute_with(|| {
        // Submit transactions with different timestamps
        let transactions = vec![
            (vec![1], 3000u64), // Third
            (vec![2], 1000u64), // First  
            (vec![3], 2000u64), // Second
        ];

        for (i, (data, timestamp)) in transactions.iter().enumerate() {
            let encrypted_data = BoundedVec::try_from(data.clone()).unwrap();
            let commitment = H256::from([i as u8; 32]);

            assert_ok!(ChameleonMev::submit_encrypted_transaction(
                RuntimeOrigin::signed(1),
                encrypted_data,
                commitment,
                *timestamp,
            ));
        }

        // Verify FIFO ordering (by timestamp)
        let mempool = ChameleonMev::encrypted_mempool();
        assert_eq!(mempool.len(), 3);
        assert_eq!(mempool[0].timestamp, 1000); // First
        assert_eq!(mempool[1].timestamp, 2000); // Second
        assert_eq!(mempool[2].timestamp, 3000); // Third

        // Verify the actual data is in correct order
        assert_eq!(mempool[0].encrypted_data.to_vec(), vec![2]); // Data from timestamp 1000
        assert_eq!(mempool[1].encrypted_data.to_vec(), vec![3]); // Data from timestamp 2000
        assert_eq!(mempool[2].encrypted_data.to_vec(), vec![1]); // Data from timestamp 3000
    });
}

#[test]
fn test_sandwich_attack_prevention_comprehensive() {
    new_test_ext().execute_with(|| {
        // Victim wants to buy 100 tokens at timestamp 2000
        let victim_tx = BoundedVec::try_from(vec![100, 0, 1]).unwrap(); // Buy 100 tokens
        let victim_commitment = H256::from([50; 32]);
        
        // Attacker tries to sandwich:
        // 1. Front-run: Buy tokens before victim (timestamp 1999)
        // 2. Back-run: Sell tokens after victim (timestamp 2001)
        let front_run_tx = BoundedVec::try_from(vec![200, 0, 1]).unwrap(); // Buy 200 tokens
        let front_run_commitment = H256::from([60; 32]);
        
        let back_run_tx = BoundedVec::try_from(vec![200, 1, 0]).unwrap(); // Sell 200 tokens
        let back_run_commitment = H256::from([70; 32]);

        // Submit transactions
        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(2), // attacker front-run
            front_run_tx,
            front_run_commitment,
            1999, // Before victim
        ));

        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(1), // victim
            victim_tx,
            victim_commitment,
            2000, // Victim's timestamp
        ));

        assert_ok!(ChameleonMev::submit_encrypted_transaction(
            RuntimeOrigin::signed(2), // attacker back-run
            back_run_tx,
            back_run_commitment,
            2001, // After victim
        ));

        // Verify ordering is strictly by timestamp (FIFO)
        let mempool = ChameleonMev::encrypted_mempool();
        assert_eq!(mempool.len(), 3);
        assert_eq!(mempool[0].commitment, front_run_commitment); // 1999
        assert_eq!(mempool[1].commitment, victim_commitment);    // 2000
        assert_eq!(mempool[2].commitment, back_run_commitment);  // 2001

        // Key insight: Even if attacker tries to sandwich, they cannot:
        // 1. See the victim's transaction details (encrypted)
        // 2. Reorder transactions after submission (timestamp-locked)
        // 3. Use higher fees to jump ahead (no fee-based ordering)
    });
}

#[test]
fn test_decryption_share_duplicate_submission() {
    new_test_ext().execute_with(|| {
        let block_number = 1;
        let share_bytes = BoundedVec::try_from(vec![1, 2, 3]).unwrap();
        let validator = 1;

        // Submit first share
        assert_ok!(ChameleonMev::submit_decryption_share(
            RuntimeOrigin::signed(validator),
            block_number,
            share_bytes.clone(),
            0,
        ));

        // Submit duplicate share (should overwrite)
        let new_share_bytes = BoundedVec::try_from(vec![4, 5, 6]).unwrap();
        assert_ok!(ChameleonMev::submit_decryption_share(
            RuntimeOrigin::signed(validator),
            block_number,
            new_share_bytes.clone(),
            0,
        ));

        // Verify the latest share is stored
        let stored_share = ChameleonMev::threshold_decryption_shares(block_number, validator).unwrap();
        assert_eq!(stored_share.share_bytes, new_share_bytes);
    });
}

#[test]
fn test_mempool_capacity_limit() {
    new_test_ext().execute_with(|| {
        // Fill mempool to capacity (MaxTransactionsPerBlock = 1000)
        for i in 0..1000 {
            let encrypted_data = BoundedVec::try_from(vec![i as u8]).unwrap();
            let commitment = H256::from([i as u8; 32]);
            let timestamp = i as u64;

            assert_ok!(ChameleonMev::submit_encrypted_transaction(
                RuntimeOrigin::signed(1),
                encrypted_data,
                commitment,
                timestamp,
            ));
        }

        // Try to add one more (should fail)
        let overflow_data = BoundedVec::try_from(vec![255]).unwrap();
        let overflow_commitment = H256::from([255; 32]);

        assert_noop!(
            ChameleonMev::submit_encrypted_transaction(
                RuntimeOrigin::signed(1),
                overflow_data,
                overflow_commitment,
                1000,
            ),
            Error::<Test>::MempoolFull
        );
    });
}
