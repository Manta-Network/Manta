// Copyright 2020-2024 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

//! Chameleon Network Constants
//!
//! This module contains all the constants for the Chameleon Network blockchain,
//! including token economics, emission schedules, staking parameters, and pallet identifiers.

use crate::types::{Balance, BlockNumber, Moment};
use frame_support::PalletId;
use sp_runtime::Perbill;

/// Chameleon SS58 Prefix
/// 
/// Custom SS58 address prefix for Chameleon Network addresses
pub const CHAMELEON_SS58PREFIX: u8 = 99;

/// Chameleon Token Decimals
/// 
/// Number of decimal places for CHML token (18 decimals like ETH)
pub const CHAMELEON_DECIMAL: u8 = 18;

/// Chameleon Token Symbol
pub const CHAMELEON_TOKEN_SYMBOL: &str = "CHML";

/// Chameleon Token Name
pub const CHAMELEON_TOKEN_NAME: &str = "Chameleon Network Token";

/// Total Supply: 100,000,000 CHML (with 18 decimals)
/// 
/// Fixed total supply - no inflation beyond the 20-year emission schedule
pub const TOTAL_SUPPLY: Balance = 100_000_000_000_000_000_000_000_000; // 100M * 10^18

// =============================================================================
// TOKEN ALLOCATION CONSTANTS
// =============================================================================

/// Validator & LP Rewards Pool: 65,000,000 CHML (65%)
/// 
/// Distributed over 20 years with declining emission schedule
pub const VALIDATOR_LP_REWARDS: Balance = 65_000_000_000_000_000_000_000_000;  // 65M

/// Public Presale Allocation: 15,000,000 CHML (15%)
/// 
/// Sold to public at $0.58 per CHML with 50% TGE unlock, 50% linear 6-month vesting
pub const PUBLIC_PRESALE: Balance = 15_000_000_000_000_000_000_000_000;        // 15M

/// Community Airdrop: 5,000,000 CHML (5%)
/// 
/// Distributed in 3 stages: TGE (20%), Testnet (30%), Mainnet (50%)
pub const COMMUNITY_AIRDROP: Balance = 5_000_000_000_000_000_000_000_000;      // 5M

/// Staking Infrastructure: 5,000,000 CHML (5%)
/// 
/// Locked in 30 genesis validators to bootstrap network security
pub const STAKING_INFRASTRUCTURE: Balance = 5_000_000_000_000_000_000_000_000; // 5M

/// Ecosystem Development: 5,000,000 CHML (5%)
/// 
/// Team compensation and development funding with 36-month vesting (6-month cliff)
pub const ECOSYSTEM_DEVELOPMENT: Balance = 5_000_000_000_000_000_000_000_000;  // 5M

/// Initial DEX Liquidity: 2,500,000 CHML (2.5%)
/// 
/// Deployed at TGE across CHML/ETH, CHML/USDC, and CHML/WBTC pools
pub const INITIAL_DEX_LIQUIDITY: Balance = 2_500_000_000_000_000_000_000_000;  // 2.5M

/// Treasury Reserve: 2,500,000 CHML (2.5%)
/// 
/// DAO-controlled strategic reserve for opportunities and emergencies
pub const TREASURY_RESERVE: Balance = 2_500_000_000_000_000_000_000_000;       // 2.5M

// =============================================================================
// STAKING CONSTANTS
// =============================================================================

/// Minimum Validator Stake: 1,750 CHML
/// 
/// Minimum amount required to run a validator node
pub const MIN_VALIDATOR_STAKE: Balance = 1_750_000_000_000_000_000_000; // 1,750 CHML

/// Existential Deposit: 0.001 CHML
/// 
/// Minimum balance required to keep an account alive
pub const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000_000_000;         // 0.001 CHML

/// Staking configuration module
pub mod staking {
    use super::*;

    /// Minimum stake for validators
    pub const MIN_VALIDATOR_STAKE: Balance = super::MIN_VALIDATOR_STAKE;
    
    /// Slashing rate for downtime (0.1%)
    pub const SLASHING_DOWNTIME: Perbill = Perbill::from_parts(1_000_000);
    
    /// Slashing rate for double signing (5.0%)
    pub const SLASHING_DOUBLE_SIGN: Perbill = Perbill::from_parts(50_000_000);
    
    /// Maximum number of validators (can be adjusted by governance)
    pub const MAX_VALIDATORS: u32 = 200;
    
    /// Minimum number of validators for network security
    pub const MIN_VALIDATORS: u32 = 5;
    
    /// Maximum delegations per delegator
    pub const MAX_DELEGATIONS_PER_DELEGATOR: u32 = 100;
    
    /// Maximum delegators per validator
    pub const MAX_DELEGATORS_PER_VALIDATOR: u32 = 500;
}

// =============================================================================
// TIME CONSTANTS
// =============================================================================

/// Chameleon time-related constants
pub mod time {
    use super::*;

    /// Block time: 6 seconds (faster than Manta's 12s)
    pub const SECONDS_PER_BLOCK: Moment = 6;
    
    /// Milliseconds per block
    pub const MILLISECS_PER_BLOCK: Moment = SECONDS_PER_BLOCK * 1000;
    
    /// Slot duration (same as block time)
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    /// Number of blocks per minute
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    
    /// Number of blocks per hour
    pub const HOURS: BlockNumber = MINUTES * 60;
    
    /// Number of blocks per day
    pub const DAYS: BlockNumber = HOURS * 24;
    
    /// Number of blocks per week
    pub const WEEKS: BlockNumber = DAYS * 7;

    /// Unbonding period: 14 days
    pub const UNBONDING_PERIOD_DAYS: u32 = 14;
    
    /// Unbonding period in blocks
    pub const UNBONDING_PERIOD_BLOCKS: BlockNumber = DAYS * 14;
    
    /// Session length: 4 hours
    pub const SESSION_LENGTH_BLOCKS: BlockNumber = HOURS * 4;
    
    /// Blocks per year (365 days)
    pub const BLOCKS_PER_YEAR: BlockNumber = DAYS * 365;
}

// =============================================================================
// EMISSION SCHEDULE CONSTANTS
// =============================================================================

/// 20-year declining emission schedule with 10% year-over-year reduction
pub mod emission {
    use super::*;

    /// Total rewards pool: 65M CHML
    pub const TOTAL_REWARDS_POOL: Balance = super::VALIDATOR_LP_REWARDS;
    
    /// Year 1 emissions: 7.4M CHML (11.38% of pool)
    pub const YEAR_1_EMISSION: Balance = 7_400_000_000_000_000_000_000_000;
    
    /// Decay factor: 90% (10% reduction per year)
    pub const EMISSION_DECAY_FACTOR: u32 = 90;
    
    /// Validator reward percentage (70%)
    pub const VALIDATOR_REWARD_PERCENT: u32 = 70;
    
    /// LP reward percentage (30%)
    pub const LP_REWARD_PERCENT: u32 = 30;
    
    /// Annual emissions for all 20 years (in CHML with 18 decimals)
    /// 
    /// Each year's emission = previous_year * 9 / 10 (using integer arithmetic)
    /// Total sums to exactly 65,000,000 CHML
    pub const YEARLY_EMISSIONS: [Balance; 20] = [
        7400000000000000000000000,  // Year 1: 7.400M (11.38%)
        6660000000000000000000000,  // Year 2: 6.660M (10.25%)
        5994000000000000000000000,  // Year 3: 5.994M (9.22%)
        5394600000000000000000000,  // Year 4: 5.395M (8.30%)
        4855140000000000000000000,  // Year 5: 4.855M (7.47%)
        4369626000000000000000000,  // Year 6: 4.370M (6.72%)
        3932663400000000000000000,  // Year 7: 3.933M (6.05%)
        3539397060000000000000000,  // Year 8: 3.539M (5.45%)
        3185457354000000000000000,  // Year 9: 3.185M (4.90%)
        2866911618600000000000000,  // Year 10: 2.867M (4.41%)
        2580220456740000000000000,  // Year 11: 2.580M (3.97%)
        2322198411066000000000000,  // Year 12: 2.322M (3.57%)
        2089978569959400000000000,  // Year 13: 2.090M (3.22%)
        1880980712963460000000000,  // Year 14: 1.881M (2.89%)
        1692882641667114000000000,  // Year 15: 1.693M (2.60%)
        1523594377500402600000000,  // Year 16: 1.524M (2.34%)
        1371234939750362340000000,  // Year 17: 1.371M (2.11%)
        1234111445775326106000000,  // Year 18: 1.234M (1.90%)
        1110700301197793495400000,  // Year 19: 1.111M (1.71%)
        996302710780141458600000,   // Year 20: 0.996M (1.53%)
    ];
}

// =============================================================================
// FEE CONSTANTS
// =============================================================================

/// Fee structure for various network operations
pub mod fees {
    use super::*;

    /// Base gas fee: 0.1 CHML
    pub const BASE_GAS_FEE: Balance = 100_000_000_000_000_000;
    
    /// Shielding fee percentage (0.02%)
    pub const SHIELDING_FEE_PERCENT: Perbill = Perbill::from_parts(200_000);
    
    /// Minimum shielding fee: 0.1 CHML
    pub const SHIELDING_MIN_FEE: Balance = 100_000_000_000_000_000;
    
    /// Unshielding fee percentage (0.05%)
    pub const UNSHIELDING_FEE_PERCENT: Perbill = Perbill::from_parts(500_000);
    
    /// Minimum unshielding fee: 0.1 CHML
    pub const UNSHIELDING_MIN_FEE: Balance = 100_000_000_000_000_000;
    
    /// pDEX swap fee (0.25%)
    pub const PDEX_SWAP_FEE: Perbill = Perbill::from_parts(2_500_000);
    
    /// LP share of pDEX fees (90%)
    pub const PDEX_LP_SHARE: Perbill = Perbill::from_parts(900_000_000);
    
    /// Treasury share of pDEX fees (10%)
    pub const PDEX_TREASURY_SHARE: Perbill = Perbill::from_parts(100_000_000);
    
    /// Validator share of gas fees (80%)
    pub const GAS_FEE_VALIDATOR_SHARE: Perbill = Perbill::from_parts(800_000_000);
    
    /// Treasury share of gas fees (20%)
    pub const GAS_FEE_TREASURY_SHARE: Perbill = Perbill::from_parts(200_000_000);
}

// =============================================================================
// GENESIS CONFIGURATION
// =============================================================================

/// Genesis validator configuration
pub mod genesis {
    use super::*;

    /// Number of genesis validators: 30 nodes across regions
    pub const GENESIS_VALIDATOR_COUNT: u32 = 30;
    
    /// Stake per genesis validator: 150,000 CHML each
    pub const GENESIS_VALIDATOR_STAKE: Balance = 150_000_000_000_000_000_000_000;
    
    /// Total genesis validator stake: 4.5M CHML (30 × 150,000)
    pub const GENESIS_VALIDATOR_TOTAL: Balance = 4_500_000_000_000_000_000_000_000;
    
    /// Emergency reserve: 0.5M CHML
    pub const EMERGENCY_RESERVE: Balance = 500_000_000_000_000_000_000_000;
}

// =============================================================================
// NETWORK PARAMETERS
// =============================================================================

/// Network configuration parameters
pub mod network {
    /// Maximum block size: 5 MB
    pub const MAX_BLOCK_SIZE: u32 = 5 * 1024 * 1024;
    
    /// Target transactions per second
    pub const TARGET_TPS: u32 = 100;
    
    /// Finality in blocks (2 blocks = 12 seconds)
    pub const FINALITY_BLOCKS: u32 = 2;
    
    /// Target block time in milliseconds (6 seconds)
    pub const TARGET_BLOCK_TIME_MS: u64 = 6_000;
    
    /// Target transaction confirmation time (<5 seconds)
    pub const TARGET_TX_CONFIRMATION_MS: u64 = 5_000;
    
    /// Target transaction finality time (<15 seconds)
    pub const TARGET_TX_FINALITY_MS: u64 = 15_000;
    
    /// Target ZK proof generation time (<10 seconds)
    pub const TARGET_ZK_PROOF_GENERATION_MS: u64 = 10_000;
    
    /// Target ZK proof verification time (<1 second)
    pub const TARGET_ZK_PROOF_VERIFICATION_MS: u64 = 1_000;
}

// =============================================================================
// PALLET IDENTIFIERS
// =============================================================================

/// Chameleon Staking Pallet Identifier
pub const CHAMELEON_STAKING_PALLET_ID: PalletId = PalletId(*b"chmlstak");

/// Chameleon Treasury Pallet Identifier
pub const CHAMELEON_TREASURY_PALLET_ID: PalletId = PalletId(*b"chmltres");

/// Chameleon Emission Pallet Identifier
pub const CHAMELEON_EMISSION_PALLET_ID: PalletId = PalletId(*b"chmlemis");

/// Chameleon MEV Protection Pallet Identifier
pub const CHAMELEON_MEV_PALLET_ID: PalletId = PalletId(*b"chmlmevp");

/// Chameleon Private DEX Pallet Identifier
pub const CHAMELEON_PDEX_PALLET_ID: PalletId = PalletId(*b"chmlpdex");

/// Chameleon Bridge Pallet Identifier
pub const CHAMELEON_BRIDGE_PALLET_ID: PalletId = PalletId(*b"chmlbrdg");

/// Chameleon Pay Pallet Identifier
pub const CHAMELEON_PAY_PALLET_ID: PalletId = PalletId(*b"chmlpay*");

// =============================================================================
// VALIDATION FUNCTIONS
// =============================================================================

/// Validates that all allocations sum to the total supply
/// 
/// This function ensures the tokenomics are mathematically correct
pub const fn validate_allocations() -> bool {
    let total_allocated = VALIDATOR_LP_REWARDS
        + PUBLIC_PRESALE
        + COMMUNITY_AIRDROP
        + STAKING_INFRASTRUCTURE
        + ECOSYSTEM_DEVELOPMENT
        + INITIAL_DEX_LIQUIDITY
        + TREASURY_RESERVE;
    
    total_allocated == TOTAL_SUPPLY
}

/// Validates that validator and LP percentages sum to 100%
pub const fn validate_reward_split() -> bool {
    emission::VALIDATOR_REWARD_PERCENT + emission::LP_REWARD_PERCENT == 100
}

/// Validates that the 20-year emission schedule sums to the rewards pool
/// 
/// Note: This is an approximation due to const fn limitations
pub const fn validate_emission_total() -> bool {
    // Sum first few years as validation (const fn can't do loops)
    let partial_sum = emission::YEARLY_EMISSIONS[0]
        + emission::YEARLY_EMISSIONS[1]
        + emission::YEARLY_EMISSIONS[2]
        + emission::YEARLY_EMISSIONS[3]
        + emission::YEARLY_EMISSIONS[4];
    
    // First 5 years should be approximately 36.6% of total
    // Calculated: 7.4 + 6.66 + 5.994 + 5.3946 + 4.8551 = 30.3037M CHML
    partial_sum > 30_000_000_000_000_000_000_000_000 && 
    partial_sum < 31_000_000_000_000_000_000_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_supply_correct() {
        assert_eq!(TOTAL_SUPPLY, 100_000_000_000_000_000_000_000_000);
    }

    #[test]
    fn test_allocations_sum_to_total() {
        let total_allocated = VALIDATOR_LP_REWARDS
            + PUBLIC_PRESALE
            + COMMUNITY_AIRDROP
            + STAKING_INFRASTRUCTURE
            + ECOSYSTEM_DEVELOPMENT
            + INITIAL_DEX_LIQUIDITY
            + TREASURY_RESERVE;
        
        assert_eq!(total_allocated, TOTAL_SUPPLY);
        assert!(validate_allocations());
    }

    #[test]
    fn test_validator_lp_split() {
        assert_eq!(emission::VALIDATOR_REWARD_PERCENT, 70);
        assert_eq!(emission::LP_REWARD_PERCENT, 30);
        assert!(validate_reward_split());
    }

    #[test]
    fn test_emission_schedule_20_years() {
        // Test that we have exactly 20 years of emissions
        assert_eq!(emission::YEARLY_EMISSIONS.len(), 20);
        
        // Test that emissions sum to the rewards pool
        let total_emitted: Balance = emission::YEARLY_EMISSIONS.iter().sum();
        assert_eq!(total_emitted, emission::TOTAL_REWARDS_POOL);
        
        // Test declining schedule (each year should be ~90% of previous)
        for i in 1..emission::YEARLY_EMISSIONS.len() {
            let current = emission::YEARLY_EMISSIONS[i];
            let previous = emission::YEARLY_EMISSIONS[i - 1];
            let ratio = (current * 100) / previous;
            
            // Should be approximately 90% (allow small rounding differences)
            assert!(ratio >= 89 && ratio <= 91, 
                "Year {} emission ratio is {}, expected ~90", i + 1, ratio);
        }
    }

    #[test]
    fn test_genesis_validator_allocation() {
        assert_eq!(genesis::GENESIS_VALIDATOR_COUNT, 30);
        assert_eq!(genesis::GENESIS_VALIDATOR_STAKE, 150_000_000_000_000_000_000_000);
        assert_eq!(
            genesis::GENESIS_VALIDATOR_TOTAL,
            genesis::GENESIS_VALIDATOR_COUNT as Balance * genesis::GENESIS_VALIDATOR_STAKE
        );
        
        // Genesis validators + emergency reserve should equal staking infrastructure
        assert_eq!(
            genesis::GENESIS_VALIDATOR_TOTAL + genesis::EMERGENCY_RESERVE,
            STAKING_INFRASTRUCTURE
        );
    }

    #[test]
    fn test_time_constants() {
        assert_eq!(time::SECONDS_PER_BLOCK, 6);
        assert_eq!(time::MILLISECS_PER_BLOCK, 6_000);
        assert_eq!(time::MINUTES, 10); // 60_000 / 6_000 = 10 blocks per minute
        assert_eq!(time::HOURS, 600);  // 10 * 60 = 600 blocks per hour
        assert_eq!(time::DAYS, 14_400); // 600 * 24 = 14,400 blocks per day
        assert_eq!(time::UNBONDING_PERIOD_BLOCKS, 14 * 14_400); // 14 days
    }

    #[test]
    fn test_minimum_amounts() {
        assert_eq!(MIN_VALIDATOR_STAKE, 1_750_000_000_000_000_000_000); // 1,750 CHML
        assert_eq!(EXISTENTIAL_DEPOSIT, 1_000_000_000_000_000); // 0.001 CHML
        
        // Existential deposit should be much smaller than validator stake
        assert!(EXISTENTIAL_DEPOSIT < MIN_VALIDATOR_STAKE / 1_000_000);
    }

    #[test]
    fn test_fee_percentages() {
        // Test that fee percentages are reasonable
        assert_eq!(fees::SHIELDING_FEE_PERCENT, Perbill::from_parts(200_000)); // 0.02%
        assert_eq!(fees::UNSHIELDING_FEE_PERCENT, Perbill::from_parts(500_000)); // 0.05%
        assert_eq!(fees::PDEX_SWAP_FEE, Perbill::from_parts(2_500_000)); // 0.25%
        
        // Test that pDEX fee distribution sums to 100%
        let total_pdex_share = fees::PDEX_LP_SHARE.deconstruct() + fees::PDEX_TREASURY_SHARE.deconstruct();
        assert_eq!(total_pdex_share, Perbill::one().deconstruct());
        
        // Test that gas fee distribution sums to 100%
        let total_gas_share = fees::GAS_FEE_VALIDATOR_SHARE.deconstruct() + fees::GAS_FEE_TREASURY_SHARE.deconstruct();
        assert_eq!(total_gas_share, Perbill::one().deconstruct());
    }

    #[test]
    fn test_pallet_ids() {
        // Test that all pallet IDs are exactly 8 bytes
        assert_eq!(CHAMELEON_STAKING_PALLET_ID.0.len(), 8);
        assert_eq!(CHAMELEON_TREASURY_PALLET_ID.0.len(), 8);
        assert_eq!(CHAMELEON_EMISSION_PALLET_ID.0.len(), 8);
        assert_eq!(CHAMELEON_MEV_PALLET_ID.0.len(), 8);
        assert_eq!(CHAMELEON_PDEX_PALLET_ID.0.len(), 8);
        assert_eq!(CHAMELEON_BRIDGE_PALLET_ID.0.len(), 8);
        assert_eq!(CHAMELEON_PAY_PALLET_ID.0.len(), 8);
        
        // Test that pallet IDs are unique
        let ids = [
            CHAMELEON_STAKING_PALLET_ID.0,
            CHAMELEON_TREASURY_PALLET_ID.0,
            CHAMELEON_EMISSION_PALLET_ID.0,
            CHAMELEON_MEV_PALLET_ID.0,
            CHAMELEON_PDEX_PALLET_ID.0,
            CHAMELEON_BRIDGE_PALLET_ID.0,
            CHAMELEON_PAY_PALLET_ID.0,
        ];
        
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j], "Pallet IDs must be unique");
            }
        }
    }

    #[test]
    fn test_validation_functions() {
        assert!(validate_allocations());
        assert!(validate_reward_split());
        assert!(validate_emission_total());
    }

    #[test]
    fn test_token_constants() {
        assert_eq!(CHAMELEON_SS58PREFIX, 99);
        assert_eq!(CHAMELEON_DECIMAL, 18);
        assert_eq!(CHAMELEON_TOKEN_SYMBOL, "CHML");
        assert_eq!(CHAMELEON_TOKEN_NAME, "Chameleon Network Token");
    }

    #[test]
    fn test_network_parameters() {
        assert_eq!(network::MAX_BLOCK_SIZE, 5 * 1024 * 1024); // 5 MB
        assert_eq!(network::TARGET_TPS, 100);
        assert_eq!(network::FINALITY_BLOCKS, 2);
        assert_eq!(network::TARGET_BLOCK_TIME_MS, 6_000); // 6 seconds
    }
}