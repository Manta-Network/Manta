// Copyright 2020-2023 Manta Network.
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

//! Dolphin Chain Specifications

use super::*;
use crate::command::{DOLPHIN_ON_BAIKAL_PARACHAIN_ID, DOLPHIN_PARACHAIN_ID};
use dolphin_runtime::{
    opaque::SessionKeys, CouncilConfig, DemocracyConfig, GenesisConfig, TechnicalCommitteeConfig,
};
use session_key_primitives::util::unchecked_account_id;

/// Dolphin Protocol Identifier
pub const DOLPHIN_PROTOCOL_ID: &str = "dolphin";

/// The default XCM version to set in genesis config.
const DOLPHIN_SAFE_XCM_VERSION: u32 = 2;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type DolphinChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Returns the [`Properties`] for the Dolphin parachain.
pub fn dolphin_properties() -> Properties {
    let mut p = Properties::new();
    p.insert("ss58format".into(), constants::CALAMARI_SS58PREFIX.into());
    p.insert("tokenDecimals".into(), constants::DOLPHIN_DECIMAL.into());
    p.insert("tokenSymbol".into(), constants::DOLPHIN_TOKEN_SYMBOL.into());
    p
}

/// Returns the Dolphin development chainspec.
pub fn dolphin_development_config() -> DolphinChainSpec {
    DolphinChainSpec::from_genesis(
        "Dolphin Parachain Development",
        "dolphin_dev",
        ChainType::Local,
        move || {
            dolphin_dev_genesis(
                vec![(
                    unchecked_account_id::<sr25519::Public>("Alice"),
                    SessionKeys::from_seed_unchecked("Alice"),
                )],
                unchecked_account_id::<sr25519::Public>("Alice"),
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
        Some(DOLPHIN_PROTOCOL_ID),
        None,
        Some(dolphin_properties()),
        Extensions {
            relay_chain: "".into(),
            para_id: DOLPHIN_PARACHAIN_ID,
        },
    )
}

/// Returns the Dolphin local chainspec.
pub fn dolphin_local_config(localdev: bool) -> DolphinChainSpec {
    let id = if localdev {
        "dolphin_localdev"
    } else {
        "dolphin_local"
    };
    DolphinChainSpec::from_genesis(
        "Dolphin Parachain Local",
        id,
        ChainType::Local,
        move || {
            let invulnerables = if localdev {
                vec![(
                    unchecked_account_id::<sr25519::Public>("Alice"),
                    SessionKeys::from_seed_unchecked("Alice"),
                )]
            } else {
                vec![
                    (
                        unchecked_account_id::<sr25519::Public>("Alice"),
                        SessionKeys::from_seed_unchecked("Alice"),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Bob"),
                        SessionKeys::from_seed_unchecked("Bob"),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Charlie"),
                        SessionKeys::from_seed_unchecked("Charlie"),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Dave"),
                        SessionKeys::from_seed_unchecked("Dave"),
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Eve"),
                        SessionKeys::from_seed_unchecked("Eve"),
                    ),
                ]
            };
            dolphin_dev_genesis(
                invulnerables,
                unchecked_account_id::<sr25519::Public>("Alice"),
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
        Some(DOLPHIN_PROTOCOL_ID),
        None,
        Some(dolphin_properties()),
        Extensions {
            relay_chain: "".into(),
            para_id: DOLPHIN_PARACHAIN_ID,
        },
    )
}

fn dolphin_dev_genesis(
    invulnerables: Vec<(AccountId, SessionKeys)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: dolphin_runtime::SystemConfig {
            code: dolphin_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: dolphin_runtime::BalancesConfig {
            balances: endowed_accounts[..endowed_accounts.len() / 2]
                .iter()
                .map(|k| {
                    (
                        k.clone(),
                        100 * DOLPHIN_ENDOWMENT / ((endowed_accounts.len() / 2) as Balance),
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        sudo: dolphin_runtime::SudoConfig {
            key: Some(root_key),
        },
        parachain_info: dolphin_runtime::ParachainInfoConfig {
            parachain_id: DOLPHIN_PARACHAIN_ID.into(),
        },
        collator_selection: dolphin_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: DOL * 1000, // How many tokens will be reserved as collator
            ..Default::default()
        },
        session: dolphin_runtime::SessionConfig {
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
        asset_manager: Default::default(),
        council_membership: Default::default(),
        technical_membership: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: dolphin_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(DOLPHIN_SAFE_XCM_VERSION),
        },
    }
}

/// Returns the Dolphin testnet chainspec.
pub fn dolphin_testnet_config() -> Result<DolphinChainSpec, String> {
    let mut spec = DolphinChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/dolphin-testnet-genesis.json")[..],
    )?;
    spec.extensions_mut().para_id = DOLPHIN_PARACHAIN_ID;
    Ok(spec)
}

pub fn dolphin_2085_config() -> Result<DolphinChainSpec, String> {
    let mut spec = DolphinChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/dolphin-2085-genesis.json")[..],
    )?;
    spec.extensions_mut().para_id = DOLPHIN_ON_BAIKAL_PARACHAIN_ID;
    Ok(spec)
}

/// Returns the Dolphin V3 2085 staging chainspec.
pub fn dolphin_v3_2085_staging_config() -> Result<DolphinChainSpec, String> {
    let mut spec = DolphinChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/dolphin-v3-2085-genesis.json")[..],
    )?;
    spec.extensions_mut().para_id = 9997;
    Ok(spec)
}
