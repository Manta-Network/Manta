# Chameleon pDEX (Privacy-preserving Decentralized Exchange)

A privacy-preserving decentralized exchange (pDEX) implementation for the Chameleon Network, featuring automated market maker (AMM) pools with constant product formula and LP rewards distribution.

## Overview

The Chameleon pDEX enables private token swaps without revealing trade details while maintaining the benefits of automated market making. Key features include:

- **AMM Model**: Uniswap v2-style constant product formula (x × y = k)
- **Privacy**: Swap amounts hidden via zkSNARKs (future implementation)
- **LP Rewards**: 30% of validator rewards distributed to liquidity providers
- **Fee Structure**: 0.25% swap fee (90% to LPs, 10% to Treasury)
- **Slippage Protection**: User-defined maximum slippage tolerance

## Supported Trading Pairs

1. **CHML/ETH** - Primary trading pair (highest rewards)
2. **CHML/USDC** - Stable pair for price discovery
3. **CHML/WBTC** - Bridge liquidity for Bitcoin exposure

## Core Components

### 1. Liquidity Pools

- Constant product AMM: `x × y = k`
- Dynamic fee collection (0.25% default)
- LP token minting/burning for position tracking
- Proportional reward distribution

### 2. AMM Mathematics

```rust
// Swap calculation with fee
let input_with_fee = input_amount * (1_000_000_000 - fee_percent) / 1_000_000_000;
let output_amount = (output_reserve * input_with_fee) / (input_reserve + input_with_fee);
```

### 3. LP Rewards

- **Source**: 30% of validator emissions (19.5M CHML over 20 years)
- **Distribution**: Proportional to LP token holdings
- **Frequency**: Claimable per session (4 hours)
- **Vesting**: 50% instant, 50% vested over 90 days

### 4. Fee Distribution

- **Swap Fee**: 0.25% of trade volume
- **LP Share**: 90% of fees (auto-compounded)
- **Treasury Share**: 10% of fees (governance controlled)

## Usage Examples

### Adding Liquidity

```rust
// Add 1000 CHML + 10 ETH to CHML/ETH pool
ChameleonPdex::add_liquidity(
    RuntimeOrigin::signed(alice()),
    CHML,              // Asset A
    ETH,               // Asset B
    1000_000_000_000_000_000_000,  // 1000 CHML (18 decimals)
    10_000_000_000_000_000_000,    // 10 ETH (18 decimals)
    950_000_000_000_000_000_000,   // Min CHML (5% slippage)
    9_500_000_000_000_000_000,     // Min ETH (5% slippage)
)?;
```

### Swapping Tokens

```rust
// Swap 100 CHML for ETH with 1% max slippage
ChameleonPdex::swap_exact_tokens_for_tokens(
    RuntimeOrigin::signed(bob()),
    100_000_000_000_000_000_000,   // Input: 100 CHML
    990_000_000_000_000_000,       // Min output: 0.99 ETH (1% slippage)
    vec![CHML, ETH],               // Path: CHML -> ETH
)?;
```

### Claiming LP Rewards

```rust
// Claim accumulated LP rewards
let pool_id = PoolId::new(CHML, ETH);
ChameleonPdex::claim_rewards(
    RuntimeOrigin::signed(alice()),
    pool_id,
)?;
```

## Economic Model

### Tokenomics Integration

- **LP Rewards Pool**: 19.5M CHML (30% of 65M validator pool)
- **Year 1 LP Rewards**: 2.22M CHML (30% of 7.4M)
- **Declining Schedule**: 10% reduction per year over 20 years

### Dynamic Yield Optimization

```rust
// Pool weight calculation for reward distribution
let pool_weight = tvl * volume_24h * utilization_rate / risk_factor;
let pool_reward_share = pool_weight / total_weight;
```

### Fee Economics

- **Trading Volume**: Target $10M+ daily volume
- **Fee Revenue**: $25K+ daily at 0.25% fee
- **LP APY**: 15-25% estimated (volume dependent)
- **Treasury Revenue**: $2.5K+ daily (10% of fees)

## Security Features

### Slippage Protection

- User-defined maximum slippage tolerance
- Price impact warnings for large trades
- MEV protection via encrypted mempool (future)

### Pool Security

- Minimum liquidity requirements
- Emergency pause functionality
- Governance-controlled parameter updates

## Technical Architecture

### Storage Items

- `LiquidityPools`: Pool state and reserves
- `LpPositions`: User LP token balances
- `PoolRewards`: Accumulated rewards per pool
- `UserRewards`: Claimable rewards per user

### Extrinsics

- `create_pool`: Initialize new trading pair
- `add_liquidity`: Provide liquidity to pool
- `remove_liquidity`: Withdraw liquidity from pool
- `swap_exact_tokens_for_tokens`: Execute token swap
- `claim_lp_rewards`: Claim accumulated rewards

### Events

- `PoolCreated`: New pool initialization
- `LiquidityAdded`: Liquidity provision
- `LiquidityRemoved`: Liquidity withdrawal
- `Swap`: Token swap execution
- `RewardsClaimed`: LP reward distribution

## Future Enhancements

### Privacy Features (Phase 2)

- zkSNARK integration for private swap amounts
- Nullifier-based double-spend prevention
- Encrypted transaction mempool

### Advanced Features (Phase 3)

- Multi-hop routing for indirect pairs
- Concentrated liquidity (Uniswap v3 style)
- Impermanent loss protection
- Flash loans for arbitrage

## Implementation Status

### ✅ Completed Features

1. **Full AMM Implementation**
   - Constant product formula (x × y = k)
   - Swap output calculation with 0.25% fee
   - LP token minting/burning
   - Slippage protection for all operations

2. **Complete Extrinsics**
   - `create_pool`: Create new liquidity pools
   - `add_liquidity`: Add liquidity with slippage protection
   - `remove_liquidity`: Remove liquidity with minimum amounts
   - `swap_exact_tokens_for_tokens`: Execute swaps with path validation
   - `claim_rewards`: Claim accumulated LP rewards

3. **Fee Distribution System**
   - 0.25% swap fee (25 basis points)
   - 90% to LPs (auto-compounded into reserves)
   - 10% to Treasury (governance controlled)

4. **LP Position Tracking**
   - Individual LP positions per user per pool
   - Automatic position creation/deletion
   - Block-level deposit tracking

5. **Pool Analytics**
   - Volume tracking for reward calculations
   - TVL calculation helpers
   - Quote functions for UI integration

6. **Comprehensive Testing**
   - AMM math validation
   - Slippage protection tests
   - Fee calculation and distribution
   - Edge case handling (empty pools, invalid paths)
   - Constant product invariant verification

### 🚧 Future Implementation

1. **Privacy Features**
   - zkSNARK integration for private amounts
   - Nullifier-based double-spend prevention
   - Encrypted mempool integration

2. **Advanced AMM Features**
   - Multi-hop routing (A→B→C swaps)
   - Concentrated liquidity positions
   - Impermanent loss protection

3. **Reward System Integration**
   - CHML token minting for rewards
   - Vesting schedule implementation
   - Dynamic yield optimization

## License

GPL-3.0
