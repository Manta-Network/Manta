use super::*;

use calamari_runtime::{CouncilConfig, DemocracyConfig, GenesisConfig, TechnicalCommitteeConfig};

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
	let num_endowed_accounts = endowed_accounts.len();

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
		democracy: DemocracyConfig::default(),
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		council_membership: Default::default(),
		technical_membership: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}

pub fn calamari_testnet_config(id: ParaId) -> Result<CalamariChainSpec, String> {
	let mut spec = CalamariChainSpec::from_json_bytes(
		&include_bytes!("../../../genesis/calamari-testnet-genesis.json")[..],
	)?;
	spec.extensions_mut().para_id = id.into();
	Ok(spec)
}

// Calamari testnet for ci jobs
#[cfg(feature = "calamari")]
pub fn calamari_testnet_ci_config() -> Result<CalamariChainSpec, String> {
	CalamariChainSpec::from_json_bytes(
		&include_bytes!("../../../genesis/calamari-testnet-ci-genesis.json")[..],
	)
}

// Calamari mainnet
pub fn calamari_config() -> Result<CalamariChainSpec, String> {
	CalamariChainSpec::from_json_bytes(
		&include_bytes!("../../../genesis/calamari-genesis.json")[..],
	)
}
