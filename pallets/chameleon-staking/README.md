# Chameleon Staking Pallet

Enhanced staking mechanism for Chameleon Network with advanced delegation features and reward optimization.

## Overview

The Chameleon Staking pallet extends the base parachain-staking functionality with:

- **Enhanced Delegation**: No minimum delegation amounts, making staking accessible to all users
- **Reward Optimization**: Performance-based rewards considering uptime, block production, and governance participation
- **Slashing Conditions**: Automated slashing for downtime (0.1%) and double-signing (5%)
- **14-Day Unbonding**: Standard unbonding period for network security

## Features

### 1. Delegation Mechanism

- Users can delegate CHML to validators without running a node
- Delegators earn proportional rewards minus validator commission
- No minimum delegation amount (accessible to all)
- Instant delegation, 14-day unbonding period

### 2. Reward Distribution

- Rewards based on stake + performance + uptime
- Validator commission system
- Bonus for governance participation
- Automatic slashing for poor performance

### 3. Validator Performance Tracking

- Uptime monitoring
- Block production success rate
- Governance participation tracking
- Automated performance-based rewards

## Constants

- **Minimum Validator Stake**: 1,750 CHML
- **Unbonding Period**: 14 days (201,600 blocks)
- **Slashing Rates**: 0.1% (downtime), 5% (double-signing)
- **Max Validators**: 200
- **Max Delegators per Validator**: 500

## Usage

### For Delegators

```rust
// Delegate to a validator
let validator = AccountId::from([1u8; 32]);
let amount = 100 * CHML; // 100 CHML
ChameleonStaking::delegate(origin, validator, amount)?;

// Undelegate from a validator
ChameleonStaking::undelegate(origin, validator)?;
```

### For Validators

```rust
// Join as validator candidate
let bond = 1750 * CHML; // Minimum stake
ChameleonStaking::join_candidates(origin, bond)?;

// Set commission rate
let commission = Perbill::from_percent(10); // 10%
ChameleonStaking::set_commission(origin, commission)?;
```

## Reward Formula

```
validator_reward = base_reward * stake_weight * uptime_multiplier * performance_multiplier * governance_bonus

where:
- stake_weight = validator_total_stake / total_network_stake
- uptime_multiplier = validator_uptime_percent / 100
- performance_multiplier = block_success_rate / 100
- governance_bonus = 1.1 if votes > 10, else 1.0
```

## Integration

This pallet extends `pallet-parachain-staking` and integrates with:

- **Emission Schedule**: Uses constants from `manta-primitives::chameleon_constants::emission`
- **Time Constants**: Uses `manta-primitives::chameleon_constants::time`
- **Staking Constants**: Uses `manta-primitives::chameleon_constants::staking`

## License

GPL-3.0