// Copyright 2020-2024 Chameleon Network.
// SPDX-License-Identifier: GPL-3.0

//! Chain specification for Chameleon Network.
//!
//! This module defines the genesis configuration for different network types.

use chameleon_runtime::{
    AccountId, AuraConfig, BalancesConfig, GrandpaConfig, RuntimeGenesisConfig,
    Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for Chameleon network.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}" , seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Development config (single validator Alice)
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::builder(wasm_binary, None)
        .with_name("Chameleon Development")
        .with_id("chameleon_dev")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_patch(testnet_genesis(
            // Initial authorities
            vec![authority_keys_from_seed("Alice")],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            // Pre-funded accounts
            vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            ],
            true,
        ))
        .build())
}

/// Local testnet config (5 validators)
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::builder(wasm_binary, None)
        .with_name("Chameleon Local Testnet")
        .with_id("chameleon_local")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_patch(testnet_genesis(
            // Initial authorities (5 validators)
            vec![
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
                authority_keys_from_seed("Charlie"),
                authority_keys_from_seed("Dave"),
                authority_keys_from_seed("Eve"),
            ],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            // Pre-funded accounts  
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
            ],
            true,
        ))
        .build())
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> serde_json::Value {
    use chameleon_runtime::UNIT;
    
    // Total supply: 100,000,000 CHML
    // Each endowed account gets equal share for dev/testing
    let endowment = 10_000_000 * UNIT; // 10M CHML each for testing
    
    serde_json::json!({
        "balances": {
            "balances": endowed_accounts.iter().cloned().map(|k| (k, endowment)).collect::<Vec<_>>(),
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key),
        },
    })
}
