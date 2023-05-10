use super::*;

pub(crate) fn calamari_dev_genesis(
    invulnerables: Vec<(AccountId, SessionKeys)>,
    delegations: Vec<(AccountId, AccountId, Balance)>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: calamari_runtime::SystemConfig {
            code: calamari_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: calamari_runtime::BalancesConfig {
            balances: endowed_accounts[..endowed_accounts.len() / 2]
                .iter()
                .map(|k| {
                    (
                        k.clone(),
                        100 * CALAMARI_ENDOWMENT / ((endowed_accounts.len() / 2) as Balance),
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        parachain_staking: ParachainStakingConfig {
            candidates: invulnerables
                .iter()
                .cloned()
                .map(|(account, _)| {
                    (
                        account,
                        calamari_runtime::staking::NORMAL_COLLATOR_MINIMUM_STAKE,
                    )
                })
                .collect(),
            delegations,
            inflation_config: calamari_runtime::staking::inflation_config::<
                calamari_runtime::Runtime,
            >(),
        },
        parachain_info: calamari_runtime::ParachainInfoConfig {
            parachain_id: CALAMARI_PARACHAIN_ID.into(),
        },
        collator_selection: calamari_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: 400_000 * KMA, // How many tokens will be reserved as collator
            ..Default::default()
        },
        session: calamari_runtime::SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, session_keys)| {
                    (
                        acc.clone(),  // account id
                        acc,          // validator id
                        session_keys, // collator session keys
                    )
                })
                .collect(),
        },
        democracy: DemocracyConfig::default(),
        council: CouncilConfig {
            members: endowed_accounts.iter().take(1).cloned().collect(),
            phantom: Default::default(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts.iter().take(1).cloned().collect(),
            phantom: Default::default(),
        },
        council_membership: Default::default(),
        technical_membership: Default::default(),
        asset_manager: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: calamari_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
        lottery: LotteryConfig {
            min_deposit: 5_000 * KMA,
            min_withdraw: 1 * KMA,
            gas_reserve: 10_000 * KMA,
        },
    }
}
