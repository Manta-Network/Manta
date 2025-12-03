# Chameleon Bridge Pallet

## Overview

The Chameleon Bridge pallet enables secure cross-chain asset transfers between Ethereum and the Chameleon Network. It supports bridging of ETH, USDC, USDT, and WBTC through a trustless multi-signature validation system.

## Features

### Supported Assets
- **ETH** → **wETH** (Wrapped Ethereum)
- **USDC** → **cUSDC** (Chameleon USDC)
- **USDT** → **cUSDT** (Chameleon USDT)
- **WBTC** → **cWBTC** (Chameleon Wrapped Bitcoin)

### Bridge Operations

#### Lock & Mint (Ethereum → Chameleon)
1. User locks assets on Ethereum smart contract
2. Ethereum emits `AssetLocked` event
3. Chameleon validators observe and verify the event
4. 5-of-9 validators must confirm the deposit
5. Wrapped tokens are minted to user's Chameleon account

#### Burn & Unlock (Chameleon → Ethereum)
1. User burns wrapped tokens on Chameleon
2. Withdrawal request is created
3. 5-of-9 validators sign the unlock transaction
4. Multi-sig transaction unlocks original assets on Ethereum

### Security Features

- **Multi-Signature Validation**: 5-of-9 validator threshold
- **Fraud Proof System**: 6-hour challenge window for disputed transactions
- **Confirmation Requirements**: Multiple block confirmations before processing
- **Emergency Pause**: Ability to halt bridge operations if needed

### Fee Structure

From the Chameleon fee structure:
- **Shielding (Lock & Mint)**: 0.02% or 0.1 CHML (whichever higher)
- **Unshielding (Burn & Unlock)**: 0.05% or 0.1 CHML (whichever higher)

**Fee Distribution (EVM Bridges)**:
- 70% → Bridge Operations Reserve (Treasury)
- 30% → Treasury

## Architecture

### Ethereum Smart Contract
- Manages asset locking/unlocking
- Emits events for Chameleon validators
- Enforces multi-sig requirements
- Implements fraud proof challenges

### Chameleon Pallet
- Monitors Ethereum events
- Manages wrapped token minting/burning
- Coordinates validator consensus
- Handles fee collection and distribution

### Validator Network
- 9 validators with 5-of-9 threshold
- Observe Ethereum events
- Sign withdrawal transactions
- Participate in fraud proof system

## Usage

### For Users

```rust
// Lock ETH on Ethereum and mint wETH on Chameleon
bridge.lock_asset(eth_address, amount, chameleon_address);

// Burn wETH on Chameleon and unlock ETH on Ethereum
bridge.burn_for_unlock(wrapped_asset_id, amount, eth_destination);
```

### For Validators

```rust
// Report observed Ethereum lock event
bridge.report_lock_event(eth_tx_hash, user, asset, amount);

// Sign withdrawal transaction
bridge.sign_withdrawal(withdrawal_id, signature);
```

## Configuration

```rust
parameter_types! {
    pub const BridgePalletId: PalletId = CHAMELEON_BRIDGE_PALLET_ID;
    pub const ConfirmationBlocks: u32 = 12; // Ethereum confirmations
    pub const ValidatorThreshold: u32 = 5;  // 5-of-9 multi-sig
    pub const ChallengePeriod: BlockNumber = 3600; // 6 hours in blocks
}

impl pallet_chameleon_bridge::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Assets = Assets;
    type PalletId = BridgePalletId;
    type ConfirmationBlocks = ConfirmationBlocks;
    type ValidatorThreshold = ValidatorThreshold;
    type ChallengePeriod = ChallengePeriod;
    type WeightInfo = ();
}
```

## Security Considerations

1. **Validator Key Management**: Secure storage and rotation of validator keys
2. **Smart Contract Audits**: Regular security audits of Ethereum contracts
3. **Monitoring**: Continuous monitoring of bridge operations
4. **Emergency Procedures**: Clear procedures for handling security incidents
5. **Fraud Detection**: Automated systems to detect suspicious activities

## Testing

The pallet includes comprehensive tests covering:
- Lock and mint operations
- Burn and unlock operations
- Multi-signature validation
- Fraud proof mechanisms
- Fee calculation and distribution
- Edge cases and error conditions

## Future Enhancements

- Support for additional EVM chains (BSC, Polygon, Arbitrum)
- Integration with other bridge protocols
- Advanced fraud detection algorithms
- Automated validator key rotation
- Cross-chain governance mechanisms
