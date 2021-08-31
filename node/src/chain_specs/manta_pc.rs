use super::*;

pub type MantaPCChainSpec =
	sc_service::GenericChainSpec<manta_pc_runtime::GenesisConfig, Extensions>;

const MANTAPC_PROTOCOL_ID: &str = "manta-pc"; // for p2p network configuration
const POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";
const POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";
#[allow(dead_code)]
const POLKADOT_RELAYCHAIN_MAIN_NET: &str = "polkadot";

/// Generate the manta-pc session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn manta_pc_session_keys(keys: AuraId) -> manta_pc_runtime::opaque::SessionKeys {
	manta_pc_runtime::opaque::SessionKeys { aura: keys }
}

/// Token
pub fn manta_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), constants::MANTAPC_SS58PREFIX.into());
	p.insert("tokenDecimals".into(), constants::MANTA_DECIMAL.into());
	p.insert("tokenSymbol".into(), constants::MANTA_TOKEN_SYMBOL.into());
	p
}

// manta-pc chain spec
pub fn manta_pc_development_config(id: ParaId) -> MantaPCChainSpec {
	let properties = manta_properties();

	MantaPCChainSpec::from_genesis(
		// Name
		"Manta Parachain Development",
		// ID
		"manta_pc_dev",
		ChainType::Local,
		move || {
			manta_pc_dev_genesis(
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
		Some(MANTAPC_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_DEV_NET.into(),
			para_id: id.into(),
		},
	)
}

pub fn manta_pc_local_config(id: ParaId) -> MantaPCChainSpec {
	let properties = manta_properties();

	MantaPCChainSpec::from_genesis(
		// Name
		"Manta Parachain Local",
		// ID
		"manta_pc_local",
		ChainType::Local,
		move || {
			manta_pc_dev_genesis(
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
		Some(MANTAPC_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_LOCAL_NET.into(),
			para_id: id.into(),
		},
	)
}

fn manta_pc_dev_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> manta_pc_runtime::GenesisConfig {
	manta_pc_runtime::GenesisConfig {
		system: manta_pc_runtime::SystemConfig {
			code: manta_pc_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: manta_pc_runtime::BalancesConfig {
			balances: endowed_accounts[..endowed_accounts.len() / 2]
				.iter()
				.map(|k| {
					(
						k.clone(),
						10 * ENDOWMENT / ((endowed_accounts.len() / 2) as Balance),
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		sudo: manta_pc_runtime::SudoConfig { key: root_key },
		parachain_info: manta_pc_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: manta_pc_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: MA * 1000, // How many tokens will be reserved as collator
			..Default::default()
		},
		session: manta_pc_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						manta_pc_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}

pub fn manta_pc_testnet_config(id: ParaId) -> MantaPCChainSpec {
	let properties = manta_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			// account id: dfcAcdrGMvrhhuVnqsXsCi53rGp9JWzyczHdRxn9THe8TiNNt
			hex!["f853f56e9bdec8841c71251dc7c0e4d5160919df9c309907a1d7ab61b35bf530"].into(),
			hex!["56bea77a6396efe47022814f7e34d5e8bae80dc89ba28fca9e413e14709b6002"]
				.unchecked_into(),
		),
		(
			// account id: dfY7Mi9craRmPdMaW8dNYp7Hj9fc8eP1Ti4hvF5ELsdLk36oN
			hex!["44e70b3104b2a6d8140fe0044e1432be8442206e7190a7edd10fa92d365ceb30"].into(),
			hex!["fe5329b7da9f11ce5d0ad8a449b197b4f5ca587ebf5a36841d4ca6dfbd2cf951"]
				.unchecked_into(),
		),
		(
			// account id: dfXk5vAbd5JyqB7XL5HvooReGxA4QdXZiuS4rPhmSs2NBL4z9
			hex!["34adba14c165d5a968dc9e19ec7268c46df418e00a6e4b730a55aa1d58dcce4a"].into(),
			hex!["c06f516066d3c99edbffdc55841107d3518830e0080c979f454f06b53c359645"]
				.unchecked_into(),
		),
		(
			// account id: dfZY9gDWqu3eunq3feCetwXppVUapUk6tJbPmrQBgUfB2JTzr
			hex!["840be78d2675c54f026fd807d2d0da145cf2577689b21cdcfe1a6680c83f9710"].into(),
			hex!["d86fc37d2bcfbcbae99eceb6255a5e063f49e9a90d0ea385343dd8805e5bc533"]
				.unchecked_into(),
		),
		(
			// account id: dfWnpKTX74uCZrWxPbNUWprLsDTJkdr69kwH3Kan8PFEEV4Go
			hex!["0a86b87ebd460a526e5d15f80586f9e0f07310ce6d364bcfe8befba3e742e822"].into(),
			hex!["b08921fb75be024361b8fe8291a5f4368b0db136e08b932fa5c769ff5e704912"]
				.unchecked_into(),
		),
	];

	// root accout: dfYKP4VdPHmSfNU3gCwb1FXqEZrnkw2fKGfQkh6JT4b2z8X4N
	let root_key: AccountId =
		hex!["4e128922a811d874f91c219aaa597ee3bd73bcb22910b3b1c57d297b9175336e"].into();

	MantaPCChainSpec::from_genesis(
		// Name
		"Manta Parachain Testnet",
		// ID
		"manta_pc_testnet",
		ChainType::Local,
		move || manta_pc_testnet_genesis(initial_authorities.clone(), root_key.clone(), id),
		vec![],
		Some(
			sc_telemetry::TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Manta testnet telemetry url is valid; qed"),
		),
		Some(MANTAPC_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: POLKADOT_RELAYCHAIN_DEV_NET.into(),
			para_id: id.into(),
		},
	)
}

fn manta_pc_testnet_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	id: ParaId,
) -> manta_pc_runtime::GenesisConfig {
	let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.cloned()
		.map(|x| (x.0, ENDOWMENT))
		.collect();
	initial_balances.push((root_key.clone(), 500_000_000 * MA));

	manta_pc_runtime::GenesisConfig {
		system: manta_pc_runtime::SystemConfig {
			code: manta_pc_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: manta_pc_runtime::BalancesConfig {
			balances: initial_balances,
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		sudo: manta_pc_runtime::SudoConfig { key: root_key },
		parachain_info: manta_pc_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: manta_pc_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, _)| acc)
				.collect(),
			candidacy_bond: MA * 1000, // How many tokens will be reserved as collator
			..Default::default()
		},
		session: manta_pc_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						manta_pc_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}
