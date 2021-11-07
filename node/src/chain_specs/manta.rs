// Copyright 2020-2021 Manta Network.
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

pub type MantaChainSpec = sc_service::GenericChainSpec<manta_runtime::GenesisConfig, Extensions>;

const MANTA_PROTOCOL_ID: &str = "manta"; // for p2p network configuration
const POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";
const POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";
#[allow(dead_code)]
const POLKADOT_RELAYCHAIN_MAIN_NET: &str = "polkadot";

/// Generate the manta session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn manta_session_keys(keys: AuraId) -> manta_runtime::opaque::SessionKeys {
	manta_runtime::opaque::SessionKeys { aura: keys }
}

/// Token
pub fn manta_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), constants::MANTA_SS58PREFIX.into());
	p.insert("tokenDecimals".into(), constants::MANTA_DECIMAL.into());
	p.insert("tokenSymbol".into(), constants::MANTA_TOKEN_SYMBOL.into());
	p
}

// manta chain spec
pub fn manta_development_config(id: ParaId) -> MantaChainSpec {
	let properties = manta_properties();

	MantaChainSpec::from_genesis(
		// Name
		"Manta Parachain Development",
		// ID
		"manta_dev",
		ChainType::Local,
		move || {
			manta_dev_genesis(
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
				id,
			)
		},
		vec![],
		None,
		Some(MANTA_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_DEV_NET.into(),
			para_id: id.into(),
		},
	)
}

pub fn manta_local_config(id: ParaId) -> MantaChainSpec {
	let properties = manta_properties();

	MantaChainSpec::from_genesis(
		// Name
		"Manta Parachain Local",
		// ID
		"manta_local",
		ChainType::Local,
		move || {
			manta_dev_genesis(
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
				id,
			)
		},
		vec![],
		None,
		Some(MANTA_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_LOCAL_NET.into(),
			para_id: id.into(),
		},
	)
}

fn manta_dev_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> manta_runtime::GenesisConfig {
	manta_runtime::GenesisConfig {
		system: manta_runtime::SystemConfig {
			code: manta_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
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
		sudo: manta_runtime::SudoConfig { key: root_key },
		parachain_info: manta_runtime::ParachainInfoConfig { parachain_id: id },
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
	}
}

pub fn manta_testnet_config(id: ParaId) -> MantaChainSpec {
	let properties = manta_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			// account id: dfZKJxcgRqQsPdC53eXBsgQWFrKTbXHm76v35ndypRPxz2JJR
			hex!["7a40f6773ffa7d13147daa0f8cf7e5ea5b54a14fb515ccded35ea7df7ce2c26a"].into(),
			hex!["a6da86747dce627b0a0cf4189ce35247a5c0c9a69570f2b5b72241beb711a141"]
				.unchecked_into(),
		),
		(
			// account id: dfWsu5CrRwsnmJHNM1xW4iUJSQLNavVC1D2PmwtRBHT7dNxRV
			hex!["0e66f3b49250bced29cff1d717b944f4f57e5ced096e4b6aeeb7d5206d7b1d0e"].into(),
			hex!["c8ddaec483dfa0a580a7c8194ee625a6251743859070415aa7f8f384abd6c550"]
				.unchecked_into(),
		),
		(
			// account id: dfbzNJu8JcX2ixaVKgbG65qBBDV9M6yYWpg2UgnyNVwPdjGWt
			hex!["f08346ce33e5c8c29d0fcb7aa70db75964d763f0537777ef9d5f0091fe3d371c"].into(),
			hex!["6c14813c02fa0b9992560cae02337c748af2e46bb5a1b26b6011bde02f92f356"]
				.unchecked_into(),
		),
		(
			// account id: dfYKdJMKT4X8inLTaSvtGjmtBGAyCbP1AuQjNu73EX4Hv6cG3
			hex!["4e4277d721cfed60407222cb7e47701a60597d7b598cda5d0ac38fc29dab8d72"].into(),
			hex!["966c68c4308b757bef26f21e4951cfd47e6a56ce6c68350dff5d3355bbf27749"]
				.unchecked_into(),
		),
		(
			// account id: dfaE1cZfyqn1taSRno43bHRKNXFxErfhtDquErcfEL11YxxMr
			hex!["a272940a6d11b48f691225841e168d0f16c8101cc034f115298c4aa53c2a5d6f"].into(),
			hex!["2e6dba967ee6ca20655e92ee82954aed4d88975435a835b97973c270dfa67402"]
				.unchecked_into(),
		),
	];

	// root accout: dfaKjznDQQFYixKSBNkfxShEbzfQ6Jvjkkn6cifV2jSCNoY1e
	let root_key: AccountId =
		hex!["a6d17ab57e1a1b7e70aea7d1c084afef514ae69613e67397fe9690ae8c0944a4"].into();
	
	let root_controllers: Vec<AccountId> = vec![
		hex!["3e3bed621633daf5ff0aa6b37d7e676aff09a942da813ab2dc6dd5e8baaf9c09"].into(),
		hex!["543e1e0ff9213cd1a3ed8cefd1443c4d7434c3d109aa665c8ec5b4ea80d37445"].into(),
		hex!["c0592dc117d6e9497d5f2ce4babadfee405761475534c7cc3b834824e845ee2c"].into(),
		hex!["5e0910c13f5f5c2b8256b8a5a1e8c9a04e377acda504f25b8a07dfc748f4382d"].into(),
		hex!["deb9e5b3f5942f66b94a5496f79053b83efc7df0eadbb4e17344b03c96efd52f"].into()
	];

	MantaChainSpec::from_genesis(
		// Name
		"Manta Parachain Testnet",
		// ID
		"manta_testnet",
		ChainType::Local,
		move || manta_testnet_genesis(initial_authorities.clone(), root_key.clone(), root_controllers.clone(), id),
		vec![],
		Some(
			sc_telemetry::TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Manta testnet telemetry url is valid; qed"),
		),
		Some(MANTA_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_DEV_NET.into(),
			para_id: id.into(),
		},
	)
}

fn manta_testnet_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	root_controllers: Vec<AccountId>,
	id: ParaId,
) -> manta_runtime::GenesisConfig {
	let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.cloned()
		.map(|x| (x.0, MANTA_ENDOWMENT))
		.collect();
	initial_balances.push((root_key.clone(), 499_500_000 * MANTA));
	for account in root_controllers {
		initial_balances.push((account, 100_000 * MANTA));
	}

	manta_runtime::GenesisConfig {
		system: manta_runtime::SystemConfig {
			code: manta_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: manta_runtime::BalancesConfig {
			balances: initial_balances,
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		sudo: manta_runtime::SudoConfig { key: root_key },
		parachain_info: manta_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: manta_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, _)| acc)
				.collect(),
			candidacy_bond: MANTA * 1000, // How many tokens will be reserved as collator
			..Default::default()
		},
		session: manta_runtime::SessionConfig {
			keys: initial_authorities
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
	}
}
