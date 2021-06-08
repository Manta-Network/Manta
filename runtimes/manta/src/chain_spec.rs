use hex_literal::hex;
use manta_primitives::{constants::currency::MA, AccountId, Balance, Signature};
use manta_runtime::{
	wasm_binary_unwrap, BabeConfig, BalancesConfig, CouncilConfig, GenesisConfig, GrandpaConfig,
	SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
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

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

fn session_keys(grandpa: GrandpaId, babe: BabeId) -> manta_runtime::opaque::SessionKeys {
	manta_runtime::opaque::SessionKeys { babe, grandpa }
}

#[allow(dead_code)]
fn staging_testnet_config_genesis() -> GenesisConfig {
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	// and
	// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)> = vec![
		(
			// 5Fbsd6WXDGiLTxunqeK5BATNiocfCqu9bS1yArVjCgeBLkVy
			hex!["9c7a2ee14e565db0c69f78c7b4cd839fbf52b607d867e9e9c5a79042898a0d12"].into(),
			// 5EnCiV7wSHeNhjW3FSUwiJNkcc2SBkPLn5Nj93FmbLtBjQUq
			hex!["781ead1e2fa9ccb74b44c19d29cb2a7a4b5be3972927ae98cd3877523976a276"].into(),
			// 5Fb9ayurnxnaXj56CjmyQLBiadfRCqUbL2VWNbbe1nZU6wiC
			hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"]
				.unchecked_into(),
			// 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
			hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
				.unchecked_into(),
		),
		(
			// 5ERawXCzCWkjVq3xz1W5KGNtVx2VdefvZ62Bw1FEuZW4Vny2
			hex!["68655684472b743e456907b398d3a44c113f189e56d1bbfd55e889e295dfde78"].into(),
			// 5Gc4vr42hH1uDZc93Nayk5G7i687bAQdHHc9unLuyeawHipF
			hex!["c8dc79e36b29395413399edaec3e20fcca7205fb19776ed8ddb25d6f427ec40e"].into(),
			// 5EockCXN6YkiNCDjpqqnbcqd4ad35nU4RmA1ikM4YeRN4WcE
			hex!["7932cff431e748892fa48e10c63c17d30f80ca42e4de3921e641249cd7fa3c2f"]
				.unchecked_into(),
			// 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
			hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
				.unchecked_into(),
		),
		(
			// 5DyVtKWPidondEu8iHZgi6Ffv9yrJJ1NDNLom3X9cTDi98qp
			hex!["547ff0ab649283a7ae01dbc2eb73932eba2fb09075e9485ff369082a2ff38d65"].into(),
			// 5FeD54vGVNpFX3PndHPXJ2MDakc462vBCD5mgtWRnWYCpZU9
			hex!["9e42241d7cd91d001773b0b616d523dd80e13c6c2cab860b1234ef1b9ffc1526"].into(),
			// 5E1jLYfLdUQKrFrtqoKgFrRvxM3oQPMbf6DfcsrugZZ5Bn8d
			hex!["5633b70b80a6c8bb16270f82cca6d56b27ed7b76c8fd5af2986a25a4788ce440"]
				.unchecked_into(),
			// 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
			hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
				.unchecked_into(),
		),
		(
			// 5HYZnKWe5FVZQ33ZRJK1rG3WaLMztxWrrNDb1JRwaHHVWyP9
			hex!["f26cdb14b5aec7b2789fd5ca80f979cef3761897ae1f37ffb3e154cbcc1c2663"].into(),
			// 5EPQdAQ39WQNLCRjWsCk5jErsCitHiY5ZmjfWzzbXDoAoYbn
			hex!["66bc1e5d275da50b72b15de072a2468a5ad414919ca9054d2695767cf650012f"].into(),
			// 5DMa31Hd5u1dwoRKgC4uvqyrdK45RHv3CpwvpUC1EzuwDit4
			hex!["3919132b851ef0fd2dae42a7e734fe547af5a6b809006100f48944d7fae8e8ef"]
				.unchecked_into(),
			// 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
			hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
				.unchecked_into(),
		),
	];

	// generated with secret: subkey inspect "$secret"
	let root_key: AccountId = hex![
		// 5Ff3iXP75ruzroPWRP2FYBHWnmGGBSb63857BgnzCoXNxfPo
		"9ee5e5bdc0ec239eb164f865ecc345ce4c88e76ee002e0f7e318097347471809"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, root_key, Some(endowed_accounts), false)
}

/// Staging testnet config.
#[allow(dead_code)]
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		Default::default(),
	)
}

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
	p.insert("tokenSymbol".into(), "MA".into());
	p
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	_enable_println: bool,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
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
		]
	});
	initial_authorities.iter().for_each(|x| {
		if !endowed_accounts.contains(&x.0) {
			endowed_accounts.push(x.0.clone())
		}
	});

	const ENDOWMENT: Balance = 100_000_000 * MA; // 10 endowment so that total supply is 1B
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|x| (x, ENDOWMENT))
				.collect(),
		}),
		pallet_session: Some(SessionConfig {
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
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_collective_Instance1: Some(CouncilConfig::default()),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		true,
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
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		false,
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
pub fn manta_testnet_config() -> ChainSpec {
	let protocol_id = Some("manta");

	ChainSpec::from_genesis(
		"Manta Testnet",
		"manta_testnet",
		ChainType::Custom("Manta Testnet".into()),
		manta_testnet_genesis,
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
pub fn manta_testnet_config_genesis(
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	initial_balances: Vec<(AccountId, Balance)>,
	root_key: AccountId,
	stash: Balance,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: initial_balances,
		}),
		pallet_session: Some(SessionConfig {
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
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), stash, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_collective_Instance1: Some(CouncilConfig::default()),
		pallet_sudo: Some(SudoConfig { key: root_key }), // we do sudo right now, this will be removed after full decentralization
		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
	}
}

/// Manta testnet genesis
pub fn manta_testnet_genesis() -> GenesisConfig {
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

	manta_testnet_config_genesis(
		initial_authorities,
		initial_balances,
		root_key,
		STASH,
		false,
	)
}

/// a single node dev testnet genesis
pub fn manta_local_dev_genesis() -> GenesisConfig {
	let stash_account: AccountId = hex![
		// 5EcVwmgGB8GTduy53PpsGBpsEEZAGEWYBeLuwSz76kxUzJid
		"70b8386b105ab594513031ed15cb9226e7db0ac285cccbcee59e55eae1e4922c"
	]
	.into();
	let controller_account: AccountId = hex![
		// 5HGdhHoyvPXkmXwNYg2vcNTU9584AfN7EsFM8DySKS53Ehxg
		"e6461a44f71ac6c43bc6c9df20310211fb3d600ad0f3a51a66e7959caa599e6f"
	]
	.into();
	let root_key: AccountId = hex![
		//root account: 5DrGMpT3dYm8cWPp6ZDbdkKVfYzAdWBGZjkrtcE5GTqS9EC1
		"4efbb0ab7942a237b3ce5b2540a0faad8cda8eeef44da6e4a614b3d8c08c0823"
	]
	.into();

	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)> = vec![(
		stash_account.clone(),
		controller_account.clone(),
		// Grandpa ID: 5DCj1vKWeHWdni8LwtKVaCuh8vq4HvBUDKAJja5S7BW6M3ho
		hex!["325a1995421793437bffa10eef55b028e61a02354e6ec66ab58b075349f6e9ca"].unchecked_into(),
		// Babe ID: 5DAA4avV1euhv9gkNfa4bGsjZRaYTHXVa8t6F6yunAmauR7v
		hex!["3064ad09d3fb2dd412aeaadf150bd6646ff2ed889e9bcea4068be8f9c2b65657"].unchecked_into(),
	)];

	let initial_balances = vec![
		(root_key.clone(), 1000 * MA),
		(stash_account, 1_000_000_000 * MA),
		(controller_account, 20_000_000 * MA),
	];

	manta_testnet_config_genesis(
		initial_authorities,
		initial_balances,
		root_key,
		100_000_000 * MA,
		false,
	)
}

/// Manta testnet dev config
pub fn manta_local_dev_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Manta local dev",
		"manta_local_dev",
		ChainType::Custom("Manta Local Dev".into()),
		manta_local_dev_genesis,
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Manta testnet telemetry url is valid; qed"),
		),
		Some("manta_local_dev"),
		Some(manta_properties()),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use sp_runtime::BuildStorage;

	#[allow(dead_code)]
	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			false,
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

	//TODO: see whether we can recover this test
	// #[test]
	// #[ignore]
	// fn test_connectivity() {
	// 	sc_service_test::connectivity(
	// 		integration_test_config_with_two_authorities(),
	// 		|config| {
	// 			let NewFullBase { task_manager, client, network, transaction_pool, .. }
	// 				= new_full_base(config,|_, _| ())?;
	// 			Ok(sc_service_test::TestNetComponents::new(task_manager, client, network, transaction_pool))
	// 		},
	// 		|config| {
	// 			let (keep_alive, _, _, client, network, transaction_pool) = new_light_base(config)?;
	// 			Ok(sc_service_test::TestNetComponents::new(keep_alive, client, network, transaction_pool))
	// 		}
	// 	);
	// }

	#[test]
	fn test_create_development_chain_spec() {
		assert!(development_config().build_storage().is_ok());
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		assert!(local_testnet_config().build_storage().is_ok());
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		assert!(staging_testnet_config().build_storage().is_ok());
	}

	#[test]
	fn test_manta_testnet_chain_spec() {
		assert!(manta_testnet_config().build_storage().is_ok());
	}

	#[test]
	fn test_manta_local_dev_genesis() {
		assert!(manta_local_dev_genesis().build_storage().is_ok());
	}

	#[test]
	fn test_manta_local_dev_config() {
		assert!(manta_local_dev_config().build_storage().is_ok());
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
