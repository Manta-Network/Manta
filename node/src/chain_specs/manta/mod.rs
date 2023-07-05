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

//! Manta Chain Specifications

use super::*;
use crate::command::MANTA_PARACHAIN_ID;
use manta_runtime::{
    opaque::SessionKeys, staking::NORMAL_COLLATOR_MINIMUM_STAKE, CouncilConfig, DemocracyConfig,
    GenesisConfig, LotteryConfig, ParachainStakingConfig, PolkadotXcmConfig,
    TechnicalCommitteeConfig,
};
use sc_network_common::config::MultiaddrWithPeerId;

mod local_testnets_geneses;
mod public_testnet_genesis;

/// Manta Protocol Identifier
pub const MANTA_PROTOCOL_ID: &str = "manta";

/// Polkadot Relaychain Local Network Identifier
pub const POLKADOT_RELAYCHAIN: &str = "polkadot";

pub const POLKADOT_STAGING_NET: &str = "polkadot_staging_testnet";

/// Polkadot Relaychain Local Network Identifier
pub const POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";

/// Polkadot Relaychain Development Network Identifier
pub const POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";

/// The default XCM version to set in genesis config.
pub const MANTA_SAFE_XCM_VERSION: u32 = 2;

/// Manta Chain Specification
pub type MantaChainSpec = sc_service::GenericChainSpec<manta_runtime::GenesisConfig, Extensions>;

#[derive(Clone)]
struct Collator {
    acc: AccountId,
    nodeid: Option<MultiaddrWithPeerId>,
    keys: SessionKeys,
}
impl Collator {
    fn new(acc: AccountId, nodeid: Option<MultiaddrWithPeerId>, keys: SessionKeys) -> Collator {
        Self { acc, nodeid, keys }
    }
}

/// Returns the [`Properties`] for the Manta parachain.
pub fn manta_properties() -> Properties {
    let mut p = Properties::new();
    p.insert("ss58format".into(), constants::MANTA_SS58PREFIX.into());
    p.insert("tokenDecimals".into(), constants::MANTA_DECIMAL.into());
    p.insert("tokenSymbol".into(), constants::MANTA_TOKEN_SYMBOL.into());
    p
}

/// Returns the Manta mainnet chainspec
pub fn manta_mainnet_config() -> Result<MantaChainSpec, String> {
    MantaChainSpec::from_json_bytes(&include_bytes!("../../../../genesis/manta-genesis.json")[..])
}
/// Returns the Manta testnet chainspec.
pub fn manta_testnet_config() -> MantaChainSpec {
    // NOTE: The public testnet is expected to be reset frequently
    // and the `manta build-spec`-generated genesis is held in the ansible deployment
    // There is no need to maintain a checked-in genesis.json in this repo as well
    public_testnet_genesis::genesis_spec()
}
/// Returns the Manta development chainspec.
pub fn manta_local_config(localdev: bool) -> MantaChainSpec {
    local_testnets_geneses::genesis_spec_local(localdev)
}
/// Returns the Manta development chainspec.
pub fn manta_development_config() -> MantaChainSpec {
    local_testnets_geneses::genesis_spec_dev()
}

// common helper to create the above configs
fn manta_devnet_genesis(genesis_collators: Vec<Collator>) -> GenesisConfig {
    let root_key = genesis_collators.first().unwrap().acc.clone();

    const INITIAL_COLLATOR_BALANCE: Balance = 1_000_000_000 * MANTA;
    let endowments = genesis_collators
        .iter()
        .map(|collator| (collator.acc.clone(), INITIAL_COLLATOR_BALANCE))
        .collect::<Vec<_>>();
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
            balances: endowments.clone(),
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
            inflation_config: manta_runtime::staking::inflation_config::<manta_runtime::Runtime>(),
        },
        lottery: LotteryConfig {
            min_deposit: 500 * MANTA,
            min_withdraw: 10 * MANTA,
            gas_reserve: 1_000 * MANTA,
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
            safe_xcm_version: Some(MANTA_SAFE_XCM_VERSION),
        },
        asset_manager: Default::default(),
        democracy: DemocracyConfig::default(),
        council: CouncilConfig {
            members: endowments
                .iter()
                .map(|endowed| endowed.0.clone())
                .take(1)
                .collect(),
            phantom: Default::default(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: endowments
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
