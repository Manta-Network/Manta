# CHAMELEON NETWORK CONSTANTS
## Technical Specification for Implementation

**Version:** 1.0  
**Status:** Approved for Implementation  
**Agent:** 1 (Tokenomics)

---

## TOKEN CONSTANTS

```rust
// Chameleon Network Token Constants
pub const CHAMELEON_SS58PREFIX: u8 = 99;  // Custom SS58 prefix for Chameleon
pub const CHAMELEON_DECIMAL: u8 = 18;
pub const CHAMELEON_TOKEN_SYMBOL: &str = "CHML";
pub const CHAMELEON_TOKEN_NAME: &str = "Chameleon Network Token";

// Total Supply: 100,000,000 CHML (with 18 decimals)
pub const TOTAL_SUPPLY: Balance = 100_000_000_000_000_000_000_000_000; // 100M * 10^18

// Allocations (in base units with 18 decimals)
pub const VALIDATOR_LP_REWARDS: Balance = 65_000_000_000_000_000_000_000_000;  // 65M
pub const PUBLIC_PRESALE: Balance = 15_000_000_000_000_000_000_000_000;        // 15M
pub const COMMUNITY_AIRDROP: Balance = 5_000_000_000_000_000_000_000_000;      // 5M
pub const STAKING_INFRASTRUCTURE: Balance = 5_000_000_000_000_000_000_000_000; // 5M
pub const ECOSYSTEM_DEVELOPMENT: Balance = 5_000_000_000_000_000_000_000_000;  // 5M
pub const INITIAL_DEX_LIQUIDITY: Balance = 2_500_000_000_000_000_000_000_000;  // 2.5M
pub const TREASURY_RESERVE: Balance = 2_500_000_000_000_000_000_000_000;       // 2.5M

// Minimum token amounts
pub const MIN_VALIDATOR_STAKE: Balance = 1_750_000_000_000_000_000_000; // 1,750 CHML
pub const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000_000_000;         // 0.001 CHML
```

---

## TIME CONSTANTS

```rust
// Chameleon time-related constants
pub mod time {
    use crate::types::{BlockNumber, Moment};

    // Block time: 6 seconds (faster than Manta's 12s)
    pub const SECONDS_PER_BLOCK: Moment = 6;
    pub const MILLISECS_PER_BLOCK: Moment = SECONDS_PER_BLOCK * 1000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // Time periods in blocks
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
    pub const WEEKS: BlockNumber = DAYS * 7;

    // Staking periods
    pub const UNBONDING_PERIOD_DAYS: u32 = 14;
    pub const UNBONDING_PERIOD_BLOCKS: BlockNumber = DAYS * 14;
    
    // Session length: 4 hours
    pub const SESSION_LENGTH_BLOCKS: BlockNumber = HOURS * 4;
    
    // Emission year (in blocks)
    pub const BLOCKS_PER_YEAR: BlockNumber = DAYS * 365;
}
```

---

## STAKING CONSTANTS

```rust
// Staking configuration
pub mod staking {
    use crate::types::Balance;
    use sp_runtime::Perbill;

    // Minimum stake for validators
    pub const MIN_VALIDATOR_STAKE: Balance = 1_750_000_000_000_000_000_000; // 1,750 CHML
    
    // Slashing rates
    pub const SLASHING_DOWNTIME: Perbill = Perbill::from_parts(1_000_000);   // 0.1%
    pub const SLASHING_DOUBLE_SIGN: Perbill = Perbill::from_parts(50_000_000); // 5.0%
    
    // Validator limits
    pub const MAX_VALIDATORS: u32 = 200;  // Can be adjusted by governance
    pub const MIN_VALIDATORS: u32 = 5;    // Minimum for network security
    
    // Delegation
    pub const MAX_DELEGATIONS_PER_DELEGATOR: u32 = 100;
    pub const MAX_DELEGATORS_PER_VALIDATOR: u32 = 500;
}
```

---

## EMISSION SCHEDULE

```rust
// 20-year declining emission schedule
// 10% year-over-year reduction
pub mod emission {
    use crate::types::Balance;

    // Total rewards pool: 65M CHML
    pub const TOTAL_REWARDS_POOL: Balance = 65_000_000_000_000_000_000_000_000;
    
    // Year 1 emissions: 7.4M CHML (11.38% of pool)
    pub const YEAR_1_EMISSION: Balance = 7_400_000_000_000_000_000_000_000;
    
    // Decay factor: 0.9 (10% reduction per year)
    pub const EMISSION_DECAY_FACTOR: u32 = 90; // 90% of previous year
    
    // Validator/LP split: 70/30
    pub const VALIDATOR_REWARD_PERCENT: u32 = 70;
    pub const LP_REWARD_PERCENT: u32 = 30;
    
    // Annual emissions (in CHML with 18 decimals)
    pub const YEARLY_EMISSIONS: [Balance; 20] = [
        7_400_000_000_000_000_000_000_000,  // Year 1:  7.4M (11.38%)
        6_660_000_000_000_000_000_000_000,  // Year 2:  6.66M (10.24%)
        6_000_000_000_000_000_000_000_000,  // Year 3:  6.0M (9.22%)
        5_390_000_000_000_000_000_000_000,  // Year 4:  5.39M (8.29%)
        4_840_000_000_000_000_000_000_000,  // Year 5:  4.84M (7.46%)
        4_360_000_000_000_000_000_000_000,  // Year 6:  4.36M (6.71%)
        3_930_000_000_000_000_000_000_000,  // Year 7:  3.93M (6.04%)
        3_530_000_000_000_000_000_000_000,  // Year 8:  3.53M (5.43%)
        3_180_000_000_000_000_000_000_000,  // Year 9:  3.18M (4.89%)
        2_860_000_000_000_000_000_000_000,  // Year 10: 2.86M (4.40%)
        2_570_000_000_000_000_000_000_000,  // Year 11: 2.57M (3.96%)
        2_310_000_000_000_000_000_000_000,  // Year 12: 2.31M (3.56%)
        2_090_000_000_000_000_000_000_000,  // Year 13: 2.09M (3.21%)
        1_870_000_000_000_000_000_000_000,  // Year 14: 1.87M (2.89%)
        1_690_000_000_000_000_000_000_000,  // Year 15: 1.69M (2.60%)
        1_520_000_000_000_000_000_000_000,  // Year 16: 1.52M (2.34%)
        1_370_000_000_000_000_000_000_000,  // Year 17: 1.37M (2.10%)
        1_230_000_000_000_000_000_000_000,  // Year 18: 1.23M (1.89%)
        1_100_000_000_000_000_000_000_000,  // Year 19: 1.1M (1.70%)
        1_000_000_000_000_000_000_000_000,  // Year 20: 1.0M (1.53%)
    ];
}
```

---

## FEE CONSTANTS

```rust
// Fee structure
pub mod fees {
    use crate::types::Balance;
    use sp_runtime::Perbill;

    // Base gas fee
    pub const BASE_GAS_FEE: Balance = 100_000_000_000_000_000; // 0.1 CHML
    
    // Shielding fees
    pub const SHIELDING_FEE_PERCENT: Perbill = Perbill::from_parts(200_000);   // 0.02%
    pub const SHIELDING_MIN_FEE: Balance = 100_000_000_000_000_000;             // 0.1 CHML
    
    // Unshielding fees
    pub const UNSHIELDING_FEE_PERCENT: Perbill = Perbill::from_parts(500_000); // 0.05%
    pub const UNSHIELDING_MIN_FEE: Balance = 100_000_000_000_000_000;           // 0.1 CHML
    
    // pDEX swap fee
    pub const PDEX_SWAP_FEE: Perbill = Perbill::from_parts(2_500_000);         // 0.25%
    pub const PDEX_LP_SHARE: Perbill = Perbill::from_parts(900_000_000);       // 90% to LPs
    pub const PDEX_TREASURY_SHARE: Perbill = Perbill::from_parts(100_000_000); // 10% to Treasury
    
    // Fee distribution
    pub const GAS_FEE_VALIDATOR_SHARE: Perbill = Perbill::from_parts(800_000_000); // 80%
    pub const GAS_FEE_TREASURY_SHARE: Perbill = Perbill::from_parts(200_000_000);  // 20%
}
```

---

## PALLET IDs

```rust
// Pallet identifiers
pub const CHAMELEON_STAKING_PALLET_ID: PalletId = PalletId(*b"chmlstak");
pub const CHAMELEON_TREASURY_PALLET_ID: PalletId = PalletId(*b"chmltres");
pub const CHAMELEON_EMISSION_PALLET_ID: PalletId = PalletId(*b"chmlemis");
pub const CHAMELEON_MEV_PALLET_ID: PalletId = PalletId(*b"chmlmevp");
pub const CHAMELEON_PDEX_PALLET_ID: PalletId = PalletId(*b"chmlpdex");
pub const CHAMELEON_BRIDGE_PALLET_ID: PalletId = PalletId(*b"chmlbrdg");
pub const CHAMELEON_PAY_PALLET_ID: PalletId = PalletId(*b"chmlpay*");
```

---

## GENESIS CONFIGURATION

```rust
// Genesis validator configuration
pub mod genesis {
    use crate::types::Balance;

    // Genesis validators: 30 nodes across regions
    pub const GENESIS_VALIDATOR_COUNT: u32 = 30;
    pub const GENESIS_VALIDATOR_STAKE: Balance = 150_000_000_000_000_000_000_000; // 150,000 CHML each
    
    // Total staking infrastructure: 5M CHML
    // - Genesis validators: 4.5M CHML (30 × 150,000)
    // - Emergency reserve: 0.5M CHML
    pub const GENESIS_VALIDATOR_TOTAL: Balance = 4_500_000_000_000_000_000_000_000; // 4.5M
    pub const EMERGENCY_RESERVE: Balance = 500_000_000_000_000_000_000_000;         // 0.5M
}
```

---

## NETWORK PARAMETERS

```rust
// Network configuration
pub mod network {
    pub const MAX_BLOCK_SIZE: u32 = 5 * 1024 * 1024; // 5 MB
    pub const TARGET_TPS: u32 = 100;
    pub const FINALITY_BLOCKS: u32 = 2;  // 2 blocks = 12 seconds finality
    
    // Performance targets
    pub const TARGET_BLOCK_TIME_MS: u64 = 6_000;    // 6 seconds
    pub const TARGET_TX_CONFIRMATION_MS: u64 = 5_000; // <5 seconds
    pub const TARGET_TX_FINALITY_MS: u64 = 15_000;   // <15 seconds
    pub const TARGET_ZK_PROOF_GENERATION_MS: u64 = 10_000; // <10 seconds
    pub const TARGET_ZK_PROOF_VERIFICATION_MS: u64 = 1_000; // <1 second
}
```

---

**Document Status:** Ready for Agent 1 implementation
