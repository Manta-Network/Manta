# Chameleon MEV Protection Pallet

## Overview

The Chameleon MEV Protection Pallet implements a commit-reveal scheme with encrypted mempool to prevent MEV (Miner/Validator Extractable Value) attacks such as front-running and sandwich attacks.

## Key Features

### 1. Encrypted Mempool
- Transactions are encrypted when submitted to the mempool
- Only decrypted when included in a block
- Prevents validators from seeing pending transaction details

### 2. Fair Ordering
- Transactions ordered by timestamp (First-In-First-Out)
- No priority gas fees - all transactions pay uniform fees
- Prevents transaction reordering for profit

### 3. Commit-Reveal Mechanism
- **Block N (Commit Phase):** Validators commit to encrypted transaction order
- **Block N+1 (Reveal Phase):** Transactions decrypted and executed
- Ensures ordering cannot be manipulated after commitment

## Architecture

```
User Transaction → Encryption → Mempool (Encrypted) → Block Proposal (Ordered by Timestamp)
                                                            ↓
Execution ← Decryption ← Reveal Phase ← Commitment Hash
```

## Constants

- **Pallet ID:** `chmlmevp` (8 bytes)
- **Reveal Delay:** 1 block (commit-reveal scheme)
- **Max Encryption Overhead:** 50ms per transaction
- **Max Decryption Time:** 200ms per block

## Dispatchable Functions

### `submit_encrypted_transaction`
Submits an encrypted transaction to the mempool with timestamp-based ordering.

### `commit_block_order`
Validators commit to the ordering of encrypted transactions in a block.

### `reveal_transactions`
Validators provide decryption shares to reveal and execute transactions.

## Storage

- **EncryptedMempool:** Pending encrypted transactions ordered by timestamp
- **BlockCommitments:** Hash commitments to transaction ordering
- **DecryptionShares:** Validator shares for threshold decryption

## MEV Attack Prevention

### Front-Running Prevention
- Attackers cannot see pending transaction details
- Ordering based on timestamp, not gas fees
- No opportunity to submit competing transactions

### Sandwich Attack Prevention
- Transaction amounts and types hidden until execution
- Cannot construct sandwich attacks without visibility
- Fair ordering prevents manipulation

## Performance

- **Target TPS:** 100+ (maintained with <10% overhead)
- **Encryption Time:** <50ms per transaction
- **Block Decryption:** <200ms for 1000 transactions
- **Finality Delay:** +1 block (acceptable for MEV protection)

## Security Model

- **Threshold Encryption:** Requires majority of validators to decrypt
- **Commitment Binding:** Cannot change order after commitment
- **Timestamp Integrity:** Prevents timestamp manipulation
- **No Single Point of Failure:** Distributed decryption

## Usage Example

```rust
// Submit encrypted transaction
let encrypted_tx = encrypt_transaction(transaction, public_key);
ChameleonMev::submit_encrypted_transaction(
    origin,
    encrypted_tx,
    timestamp,
    commitment_hash
)?;

// Validators commit to ordering (automatic)
// Validators reveal transactions (automatic)
```

## Integration

This pallet integrates with:
- **Transaction Pool:** Custom encrypted mempool
- **Block Production:** Modified to handle commit-reveal
- **Consensus:** Ensures validator participation in decryption

## License

GPL-3.0 (same as Manta Network)
