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

//! Chameleon Network Chain Specifications
//!
//! This module contains the chain specification and genesis configuration
//! for the Chameleon Network blockchain, including token allocations,
//! validator setup, and system account initialization.

use crate::chain_specs::*;
use crate::chameleon_accounts::*;
use core::marker::PhantomData;
use manta_primitives::{
    chameleon_constants::*,
    types::{AccountId, Balance},
};
use manta_runtime::{
    opaque::SessionKeys, CouncilConfig, DemocracyConfig, ParachainStakingConfig,
    PolkadotXcmConfig, Runtime, RuntimeGenesisConfig, TechnicalCommitteeConfig, WASM_BINARY,
};
use sc_service::config::MultiaddrWithPeerId;
use sc_service::{ChainType, Properties};
use session_key_primitives::util::unchecked_account_id;
use sp_core::sr25519;

/// Chameleon Protocol Identifier
pub const CHAMELEON_PROTOCOL_ID: &str = "chameleon";

/// Chameleon Parachain ID (using a unique ID different from Manta)
pub const CHAMELEON_PARACHAIN_ID: u32 = 2105;

/// Polkadot Relaychain Local Network Identifier for Chameleon
pub const CHAMELEON_POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";

/// Polkadot Relaychain Development Network Identifier for Chameleon
pub const CHAMELEON_POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";

/// The default XCM version to set in genesis config.
pub const CHAMELEON_SAFE_XCM_VERSION: u32 = 3;

/// Chameleon Chain Specification
pub type ChameleonChainSpec =
    sc_service::GenericChainSpec<manta_runtime::RuntimeGenesisConfig, Extensions>;

#[derive(Clone)]
struct ChameleonValidator {
    acc: AccountId,
    nodeid: Option<MultiaddrWithPeerId>,
    keys: SessionKeys,
    stake: Balance,
}

impl ChameleonValidator {
    fn new(
        acc: AccountId,
        nodeid: Option<MultiaddrWithPeerId>,
        keys: SessionKeys,
        stake: Balance,
    ) -> ChameleonValidator {
        Self {
            acc,
            nodeid,
            keys,
            stake,
        }
    }
}

/// Returns the [`Properties`] for the Chameleon parachain.
pub fn chameleon_properties() -> Properties {
    let mut p = Properties::new();
    p.insert("ss58format".into(), CHAMELEON_SS58PREFIX.into());
    p.insert("tokenDecimals".into(), CHAMELEON_DECIMAL.into());
    p.insert("tokenSymbol".into(), CHAMELEON_TOKEN_SYMBOL.into());
    p
}

/// Returns the Chameleon development chainspec.
pub fn chameleon_development_config() -> ChameleonChainSpec {
    let genesis_validators: Vec<ChameleonValidator> = vec![ChameleonValidator::new(
        unchecked_account_id::<sr25519::Public>("Alice"),
        None,
        SessionKeys::from_seed_unchecked("Alice"),
        1_000_000 * 10u128.pow(18), // 1M CHML
    )];
    let genesis_validators_clone = genesis_validators.clone();

    #[allow(deprecated)]
    ChameleonChainSpec::from_genesis(
        "Chameleon Network Dev",
        "chameleon-dev",
        ChainType::Development,
        move || chameleon_devnet_genesis(genesis_validators_clone.clone()),
        genesis_validators
            .into_iter()
            .filter_map(|validator| validator.nodeid)
            .collect(),
        None,
        Some(CHAMELEON_PROTOCOL_ID),
        None,
        Some(chameleon_properties()),
        Extensions {
            relay_chain: CHAMELEON_POLKADOT_RELAYCHAIN_DEV_NET.into(),
            para_id: CHAMELEON_PARACHAIN_ID,
        },
        WASM_BINARY.expect("WASM binary was not built, please build it!"),
    )
}

/// Returns the Chameleon local testnet chainspec.
pub fn chameleon_local_config() -> ChameleonChainSpec {
    let genesis_validators: Vec<ChameleonValidator> = vec![
        ChameleonValidator::new(
            get_validator_account_id(0),
            None,
            SessionKeys::from_seed_unchecked("Validator0"),
            1_000_000 * 10u128.pow(18), // 1M CHML
        ),
        ChameleonValidator::new(
            get_validator_account_id(1),
            None,
            SessionKeys::from_seed_unchecked("Validator1"),
            1_000_000 * 10u128.pow(18), // 1M CHML
        ),
        ChameleonValidator::new(
            get_validator_account_id(2),
            None,
            SessionKeys::from_seed_unchecked("Validator2"),
            1_000_000 * 10u128.pow(18), // 1M CHML
        ),
        ChameleonValidator::new(
            get_validator_account_id(3),
            None,
            SessionKeys::from_seed_unchecked("Validator3"),
            1_000_000 * 10u128.pow(18), // 1M CHML
        ),
        ChameleonValidator::new(
            get_validator_account_id(4),
            None,
            SessionKeys::from_seed_unchecked("Validator4"),
            1_000_000 * 10u128.pow(18), // 1M CHML
        ),
    ];
    let genesis_validators_clone = genesis_validators.clone();

    #[allow(deprecated)]
    ChameleonChainSpec::from_genesis(
        "Chameleon Network Local",
        "chameleon-local",
        ChainType::Local,
        move || chameleon_devnet_genesis(genesis_validators_clone.clone()),
        genesis_validators
            .into_iter()
            .filter_map(|validator| validator.nodeid)
            .collect(),
        None,
        Some(CHAMELEON_PROTOCOL_ID),
        None,
        Some(chameleon_properties()),
        Extensions {
            relay_chain: CHAMELEON_POLKADOT_RELAYCHAIN_LOCAL_NET.into(),
            para_id: CHAMELEON_PARACHAIN_ID,
        },
        WASM_BINARY.expect("WASM binary was not built, please build it!"),
    )
}

/// Returns the Chameleon devnet chainspec with full 5-validator setup.
pub fn chameleon_devnet_config() -> ChameleonChainSpec {
    let genesis_validators: Vec<ChameleonValidator> = (0..5)
        .map(|i| {
            ChameleonValidator::new(
                get_validator_account_id(i),
                None,
                SessionKeys::from_seed_unchecked(&format!("Validator{}", i)),
                1_000_000 * 10u128.pow(18), // 1M CHML each
            )
        })
        .collect();
    let genesis_validators_clone = genesis_validators.clone();

    #[allow(deprecated)]
    ChameleonChainSpec::from_genesis(
        "Chameleon Network Devnet",
        "chameleon-devnet",
        ChainType::Live,
        move || chameleon_devnet_genesis(genesis_validators_clone.clone()),
        genesis_validators
            .into_iter()
            .filter_map(|validator| validator.nodeid)
            .collect(),
        None,
        Some(CHAMELEON_PROTOCOL_ID),
        None,
        Some(chameleon_properties()),
        Extensions {
            relay_chain: "polkadot".into(),
            para_id: CHAMELEON_PARACHAIN_ID,
        },
        WASM_BINARY.expect("WASM binary was not built, please build it!"),
    )
}

/// Common helper to create the Chameleon genesis configuration
fn chameleon_devnet_genesis(genesis_validators: Vec<ChameleonValidator>) -> RuntimeGenesisConfig {
    // Use first validator as root key for development
    let root_key = genesis_validators.first().unwrap().acc.clone();

    // Create validator endowments with their stakes
    let validator_endowments: Vec<(AccountId, Balance)> = genesis_validators
        .iter()
        .map(|validator| (validator.acc.clone(), validator.stake))
        .collect();

    // Create system account endowments according to tokenomics
    let mut system_endowments = vec![
        // Liquidity Pool Account: 2.5M CHML
        (get_liquidity_pool_account_id(), INITIAL_DEX_LIQUIDITY),
        // Treasury Account: 2.5M CHML
        (get_treasury_account_id(), TREASURY_RESERVE),
        // Ecosystem Vesting Account: 5M CHML
        (get_ecosystem_account_id(), ECOSYSTEM_DEVELOPMENT),
        // Presale Vesting Account: 15M CHML
        (get_presale_account_id(), PUBLIC_PRESALE),
        // Airdrop Distribution Account: 5M CHML
        (get_airdrop_account_id(), COMMUNITY_AIRDROP),
        // Emission Pallet Account: 65M CHML
        (get_emission_account_id(), VALIDATOR_LP_REWARDS),
    ];

    // Combine all endowments
    let mut all_endowments = validator_endowments;
    all_endowments.append(&mut system_endowments);

    // Verify total allocation equals 100M CHML
    let total_allocated: Balance = all_endowments.iter().map(|(_, balance)| *balance).sum();
    assert_eq!(
        total_allocated, TOTAL_SUPPLY,
        "Total allocation ({}) must equal total supply ({})",
        total_allocated, TOTAL_SUPPLY
    );

    RuntimeGenesisConfig {
        system: manta_runtime::SystemConfig {
            _config: PhantomData::<Runtime>,
        },
        balances: manta_runtime::BalancesConfig {
            balances: all_endowments.clone(),
        },
        // Empty aura authorities, validators registered with parachain staking instead
        aura: Default::default(),
        sudo: manta_runtime::SudoConfig {
            key: Some(root_key),
        },
        parachain_staking: ParachainStakingConfig {
            candidates: genesis_validators
                .iter()
                .map(|validator| (validator.acc.clone(), validator.stake))
                .collect(),
            delegations: vec![],
            inflation_config: manta_runtime::staking::inflation_config::<manta_runtime::Runtime>(),
        },
        parachain_info: manta_runtime::ParachainInfoConfig {
            parachain_id: CHAMELEON_PARACHAIN_ID.into(),
            _config: PhantomData::<Runtime>,
        },
        collator_selection: manta_runtime::CollatorSelectionConfig {
            invulnerables: vec![],
            candidacy_bond: 0,
            ..Default::default()
        },
        session: manta_runtime::SessionConfig {
            keys: genesis_validators
                .iter()
                .map(|validator| {
                    (
                        validator.acc.clone(), // account id
                        validator.acc.clone(), // validator id
                        validator.keys.clone(), // validator session keys
                    )
                })
                .collect(),
        },
        parachain_system: Default::default(),
        polkadot_xcm: PolkadotXcmConfig {
            safe_xcm_version: Some(CHAMELEON_SAFE_XCM_VERSION),
            _config: PhantomData::<Runtime>,
        },
        asset_manager: Default::default(),
        democracy: DemocracyConfig::default(),
        council: CouncilConfig {
            members: all_endowments
                .iter()
                .map(|endowed| endowed.0.clone())
                .take(1)
                .collect(),
            phantom: Default::default(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: all_endowments
                .iter()
                .map(|endowed| endowed.0.clone())
                .take(1)
                .collect(),
            phantom: Default::default(),
        },
        council_membership: Default::default(),
        technical_membership: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chameleon_properties() {
        let props = chameleon_properties();
        assert_eq!(props.get("ss58format"), Some(&CHAMELEON_SS58PREFIX.into()));
        assert_eq!(props.get("tokenDecimals"), Some(&CHAMELEON_DECIMAL.into()));
        assert_eq!(props.get("tokenSymbol"), Some(&CHAMELEON_TOKEN_SYMBOL.into()));
    }

    #[test]
    fn test_genesis_allocation_totals() {
        let genesis_validators: Vec<ChameleonValidator> = (0..5)
            .map(|i| {
                ChameleonValidator::new(
                    get_validator_account_id(i),
                    None,
                    SessionKeys::from_seed_unchecked(&format!("Validator{}", i)),
                    1_000_000 * 10u128.pow(18),
                )
            })
            .collect();

        let genesis = chameleon_devnet_genesis(genesis_validators);
        let total_allocated: Balance = genesis.balances.balances.iter().map(|(_, balance)| *balance).sum();
        
        assert_eq!(total_allocated, TOTAL_SUPPLY, "Genesis allocation must equal total supply");
    }

    #[test]
    fn test_validator_stakes() {
        let genesis_validators: Vec<ChameleonValidator> = (0..5)
            .map(|i| {
                ChameleonValidator::new(
                    get_validator_account_id(i),
                    None,
                    SessionKeys::from_seed_unchecked(&format!("Validator{}", i)),
                    1_000_000 * 10u128.pow(18),
                )
            })
            .collect();

        let total_validator_stake: Balance = genesis_validators.iter().map(|v| v.stake).sum();
        assert_eq!(total_validator_stake, 5_000_000 * 10u128.pow(18), "Total validator stake must be 5M CHML");
    }

    #[test]
    fn test_parachain_id_unique() {
        assert_ne!(CHAMELEON_PARACHAIN_ID, 2104, "Chameleon parachain ID must be different from Manta");
        assert_eq!(CHAMELEON_PARACHAIN_ID, 2105, "Chameleon parachain ID should be 2105");
    }
}
