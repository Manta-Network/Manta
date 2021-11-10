use hex_literal::hex;
use manta_primitives::{
	constants::currency::MA, AccountId, AssetBalance, AssetId, Balance, Signature,
};
use manta_runtime::{
	wasm_binary_unwrap, BabeConfig, BalancesConfig, Block, CouncilConfig, GenesisConfig,
	GrandpaConfig, MantaPayConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig, MAX_NOMINATIONS,
};
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

type AccountPublic = <Signature as Verify>::Signer;

// The URL for the telemetry server.
#[allow(dead_code)]
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
// #[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
// #[serde(rename_all = "camelCase")]
// pub struct Extensions {
// 	/// Block numbers with known hashes.
// 	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
// 	/// Known bad block hashes.
// 	pub bad_blocks: sc_client_api::BadBlocks<Block>,
// }

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn session_keys(grandpa: GrandpaId, babe: BabeId) -> manta_runtime::opaque::SessionKeys {
	manta_runtime::opaque::SessionKeys { babe, grandpa }
}

const TESTNET_ASSETS: &[(AssetId, AssetBalance)] = &[
	// DOT, 1.2B, decimal 10
    (0, 11_200_000_000_000_000_000),
	// KSM, 11.6M, decimal 12
    (1, 11_600_000_000_000_000_000),
	// BTC, 21M, decimal 8
    (2, 2_100_000_000_000_000),
	// ETH, 112M, decimal 18
    (3, 112_000_000_000_000_000_000_000_000),
	// ACA, 1B, decimal 18
    (4, 1_000_000_000_000_000_000_000_000_000),
	// // GLMR, 1B, decimal 18
    (5, 1_000_000_000_000_000_000_000_000_000),
];

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
	)
}

/// Token
pub fn manta_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), 77.into());
	p.insert("tokenDecimals".into(), 12.into());
	p.insert("tokenSymbol".into(), "DOL".into());
	p
}

/// Helper function to create GenesisConfig for testing
pub fn devnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(&x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MAX_NOMINATIONS as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(
				x.clone(),
				x.clone(),
				STASH,
				StakerStatus::Nominator(nominations),
			)
		}))
		.collect::<Vec<_>>();

	const ENDOWMENT: Balance = 10_000_000 * MA;
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|x| (x, ENDOWMENT))
				.collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		council: CouncilConfig::default(),
		sudo: SudoConfig { key: root_key },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(manta_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		manta_pay: MantaPayConfig {
			owner: get_account_id_from_seed::<sr25519::Public>("Alice"),
			assets: TESTNET_ASSETS.to_vec(),
		},
	}
}

fn development_config_genesis() -> GenesisConfig {
	devnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		None,
		Some(manta_properties()),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	devnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		Some(manta_properties()),
		Default::default(),
	)
}

/// Manta testnet config
pub fn dolphin_testnet_config() -> ChainSpec {
	let protocol_id = Some("dolphin");

	ChainSpec::from_genesis(
		"Dolphin Testnet",
		"dolphin_testnet",
		ChainType::Custom("Dolphin Testnet".into()),
		dolphin_testnet_genesis,
		vec![
			"/dns/n1.testnet.manta.network/tcp/30333/p2p/12D3KooWBV7qb2LshmqCr74edBk5h4Fi1Zt71fhpvdyi8ah3KzAa".parse().expect("failed to parse multiaddress."),
			"/dns/n2.testnet.manta.network/tcp/30333/p2p/12D3KooWBGhNQyzkKEpN7QQnP94BhM8wyhpJwsZ58wbr1r3Pi6gV".parse().expect("failed to parse multiaddress."),
			"/dns/n3.testnet.manta.network/tcp/30333/p2p/12D3KooWSBpCHCHi4jmwJTkdMmb7vWBBjPJnoGsRE1VwERMTgvVD".parse().expect("failed to parse multiaddress."),
			"/dns/n4.testnet.manta.network/tcp/30333/p2p/12D3KooWNkupfxbGwPLBhkXLV7c9P2cGHm8JbGadSXQG854F2nrM".parse().expect("failed to parse multiaddress."),
			"/dns/n5.testnet.manta.network/tcp/30333/p2p/12D3KooWBwTA8KdcBhRn6tJSRqQ8JTdxy1MKRHjHyjqcDcRcSuC6".parse().expect("failed to parse multiaddress."),
		],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Manta testnet telemetry url is valid; qed")),
		protocol_id,
		Some(manta_properties()),
		Default::default(),
	)
}

/// Helper function to create GenesisConfig for manta testnets
pub fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	initial_balances: Vec<(AccountId, Balance)>,
	root_key: AccountId,
	stash: Balance,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: initial_balances,
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), stash, StakerStatus::Validator))
				.collect(),
			..Default::default()
		},
		council: CouncilConfig::default(),
		sudo: SudoConfig { key: root_key },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(manta_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		manta_pay: MantaPayConfig {
			owner: hex!["12b73670c56f4fcd319636bdd6ec4a803ae2d06fdbc715087524a5151395d16c"].into(),
			assets: TESTNET_ASSETS.to_vec(),
		},
	}
}

/// Manta testnet genesis
pub fn dolphin_testnet_genesis() -> GenesisConfig {
	// (stash_account, controller_account, grandpa_id, babe_id)
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)> = vec![
		(
			hex!["16b77c266c577ad605bec26cd2421a9b405d102bd54663c5f242454e0de81376"].into(),
			hex!["7a40f6773ffa7d13147daa0f8cf7e5ea5b54a14fb515ccded35ea7df7ce2c26a"].into(),
			hex!["c5189d7881d966d8355c403a8b490267e1ca28b471d948f1a054f536fef0ecdc"]
				.unchecked_into(),
			hex!["a6da86747dce627b0a0cf4189ce35247a5c0c9a69570f2b5b72241beb711a141"]
				.unchecked_into(),
		),
		(
			hex!["c233dbba1667da231e1091fdd99e1ead60270c836ee809521b40a5c89cde497c"].into(),
			hex!["0e66f3b49250bced29cff1d717b944f4f57e5ced096e4b6aeeb7d5206d7b1d0e"].into(),
			hex!["6725d2323bc3e69d1017a47cefe70a4ee5760ffd4175852370c439132fe06916"]
				.unchecked_into(),
			hex!["c8ddaec483dfa0a580a7c8194ee625a6251743859070415aa7f8f384abd6c550"]
				.unchecked_into(),
		),
		(
			hex!["088eb36dcb104076d56705d27c7fe94db3f32a399d48a21ac4b1470a231c0a54"].into(),
			hex!["f08346ce33e5c8c29d0fcb7aa70db75964d763f0537777ef9d5f0091fe3d371c"].into(),
			hex!["06a368a12a24785b2be5f332ae51d947c49d2aac1d8b5804c25a1c47bb838272"]
				.unchecked_into(),
			hex!["6c14813c02fa0b9992560cae02337c748af2e46bb5a1b26b6011bde02f92f356"]
				.unchecked_into(),
		),
		(
			hex!["ccc16c960eed8939a66043b7a26d97f7363ac862b50bf50a8ecceff4a6f1d44a"].into(),
			hex!["4e4277d721cfed60407222cb7e47701a60597d7b598cda5d0ac38fc29dab8d72"].into(),
			hex!["290ed0c0ce03c67d598f31321fe77f79684ffe9cdb5824d02707dc21e1843823"]
				.unchecked_into(),
			hex!["966c68c4308b757bef26f21e4951cfd47e6a56ce6c68350dff5d3355bbf27749"]
				.unchecked_into(),
		),
		(
			hex!["fe66a8f15b1c29b69fdb246a7368316192db12b98fca934a6f1e4c5863a2885c"].into(),
			hex!["a272940a6d11b48f691225841e168d0f16c8101cc034f115298c4aa53c2a5d6f"].into(),
			hex!["d76c05af97a59a4a3bb8ccbe5811547e26bc185f3acf7b401ad0e40f17ac880b"]
				.unchecked_into(),
			hex!["2e6dba967ee6ca20655e92ee82954aed4d88975435a835b97973c270dfa67402"]
				.unchecked_into(),
		),
	];

	let root_key: AccountId =
		hex!["7200ed745a32b3843eed5889b48185dca0519412b673d1650a0986ac361ffd32"].into();

	const ENDOWMENT: Balance = 100_000_000 * MA; // 5 initial validators
	const STASH: Balance = ENDOWMENT / 2; // every initial validator use half of their tokens to stake

	let mut initial_balances: Vec<(AccountId, Balance)> = initial_authorities
		.iter()
		.cloned()
		.map(|x| (x.0, ENDOWMENT))
		.collect();

	initial_balances.push((root_key.clone(), 500_000_000 * MA)); // root_key get half of the stake

	testnet_genesis(
		initial_authorities,
		initial_balances,
		root_key,
		STASH,
		false,
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use sp_runtime::BuildStorage;

	#[allow(dead_code)]
	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		devnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	/// Local testnet config (single validator - Alice)
	#[allow(dead_code)]
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	#[allow(dead_code)]
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	fn test_create_development_chain_spec() {
		assert!(development_config().build_storage().is_ok());
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		assert!(local_testnet_config().build_storage().is_ok());
	}

	#[test]
	fn test_manta_testnet_chain_spec() {
		assert!(dolphin_testnet_config().build_storage().is_ok());
	}

	#[test]
	fn test_integration_config_with_single_authority() {
		assert!(integration_test_config_with_single_authority()
			.build_storage()
			.is_ok());
	}

	#[test]
	fn test_integration_config_with_two_authorities() {
		assert!(integration_test_config_with_two_authorities()
			.build_storage()
			.is_ok());
	}
}
