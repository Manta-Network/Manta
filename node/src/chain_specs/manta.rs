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
		hex!["deb9e5b3f5942f66b94a5496f79053b83efc7df0eadbb4e17344b03c96efd52f"].into(),
	];

	MantaChainSpec::from_genesis(
		// Name
		"Manta Parachain Testnet",
		// ID
		"manta_testnet",
		ChainType::Local,
		move || {
			manta_testnet_genesis(
				initial_authorities.clone(),
				root_key.clone(),
				root_controllers.clone(),
				id,
			)
		},
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

pub fn manta_config(id: ParaId) -> MantaChainSpec {
	let properties = manta_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			hex!["0a667c67f4b0499b222eb2f4e7fa23cc2404587c7b3c38a4c80d2a4bf1173415"].into(),
			hex!["be333c8657539b601306850f6fd0e939541fddc139861ea36c44b1dab880990a"]
				.unchecked_into(),
		),
		(
			hex!["24bbcf0d98d8a4c88c32e49693eb9d45526718eb7113e890473dddcad5d8e90c"].into(),
			hex!["628dd3b79748ce21a74107f10e417107a59c83988ef57598482f47eb4506dd2f"]
				.unchecked_into(),
		),
		(
			hex!["fa4a27839a0fb276700fe3f89dd9483ee6e125b6805a3e56abde8194c8156421"].into(),
			hex!["deff6eeef76208eb1e76013519a54ecacfc87d070902e305dfc65df707c5cb27"]
				.unchecked_into(),
		),
		(
			hex!["dee345754f20c0b7bbd26af13af1703093f066fb45f1401c4ecea0451932e92a"].into(),
			hex!["82ea3434902300b10cca7080af294f41ad478d4dde5bc85231820a9a3d434529"]
				.unchecked_into(),
		),
		(
			hex!["0206514b020cb93e5ed853954f9f461d694b1016cff6cc108ad6ba905f660a52"].into(),
			hex!["fefd3f734afedea792f94d1f5c0e622b97decab99a116337996ceac044893811"]
				.unchecked_into(),
		),
	];

	// root accout:
	let root_key: AccountId =
		hex!["6f29ececbfe810fc957e53bc66af53f9f6722d51d1b7417b0caa179770d190b0"].into();
	// treasury account:
	let treasury_key: AccountId =
		hex!["4e8a2a12d210f77c7c051406061c605f7c2fe41165f61abd1591292d0acc01ac"].into();

	// first 5 are root controllers, next 5 are treasury controllers
	let controllers: Vec<AccountId> = vec![
		hex!["8421393ff776ee8cdc4116d3eeb62afdf014aed077a13907308437ba560b8c61"].into(),
		hex!["50049deef9dc52327ceee49fb3934860678434fef94f0383bd7d2844c8e89f24"].into(),
		hex!["3c4dffe781018bb43259dd18ff566d5290fe3a7bdcd40c59362c7a3947af314a"].into(),
		hex!["2cf40ae574a3a762fb70e5cb577bafce2cf930e606ec581fa421ef770cd40840"].into(),
		hex!["4eccdc12de7dd755993e2d6cd3aa941005ded69c1fd3abbc8854cafaa9e70e29"].into(),
		hex!["e0b4301d7cd7ea22ca16b1424ef0dfb9d58575054cbcfaca1d97b2cda292cb68"].into(),
		hex!["10a7075d5206123d836d442e44bed9e48cafa5dc73730d08cf77871f28568249"].into(),
		hex!["ee73111842e7d40653c4680bfd94865fe8c99c867adfcac3ebe6497b6b4ae51f"].into(),
		hex!["2e7d715571d612051f7a4c951769f090e82b41803adf1e2da92f52629ff28611"].into(),
		hex!["5c555792cf64b4e1ac1e17afdd35a661ecb4610e21dd3d48f9898feb21295539"].into(),
	];

	MantaChainSpec::from_genesis(
		// Name
		"Manta Parachain",
		// ID
		"manta",
		ChainType::Live,
		move || {
			manta_genesis(
				initial_authorities.clone(),
				root_key.clone(),
				controllers.clone(),
				treasury_key.clone(),
				id,
			)
		},
		vec![],
		Some(
			sc_telemetry::TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Manta telemetry url is valid; qed"),
		),
		Some(MANTA_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_MAIN_NET.into(),
			para_id: id.into(),
		},
	)
}

fn manta_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	controllers: Vec<AccountId>,
	treasury: AccountId,
	id: ParaId,
) -> manta_runtime::GenesisConfig {
	const TOTAL_SUPPLY: u128 = 1000_000_000 * MANTA;
	// default initial balances for root, controller, and collators
	const DEFAULT_INITIAL_BALANCE: u128 = 100_000 * MANTA;

	let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.cloned()
		.map(|x| (x.0, DEFAULT_INITIAL_BALANCE))
		.collect();

	initial_balances.push((root_key.clone(), DEFAULT_INITIAL_BALANCE));
	for account in controllers.clone() {
		initial_balances.push((account, DEFAULT_INITIAL_BALANCE));
	}

	let treasury_balance = TOTAL_SUPPLY
		- (initial_authorities.len() as u128 + controllers.len() as u128 + 1)
			* DEFAULT_INITIAL_BALANCE;
	initial_balances.push((treasury.clone(), treasury_balance));

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
			candidacy_bond: MANTA * 10_000, // How many tokens will be reserved as collator
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
