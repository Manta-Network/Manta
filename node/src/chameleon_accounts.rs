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

//! Chameleon Network Genesis Accounts
//!
//! This module provides deterministic account generation for all genesis accounts
//! in the Chameleon Network, including validators and system accounts.

use manta_primitives::types::AccountId;
use session_key_primitives::util::unchecked_account_id;
use sp_core::sr25519;
use sp_runtime::traits::AccountIdConversion;
use frame_support::PalletId;

/// Validator account seeds for deterministic generation
const VALIDATOR_SEEDS: [&str; 5] = [
    "ChameleonValidator0//Alice//Stash",
    "ChameleonValidator1//Bob//Stash", 
    "ChameleonValidator2//Charlie//Stash",
    "ChameleonValidator3//Dave//Stash",
    "ChameleonValidator4//Eve//Stash",
];

/// System account pallet IDs for deterministic generation
const LIQUIDITY_POOL_PALLET_ID: PalletId = PalletId(*b"chml/liq");
const TREASURY_PALLET_ID: PalletId = PalletId(*b"chml/tre");
const ECOSYSTEM_PALLET_ID: PalletId = PalletId(*b"chml/eco");
const PRESALE_PALLET_ID: PalletId = PalletId(*b"chml/pre");
const AIRDROP_PALLET_ID: PalletId = PalletId(*b"chml/air");
const EMISSION_PALLET_ID: PalletId = PalletId(*b"chml/emi");

/// Get validator account ID by index (0-4)
pub fn get_validator_account_id(index: usize) -> AccountId {
    assert!(index < 5, "Validator index must be 0-4");
    unchecked_account_id::<sr25519::Public>(VALIDATOR_SEEDS[index])
}

/// Get all validator account IDs
pub fn get_all_validator_account_ids() -> Vec<AccountId> {
    (0..5).map(get_validator_account_id).collect()
}

/// Get liquidity pool account ID (holds 2.5M CHML for initial DEX liquidity)
pub fn get_liquidity_pool_account_id() -> AccountId {
    LIQUIDITY_POOL_PALLET_ID.into_account_truncating()
}

/// Get treasury account ID (holds 2.5M CHML for DAO-controlled strategic reserve)
pub fn get_treasury_account_id() -> AccountId {
    TREASURY_PALLET_ID.into_account_truncating()
}

/// Get ecosystem development account ID (holds 5M CHML with 36-month vesting)
pub fn get_ecosystem_account_id() -> AccountId {
    ECOSYSTEM_PALLET_ID.into_account_truncating()
}

/// Get presale vesting account ID (holds 15M CHML with 6-month linear vesting)
pub fn get_presale_account_id() -> AccountId {
    PRESALE_PALLET_ID.into_account_truncating()
}

/// Get airdrop distribution account ID (holds 5M CHML for staged distribution)
pub fn get_airdrop_account_id() -> AccountId {
    AIRDROP_PALLET_ID.into_account_truncating()
}

/// Get emission pallet account ID (holds 65M CHML for 20-year validator/LP rewards)
pub fn get_emission_account_id() -> AccountId {
    EMISSION_PALLET_ID.into_account_truncating()
}

/// Get all system account IDs
pub fn get_all_system_account_ids() -> Vec<AccountId> {
    vec![
        get_liquidity_pool_account_id(),
        get_treasury_account_id(),
        get_ecosystem_account_id(),
        get_presale_account_id(),
        get_airdrop_account_id(),
        get_emission_account_id(),
    ]
}

/// Get all genesis account IDs (validators + system accounts)
pub fn get_all_genesis_account_ids() -> Vec<AccountId> {
    let mut accounts = get_all_validator_account_ids();
    accounts.extend(get_all_system_account_ids());
    accounts
}

/// Validator information structure for JSON export
#[derive(Clone, Debug)]
pub struct ValidatorInfo {
    pub index: usize,
    pub name: String,
    pub account_id: AccountId,
    pub seed: String,
    pub region: String,
}

/// Get validator information for all 5 validators
pub fn get_validator_info() -> Vec<ValidatorInfo> {
    vec![
        ValidatorInfo {
            index: 0,
            name: "Validator-US-East-1".to_string(),
            account_id: get_validator_account_id(0),
            seed: VALIDATOR_SEEDS[0].to_string(),
            region: "US-East".to_string(),
        },
        ValidatorInfo {
            index: 1,
            name: "Validator-US-West-1".to_string(),
            account_id: get_validator_account_id(1),
            seed: VALIDATOR_SEEDS[1].to_string(),
            region: "US-West".to_string(),
        },
        ValidatorInfo {
            index: 2,
            name: "Validator-EU-Central-1".to_string(),
            account_id: get_validator_account_id(2),
            seed: VALIDATOR_SEEDS[2].to_string(),
            region: "EU-Central".to_string(),
        },
        ValidatorInfo {
            index: 3,
            name: "Validator-Asia-East-1".to_string(),
            account_id: get_validator_account_id(3),
            seed: VALIDATOR_SEEDS[3].to_string(),
            region: "Asia-East".to_string(),
        },
        ValidatorInfo {
            index: 4,
            name: "Validator-Asia-Southeast-1".to_string(),
            account_id: get_validator_account_id(4),
            seed: VALIDATOR_SEEDS[4].to_string(),
            region: "Asia-Southeast".to_string(),
        },
    ]
}

/// System account information structure for JSON export
#[derive(Clone, Debug)]
pub struct SystemAccountInfo {
    pub name: String,
    pub account_id: AccountId,
    pub purpose: String,
    pub allocation_chml: u64,
}

/// Get system account information
pub fn get_system_account_info() -> Vec<SystemAccountInfo> {
    vec![
        SystemAccountInfo {
            name: "Liquidity Pool".to_string(),
            account_id: get_liquidity_pool_account_id(),
            purpose: "Initial DEX liquidity across CHML/ETH, CHML/USDC, CHML/WBTC pools".to_string(),
            allocation_chml: 2_500_000,
        },
        SystemAccountInfo {
            name: "Treasury Reserve".to_string(),
            account_id: get_treasury_account_id(),
            purpose: "DAO-controlled strategic reserve for opportunities and emergencies".to_string(),
            allocation_chml: 2_500_000,
        },
        SystemAccountInfo {
            name: "Ecosystem Development".to_string(),
            account_id: get_ecosystem_account_id(),
            purpose: "Team compensation and development funding (36-month vesting, 6-month cliff)".to_string(),
            allocation_chml: 5_000_000,
        },
        SystemAccountInfo {
            name: "Presale Vesting".to_string(),
            account_id: get_presale_account_id(),
            purpose: "Public presale allocation (50% TGE unlock, 50% linear 6-month vesting)".to_string(),
            allocation_chml: 15_000_000,
        },
        SystemAccountInfo {
            name: "Airdrop Distribution".to_string(),
            account_id: get_airdrop_account_id(),
            purpose: "Community airdrop in 3 stages: TGE (20%), Testnet (30%), Mainnet (50%)".to_string(),
            allocation_chml: 5_000_000,
        },
        SystemAccountInfo {
            name: "Emission Rewards".to_string(),
            account_id: get_emission_account_id(),
            purpose: "20-year declining emission schedule for validator (70%) and LP (30%) rewards".to_string(),
            allocation_chml: 65_000_000,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use manta_primitives::chameleon_constants::*;

    #[test]
    fn test_validator_account_generation() {
        let validators = get_all_validator_account_ids();
        assert_eq!(validators.len(), 5, "Must have exactly 5 validators");
        
        // Ensure all validator accounts are unique
        for i in 0..validators.len() {
            for j in (i + 1)..validators.len() {
                assert_ne!(validators[i], validators[j], "Validator accounts must be unique");
            }
        }
    }

    #[test]
    fn test_system_account_generation() {
        let system_accounts = get_all_system_account_ids();
        assert_eq!(system_accounts.len(), 6, "Must have exactly 6 system accounts");
        
        // Ensure all system accounts are unique
        for i in 0..system_accounts.len() {
            for j in (i + 1)..system_accounts.len() {
                assert_ne!(system_accounts[i], system_accounts[j], "System accounts must be unique");
            }
        }
    }

    #[test]
    fn test_all_accounts_unique() {
        let all_accounts = get_all_genesis_account_ids();
        assert_eq!(all_accounts.len(), 11, "Must have exactly 11 genesis accounts (5 validators + 6 system)");
        
        // Ensure all accounts are unique
        for i in 0..all_accounts.len() {
            for j in (i + 1)..all_accounts.len() {
                assert_ne!(all_accounts[i], all_accounts[j], "All genesis accounts must be unique");
            }
        }
    }

    #[test]
    fn test_validator_info_consistency() {
        let validator_info = get_validator_info();
        assert_eq!(validator_info.len(), 5, "Must have info for 5 validators");
        
        for (i, info) in validator_info.iter().enumerate() {
            assert_eq!(info.index, i, "Validator index must match array position");
            assert_eq!(info.account_id, get_validator_account_id(i), "Account ID must match generated ID");
            assert_eq!(info.seed, VALIDATOR_SEEDS[i], "Seed must match predefined seed");
        }
    }

    #[test]
    fn test_system_account_info_consistency() {
        let system_info = get_system_account_info();
        assert_eq!(system_info.len(), 6, "Must have info for 6 system accounts");
        
        let expected_allocations = [
            2_500_000, // Liquidity Pool
            2_500_000, // Treasury
            5_000_000, // Ecosystem
            15_000_000, // Presale
            5_000_000, // Airdrop
            65_000_000, // Emission
        ];
        
        let total_allocation: u64 = system_info.iter().map(|info| info.allocation_chml).sum();
        let expected_total: u64 = expected_allocations.iter().sum();
        assert_eq!(total_allocation, expected_total, "System account allocations must sum correctly");
        assert_eq!(total_allocation + 5_000_000, 100_000_000, "Total allocation including validators must be 100M CHML");
    }

    #[test]
    fn test_pallet_id_uniqueness() {
        let pallet_ids = [
            LIQUIDITY_POOL_PALLET_ID.0,
            TREASURY_PALLET_ID.0,
            ECOSYSTEM_PALLET_ID.0,
            PRESALE_PALLET_ID.0,
            AIRDROP_PALLET_ID.0,
            EMISSION_PALLET_ID.0,
        ];
        
        // Ensure all pallet IDs are unique
        for i in 0..pallet_ids.len() {
            for j in (i + 1)..pallet_ids.len() {
                assert_ne!(pallet_ids[i], pallet_ids[j], "Pallet IDs must be unique");
            }
        }
        
        // Ensure all pallet IDs are exactly 8 bytes
        for id in &pallet_ids {
            assert_eq!(id.len(), 8, "Pallet IDs must be exactly 8 bytes");
        }
    }

    #[test]
    fn test_validator_index_bounds() {
        // Valid indices should work
        for i in 0..5 {
            let _ = get_validator_account_id(i);
        }
    }

    #[test]
    #[should_panic(expected = "Validator index must be 0-4")]
    fn test_validator_index_out_of_bounds() {
        let _ = get_validator_account_id(5);
    }
}
