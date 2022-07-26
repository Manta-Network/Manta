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

//! Dolphin Chain Specifications

use super::*;
use crate::command::{DOLPHIN_ON_BAIKAL_PARACHAIN_ID, DOLPHIN_PARACHAIN_ID};
use dolphin_runtime::{
    opaque::SessionKeys, CouncilConfig, DemocracyConfig, GenesisConfig, TechnicalCommitteeConfig,
};
use session_key_primitives::helpers::{get_account_id_from_seed, get_collator_keys_from_seed};
/// Dolphin Protocol Identifier
pub const DOLPHIN_PROTOCOL_ID: &str = "dolphin";
use sp_core::crypto::UncheckedInto;
/// Kusama Relaychain Local Network Identifier
pub const KUSAMA_RELAYCHAIN_LOCAL_NET: &str = "kusama-local";

/// Kusama Relaychain Development Network Identifier
pub const KUSAMA_RELAYCHAIN_DEV_NET: &str = "kusama-dev";

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = 2;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type DolphinChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate the dolphin session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn dolphin_session_keys(keys: AuraId) -> SessionKeys {
    SessionKeys {
        aura: keys.clone(),
        nimbus: session_key_primitives::nimbus::dummy_key_from(keys.clone()),
        vrf: session_key_primitives::vrf::dummy_key_from(keys),
    }
}

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
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    SessionKeys::new(get_collator_keys_from_seed("Alice")).aura,
                )],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
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
pub fn dolphin_local_config() -> DolphinChainSpec {
    DolphinChainSpec::from_genesis(
        "Dolphin Parachain Local",
        "dolphin_local",
        ChainType::Local,
        move || {
            dolphin_dev_genesis(
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        SessionKeys::new(get_collator_keys_from_seed("Alice")).aura,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        SessionKeys::new(get_collator_keys_from_seed("Bob")).aura,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Charlie"),
                        SessionKeys::new(get_collator_keys_from_seed("Charlie")).aura,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Dave"),
                        SessionKeys::new(get_collator_keys_from_seed("Dave")).aura,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Eve"),
                        SessionKeys::new(get_collator_keys_from_seed("Eve")).aura,
                    ),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
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
    invulnerables: Vec<(AccountId, AuraId)>,
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
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                // account id
                        acc,                        // validator id
                        dolphin_session_keys(aura), // session keys
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
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: dolphin_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
    }
}

// /// Returns the Dolphin testnet chainspec.
// pub fn dolphin_testnet_config() -> Result<DolphinChainSpec, String> {
//     let mut spec = DolphinChainSpec::from_json_bytes(
//         &include_bytes!("../../../genesis/dolphin-testnet-genesis.json")[..],
//     )?;
//     spec.extensions_mut().para_id = DOLPHIN_PARACHAIN_ID;
//     Ok(spec)
// }

// pub fn dolphin_testnet_ci_config() -> Result<DolphinChainSpec, String> {
//     let mut spec = DolphinChainSpec::from_json_bytes(
//         &include_bytes!("../../../genesis/dolphin-testnet-ci-genesis.json")[..],
//     )?;
//     spec.extensions_mut().para_id = DOLPHIN_PARACHAIN_ID;
//     Ok(spec)
// }

pub fn dolphin_testnet_config() -> DolphinChainSpec {
    let properties = dolphin_properties();

    // (controller_account, aura_id)
    let initial_authorities: Vec<(AccountId, AuraId)> = vec![
        (
            // account id: dmvSXhJWeJEKTZT8CCUieJDaNjNFC4ZFqfUm4Lx1z7J7oFzBf
            hex_literal::hex!["4294b2a716cea91dd008d694d264feeaf9f0baf9c0b8cbe3e107515947ed440d"]
                .into(),
            hex_literal::hex!["10814b2b41bf39155ef7b38bb2431056894ba71acc35cf0101c999fd69f9c357"]
                .unchecked_into(),
        ),
        (
            // account id: dmxvZaMQir24EPxvFiCzkhDZaiScPB7ZWpHXUv5x8uct2A3du
            hex_literal::hex!["b06e5d852078f64ab74af9b31add10e36d0438b847bc925fbacbf1e14963e379"]
                .into(),
            hex_literal::hex!["f2ac4141fee9f9ba42e830f39f00f316e45d280db1464a9148702ab7c4fcde52"]
                .unchecked_into(),
        ),
        (
            // account id: dmud2BmjLyMtbAX2FaVTUtvmutoCKvR3GbARLc4crzGvVMCwu
            hex_literal::hex!["1e58d3c3900c7ce6c6d82152becb45bf7bd3453fb2d267e5f72ca51285bca173"]
                .into(),
            hex_literal::hex!["f6284f9446db8f895c6cf02d0d6de6e67885a1e55c880ccac640ff4bc076df68"]
                .unchecked_into(),
        ),
        (
            // account id: dmx4vuA3PnQmraqJqeJaKRydUjP1AW4wMVTPLQWgZSpDyQUrp
            hex_literal::hex!["8a93e0f756448030dcb3018d25d75c7bf97a2e2ff15d02fd1f55bf3f2104fb5b"]
                .into(),
            hex_literal::hex!["741101a186479f4f28aa40fc78f02d7307ed3574e829aed76fdede5876e46a43"]
                .unchecked_into(),
        ),
        (
            // account id: dmtwRyEeNyRW3KApnTxjHahWCjN5b9gDjdvxpizHt6E9zYkXj
            hex_literal::hex!["0027131c176c0d19a2a5cc475ecc657f936085912b846839319249e700f37e79"]
                .into(),
            hex_literal::hex!["8ebf03bda1702d719f428bc0a4c7cfca010c44a48ef79752490818c901548d20"]
                .unchecked_into(),
        ),
    ];

    // root account: dmyBqgFxMPZs1wKz8vFjv7nD4RBu4HeYhZTsGxSDU1wXQV15R
    let root_key: AccountId =
        hex_literal::hex!["bc153ffd4c96de7496df009c6f4ecde6f95bf67b60e0c1025a7552d0b6926e04"]
            .into();

    DolphinChainSpec::from_genesis(
        // Name
        "Dolphin-2085 Parachain",
        // ID
        "dolphin_2085",
        ChainType::Live,
        move || dolphin_testnet_genesis(initial_authorities.clone(), root_key.clone()),
        vec![],
        Some(
            sc_telemetry::TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Dolphin testnet telemetry url is valid; qed"),
        ),
        Some(DOLPHIN_PROTOCOL_ID),
        None,
        Some(properties),
        Extensions {
            relay_chain: "".into(),
            para_id: DOLPHIN_ON_BAIKAL_PARACHAIN_ID,
        },
    )
}

fn dolphin_testnet_genesis(
    initial_authorities: Vec<(AccountId, AuraId)>,
    root_key: AccountId,
) -> dolphin_runtime::GenesisConfig {
    let endowment = 5_000_000_000 * DOL;
    let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
        .iter()
        .cloned()
        .map(|x| (x.0, endowment))
        .collect();
    initial_balances.push((root_key.clone(), endowment));

    dolphin_runtime::GenesisConfig {
        system: dolphin_runtime::SystemConfig {
            code: dolphin_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: dolphin_runtime::BalancesConfig {
            balances: initial_balances,
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        sudo: dolphin_runtime::SudoConfig {
            key: Some(root_key),
        },
        parachain_info: dolphin_runtime::ParachainInfoConfig {
            parachain_id: DOLPHIN_ON_BAIKAL_PARACHAIN_ID.into(),
        },
        collator_selection: dolphin_runtime::CollatorSelectionConfig {
            invulnerables: initial_authorities
                .iter()
                .cloned()
                .map(|(acc, _)| acc)
                .collect(),
            candidacy_bond: DOL * 1000, // How many tokens will be reserved as collator
            ..Default::default()
        },
        democracy: DemocracyConfig::default(),
        council: CouncilConfig {
            members: Default::default(),
            phantom: Default::default(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: Default::default(),
            phantom: Default::default(),
        },
        council_membership: Default::default(),
        technical_membership: Default::default(),
        session: dolphin_runtime::SessionConfig {
            keys: initial_authorities
                .iter()
                .cloned()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                // account id
                        acc,                        // validator id
                        dolphin_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        asset_manager: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: dolphin_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
    }
}
