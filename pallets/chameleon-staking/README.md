# Chameleon Staking Pallet

## Overview

The Chameleon Staking Pallet is an enhanced staking mechanism that improves upon Substrate's default staking with advanced delegation features, optimized reward distribution, and user-friendly design.

## Key Features

### 🚀 Enhanced Delegation System
- **No Minimum Amounts**: Anyone can delegate any amount (no barriers to entry)
- **Instant Delegation**: Immediate delegation without waiting periods
- **Multiple Delegations**: Delegate to multiple validators simultaneously
- **14-Day Unbonding**: Security-focused unbonding period

### 💰 Optimized Reward Distribution
- **Performance-Based**: Rewards based on stake ratio and validator performance
- **Commission System**: Validators earn commission from delegator rewards
- **Era-Based Distribution**: Regular reward distribution cycles
- **Fair Allocation**: Proportional rewards for all participants

### ⚡ Advanced Features
- **Slashing Protection**: Penalties for downtime (0.1%) and double-signing (5%)
- **Validator Management**: Easy join/leave mechanisms
- **Reward Claiming**: Separate reward claiming for better UX
- **State Tracking**: Comprehensive validator and delegation state

## Architecture

### Storage Items

```rust
/// Validator information
Validators: Map<AccountId, ValidatorInfo>

/// Delegations: (delegator, validator) -> amount
Delegations: DoubleMap<AccountId, AccountId, Balance>

/// Unbonding requests with unlock times
UnbondingRequests: Map<AccountId, Vec<UnbondingRequest>>

/// Pending rewards per account
PendingRewards: Map<AccountId, Balance>
```

### Core Types

```rust
pub struct ValidatorInfo {
    pub controller: AccountId,
    pub self_stake: Balance,
    pub total_stake: Balance,
    pub delegator_count: u32,
    pub commission: Perbill,
    pub status: ValidatorStatus,
}

pub struct UnbondingRequest {
    pub amount: Balance,
    pub unlock_at: BlockNumber,
}
```

## Usage

### For Validators

```rust
// Join as validator
ChameleonStaking::join_candidates(origin, stake_amount)?;

// Set commission rate
ChameleonStaking::set_commission(origin, Perbill::from_percent(10))?;

// Leave validator set
ChameleonStaking::leave_candidates(origin)?;
```

### For Delegators

```rust
// Delegate to validator
ChameleonStaking::delegate(origin, validator_id, amount)?;

// Remove delegation
ChameleonStaking::undelegate(origin, validator_id)?;

// Withdraw after unbonding period
ChameleonStaking::withdraw_unbonded(origin)?;
```

### For Everyone

```rust
// Claim pending rewards
ChameleonStaking::claim_rewards(origin)?;
```

## Configuration

```rust
impl pallet_chameleon_staking::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinValidatorStake = MinValidatorStake; // 1,750 CHML
    type UnbondingPeriod = UnbondingPeriod; // 14 days
    type MaxDelegatorsPerValidator = ConstU32<500>;
    type MaxDelegationsPerDelegator = ConstU32<100>;
    type MaxUnbondingRequests = ConstU32<10>;
}
```

## Security

### Slashing Conditions
- **Extended Downtime**: 0.1% slash for >12 hours offline
- **Double Signing**: 5% slash for equivocation
- **Proportional Impact**: Affects both validator and delegators

### Safety Mechanisms
- **Token Locking**: Secure token locking via Currency trait
- **Unbonding Period**: 14-day security delay
- **Overflow Protection**: Safe arithmetic operations
- **State Consistency**: Atomic operations with rollback

## Events

```rust
ValidatorJoined { validator, stake }
Delegated { delegator, validator, amount }
Undelegated { delegator, validator, amount }
UnbondingStarted { who, amount, unlock_at }
RewardsDistributed { era, total_reward }
ValidatorSlashed { validator, amount, offense }
CommissionSet { validator, commission }
RewardsClaimed { who, amount }
UnbondedWithdrawn { who, amount }
ValidatorLeft { validator }
```

## Testing

Run the test suite:

```bash
cargo test -p pallet-chameleon-staking --lib
```

The tests cover:
- Validator lifecycle management
- Delegation mechanics
- Unbonding and withdrawal
- Reward distribution
- Slashing scenarios
- Error conditions
- Edge cases

## Integration

1. **Add to Runtime**: Include in `construct_runtime!` macro
2. **Configure Parameters**: Set constants for your network
3. **Genesis Setup**: Initialize with genesis validators
4. **Connect Currency**: Ensure proper Currency trait implementation
5. **Monitor Performance**: Track validator performance for rewards

## Comparison with Default Staking

| Feature | Default Substrate | Chameleon Staking |
|---------|------------------|-------------------|
| Minimum Delegation | High barriers | No minimum |
| Delegation UX | Complex | User-friendly |
| Reward Distribution | Manual | Automated |
| Commission System | Basic | Advanced |
| Unbonding | 28 days | 14 days |
| Multiple Delegations | Limited | Unlimited |

## License

GPL-3.0 - See [LICENSE](../../LICENSE) for details.

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines.
