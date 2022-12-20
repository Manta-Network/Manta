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

//! Manta Chain Specifications

use super::*;
use crate::command::MANTA_PARACHAIN_ID;
use manta_runtime::opaque::SessionKeys;
use session_key_primitives::util::{unchecked_account_id, unchecked_collator_keys};
/// Manta Protocol Identifier
pub const MANTA_PROTOCOL_ID: &str = "manta";

/// Polkadot Relaychain Local Network Identifier
pub const POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";

/// Polkadot Relaychain Development Network Identifier
pub const POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";

/// The default XCM version to set in genesis config.
pub const SAFE_XCM_VERSION: u32 = 2;

/// Manta Chain Specification
pub type MantaChainSpec = sc_service::GenericChainSpec<manta_runtime::GenesisConfig, Extensions>;

/// Generate the manta session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn manta_session_keys(keys: AuraId) -> SessionKeys {
    SessionKeys { aura: keys }
}

/// Returns the [`Properties`] for the Manta parachain.
pub fn manta_properties() -> Properties {
    let mut p = Properties::new();
    p.insert("ss58format".into(), constants::MANTA_SS58PREFIX.into());
    p.insert("tokenDecimals".into(), constants::MANTA_DECIMAL.into());
    p.insert("tokenSymbol".into(), constants::MANTA_TOKEN_SYMBOL.into());
    p
}

/// Returns the Manta development chainspec.
pub fn manta_development_config() -> MantaChainSpec {
    MantaChainSpec::from_genesis(
        "Manta Parachain Development",
        "manta_dev",
        ChainType::Local,
        move || {
            manta_dev_genesis(
                vec![(
                    unchecked_account_id::<sr25519::Public>("Alice"),
                    unchecked_collator_keys("Alice").0,
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
        Some(MANTA_PROTOCOL_ID),
        None,
        Some(manta_properties()),
        Extensions {
            relay_chain: POLKADOT_RELAYCHAIN_DEV_NET.into(),
            para_id: MANTA_PARACHAIN_ID,
        },
    )
}

/// Returns the Manta local chainspec.
pub fn manta_local_config() -> MantaChainSpec {
    MantaChainSpec::from_genesis(
        "Manta Parachain Local",
        "manta_local",
        ChainType::Local,
        move || {
            manta_dev_genesis(
                vec![
                    (
                        unchecked_account_id::<sr25519::Public>("Alice"),
                        unchecked_collator_keys("Alice").0,
                    ),
                    (
                        unchecked_account_id::<sr25519::Public>("Bob"),
                        unchecked_collator_keys("Bob").0,
                    ),
                ],
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
        Some(MANTA_PROTOCOL_ID),
        None,
        Some(manta_properties()),
        Extensions {
            relay_chain: POLKADOT_RELAYCHAIN_LOCAL_NET.into(),
            para_id: MANTA_PARACHAIN_ID,
        },
    )
}

fn manta_dev_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> manta_runtime::GenesisConfig {
    manta_runtime::GenesisConfig {
        system: manta_runtime::SystemConfig {
            code: manta_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: manta_runtime::BalancesConfig {
            balances: endowed_accounts[..endowed_accounts.len() / 2]
                .iter()
                .map(|k| {
                    (
                        k.clone(),
                        10 * MANTA_ENDOWMENT / ((endowed_accounts.len() / 2) as Balance),
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        sudo: manta_runtime::SudoConfig {
            key: Some(root_key),
        },
        parachain_info: manta_runtime::ParachainInfoConfig {
            parachain_id: MANTA_PARACHAIN_ID.into(),
        },
        collator_selection: manta_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: MANTA * 10000, // How many tokens will be reserved as collator
            ..Default::default()
        },
        session: manta_runtime::SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),              // account id
                        acc,                      // validator id
                        manta_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: manta_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
    }
}

/// Returns the Manta testnet chainspec.
pub fn manta_testnet_config() -> Result<MantaChainSpec, String> {
    let mut spec = MantaChainSpec::from_json_bytes(
        &include_bytes!("../../../genesis/manta-testnet-genesis.json")[..],
    )?;
    spec.extensions_mut().para_id = MANTA_PARACHAIN_ID;
    Ok(spec)
}

/// Returns the Manta mainnet chainspec.
pub fn manta_config() -> Result<MantaChainSpec, String> {
    MantaChainSpec::from_json_bytes(&include_bytes!("../../../genesis/manta-genesis.json")[..])
}
