use super::*;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type CalamariChainSpec =
	sc_service::GenericChainSpec<calamari_runtime::GenesisConfig, Extensions>;

const CALAMARI_PROTOCOL_ID: &str = "calamari"; // for p2p network configuration
const KUSAMA_RELAYCHAIN_LOCAL_NET: &str = "kusama-local";
const KUSAMA_RELAYCHAIN_DEV_NET: &str = "kusama-dev";

/// Generate the calamari session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn calamari_session_keys(keys: AuraId) -> calamari_runtime::opaque::SessionKeys {
	calamari_runtime::opaque::SessionKeys { aura: keys }
}

// calamari chain specs
pub fn calamari_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), constants::CALAMARI_SS58PREFIX.into());
	p.insert("tokenDecimals".into(), constants::MANTA_DECIMAL.into());
	p.insert(
		"tokenSymbol".into(),
		constants::CALAMARI_TOKEN_SYMBOL.into(),
	);
	p
}

pub fn calamari_development_config(id: ParaId) -> CalamariChainSpec {
	let properties = calamari_properties();

	CalamariChainSpec::from_genesis(
		// Name
		"Calamari Parachain Development",
		// ID
		"calamari_dev",
		ChainType::Local,
		move || {
			calamari_dev_genesis(
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
		Some(CALAMARI_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: KUSAMA_RELAYCHAIN_DEV_NET.into(),
			para_id: id.into(),
		},
	)
}

pub fn calamari_local_config(id: ParaId) -> CalamariChainSpec {
	let properties = calamari_properties();

	CalamariChainSpec::from_genesis(
		// Name
		"Calamari Parachain Local",
		// ID
		"calamari_local",
		ChainType::Local,
		move || {
			calamari_dev_genesis(
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
				id,
			)
		},
		vec![],
		None,
		Some(CALAMARI_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: KUSAMA_RELAYCHAIN_LOCAL_NET.into(),
			para_id: id.into(),
		},
	)
}

fn calamari_dev_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> calamari_runtime::GenesisConfig {
	calamari_runtime::GenesisConfig {
		system: calamari_runtime::SystemConfig {
			code: calamari_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: calamari_runtime::BalancesConfig {
			balances: endowed_accounts[..endowed_accounts.len() / 2]
				.iter()
				.map(|k| {
					(
						k.clone(),
						100 * ENDOWMENT / ((endowed_accounts.len() / 2) as Balance),
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		sudo: calamari_runtime::SudoConfig { key: root_key },
		parachain_info: calamari_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: calamari_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: MA * 1000, // How many tokens will be reserved as collator
			..Default::default()
		},
		session: calamari_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						calamari_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}

pub fn calamari_testnet_config(id: ParaId) -> CalamariChainSpec {
	let properties = calamari_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			// account id: dmvSXhJWeJEKTZT8CCUieJDaNjNFC4ZFqfUm4Lx1z7J7oFzBf
			hex!["4294b2a716cea91dd008d694d264feeaf9f0baf9c0b8cbe3e107515947ed440d"].into(),
			hex!["10814b2b41bf39155ef7b38bb2431056894ba71acc35cf0101c999fd69f9c357"]
				.unchecked_into(),
		),
		(
			// account id: dmxvZaMQir24EPxvFiCzkhDZaiScPB7ZWpHXUv5x8uct2A3du
			hex!["b06e5d852078f64ab74af9b31add10e36d0438b847bc925fbacbf1e14963e379"].into(),
			hex!["f2ac4141fee9f9ba42e830f39f00f316e45d280db1464a9148702ab7c4fcde52"]
				.unchecked_into(),
		),
		(
			// account id: dmud2BmjLyMtbAX2FaVTUtvmutoCKvR3GbARLc4crzGvVMCwu
			hex!["1e58d3c3900c7ce6c6d82152becb45bf7bd3453fb2d267e5f72ca51285bca173"].into(),
			hex!["f6284f9446db8f895c6cf02d0d6de6e67885a1e55c880ccac640ff4bc076df68"]
				.unchecked_into(),
		),
		(
			// account id: dmx4vuA3PnQmraqJqeJaKRydUjP1AW4wMVTPLQWgZSpDyQUrp
			hex!["8a93e0f756448030dcb3018d25d75c7bf97a2e2ff15d02fd1f55bf3f2104fb5b"].into(),
			hex!["741101a186479f4f28aa40fc78f02d7307ed3574e829aed76fdede5876e46a43"]
				.unchecked_into(),
		),
		(
			// account id: dmtwRyEeNyRW3KApnTxjHahWCjN5b9gDjdvxpizHt6E9zYkXj
			hex!["0027131c176c0d19a2a5cc475ecc657f936085912b846839319249e700f37e79"].into(),
			hex!["8ebf03bda1702d719f428bc0a4c7cfca010c44a48ef79752490818c901548d20"]
				.unchecked_into(),
		),
	];

	// root account: dmyBqgFxMPZs1wKz8vFjv7nD4RBu4HeYhZTsGxSDU1wXQV15R
	let root_key: AccountId =
		hex!["bc153ffd4c96de7496df009c6f4ecde6f95bf67b60e0c1025a7552d0b6926e04"].into();

	CalamariChainSpec::from_genesis(
		// Name
		"Calamari Parachain Testnet",
		// ID
		"calamari_testnet",
		ChainType::Local,
		move || calamari_testnet_genesis(initial_authorities.clone(), root_key.clone(), id),
		vec![],
		Some(
			sc_telemetry::TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Calamari testnet telemetry url is valid; qed"),
		),
		Some(CALAMARI_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: KUSAMA_RELAYCHAIN_DEV_NET.into(),
			para_id: id.into(),
		},
	)
}

fn calamari_testnet_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	id: ParaId,
) -> calamari_runtime::GenesisConfig {
	let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.cloned()
		.map(|x| (x.0, ENDOWMENT))
		.collect();
	initial_balances.push((root_key.clone(), 5_000_000_000 * MA));

	calamari_runtime::GenesisConfig {
		system: calamari_runtime::SystemConfig {
			code: calamari_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: calamari_runtime::BalancesConfig {
			balances: initial_balances,
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		sudo: calamari_runtime::SudoConfig { key: root_key },
		parachain_info: calamari_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: calamari_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, _)| acc)
				.collect(),
			candidacy_bond: MA * 1000, // How many tokens will be reserved as collator
			..Default::default()
		},
		session: calamari_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						calamari_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}

// Calamari mainnet
pub fn calamari_config() -> CalamariChainSpec {
	CalamariChainSpec::from_json_bytes(
		&include_bytes!("../../../genesis/calamari-genesis.json")[..],
	)
	.unwrap()
}
