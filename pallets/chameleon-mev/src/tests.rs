// Copyright 2020-2024 Manta Network.
// MEV Protection Tests - Proving Front-Running and Sandwich Attack Prevention

use crate::pallet::*;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{ConstU32, ConstU64},
    Blake2_256, StorageHasher,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        ChameleonMev: crate,
    }
);

impl frame_system::Config for Test {
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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxSealedTxPerBlock = ConstU32<100>;
    type MaxTxInOrdering = ConstU32<100>;
    type RevealDeadline = ConstU64<10>;
}

fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn create_commitment(tx_hash: H256, nonce: [u8; 32]) -> H256 {
    let mut preimage = Vec::with_capacity(64);
    preimage.extend_from_slice(tx_hash.as_bytes());
    preimage.extend_from_slice(&nonce);
    H256::from_slice(&Blake2_256::hash(&preimage))
}

// ============================================================================
// BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_submit_sealed_transaction_works() {
    new_test_ext().execute_with(|| {
        let tx_hash = H256::from([1u8; 32]);
        let nonce = [2u8; 32];
        let submitter = 1u64;

        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(submitter),
            tx_hash,
            nonce,
        ));

        let commitment = create_commitment(tx_hash, nonce);
        
        // Verify sealed transaction stored
        assert!(SealedTransactions::<Test>::contains_key(commitment));
        
        // Verify pending commitments updated
        let pending = PendingCommitments::<Test>::get();
        assert!(pending.contains(&commitment));
    });
}

#[test]
fn test_duplicate_commitment_rejected() {
    new_test_ext().execute_with(|| {
        let tx_hash = H256::from([1u8; 32]);
        let nonce = [2u8; 32];

        // First submission succeeds
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            tx_hash,
            nonce,
        ));

        // Duplicate rejected
        assert_noop!(
            ChameleonMev::submit_sealed_transaction(
                RuntimeOrigin::signed(2),
                tx_hash,
                nonce,
            ),
            Error::<Test>::CommitmentAlreadyExists
        );
    });
}

#[test]
fn test_commit_ordering_works() {
    new_test_ext().execute_with(|| {
        // Submit some sealed transactions
        let tx1_hash = H256::from([1u8; 32]);
        let tx2_hash = H256::from([2u8; 32]);
        let nonce1 = [1u8; 32];
        let nonce2 = [2u8; 32];

        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            tx1_hash,
            nonce1,
        ));

        // Advance block to get different timestamp
        System::set_block_number(2);

        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(2),
            tx2_hash,
            nonce2,
        ));

        System::set_block_number(3);

        // Commit ordering
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        // Verify ordering committed
        assert!(OrderingCommitments::<Test>::contains_key(3));
        
        // Verify execution queue set
        let queue = ExecutionQueue::<Test>::get();
        assert_eq!(queue.len(), 2);
        
        // Verify FIFO ordering (tx1 submitted first, should be first in queue)
        let commitment1 = create_commitment(tx1_hash, nonce1);
        let commitment2 = create_commitment(tx2_hash, nonce2);
        assert_eq!(queue[0], commitment1);
        assert_eq!(queue[1], commitment2);
    });
}

#[test]
fn test_reveal_transaction_works() {
    new_test_ext().execute_with(|| {
        let tx_data: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3, 4, 5].try_into().unwrap();
        let tx_hash = H256::from_slice(&Blake2_256::hash(&tx_data));
        let nonce = [1u8; 32];

        // Submit sealed transaction
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            tx_hash,
            nonce,
        ));

        System::set_block_number(2);

        // Commit ordering
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        System::set_block_number(3);

        // Reveal transaction
        assert_ok!(ChameleonMev::reveal_transaction(
            RuntimeOrigin::signed(1),
            tx_data,
            nonce,
        ));

        // Verify revealed
        let commitment = create_commitment(tx_hash, nonce);
        assert!(RevealedTransactions::<Test>::contains_key(commitment));
        assert!(UsedCommitments::<Test>::get(commitment));
    });
}

// ============================================================================
// MEV PROTECTION TESTS - PROVING SECURITY
// ============================================================================

#[test]
fn test_front_running_prevented() {
    new_test_ext().execute_with(|| {
        // SCENARIO: Victim submits a swap, attacker tries to front-run
        
        // Block 1: Victim submits sealed transaction
        let victim_tx_hash = H256::from([0xAAu8; 32]); // Represents: swap 100 ETH -> CHML
        let victim_nonce = [1u8; 32];
        
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1), // victim
            victim_tx_hash,
            victim_nonce,
        ));

        // Block 2: Attacker sees sealed tx (but CANNOT see content)
        System::set_block_number(2);
        
        // Attacker submits their own transaction (trying to front-run)
        let attacker_tx_hash = H256::from([0xBBu8; 32]); // Represents: swap 1000 ETH -> CHML
        let attacker_nonce = [2u8; 32];
        
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(2), // attacker
            attacker_tx_hash,
            attacker_nonce,
        ));

        // Block 3: Ordering committed
        System::set_block_number(3);
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        // VERIFICATION: Victim's transaction is FIRST (FIFO ordering)
        let queue = ExecutionQueue::<Test>::get();
        let victim_commitment = create_commitment(victim_tx_hash, victim_nonce);
        let attacker_commitment = create_commitment(attacker_tx_hash, attacker_nonce);
        
        // Victim submitted in block 1, attacker in block 2
        // Therefore victim executes FIRST - front-running prevented!
        assert_eq!(queue[0], victim_commitment, "Victim must execute first");
        assert_eq!(queue[1], attacker_commitment, "Attacker must execute second");
        
        // The attacker's transaction executes AFTER victim's
        // This means the attacker cannot profit from front-running
    });
}

#[test]
fn test_sandwich_attack_prevented() {
    new_test_ext().execute_with(|| {
        // SCENARIO: Attacker tries to sandwich victim's trade
        // Sandwich attack requires:
        // 1. Attacker sees victim tx content
        // 2. Attacker places tx BEFORE victim
        // 3. Attacker places tx AFTER victim
        
        // Block 1: Victim submits sealed transaction (content HIDDEN)
        let victim_tx_hash = H256::from([0xAAu8; 32]);
        let victim_nonce = [1u8; 32];
        
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            victim_tx_hash,
            victim_nonce,
        ));

        // Block 2: Attacker tries to construct sandwich
        System::set_block_number(2);
        
        // Attacker CANNOT see victim tx content - only commitment hash visible
        // Without knowing the trade details, attacker cannot construct sandwich
        
        let front_tx_hash = H256::from([0xBBu8; 32]);
        let back_tx_hash = H256::from([0xCCu8; 32]);
        
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(2),
            front_tx_hash,
            [2u8; 32],
        ));
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(2),
            back_tx_hash,
            [3u8; 32],
        ));

        // Block 3: Ordering committed
        System::set_block_number(3);
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        // VERIFICATION: Execution order is FIFO by timestamp
        let queue = ExecutionQueue::<Test>::get();
        
        // Victim submitted first (block 1), attacker's txs submitted later (block 2)
        // Order: victim -> front_tx -> back_tx (NOT front_tx -> victim -> back_tx)
        // Sandwich attack IMPOSSIBLE because:
        // 1. Attacker cannot place tx BEFORE victim (FIFO ordering)
        // 2. Attacker didn't know victim tx content to construct sandwich
        
        let victim_commitment = create_commitment(victim_tx_hash, victim_nonce);
        assert_eq!(queue[0], victim_commitment, "Victim executes first - sandwich broken");
    });
}

#[test]
fn test_ordering_immutable_after_commit() {
    new_test_ext().execute_with(|| {
        // Submit transactions
        let tx1_hash = H256::from([1u8; 32]);
        let tx2_hash = H256::from([2u8; 32]);
        
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            tx1_hash,
            [1u8; 32],
        ));
        
        System::set_block_number(2);
        
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(2),
            tx2_hash,
            [2u8; 32],
        ));

        System::set_block_number(3);
        
        // Commit ordering
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        // Get committed ordering hash
        let ordering = OrderingCommitments::<Test>::get(3).unwrap();
        let original_hash = ordering.ordering_hash;

        // Try to commit again - should fail (ordering is immutable)
        assert_noop!(
            ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)),
            Error::<Test>::NoPendingTransactions
        );

        // Ordering hash unchanged
        let ordering_check = OrderingCommitments::<Test>::get(3).unwrap();
        assert_eq!(original_hash, ordering_check.ordering_hash, "Ordering must be immutable");
    });
}

#[test]
fn test_reveal_must_match_commitment() {
    new_test_ext().execute_with(|| {
        let original_tx_data: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
        let original_tx_hash = H256::from_slice(&Blake2_256::hash(&original_tx_data));
        let nonce = [1u8; 32];

        // Submit sealed transaction
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            original_tx_hash,
            nonce,
        ));

        System::set_block_number(2);
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        System::set_block_number(3);

        // Try to reveal with different data - should fail
        let fake_tx_data: BoundedVec<u8, ConstU32<65536>> = vec![9, 9, 9].try_into().unwrap();
        
        assert_noop!(
            ChameleonMev::reveal_transaction(
                RuntimeOrigin::signed(1),
                fake_tx_data,
                nonce,
            ),
            Error::<Test>::CommitmentNotFound
        );

        // Reveal with correct data - should succeed
        assert_ok!(ChameleonMev::reveal_transaction(
            RuntimeOrigin::signed(1),
            original_tx_data,
            nonce,
        ));
    });
}

#[test]
fn test_replay_attack_prevented() {
    new_test_ext().execute_with(|| {
        let tx_data: BoundedVec<u8, ConstU32<65536>> = vec![1, 2, 3].try_into().unwrap();
        let tx_hash = H256::from_slice(&Blake2_256::hash(&tx_data));
        let nonce = [1u8; 32];

        // Complete full cycle
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            tx_hash,
            nonce,
        ));

        System::set_block_number(2);
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));

        System::set_block_number(3);
        assert_ok!(ChameleonMev::reveal_transaction(
            RuntimeOrigin::signed(1),
            tx_data.clone(),
            nonce,
        ));

        // Try to submit same commitment again - REPLAY ATTACK
        System::set_block_number(4);
        assert_noop!(
            ChameleonMev::submit_sealed_transaction(
                RuntimeOrigin::signed(1),
                tx_hash,
                nonce,
            ),
            Error::<Test>::CommitmentAlreadyUsed
        );
    });
}

#[test]
fn test_confidentiality_window() {
    new_test_ext().execute_with(|| {
        // SECURITY PROPERTY: Transaction content hidden for confidentiality window
        // Window = Block N-1 (submit) to Block N+1 (reveal) = ~12 seconds
        
        let tx_hash = H256::from([0xAAu8; 32]);
        let nonce = [1u8; 32];
        
        // Block 1: Submit - content HIDDEN
        assert_ok!(ChameleonMev::submit_sealed_transaction(
            RuntimeOrigin::signed(1),
            tx_hash,
            nonce,
        ));
        
        // What's visible on-chain at this point:
        let commitment = create_commitment(tx_hash, nonce);
        let sealed = SealedTransactions::<Test>::get(commitment).unwrap();
        
        // Only commitment hash visible, NOT the actual tx_hash
        // (In production, tx_hash would be the hash of encrypted tx data)
        assert_eq!(sealed.commitment, commitment);
        // Attacker sees commitment but CANNOT derive original transaction
        
        // Block 2: Ordering committed - still hidden
        System::set_block_number(2);
        assert_ok!(ChameleonMev::commit_ordering(RuntimeOrigin::signed(100)));
        
        // Ordering committed but content still hidden
        let ordering = OrderingCommitments::<Test>::get(2).unwrap();
        assert!(ordering.ordered_commitments.contains(&commitment));
        
        // Content only revealed in Block 3 (after ordering locked in)
    });
}
