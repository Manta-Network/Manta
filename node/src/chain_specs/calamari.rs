// Copyright 2020-2022 Manta Network.
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

//! Calamari Chain Specifications

use super::*;
use crate::command::CALAMARI_PARACHAIN_ID;
#[allow(unused_imports)]
use calamari_runtime::{
    currency::KMA, opaque::SessionKeys, CouncilConfig, DemocracyConfig, GenesisConfig,
    ParachainStakingConfig, TechnicalCommitteeConfig,
};
use session_key_primitives::util::{unchecked_account_id, unchecked_collator_keys};
/// Calamari Protocol Identifier
pub const CALAMARI_PROTOCOL_ID: &str = "calamari";

/// Kusama Relaychain Local Network Identifier
pub const KUSAMA_RELAYCHAIN_LOCAL_NET: &str = "kusama-local";

/// Kusama Relaychain Development Network Identifier
pub const KUSAMA_RELAYCHAIN_DEV_NET: &str = "kusama-dev";

/// The default XCM version to set in genesis config.
pub const SAFE_XCM_VERSION: u32 = 2;

/// Calamari Chain Spec
pub type CalamariChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Returns the [`Properties`] for the Calamari parachain.
pub fn calamari_properties() -> Properties {
    let mut p = Properties::new();
    p.insert("ss58format".into(), constants::CALAMARI_SS58PREFIX.into());
    p.insert("tokenDecimals".into(), constants::CALAMARI_DECIMAL.into());
    p.insert(
        "tokenSymbol".into(),
        constants::CALAMARI_TOKEN_SYMBOL.into(),
    );
    p
}

/// Returns the Calamari development chainspec.
pub fn calamari_development_config() -> CalamariChainSpec {
    CalamariChainSpec::from_genesis(
        "Calamari Parachain Development",
        "calamari_dev",
        ChainType::Local,
        move || {
            calamari_dev_genesis(
                vec![(
                    unchecked_account_id::<sr25519::Public>("Alice"),
                    SessionKeys::new(unchecked_collator_keys("Alice")),
                )],
                // Delegations
                vec![],
                vec![
                    unchecked_account_id::<sr25519::Public>("Alice"),
                    unchecked_account_id::<sr25519::Public>("Bob"),
                    unchecked_account_id::<sr25519::Public>("Alice//stash"),
                    unchecked_account_id::<sr25519::Public>("Bob//stash"),
                ],
            )
        },
        vec![],
        None,
        Some(CALAMARI_PROTOCOL_ID),
        None,
        Some(calamari_properties()),
        Extensions {
            relay_chain: KUSAMA_RELAYCHAIN_DEV_NET.into(),
            para_id: CALAMARI_PARACHAIN_ID,
        },
    )
}

/// Returns the Calamari local chainspec.
pub fn calamari_local_config() -> CalamariChainSpec {
    CalamariChainSpec::from_genesis(
        "Calamari Parachain Local",
        "calamari_local",
        ChainType::Local,
        move || {
            calamari_dev_genesis(
                vec![
                    (
                        unchecked_account_id::<sr25519::Public>("Alice"),
                        SessionKeys::new(unchecked_collator_keys("Alice")),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Bob"),
                        SessionKeys::new(unchecked_collator_keys("Bob")),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Charlie"),
                        SessionKeys::new(unchecked_collator_keys("Charlie")),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Dave"),
                        SessionKeys::new(unchecked_collator_keys("Dave")),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Eve"),
                        SessionKeys::new(unchecked_collator_keys("Eve")),
                    ),
                ],
                // Delegations
                vec![],
                vec![
                    unchecked_account_id::<sr25519::Public>("Alice"),
                    unchecked_account_id::<sr25519::Public>("Bob"),
                    unchecked_account_id::<sr25519::Public>("Charlie"),
                    unchecked_account_id::<sr25519::Public>("Dave"),
                    unchecked_account_id::<sr25519::Public>("Eve"),
                    unchecked_account_id::<sr25519::Public>("Alice//stash"),
                    unchecked_account_id::<sr25519::Public>("Bob//stash"),
                    unchecked_account_id::<sr25519::Public>("Charlie//stash"),
                    unchecked_account_id::<sr25519::Public>("Dave//stash"),
                    unchecked_account_id::<sr25519::Public>("Eve//stash"),
                ],
            )
        },
        vec![],
        None,
        Some(CALAMARI_PROTOCOL_ID),
        None,
        Some(calamari_properties()),
        Extensions {
            relay_chain: KUSAMA_RELAYCHAIN_LOCAL_NET.into(),
            para_id: CALAMARI_PARACHAIN_ID,
        },
    )
}

fn calamari_dev_genesis(
    invulnerables: Vec<(AccountId, SessionKeys)>,

    delegations: Vec<(AccountId, AccountId, Balance)>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: calamari_runtime::SystemConfig {
            code: calamari_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: calamari_runtime::BalancesConfig {
            balances: endowed_accounts[..endowed_accounts.len() / 2]
                .iter()
                .map(|k| {
                    (
                        k.clone(),
                        100 * CALAMARI_ENDOWMENT / ((endowed_accounts.len() / 2) as Balance),
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        parachain_staking: ParachainStakingConfig {
            candidates: invulnerables
                .iter()
                .cloned()
                .map(|(account, _)| (account, 4_000_000 * KMA)) // TODO: Change to use constant from primitives
                .collect(),
            delegations,
            inflation_config: calamari_runtime::currency::inflation_config::<
                calamari_runtime::Runtime,
            >(),
        },
        parachain_info: calamari_runtime::ParachainInfoConfig {
            parachain_id: CALAMARI_PARACHAIN_ID.into(),
        },
        collator_selection: calamari_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: 400_000 * KMA, // How many tokens will be reserved as collator
            ..Default::default()
        },
        session: calamari_runtime::SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, session_keys)| {
                    (
                        acc.clone(),  // account id
                        acc,          // validator id
                        session_keys, // collator session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        council: CouncilConfig {
            members: endowed_accounts.iter().take(1).cloned().collect(),
            phantom: Default::default(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts.iter().take(1).cloned().collect(),
            phantom: Default::default(),
        },
        council_membership: Default::default(),
        technical_membership: Default::default(),
        asset_manager: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: calamari_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
    }
}
/// Returns the Calamari testnet chainspec.
pub fn calamari_testnet_config() -> Result<CalamariChainSpec, String> {
    let mut spec = CalamariChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/calamari-testnet-genesis.json")[..],
    )?;
    spec.extensions_mut().para_id = CALAMARI_PARACHAIN_ID;
    Ok(spec)
}

/// Returns the Calamari testnet for CI chainspec.
pub fn calamari_testnet_ci_config() -> Result<CalamariChainSpec, String> {
    CalamariChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/calamari-testnet-ci-genesis.json")[..],
    )
}

/// Returns the Calamari mainnet chainspec.
pub fn calamari_config() -> Result<CalamariChainSpec, String> {
    CalamariChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/calamari-genesis.json")[..],
    )
}
