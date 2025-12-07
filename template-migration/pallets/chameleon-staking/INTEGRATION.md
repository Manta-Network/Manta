# Chameleon Staking Pallet Integration Guide

## Overview

The Chameleon Staking Pallet provides enhanced staking functionality with delegation, unbonding, rewards, and slashing mechanisms.

## Features Implemented

### ✅ Core Storage
- `Validators`: Validator information storage
- `Delegations`: Delegation mapping (delegator, validator) -> amount
- `UnbondingRequests`: Unbonding requests with 14-day period
- `PendingRewards`: Pending rewards per account
- `TotalStaked`: Global staked amount tracking
- `CurrentEra`: Era tracking for rewards

### ✅ Extrinsics (Callable Functions)

1. **`join_candidates(stake)`**
   - Allows users to become validator candidates
   - Requires minimum stake of 1,750 CHML
   - Locks tokens using Currency trait
   - Creates ValidatorInfo with default 10% commission

2. **`delegate(validator, amount)`**
   - Delegate tokens to a validator without running a node
   - No minimum delegation amount (accessible to all)
   - Prevents self-delegation
   - Updates validator's total stake

3. **`undelegate(validator)`**
   - Remove delegation and start 14-day unbonding period
   - Creates UnbondingRequest with unlock time
   - Updates validator's total stake

4. **`withdraw_unbonded()`**
   - Withdraw tokens after unbonding period completes
   - Removes completed unbonding requests
   - Unlocks tokens

5. **`claim_rewards()`**
   - Claim pending staking rewards
   - Transfers rewards to account balance
   - Clears pending rewards

6. **`set_commission(commission)`**
   - Validators can set commission rate (0-100%)
   - Commission taken from delegator rewards

7. **`leave_candidates()`**
   - Validators can leave the candidate pool
   - Requires no active delegators
   - Starts unbonding for self-stake

### ✅ Reward Distribution

- **Era-based rewards**: `distribute_rewards(total_reward)`
- **Proportional distribution**: Based on stake ratio
- **Commission system**: Validators earn commission from delegator rewards
- **Fair allocation**: Validators get their proportional share + commission

### ✅ Slashing Mechanism

- **`slash_validator(validator, offense)`**
- **Downtime slashing**: 0.1% for extended downtime (>12 hours)
- **Double-sign slashing**: 5% for double signing
- **Proportional slashing**: Affects both validator and delegators
- **Status update**: Marks validator as slashed

### ✅ Helper Functions

- `is_validator(account)`: Check if account is a validator
- `get_validator_info(account)`: Get validator information
- `get_delegation(delegator, validator)`: Get delegation amount

## Configuration Requirements

```rust
impl pallet_chameleon_staking::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances; // Must implement LockableCurrency + ReservableCurrency
    type MinValidatorStake = MinValidatorStake; // 1,750 CHML
    type UnbondingPeriod = UnbondingPeriod; // 14 days in blocks
    type MaxDelegatorsPerValidator = MaxDelegatorsPerValidator; // e.g., 500
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator; // e.g., 100
    type MaxUnbondingRequests = MaxUnbondingRequests; // e.g., 10
}
```

## Usage Examples

### Becoming a Validator

```rust
// Join as validator with 2000 CHML stake
let stake = 2_000_000_000_000_000_000_000; // 2000 CHML with 18 decimals
ChameleonStaking::join_candidates(origin, stake)?;

// Set commission to 15%
let commission = Perbill::from_percent(15);
ChameleonStaking::set_commission(origin, commission)?;
```

### Delegating Tokens

```rust
// Delegate 1000 CHML to validator
let validator_account = AccountId::from([1u8; 32]);
let delegation_amount = 1_000_000_000_000_000_000_000; // 1000 CHML
ChameleonStaking::delegate(origin, validator_account, delegation_amount)?;
```

### Unbonding and Withdrawal

```rust
// Start unbonding (14-day period)
ChameleonStaking::undelegate(origin, validator_account)?;

// After 14 days, withdraw unbonded tokens
ChameleonStaking::withdraw_unbonded(origin)?;
```

### Claiming Rewards

```rust
// Claim pending staking rewards
ChameleonStaking::claim_rewards(origin)?;
```

## Events

The pallet emits the following events:

- `ValidatorJoined { validator, stake }`
- `Delegated { delegator, validator, amount }`
- `Undelegated { delegator, validator, amount }`
- `UnbondingStarted { who, amount, unlock_at }`
- `RewardsDistributed { era, total_reward }`
- `ValidatorSlashed { validator, amount, offense }`
- `CommissionSet { validator, commission }`
- `RewardsClaimed { who, amount }`
- `UnbondedWithdrawn { who, amount }`
- `ValidatorLeft { validator }`

## Error Handling

The pallet includes comprehensive error handling:

- `ValidatorNotFound`: Validator doesn't exist
- `InsufficientStake`: Below minimum stake requirement
- `TooManyDelegators`: Validator has too many delegators
- `TooManyDelegations`: Delegator has too many delegations
- `AlreadyDelegated`: Already delegated to this validator
- `NotDelegated`: No delegation exists
- `InvalidCommission`: Commission rate invalid (>100%)
- `AlreadyValidator`: Account is already a validator
- `CannotDelegateToSelf`: Cannot delegate to own account
- `NoUnbondedTokens`: No tokens ready for withdrawal
- `NoRewardsToClaim`: No pending rewards
- `ArithmeticOverflow/Underflow`: Math operation errors

## Security Features

1. **Token Locking**: Uses Currency trait's locking mechanism
2. **Unbonding Period**: 14-day security delay
3. **Slashing Protection**: Penalties for misbehavior
4. **Overflow Protection**: Safe arithmetic operations
5. **Access Control**: Proper origin checking
6. **State Consistency**: Atomic operations and rollback on errors

## Testing

The pallet includes comprehensive tests covering:

- Validator lifecycle (join/leave)
- Delegation mechanics
- Unbonding and withdrawal
- Reward distribution
- Slashing scenarios
- Error conditions
- Edge cases

## Integration Status

**Status**: ✅ **COMPLETE** - Full implementation ready for integration

**Compilation**: The pallet compiles successfully with proper dependencies

**Testing**: Comprehensive test suite covers all functionality

**Documentation**: Complete API documentation and integration guide

## Next Steps

1. **Runtime Integration**: Add to runtime configuration
2. **Genesis Configuration**: Set initial validators and parameters
3. **UI Integration**: Connect to frontend for user interactions
4. **Monitoring**: Add metrics and monitoring for validator performance
5. **Governance**: Connect to governance for parameter updates

## Constants from Chameleon Network

The pallet uses constants from `manta_primitives::chameleon_constants`:

- `MIN_VALIDATOR_STAKE`: 1,750 CHML
- `UNBONDING_PERIOD_BLOCKS`: 14 days in blocks
- `SLASHING_DOWNTIME`: 0.1%
- `SLASHING_DOUBLE_SIGN`: 5%
- `MAX_VALIDATORS`: 200
- `MAX_DELEGATIONS_PER_DELEGATOR`: 100
- `MAX_DELEGATORS_PER_VALIDATOR`: 500

This implementation provides a robust, secure, and user-friendly staking system that enhances the default Substrate staking with advanced delegation features and optimized reward distribution.
