# Chameleon Staking Pallet Integration Guide

## Overview

This guide shows how to integrate the Chameleon Staking pallet into a Substrate runtime.

## Runtime Integration

### 1. Add to Runtime Cargo.toml

```toml
[dependencies]
pallet-chameleon-staking = { path = "../../pallets/chameleon-staking", default-features = false }

[features]
std = [
    # ... other pallets
    "pallet-chameleon-staking/std",
]
runtime-benchmarks = [
    # ... other pallets
    "pallet-chameleon-staking/runtime-benchmarks",
]
try-runtime = [
    # ... other pallets
    "pallet-chameleon-staking/try-runtime",
]
```

### 2. Configure in Runtime

```rust
use manta_primitives::chameleon_constants::{
    staking::*,
    time::*,
    CHAMELEON_STAKING_PALLET_ID,
};

// Parameter types
parameter_types! {
    pub const ChameleonStakingPalletId: PalletId = CHAMELEON_STAKING_PALLET_ID;
    pub const MaxDelegationsPerDelegator: u32 = MAX_DELEGATIONS_PER_DELEGATOR;
    pub const MaxDelegatorsPerValidator: u32 = MAX_DELEGATORS_PER_VALIDATOR;
    pub const MinValidatorStake: Balance = MIN_VALIDATOR_STAKE;
    pub const UnbondingPeriod: BlockNumber = UNBONDING_PERIOD_BLOCKS;
    pub const SlashingDowntime: Perbill = SLASHING_DOWNTIME;
    pub const SlashingDoubleSign: Perbill = SLASHING_DOUBLE_SIGN;
}

// Pallet configuration
impl pallet_chameleon_staking::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = pallet_chameleon_staking::weights::SubstrateWeight<Runtime>;
    type PalletId = ChameleonStakingPalletId;
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
    type MaxDelegatorsPerValidator = MaxDelegatorsPerValidator;
    type MinValidatorStake = MinValidatorStake;
    type UnbondingPeriod = UnbondingPeriod;
    type SlashingDowntime = SlashingDowntime;
    type SlashingDoubleSign = SlashingDoubleSign;
}

// Add to construct_runtime! macro
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // ... other pallets
        ChameleonStaking: pallet_chameleon_staking,
    }
);
```

### 3. Genesis Configuration

```rust
// In genesis configuration
use pallet_chameleon_staking::GenesisConfig as ChameleonStakingConfig;

ChameleonStakingConfig {
    validators: vec![
        // (validator_account, initial_stake)
        (get_account_id_from_seed::<sr25519::Public>("Alice"), 150_000 * CHML),
        (get_account_id_from_seed::<sr25519::Public>("Bob"), 150_000 * CHML),
        // ... more genesis validators
    ],
}
```

## Usage Examples

### For Delegators

```rust
// Delegate 100 CHML to a validator
let validator = AccountId::from([1u8; 32]);
let amount = 100 * CHML;

ChameleonStaking::delegate(
    RuntimeOrigin::signed(delegator),
    validator,
    amount
)?;

// Undelegate from validator (starts 14-day unbonding)
ChameleonStaking::undelegate(
    RuntimeOrigin::signed(delegator),
    validator
)?;

// Withdraw unbonded funds after unbonding period
ChameleonStaking::withdraw_unbonded(
    RuntimeOrigin::signed(delegator)
)?;
```

### For Validators

```rust
// Join as validator candidate
let bond = 1750 * CHML; // Minimum stake
ChameleonStaking::join_candidates(
    RuntimeOrigin::signed(validator),
    bond
)?;

// Set commission rate (10%)
let commission = Perbill::from_percent(10);
ChameleonStaking::set_commission(
    RuntimeOrigin::signed(validator),
    commission
)?;

// Leave validator pool
ChameleonStaking::leave_candidates(
    RuntimeOrigin::signed(validator)
)?;
```

### Query Functions

```rust
// Get validator information
let validator_info = ChameleonStaking::validators(&validator_account);

// Get delegation information
let delegation = ChameleonStaking::delegations(&delegator, &validator);

// Get unbonding requests
let unbonding = ChameleonStaking::unbonding_requests(&account);

// Get total staked in network
let total_staked = ChameleonStaking::total_staked();

// Calculate validator APY
let apy = ChameleonStaking::calculate_validator_apy(&validator)?;
```

## Events

The pallet emits the following events:

- `Delegated { delegator, validator, amount }` - When delegation is made
- `Undelegated { delegator, validator, amount }` - When delegation is removed
- `UnbondingStarted { who, amount, unlock_at }` - When unbonding starts
- `UnbondingCompleted { who, amount }` - When unbonding completes
- `RewardsDistributed { era, total_reward }` - When era rewards are distributed
- `ValidatorSlashed { validator, amount, offense }` - When validator is slashed
- `CommissionSet { validator, commission }` - When commission is updated

## Constants

Key constants from `manta_primitives::chameleon_constants::staking`:

- `MIN_VALIDATOR_STAKE`: 1,750 CHML
- `MAX_VALIDATORS`: 200
- `MAX_DELEGATORS_PER_VALIDATOR`: 500
- `MAX_DELEGATIONS_PER_DELEGATOR`: 100
- `SLASHING_DOWNTIME`: 0.1%
- `SLASHING_DOUBLE_SIGN`: 5.0%
- `UNBONDING_PERIOD_BLOCKS`: 201,600 (14 days)

## Reward Distribution

Rewards are automatically distributed every era based on:

1. **Stake Weight**: Validator's share of total network stake
2. **Performance**: Block production success rate and uptime
3. **Governance**: Bonus for active governance participation
4. **Commission**: Validator's cut of delegator rewards

### Reward Formula

```
validator_reward = base_reward × stake_weight × uptime_multiplier × performance_multiplier × governance_bonus

where:
- stake_weight = validator_total_stake / total_network_stake
- uptime_multiplier = validator_uptime_percent / 100
- performance_multiplier = block_success_rate / 100
- governance_bonus = 1.1 if votes > 10, else 1.0
```

## Slashing Conditions

Automatic slashing occurs for:

1. **Extended Downtime** (>12 hours): 0.1% slash
2. **Double Signing**: 5% slash + validator deactivation

Slashing affects both validator self-stake and delegator stakes proportionally.

## Migration from Parachain Staking

If migrating from the existing parachain-staking pallet:

1. Export existing validator and delegator data
2. Configure genesis with current validators
3. Migrate delegations using the new delegation system
4. Update any dependent pallets to use new interfaces

## Testing

Run the pallet tests:

```bash
cargo test -p pallet-chameleon-staking
```

For integration testing, see the mock runtime in `src/mock.rs`.
