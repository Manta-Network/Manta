use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use manta_primitives::{
	constants::{MANTAPC_SS58PREFIX, MANTA_DECIMAL, MANTA_TOKEN_SYMBOL},
	currency::MA,
	AccountId, AuraId, Balance, Signature,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<manta_pc_runtime::GenesisConfig, Extensions>;

const ENDOWMENT: Balance = 100_000_000 * MA; // 10 endowment so that total supply is 1B
const MANTAPC_PROTOCOL_ID: &str = "manta-pc"; // for p2p network configuration
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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
pub fn manta_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), MANTAPC_SS58PREFIX.into());
	p.insert("tokenDecimals".into(), MANTA_DECIMAL.into());
	p.insert("tokenSymbol".into(), MANTA_TOKEN_SYMBOL.into());
	p
}

pub fn manta_pc_development_config(id: ParaId) -> ChainSpec {
	let properties = manta_properties();

	ChainSpec::from_genesis(
		// Name
		"Manta-Parachain Development",
		// ID
		"manta_pc_dev",
		ChainType::Local,
		move || {
			testnet_genesis(
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
			relay_chain: "polkadot-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn manta_pc_local_config(id: ParaId) -> ChainSpec {
	let properties = manta_properties();

	ChainSpec::from_genesis(
		// Name
		"Manta-Parachain Local",
		// ID
		"manta_pc_local",
		ChainType::Local,
		move || {
			testnet_genesis(
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
			relay_chain: "polkadot-local".into(),
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> manta_pc_runtime::GenesisConfig {
	manta_pc_runtime::GenesisConfig {
		frame_system: manta_pc_runtime::SystemConfig {
			code: manta_pc_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: manta_pc_runtime::BalancesConfig {
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
		pallet_aura: manta_pc_runtime::AuraConfig {
			authorities: invulnerables.iter().map(|v| v.1.clone()).collect(),
		},
		pallet_sudo: manta_pc_runtime::SudoConfig { key: root_key },
		parachain_info: manta_pc_runtime::ParachainInfoConfig { parachain_id: id },
		cumulus_pallet_aura_ext: Default::default(),
	}
}

pub fn manta_pc_testnet_config(id: ParaId) -> ChainSpec {
	let properties = manta_properties();

	// (controller_account, aura_id)
	let initial_authorities: Vec<(AccountId, AuraId)> = vec![
		(
			hex!["16b77c266c577ad605bec26cd2421a9b405d102bd54663c5f242454e0de81376"].into(),
			hex!["7a40f6773ffa7d13147daa0f8cf7e5ea5b54a14fb515ccded35ea7df7ce2c26a"]
				.unchecked_into(),
		),
		(
			hex!["c233dbba1667da231e1091fdd99e1ead60270c836ee809521b40a5c89cde497c"].into(),
			hex!["0e66f3b49250bced29cff1d717b944f4f57e5ced096e4b6aeeb7d5206d7b1d0e"]
				.unchecked_into(),
		),
		(
			hex!["088eb36dcb104076d56705d27c7fe94db3f32a399d48a21ac4b1470a231c0a54"].into(),
			hex!["f08346ce33e5c8c29d0fcb7aa70db75964d763f0537777ef9d5f0091fe3d371c"]
				.unchecked_into(),
		),
		(
			hex!["ccc16c960eed8939a66043b7a26d97f7363ac862b50bf50a8ecceff4a6f1d44a"].into(),
			hex!["4e4277d721cfed60407222cb7e47701a60597d7b598cda5d0ac38fc29dab8d72"]
				.unchecked_into(),
		),
		(
			hex!["fe66a8f15b1c29b69fdb246a7368316192db12b98fca934a6f1e4c5863a2885c"].into(),
			hex!["a272940a6d11b48f691225841e168d0f16c8101cc034f115298c4aa53c2a5d6f"]
				.unchecked_into(),
		),
	];

	let root_key: AccountId =
		hex!["7200ed745a32b3843eed5889b48185dca0519412b673d1650a0986ac361ffd32"].into();

	ChainSpec::from_genesis(
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
			relay_chain: "polkadot-dev".into(),
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
		frame_system: manta_pc_runtime::SystemConfig {
			code: manta_pc_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: manta_pc_runtime::BalancesConfig {
			balances: initial_balances,
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		pallet_aura: manta_pc_runtime::AuraConfig {
			authorities: initial_authorities.iter().map(|v| v.1.clone()).collect(),
		},
		pallet_sudo: manta_pc_runtime::SudoConfig { key: root_key },
		parachain_info: manta_pc_runtime::ParachainInfoConfig { parachain_id: id },
		cumulus_pallet_aura_ext: Default::default(),
	}
}
