use cumulus_primitives_core::ParaId;
use manta_primitives::{AccountId, Signature, AuraId, Balance};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<manta_pc_runtime::GenesisConfig, Extensions>;

const MANTAPC_ED: Balance = manta_pc_runtime::NativeTokenExistentialDeposit::get();

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

pub fn manta_pc_development_config(id: ParaId) -> ChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "MA".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		// Name
		"Manta-Parachain Development",
		// ID
		"manta_pc_dev",
		ChainType::Local,
		move || {
			manta_pc_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					)
				],
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
		None,
		Some(properties),
		Extensions {
			relay_chain: "polkadot-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn manta_pc_local_config(id: ParaId) -> ChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "MA".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		// Name
		"Manta-Parachain Local",
		// ID
		"manta_pc_local",
		ChainType::Local,
		move || {
			manta_pc_genesis(
				// initial collators.
				vec![(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob")
					),
				],
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
				],
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "polkadot-local".into(),
			para_id: id.into(),
		},
	)
}

fn manta_pc_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
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
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, MANTAPC_ED * 4096))
				.collect(),
		},
		parachain_info: manta_pc_runtime::ParachainInfoConfig { parachain_id: id },
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		pallet_aura: manta_pc_runtime::AuraConfig {
			authorities: invulnerables.iter().map(|v| v.1.clone()).collect(),
		},
		cumulus_pallet_aura_ext: Default::default(),
	}
}
