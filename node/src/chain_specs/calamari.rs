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
use calamari_runtime::{
    opaque::SessionKeys, CouncilConfig, DemocracyConfig, GenesisConfig, TechnicalCommitteeConfig,
};
use session_key_primitives::helpers::{get_account_id_from_seed, get_collator_keys_from_seed};
use sp_core::crypto::UncheckedInto;

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
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    SessionKeys::new(get_collator_keys_from_seed("Alice")),
                )],
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
        "Calamari Parachain Staging",
        "calamari_staging",
        ChainType::Live,
        move || {
            calamari_dev_genesis(
                vec![
                    (
                        // account id: dmvSXhJWeJEKTZT8CCUieJDaNjNFC4ZFqfUm4Lx1z7J7oFzBf
                        hex_literal::hex![
                            "4294b2a716cea91dd008d694d264feeaf9f0baf9c0b8cbe3e107515947ed440d"
                        ]
                        .into(),
                        SessionKeys::new(
                            hex_literal::hex![
                                "10814b2b41bf39155ef7b38bb2431056894ba71acc35cf0101c999fd69f9c357"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "10814b2b41bf39155ef7b38bb2431056894ba71acc35cf0101c999fd69f9c357"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "10814b2b41bf39155ef7b38bb2431056894ba71acc35cf0101c999fd69f9c357"
                            ]
                            .unchecked_into(),
                        ),
                    ),
                    (
                        // account id: dmxvZaMQir24EPxvFiCzkhDZaiScPB7ZWpHXUv5x8uct2A3du
                        hex_literal::hex![
                            "b06e5d852078f64ab74af9b31add10e36d0438b847bc925fbacbf1e14963e379"
                        ]
                        .into(),
                        SessionKeys::new(
                            hex_literal::hex![
                                "f2ac4141fee9f9ba42e830f39f00f316e45d280db1464a9148702ab7c4fcde52"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "f2ac4141fee9f9ba42e830f39f00f316e45d280db1464a9148702ab7c4fcde52"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "f2ac4141fee9f9ba42e830f39f00f316e45d280db1464a9148702ab7c4fcde52"
                            ]
                            .unchecked_into(),
                        ),
                    ),
                    (
                        // account id: dmud2BmjLyMtbAX2FaVTUtvmutoCKvR3GbARLc4crzGvVMCwu
                        hex_literal::hex![
                            "1e58d3c3900c7ce6c6d82152becb45bf7bd3453fb2d267e5f72ca51285bca173"
                        ]
                        .into(),
                        SessionKeys::new(
                            hex_literal::hex![
                                "f6284f9446db8f895c6cf02d0d6de6e67885a1e55c880ccac640ff4bc076df68"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "f6284f9446db8f895c6cf02d0d6de6e67885a1e55c880ccac640ff4bc076df68"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "f6284f9446db8f895c6cf02d0d6de6e67885a1e55c880ccac640ff4bc076df68"
                            ]
                            .unchecked_into(),
                        ),
                    ),
                    (
                        // account id: dmx4vuA3PnQmraqJqeJaKRydUjP1AW4wMVTPLQWgZSpDyQUrp
                        hex_literal::hex![
                            "8a93e0f756448030dcb3018d25d75c7bf97a2e2ff15d02fd1f55bf3f2104fb5b"
                        ]
                        .into(),
                        SessionKeys::new(
                            hex_literal::hex![
                                "741101a186479f4f28aa40fc78f02d7307ed3574e829aed76fdede5876e46a43"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "741101a186479f4f28aa40fc78f02d7307ed3574e829aed76fdede5876e46a43"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "741101a186479f4f28aa40fc78f02d7307ed3574e829aed76fdede5876e46a43"
                            ]
                            .unchecked_into(),
                        ),
                    ),
                    (
                        // account id: dmtwRyEeNyRW3KApnTxjHahWCjN5b9gDjdvxpizHt6E9zYkXj
                        hex_literal::hex![
                            "0027131c176c0d19a2a5cc475ecc657f936085912b846839319249e700f37e79"
                        ]
                        .into(),
                        SessionKeys::new(
                            hex_literal::hex![
                                "8ebf03bda1702d719f428bc0a4c7cfca010c44a48ef79752490818c901548d20"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "8ebf03bda1702d719f428bc0a4c7cfca010c44a48ef79752490818c901548d20"
                            ]
                            .unchecked_into(),
                            hex_literal::hex![
                                "8ebf03bda1702d719f428bc0a4c7cfca010c44a48ef79752490818c901548d20"
                            ]
                            .unchecked_into(),
                        ),
                    ),
                ],
                vec![
                    hex_literal::hex![
                        "4294b2a716cea91dd008d694d264feeaf9f0baf9c0b8cbe3e107515947ed440d"
                    ],
                    hex_literal::hex![
                        "b06e5d852078f64ab74af9b31add10e36d0438b847bc925fbacbf1e14963e379"
                    ],
                    hex_literal::hex![
                        "1e58d3c3900c7ce6c6d82152becb45bf7bd3453fb2d267e5f72ca51285bca173"
                    ],
                    hex_literal::hex![
                        "8a93e0f756448030dcb3018d25d75c7bf97a2e2ff15d02fd1f55bf3f2104fb5b"
                    ],
                    hex_literal::hex![
                        "0027131c176c0d19a2a5cc475ecc657f936085912b846839319249e700f37e79"
                    ],
                ]
                .into(),
            )
        },
        vec![],
        None,
        Some(CALAMARI_PROTOCOL_ID),
        None,
        Some(calamari_properties()),
        Extensions {
            relay_chain: "kusama-staging".into(),
            para_id: CALAMARI_PARACHAIN_ID,
        },
    )
}

fn calamari_dev_genesis(
    invulnerables: Vec<(AccountId, SessionKeys)>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: calamari_runtime::SystemConfig {
            code: calamari_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: calamari_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .map(|k| {
                    (
                        k.clone(),
                        10 * CALAMARI_ENDOWMENT / ((endowed_accounts.len()) as Balance),
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        parachain_info: calamari_runtime::ParachainInfoConfig {
            parachain_id: CALAMARI_PARACHAIN_ID.into(),
        },
        collator_selection: calamari_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: KMA * 400_000, // How many tokens will be reserved as collator
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
