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

//! Calamari Chain Specifications

mod dev_genesis;
use dev_genesis::calamari_dev_genesis;

use super::*;
use crate::command::CALAMARI_PARACHAIN_ID;
#[allow(unused_imports)]
use calamari_runtime::{
    currency::KMA, opaque::SessionKeys, CouncilConfig, DemocracyConfig, GenesisConfig,
    LotteryConfig, ParachainStakingConfig, TechnicalCommitteeConfig,
};
use sc_telemetry::TelemetryEndpoints;
use session_key_primitives::util::unchecked_account_id;
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
                    SessionKeys::from_seed_unchecked("Alice"),
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
pub fn calamari_local_config(localdev: bool) -> CalamariChainSpec {
    let id = if localdev {
        "calamari_localdev"
    } else {
        "calamari_local"
    };
    CalamariChainSpec::from_genesis(
        "Calamari Parachain Local",
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
            calamari_dev_genesis(
                invulnerables,
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

/// Returns the Calamari testnet chainspec.
pub fn calamari_testnet_config() -> Result<CalamariChainSpec, String> {
    let mut spec = CalamariChainSpec::from_json_bytes(
        &include_bytes!("../../../../genesis/calamari-testnet-genesis.json")[..],
    )?;
    spec.extensions_mut().para_id = CALAMARI_PARACHAIN_ID;
    Ok(spec)
}

/// Returns the Calamari mainnet chainspec.
pub fn calamari_config() -> Result<CalamariChainSpec, String> {
    CalamariChainSpec::from_json_bytes(
        &include_bytes!("../../../../genesis/calamari-genesis.json")[..],
    )
}

/// Returns the Calamari Baikal internal testnet chainspec.
pub fn calamari_baikal_config() -> CalamariChainSpec {
    let boot_nodes = vec![
		"/dns/crispy.baikal.testnet.calamari.systems/tcp/30333/p2p/12D3KooWSxTYS1UrAeowqsmetyQidbQsztHZcUj7sDzuGcVRoyp2".parse().unwrap(),
		"/dns/crunchy.baikal.testnet.calamari.systems/tcp/30333/p2p/12D3KooWQp2SFroUMVpS5BBHEa4sGexsUzxo9aVoD68BiuWTABqB".parse().unwrap(),
		"/dns/hotdog.baikal.testnet.calamari.systems/tcp/30333/p2p/12D3KooWEpQZzewqvRoMC4DTdm9MAYYGnLSZsVqLeVd2SRfMrY21".parse().unwrap(),
		"/dns/tasty.baikal.testnet.calamari.systems/tcp/30333/p2p/12D3KooWGLKo6GXmP8g79jHrUTgScz4EdbkE9rcbsRyxZCKVQ3nn".parse().unwrap(),
		"/dns/tender.baikal.testnet.calamari.systems/tcp/30333/p2p/12D3KooWRSpMo8JrcNNdnv4oE5ZMzBFToPXsKvzPx4MCsDphPRHm".parse().unwrap(),
	];

    CalamariChainSpec::from_genesis(
        "Calamari Baikal Parachain",
        "calamari_baikal",
        ChainType::Live,
        move || {
            let invulnerables = vec![
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
            ];
            calamari_dev_genesis(
                invulnerables,
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
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(
                "/dns/api.telemetry.pelagos.systems/tcp/443/x-parity-wss/%2Fsubmit%2F".to_string(),
                0,
            )])
            .unwrap(),
        ),
        Some(CALAMARI_PROTOCOL_ID),
        None,
        Some(calamari_properties()),
        Extensions {
            relay_chain: KUSAMA_RELAYCHAIN_LOCAL_NET.into(),
            para_id: CALAMARI_PARACHAIN_ID,
        },
    )
}
