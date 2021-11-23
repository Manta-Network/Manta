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
	// GLMR(Moonbeam), 1B, decimal 18
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
			"/dns/n1.testnet.manta.network/tcp/30333/p2p/12D3KooWNXxthQYVaXA1yDD7q6VfDK9iud7t6aUAmyyeabeT5PJ9".parse().expect("failed to parse multiaddress."),
			"/dns/n2.testnet.manta.network/tcp/30333/p2p/12D3KooWBKvCn32z1RZzWuM84VzzK1JubokjLKfaeq9fVJMWnZje".parse().expect("failed to parse multiaddress."),
			"/dns/n3.testnet.manta.network/tcp/30333/p2p/12D3KooWS3BxpSPeDU59KmocL1RMdK4wucw34abfyayUt9kY6vg1".parse().expect("failed to parse multiaddress."),
			"/dns/n4.testnet.manta.network/tcp/30333/p2p/12D3KooWDT6En2QzURmh7FDZjkAXVsk6Qy68MYqVhVRMrsTq8oF4".parse().expect("failed to parse multiaddress."),
			"/dns/n5.testnet.manta.network/tcp/30333/p2p/12D3KooWQQgTNm1jeQFekLHsqU9BrR8wrVTNkhdCvwbB3hNA5eSZ".parse().expect("failed to parse multiaddress."),
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
			owner: hex!["fe25242cf9cb6239520319037206cae7b0cd2c2be218c0d2e5c26f9d86d41f0e"].into(),
			assets: TESTNET_ASSETS.to_vec(),
		},
	}
}

/// Manta testnet genesis
pub fn dolphin_testnet_genesis() -> GenesisConfig {
	// (stash_account, controller_account, grandpa_id, babe_id)
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)> = vec![
		(
			hex!["10d9b9a1d7ab94c2ee0cd003e5a29a3a7d686c07bfafe8839484980023557202"].into(),
			hex!["3c6fbc3d0b8931885d93ed15af934a5fe19d167995728b3ee0acb095e5b96d70"].into(),
			hex!["099795fb88bbb7c37f06237109e013c84bbaa4f59a4e094bdff1212193b4e680"]
				.unchecked_into(),
			hex!["947569714b326872b281b368ea5e3251df33beb7c78246d18aeb5fa90f0f456a"]
				.unchecked_into(),
		),
		(
			hex!["3223497291b5e36ed927d48f216085fdbff66cea340229366a7b392651e25a60"].into(),
			hex!["42b48222eb545172e64f54e2158258ea5d4136ad0b343366f817c278013c5e0c"].into(),
			hex!["dba8d8d14157c94f08fff43f3f4a61f47ddcb59e2887cc90a73474dd32e4f839"]
				.unchecked_into(),
			hex!["fa9318d4dfffd8eab186fb9f6102db06ee8482b03a8292f40de0226c3d12167e"]
				.unchecked_into(),
		),
		(
			hex!["0a005d4883ac8d869e4fd4b23d71d6c76252c02c98aeefeabf59005aa2c2130f"].into(),
			hex!["0c8c1b8a94d5c549cc63ce4aedfc5b018ae09ab7cfefc02e958ef900e55ac240"].into(),
			hex!["a9a1faa6cf8d1ca76cc32ca057469f22d0481dbbd5dbb33a6e6981f7c9f562c5"]
				.unchecked_into(),
			hex!["5201be9c473b98c043a47bee3c4a07b06d748068a838f59b31b0e636a617b811"]
				.unchecked_into(),
		),
		(
			hex!["d842a1aecdde6e437fa89afbeb26479343c8220573f24eb0ae767c75a9d41723"].into(),
			hex!["9eccf7214c8ac17b57205a0cf8afe9012e0d79e7b89234722541274e104c7464"].into(),
			hex!["a6c42d155cf067f83ab01064591d3f92dd39c4671be51d4f8a9ae0be96d0f6f5"]
				.unchecked_into(),
			hex!["183fae74e24e19001f131b9f0105b058da622ce458f8500cb7aa07b006c71810"]
				.unchecked_into(),
		),
		(
			hex!["827a761c6f4a579f3caa1c5e59ddf290aeda35435df50162d2682c494918610c"].into(),
			hex!["8c9b0fad7780e2c20a16d9160014558739b3006356279822a01c54f20908f660"].into(),
			hex!["f42ed04232657986efb42181bdbdddac1d820f5ed4963a0d4a059a2bf4258aa7"]
				.unchecked_into(),
			hex!["d26915a544c5ffc1ec6e60d814620d7bb1b0bc8271a017ae80354b239f6f4d03"]
				.unchecked_into(),
		),
	];

	let root_key: AccountId =
		hex!["fe25242cf9cb6239520319037206cae7b0cd2c2be218c0d2e5c26f9d86d41f0e"].into();

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
