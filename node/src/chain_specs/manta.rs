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
use manta_runtime::{
    opaque::SessionKeys, staking::NORMAL_COLLATOR_MINIMUM_STAKE, GenesisConfig,
    ParachainStakingConfig, PolkadotXcmConfig,
};
use session_key_primitives::util::unchecked_account_id;
/// Manta Protocol Identifier
pub const MANTA_PROTOCOL_ID: &str = "manta";

/// Polkadot Relaychain Local Network Identifier
pub const POLKADOT_RELAYCHAIN: &str = "polkadot";

/// Polkadot Relaychain Local Network Identifier
pub const POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";

/// Polkadot Relaychain Development Network Identifier
pub const POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";

/// The default XCM version to set in genesis config.
pub const SAFE_XCM_VERSION: u32 = 2;

/// Manta Chain Specification
pub type MantaChainSpec = sc_service::GenericChainSpec<manta_runtime::GenesisConfig, Extensions>;

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
                    SessionKeys::from_seed_unchecked("Alice"),
                )],
                unchecked_account_id::<sr25519::Public>("Alice"),
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
                ],
                unchecked_account_id::<sr25519::Public>("Alice"),
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
    invulnerables: Vec<(AccountId, SessionKeys)>,
    root_key: AccountId,
    delegations: Vec<(AccountId, AccountId, Balance)>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
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
        parachain_staking: ParachainStakingConfig {
            candidates: invulnerables
                .iter()
                .cloned()
                .map(|(account, _)| (account, NORMAL_COLLATOR_MINIMUM_STAKE))
                .collect(),
            delegations,
            inflation_config: manta_runtime::staking::inflation_config::<manta_runtime::Runtime>(),
        },
        parachain_info: manta_runtime::ParachainInfoConfig {
            parachain_id: MANTA_PARACHAIN_ID.into(),
        },
        collator_selection: manta_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: 10_000 * MANTA, // How many tokens will be reserved as collator
            ..Default::default()
        },
        session: manta_runtime::SessionConfig {
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
        parachain_system: Default::default(),
        polkadot_xcm: PolkadotXcmConfig {
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
    // Ok(mainnet::manta_mainnet_config())
}

pub mod mainnet {
    use super::*;
    use hex_literal::hex;
    use sc_network::config::MultiaddrWithPeerId;
    use sc_telemetry::TelemetryEndpoints;
    use sp_core::crypto::UncheckedInto;
    #[derive(Clone)]
    struct Collator {
        acc: AccountId,
        nodeid: MultiaddrWithPeerId,
        keys: SessionKeys,
    }
    impl Collator {
        fn new(acc: AccountId, nodeid: MultiaddrWithPeerId, keys: SessionKeys) -> Collator {
            Self { acc, nodeid, keys }
        }
    }

    fn manta_mainnet_genesis(genesis_collators: Vec<Collator>) -> GenesisConfig {
        const TOTAL_ISSUANCE: Balance = 1_000_000_000 * MANTA;
        const INITIAL_CONTROLLER_BALANCE: Balance = 5_000 * MANTA;
        const INITIAL_COLLATOR_BALANCE: Balance = 100 * MANTA;
        let root_key: AccountId =
            hex!("fb1818aca757d6bc9aa65c3a94f5a39ff1edbd92e47311c91ace601676168703").into(); // MULTISIG // ss58: 16gEBH3Nq4mtWGg9oFBGveSSoUDcKRABwAq9KESiWKEwb1vQ
        let endowments = vec![
            (
                hex!("4a5d27062c9d0b5747566139c683c17a0e6da8c440c41dfa01725ba664af6977").into(),
                INITIAL_CONTROLLER_BALANCE,
            ), // CONTROLLER 1 ss58 12gWDW1z7T1AN1Vm86RcvRdSmyjtJ1Mv4LHmYvmq9LJG8czh
            (
                hex!("a4c2839433ae05aa48e615e5a6917cc98e4a2886272f2519347f2be7c0c6853d").into(),
                INITIAL_CONTROLLER_BALANCE,
            ), // CONTROLLER 2 ss58 14j2dssnCyQD2xWezTvZmbZPSKDBrwKcAcBtXrE6XgeGfzGx
            (
                hex!("3e31eaf98e0f6dd9b088c095757fac00dc9e3fe6af29e42caea94f7d76dd3444").into(),
                INITIAL_CONTROLLER_BALANCE,
            ), // CONTROLLER 3 ss58 12QYoFyrrEomSq4tMPxVxnWvPwVVKQHUpDWZuDxcg5Gv3c9E
            (
                hex!("0619e4018531059b677d70b476b72431d3e88fe71d2c27bba5d945975e824956").into(),
                INITIAL_CONTROLLER_BALANCE,
            ), // COLLATOR 1
            (
                hex!("0202d1760e33f60f977fcff88b611cb95e66d0b3b74c12ac6c8037b8b9764a31").into(),
                INITIAL_COLLATOR_BALANCE,
            ), // COLLATOR 2
            (
                hex!("4ece9156edd66af83b0c6afe17878844d873a08fd05bd9dbc17f9712814e9952").into(),
                INITIAL_COLLATOR_BALANCE,
            ), // COLLATOR 3
            (
                hex!("7a5cef43508640b6839c37a41272dec83dda4df28503bd87436c197d213f692c").into(),
                INITIAL_COLLATOR_BALANCE,
            ), // COLLATOR 4
            (
                hex!("4039a894994d326e32801a0fc19a267383b6231e74bc9c3fd1021d949d849406").into(),
                INITIAL_COLLATOR_BALANCE,
            ), // COLLATOR 5
            (
                root_key.clone(),
                TOTAL_ISSUANCE - 4 * INITIAL_CONTROLLER_BALANCE - 4 * INITIAL_COLLATOR_BALANCE,
            ), // SUDO ACCOUNT
        ];
        #[allow(clippy::assertions_on_constants)]
        const _: () = assert!(
            NORMAL_COLLATOR_MINIMUM_STAKE < INITIAL_COLLATOR_BALANCE,
            "won't be able to register collator, balance in account set too low"
        );

        GenesisConfig {
            system: manta_runtime::SystemConfig {
                code: manta_runtime::WASM_BINARY
                    .expect("WASM binary was not build, please build it!")
                    .to_vec(),
            },
            balances: manta_runtime::BalancesConfig {
                balances: endowments,
            },
            // empty aura authorities, collators registered with parachain staking instead
            aura: Default::default(),
            sudo: manta_runtime::SudoConfig {
                key: Some(root_key),
            },
            parachain_staking: ParachainStakingConfig {
                candidates: genesis_collators
                    .iter()
                    .map(|collator| (collator.acc.clone(), NORMAL_COLLATOR_MINIMUM_STAKE))
                    .collect(),
                delegations: vec![],
                // set to 0 inflation at genesis
                inflation_config: manta_runtime::staking::inflation_config::<manta_runtime::Runtime>(
                ),
            },
            parachain_info: manta_runtime::ParachainInfoConfig {
                parachain_id: MANTA_PARACHAIN_ID.into(),
            },
            collator_selection: manta_runtime::CollatorSelectionConfig {
                invulnerables: vec![],
                candidacy_bond: 0,
                ..Default::default()
            },
            session: manta_runtime::SessionConfig {
                keys: genesis_collators
                    .iter()
                    .map(|collator| {
                        (
                            collator.acc.clone(),  // account id
                            collator.acc.clone(),  // validator id
                            collator.keys.clone(), // collator session keys
                        )
                    })
                    .collect(),
            },
            parachain_system: Default::default(),
            polkadot_xcm: PolkadotXcmConfig {
                safe_xcm_version: Some(SAFE_XCM_VERSION),
            },
        }
    }

    pub fn manta_mainnet_config() -> MantaChainSpec {
        let genesis_collators: Vec<Collator> = vec![
        Collator::new( // c1: dfWh1oFWNxrmrHdiGdVpQQD6Pp3SmY5NrNsAGNRCxgaqJuZif
            hex!("0619e4018531059b677d70b476b72431d3e88fe71d2c27bba5d945975e824956").into(),
            "/dns/c1.manta.systems/tcp/30333/p2p/12D3KooWSNwD7tJkqKGdMfCVTJbbzrGFTGbXoeMFZCTwEytpFCM4".parse().unwrap(),
            SessionKeys::new((
                hex!("4852f3c0a603d7da194fbff174c24700f3f89d86b462760033bdcb1663c76c60").unchecked_into(), // ss58: dfYBquhamT9Tm8vZy2aNZsLA8ti9TS1DH2Sb5CrdqEGkYxkS2
                hex!("ba09b2e06ab0fa1eefa8e8722f82144a3a25fd3703b28d7377780ad0c2a91256").unchecked_into() // ss58: dfakwcBogWNbwGEbKVAwoKPf2U4Hf6vz9szfVhhdiQwtg2rEL
            ))
        ),
        Collator::new( // c2: dfWbekcVeiw3kzPEw6SiDafzrNM8eWAHTM2ideHQF6jWgS85f
            hex!("0202d1760e33f60f977fcff88b611cb95e66d0b3b74c12ac6c8037b8b9764a31").into(),
            "/dns/c2.manta.systems/tcp/30333/p2p/12D3KooWSyPTkVytQwurRBt73wPQDTgypw88bdhsE4Rb6RnQvCJ9".parse().unwrap(),
            SessionKeys::new((
                hex!("8a8af929d1219802dc8155d0b5381c41d49a185aae2c8bf3537d5c124a63a63e").unchecked_into(), // ss58: dfZgfiDzgdPNwREcJ9hqobyufda6EH2C4p6vDrzeMDvbJwpdm"
                hex!("48c832d59679790f579d6009f5bb56ef0b52da49d7e4c504fe50847d04b12474").unchecked_into(), // ss58: dfYCSjotLLwrHFhkskh7R6HE5ibUDVNTaZ5DqMH2rNseViATG"
            ))
        ),
        Collator::new( // c3: dfYLLvDCZ847eDqvGAf9h9eGmaGi7RgdfAyvfqVqUBFzsEipm
            hex!("4ece9156edd66af83b0c6afe17878844d873a08fd05bd9dbc17f9712814e9952").into(),
            "/dns/c3.manta.systems/tcp/30333/p2p/12D3KooWJwHqCEjTF46eAUDspKKwxa15TMfs7x8DNr3Gs71Qr64j".parse().unwrap(),
            SessionKeys::new((
                hex!("b24d5e9296399d0b66e21dd2b3498d43a67653605455203348d141ef520a9a15").unchecked_into(), // ss58: dfaaoKuam626pYJGb4enxS8NmEGvsWcHADFptCm4RVAWmsLEG"
                hex!("c082e23285092adfa2faebc7683e6a3012e88b59f25c4eb6655c8266da23d30b").unchecked_into(), // ss58: dfauRtpMasGNxTTa5kBjR4LcqGmGrnA53vd3UP9NqF6WctyZ3"
            ))
        ),
        Collator::new( // c4: dfZKTGZS2A7YJmiunthXUNkongd1VMMdrhn7cPxzG2HYxHant
            hex!("7a5cef43508640b6839c37a41272dec83dda4df28503bd87436c197d213f692c").into(),
            "/dns/c4.manta.systems/tcp/30333/p2p/12D3KooWAgZYhwfUo7brgZK2TvArK6XNUtZnzk1cSNfyD9kX1rDE".parse().unwrap(),
            SessionKeys::new((
                hex!("a2d32cfe62273ab87ee00d33bc81abb038292e2dd1e9584b1e81f295ba8ee25a").unchecked_into(), // ss58: dfaEWJu9HfkmHyR2aQac3ZRCnBqsgFS6qncpGRYC2jpv2W1uS
                hex!("a82c54273056aa45a1f43bbead30b749ab2750f1091e21ae7ad17c686c4e2f5d").unchecked_into(), // ss58: dfaMX2rcfXj3aYaMp49iWVrRgkxRj7eeYRnGPTAbUVBXTNQnq
            ))
        ),
        Collator::new( // c5: dfY1E1gYvTvJqnwpSnLfYEHpHEdSNW69dcJQRRKKW1UN7ts9s
            hex!("4039a894994d326e32801a0fc19a267383b6231e74bc9c3fd1021d949d849406").into(),
            "/dns/c5.manta.systems/tcp/30333/p2p/12D3KooWNHVexSGRVeLb7rt7tYS5Y3k5Up9amQn1GyTDCi7L9LLf".parse().unwrap(),
            SessionKeys::new((
                hex!("1eda33797b226fed66ea3ac327a77f26b594b90bc9e9af7416a651bef07a7c72").unchecked_into(), // ss58: dfXFU5LYyQj5vzHeaHbVfhyenuibJJPGSpPonKCPNw6qsUwSm
                hex!("bec2a422e8b72b6f5423eb2ec59d9727e580227e0e0327247d516593ec7b5312").unchecked_into(), // ss58: dfas8jnCTfPXtVvwAu15PtX6WAc3rw4iacCHeuj7qniepWtJE
            ))
        ),
    ];
        let genesis_collators_clone = genesis_collators.clone(); // so we can move it into the constructor closure

        MantaChainSpec::from_genesis(
            "Shark Parachain",
            "manta",
            ChainType::Live,
            move || manta_mainnet_genesis(genesis_collators.clone()),
            genesis_collators_clone
                .into_iter()
                .map(|collator| collator.nodeid)
                .collect(),
            Some(
                TelemetryEndpoints::new(vec![
                    (
                        "/dns/api.telemetry.manta.systems/tcp/443/x-parity-wss/%2Fsubmit%2F"
                            .to_string(),
                        0,
                    ),
                    (
                        "/dns/telemetry.polkadot.io/tcp/443/x-parity-wss/%2Fsubmit%2F".to_string(),
                        0,
                    ),
                ])
                .unwrap(),
            ),
            Some(MANTA_PROTOCOL_ID),
            None,
            Some(manta_properties()),
            Extensions {
                relay_chain: POLKADOT_RELAYCHAIN.into(),
                para_id: MANTA_PARACHAIN_ID,
            },
        )
    }
}
