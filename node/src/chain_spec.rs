use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use manta_primitives::{constants, currency::MA, AccountId, AuraId, Balance, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
#[cfg(feature = "calamari")]
pub type CalamariChainSpec =
	sc_service::GenericChainSpec<calamari_runtime::GenesisConfig, Extensions>;
#[cfg(feature = "manta-pc")]
pub type MantaPCChainSpec =
	sc_service::GenericChainSpec<manta_pc_runtime::GenesisConfig, Extensions>;

const ENDOWMENT: Balance = 100_000_000 * MA; // 10 endowment so that total supply is 1B
#[cfg(feature = "calamari")]
const CALAMARI_PROTOCOL_ID: &str = "calamari"; // for p2p network configuration
#[cfg(feature = "manta-pc")]
const MANTAPC_PROTOCOL_ID: &str = "manta-pc"; // for p2p network configuration
const STAGING_TELEMETRY_URL: &str = "wss://api.telemetry.manta.systems/submit/";

#[cfg(feature = "manta-pc")]
const POLKADOT_RELAYCHAIN_LOCAL_NET: &str = "polkadot-local";
#[cfg(feature = "manta-pc")]
const POLKADOT_RELAYCHAIN_DEV_NET: &str = "polkadot-dev";
#[cfg(feature = "manta-pc")]
const POLKADOT_RELAYCHAIN_MAIN_NET: &str = "polkadot";

#[cfg(feature = "calamari")]
const KUSAMA_RELAYCHAIN_LOCAL_NET: &str = "kusama-local";
#[cfg(feature = "calamari")]
const KUSAMA_RELAYCHAIN_DEV_NET: &str = "kusama-dev";
#[cfg(feature = "calamari")]
const KUSAMA_RELAYCHAIN_MAIN_NET: &str = "kusama";

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_pair_from_seed::<AuraId>(seed)
}

/// Generate the manta-pc session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
#[cfg(feature = "manta-pc")]
pub fn manta_pc_session_keys(keys: AuraId) -> manta_pc_runtime::opaque::SessionKeys {
	manta_pc_runtime::opaque::SessionKeys { aura: keys }
}

/// Generate the calamari session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
#[cfg(feature = "calamari")]
pub fn calamari_session_keys(keys: AuraId) -> calamari_runtime::opaque::SessionKeys {
	calamari_runtime::opaque::SessionKeys { aura: keys }
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;
/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

/// Token
#[cfg(feature = "manta-pc")]
pub fn manta_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), constants::MANTAPC_SS58PREFIX.into());
	p.insert("tokenDecimals".into(), constants::MANTA_DECIMAL.into());
	p.insert("tokenSymbol".into(), constants::MANTA_TOKEN_SYMBOL.into());
	p
}

// manta-pc chain spec
#[cfg(feature = "manta-pc")]
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

#[cfg(feature = "manta-pc")]
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

#[cfg(feature = "manta-pc")]
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

#[cfg(feature = "manta-pc")]
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

#[cfg(feature = "manta-pc")]
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

// calamari chain specs
#[cfg(feature = "calamari")]
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

#[cfg(feature = "calamari")]
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

#[cfg(feature = "calamari")]
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

#[cfg(feature = "calamari")]
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

#[cfg(feature = "calamari")]
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

#[cfg(feature = "calamari")]
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

#[cfg(feature = "calamari")]
fn calamari_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	id: ParaId,
) -> calamari_runtime::GenesisConfig {
	// collator stake
	let collator_stake = 20_000 * MA;

	let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.cloned()
		.map(|x| (x.0, collator_stake))
		.collect();

	initial_balances.push((
		root_key.clone(),
		10_000_000_000 * MA - collator_stake * (initial_authorities.len() as u128),
	));

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
			candidacy_bond: MA * 10_000, // How many tokens will be reserved as collator
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

#[cfg(feature = "calamari")]
pub fn calamari_config(id: ParaId) -> CalamariChainSpec {
	let properties = calamari_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			// collator 1
			// Account ID: dmxjZSec4Xj3xz3nBEwSHjQSnRGhvcoB4eRabkiw7pSDuv8fW
			hex!["a80a38004dcea90dd0b91c5194ad577fb7b19517ea97e2ad263fcc5df7f57e06"].into(),
			hex!["7c4e5ea112f816c85f2bc77383cca50b73af038e327d39dca94252a4553b897e"]
				.unchecked_into(),
		),
		(
			// collator 2
			// Account ID: dmu63DLez715hRyhzdigz6akxS2c9W6RQvrToUWuQ1hntcBwF
			hex!["06b7ad4ce692a1653f7e2943b05c466c76c083238837af9a69ccba80185d2e6a"].into(),
			hex!["e6cd4aa48cfb4638c90b2b4965e28f6f0eabdc261c545a31917243ad7c45d633"]
				.unchecked_into(),
		),
		(
			// collator 3
			// Account ID: dmxvivs72h11DBNyKbeF8KQvcksoZsK9uejLpaWygFHZ2fU9z
			hex!["b08dda3edc4405b4283e0e3ee7a4eddf850ccb01cda1b5716a21e033f47e7912"].into(),
			hex!["ba3ca0dcf9e7515da2ad6ad37aba358ac8dfc727d791f6607d5779f934323859"]
				.unchecked_into(),
		),
		(
			// collator 4
			// Account ID: dmyhGnuox8ny9R1efVsWKxNU2FevMxcPZaB66uEJqJhgC4a1W
			hex!["d287e909d2ac9ad80917aa96c49130890b0cbe025c8613aceb414c9d78836a22"].into(),
			hex!["ae06f5c31189ad71a94c3dee0e462619694db71821467dbe3d49ab06319add18"]
				.unchecked_into(),
		),
		(
			// collator 5
			// Account ID: dmzbLejekGYZmfo5FoSznv5bBik7vGowuLxvzqFs2gZo2kANh
			hex!["fa3da97c21b48c74aec68124ea2102691fe44ef9aed1dd206a06fe21925c2024"].into(),
			hex!["6efd8d34a7139069ff8eb2cfe94af804a74f3084db80b6d052c0b5e300e78602"]
				.unchecked_into(),
		),
	];

	let root_key: AccountId =
		// sudo account: 
		// Account ID: dmv5qjXCqUwesFY56U9AyVsa2We7D55vYnkd5kBTdkiMyAWaF
		hex!["32cd443cce01db659930f0391edde50dac2e511b12301bd40736c68b8a241717"].into();

	CalamariChainSpec::from_genesis(
		// Name
		"Calamari Parachain",
		// ID
		"calamari",
		ChainType::Live,
		move || calamari_genesis(initial_authorities.clone(), root_key.clone(), id),
		vec![],
		Some(
			sc_telemetry::TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Calamari testnet telemetry url is valid; qed"),
		),
		Some(CALAMARI_PROTOCOL_ID),
		Some(properties),
		Extensions {
			relay_chain: KUSAMA_RELAYCHAIN_MAIN_NET.into(),
			para_id: id.into(),
		},
	)
}
