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

use super::*;
use crate::command::DOLPHIN_PARACHAIN_ID;

use dolphin_runtime::{
	AssetManagerConfig, CouncilConfig, DemocracyConfig, GenesisConfig, TechnicalCommitteeConfig,
};
use manta_primitives::helpers::{get_account_id_from_seed, get_collator_keys_from_seed};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type DolphinChainSpec =
	sc_service::GenericChainSpec<dolphin_runtime::GenesisConfig, Extensions>;

const DOLPHIN_PROTOCOL_ID: &str = "dolphin"; // for p2p network configuration
const KUSAMA_RELAYCHAIN_LOCAL_NET: &str = "kusama-local";
const KUSAMA_RELAYCHAIN_DEV_NET: &str = "kusama-dev";

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = 2;

/// Generate the dolphin session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn dolphin_session_keys(keys: AuraId) -> dolphin_runtime::opaque::SessionKeys {
	dolphin_runtime::opaque::SessionKeys { aura: keys }
}

// dolphin chain specs
pub fn dolphin_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), constants::CALAMARI_SS58PREFIX.into());
	p.insert("tokenDecimals".into(), constants::DOLPHIN_DECIMAL.into());
	p.insert("tokenSymbol".into(), constants::DOLPHIN_TOKEN_SYMBOL.into());
	p
}

pub fn dolphin_development_config() -> DolphinChainSpec {
	let properties = dolphin_properties();

	DolphinChainSpec::from_genesis(
		// Name
		"Dolphin Parachain Development",
		// ID
		"dolphin_dev",
		ChainType::Local,
		move || {
			dolphin_dev_genesis(
				// initial collators.
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_collator_keys_from_seed("Alice"),
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
		Some(properties),
		Extensions {
			relay_chain: "".into(),
			para_id: DOLPHIN_PARACHAIN_ID,
		},
	)
}

pub fn dolphin_local_config() -> DolphinChainSpec {
	let properties = dolphin_properties();

	DolphinChainSpec::from_genesis(
		// Name
		"Dolphin Parachain Local",
		// ID
		"dolphin_local",
		ChainType::Local,
		move || {
			dolphin_dev_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_collator_keys_from_seed("Charlie"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						get_collator_keys_from_seed("Dave"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						get_collator_keys_from_seed("Eve"),
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
		Some(properties),
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
) -> dolphin_runtime::GenesisConfig {
	dolphin_runtime::GenesisConfig {
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

// pub fn dolphin_testnet_config() -> Result<DolphinChainSpec, String> {
// 	let mut spec = DolphinChainSpec::from_json_bytes(
// 		&include_bytes!("../../../genesis/dolphin-testnet-genesis.json")[..],
// 	)?;
// 	spec.extensions_mut().para_id = DOLPHIN_PARACHAIN_ID;
// 	Ok(spec)
// }

pub fn dolphin_testnet_config() -> DolphinChainSpec {
	let properties = dolphin_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			// account id: dmuXsDLj4oAVr81s5cDGF8Q7G7WxCh1TFMULdhsrGQLzfUWv9
			hex_literal::hex!["fc54607744a44b252d4f3ea13fdabf00a15aff220ea0cecd1ca83baf9d96a34f"]
				.into(),
			hex_literal::hex!["10eb9ee3a4beaad720b6f193384aba954ea5df7dc8d8c19b8a7a8a2ac9a35d21"]
				.unchecked_into(),
		),
		(
			// account id: dmuQh7dKho8ZzSu16iGR18skWqm28FeqyFjpRYbiKNm5M8vY9
			hex_literal::hex!["0a08c73e1955d586cbc9cf22aad2c278b7ccc68aee2b40ddcd680c7ab92e6a60"]
				.into(),
			hex_literal::hex!["96d512717c2d5b232bacb3bb164b18ca9ee77c4ee8c73a0e43e6c89b7dbf2562"]
				.unchecked_into(),
		),
		(
			// account id: dmudPU6mEcENzH1fvePtBaQYvwRY4rdm2wE3M1zBu4MXXbAHT
			hex_literal::hex!["064284a5fffe704ca7fb2ec32bc0b1164aaceba0d8ae7e3c191724c8255f787e"]
				.into(),
			hex_literal::hex!["90c1fc7a8efdde12ab3003cb7c0eaa171ae794497f8a456d4ec01923edf1c327"]
				.unchecked_into(),
		),
		(
			// account id: dmukcPL97Mr5g3kg8ct6kiXciZV2n7Vnd6ZPpa7ogUhgWuZ1i
			hex_literal::hex!["f6cbc80ab3d92c97029a32da5f9f986fc4d0d697d7b4ddb0028f415459f73433"]
				.into(),
			hex_literal::hex!["3072406e3387cca8e889671f3dbfbce600958c3cf614f8e2eb1f8ad8b19c4615"]
				.unchecked_into(),
		),
		(
			// account id: dmyWaw9MZ2AwHqQvULhGDyeXvLGkEXfrHApAHsaB9GBASwUtz
			hex_literal::hex!["2229a7740efa90b3449b3fb4ed927d82cc3e8a369cae7eda79ff47a29cea8d11"]
				.into(),
			hex_literal::hex!["42d4506d4eac8d5be3cd11dcd9038971691690d0725be9e9fb4e468249d79b7c"]
				.unchecked_into(),
		),
	];

	// root account: dmwSWSApwV9LdbVbEkWAkg1HznrHMNDn6yA8VEQ7UgfFcqqkY
	let root_key: AccountId =
		hex_literal::hex!["82a9cd66c8959be784dd5129ef00eb9d919909a90fc9319ce5e2fb89b713c73c"]
			.into();

	DolphinChainSpec::from_genesis(
		// Name
		"Dolphin Parachain Temp",
		// ID
		"dolphin_temp",
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
			relay_chain: "kusama".into(),
			para_id: DOLPHIN_PARACHAIN_ID,
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
			parachain_id: DOLPHIN_PARACHAIN_ID.into(),
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
