# Chameleon Bridge Ethereum Contracts

Solidity smart contracts for the Chameleon Network cross-chain bridge.

## Overview

The ChameleonBridge contract enables secure cross-chain asset transfers between Ethereum and Chameleon Network using a multi-signature validation system.

## Supported Assets

- **ETH** (Native Ethereum)
- **USDC** (USD Coin) - `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- **USDT** (Tether USD) - `0xdAC17F958D2ee523a2206206994597C13D831ec7`
- **WBTC** (Wrapped Bitcoin) - `0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599`

## Security Features

- **Multi-Signature Validation**: 5-of-9 validator threshold
- **Reentrancy Protection**: OpenZeppelin ReentrancyGuard
- **Pausable**: Emergency pause functionality
- **Minimum Amounts**: Dust attack prevention
- **Signature Verification**: ECDSA signature recovery

## Contract Functions

### Lock Functions

- `lockETH(bytes32 chameleonRecipient)` - Lock ETH for bridging
- `lockToken(address token, uint256 amount, bytes32 chameleonRecipient)` - Lock ERC20 tokens

### Unlock Functions

- `unlock(address token, address recipient, uint256 amount, bytes32 withdrawalId, bytes[] signatures)` - Unlock assets with multi-sig

### Admin Functions

- `addValidator(address validator)` - Add new validator
- `removeValidator(address validator)` - Remove validator
- `addSupportedToken(address token, uint256 minAmount)` - Add token support
- `pause()` / `unpause()` - Emergency controls

## Installation

```bash
cd /app/contracts/ethereum
npm install
```

## Compilation

```bash
npx hardhat compile
```

## Testing

```bash
npx hardhat test
```

## Deployment

1. Set environment variables:
```bash
export PRIVATE_KEY="your_private_key"
export GOERLI_URL="your_goerli_rpc_url"
export ETHERSCAN_API_KEY="your_etherscan_api_key"
```

2. Deploy to testnet:
```bash
npx hardhat run scripts/deploy.js --network goerli
```

## Validator Setup

Validators must:
1. Monitor Ethereum events (`Locked` events)
2. Validate transactions on Chameleon Network
3. Sign withdrawal requests with their private keys
4. Submit signatures for multi-sig unlocks

## Event Monitoring

### Lock Events
```solidity
event Locked(
    address indexed token,
    address indexed sender,
    bytes32 indexed recipient,
    uint256 amount,
    uint256 nonce
);
```

### Unlock Events
```solidity
event Unlocked(
    address indexed token,
    address indexed recipient,
    uint256 amount,
    bytes32 withdrawalId
);
```

## Security Considerations

- Always verify validator signatures
- Implement proper nonce management
- Monitor for unusual transaction patterns
- Regular security audits recommended
- Emergency pause in case of issues

## Gas Optimization

- Batch operations when possible
- Use events for off-chain monitoring
- Optimize signature verification
- Consider gas limits for multi-sig operations

## Integration with Chameleon Network

The Ethereum contract works in conjunction with the Chameleon Bridge pallet:

1. **Lock on Ethereum** → **Mint on Chameleon**
2. **Burn on Chameleon** → **Unlock on Ethereum**

Validators observe both chains and facilitate secure cross-chain transfers.