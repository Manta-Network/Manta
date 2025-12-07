//! Genesis configuration presets for the Chameleon solochain runtime.

use crate::{AccountId, Balance, RuntimeGenesisConfig, EXISTENTIAL_DEPOSIT};
use alloc::{vec, vec::Vec};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::PresetId;
use sp_keyring::AccountKeyring;

/// Development genesis preset.
pub fn development_config_genesis() -> serde_json::Value {
    let endowed_accounts = vec![
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
        AccountKeyring::Charlie.to_account_id(),
        AccountKeyring::Dave.to_account_id(),
        AccountKeyring::Eve.to_account_id(),
        AccountKeyring::Ferdie.to_account_id(),
    ];

    let initial_authorities: Vec<(AuraId, GrandpaId)> = vec![(
        sp_keyring::Sr25519Keyring::Alice.public().into(),
        sp_keyring::Ed25519Keyring::Alice.public().into(),
    )];

    testnet_genesis(initial_authorities, AccountKeyring::Alice.to_account_id(), endowed_accounts)
}

/// Local testnet genesis preset.
pub fn local_testnet_genesis() -> serde_json::Value {
    let endowed_accounts = vec![
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
        AccountKeyring::Charlie.to_account_id(),
        AccountKeyring::Dave.to_account_id(),
        AccountKeyring::Eve.to_account_id(),
        AccountKeyring::Ferdie.to_account_id(),
    ];

    let initial_authorities: Vec<(AuraId, GrandpaId)> = vec![
        (
            sp_keyring::Sr25519Keyring::Alice.public().into(),
            sp_keyring::Ed25519Keyring::Alice.public().into(),
        ),
        (
            sp_keyring::Sr25519Keyring::Bob.public().into(),
            sp_keyring::Ed25519Keyring::Bob.public().into(),
        ),
    ];

    testnet_genesis(initial_authorities, AccountKeyring::Alice.to_account_id(), endowed_accounts)
}

/// Helper function to create genesis configuration.
fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> serde_json::Value {
    let config = RuntimeGenesisConfig {
        system: Default::default(),
        balances: pallet_balances::GenesisConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1_000_000_000_000_000_000u128)) // 1000 units
                .collect(),
        },
        aura: pallet_aura::GenesisConfig {
            authorities: initial_authorities.iter().map(|x| x.0.clone()).collect(),
        },
        grandpa: pallet_grandpa::GenesisConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
            ..Default::default()
        },
        sudo: pallet_sudo::GenesisConfig { key: Some(root_key) },
        transaction_payment: Default::default(),
    };

    serde_json::to_value(config).expect("Could not serialize genesis config")
}

/// Get preset by id.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_testnet_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&patch)
            .expect("serialization to json is expected to work")
            .into_bytes(),
    )
}

/// Get list of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}
