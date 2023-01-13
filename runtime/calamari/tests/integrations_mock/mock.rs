// Copyright 2020-2023 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(test)]

use crate::integrations_mock::*;

use calamari_runtime::opaque::SessionKeys;
pub use calamari_runtime::{
    assets_config::CalamariAssetConfig, currency::KMA, AuthorInherent, Call, CollatorSelection,
    Democracy, InflationInfo, Range, Runtime, Scheduler, Session, System, TransactionPayment,
};
use frame_support::traits::{GenesisBuild, OnFinalize, OnInitialize};
use manta_primitives::{
    assets::AssetConfig,
    types::{AccountId, Balance},
};
use sp_arithmetic::Perbill;
pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
    authorities: Vec<(AccountId, SessionKeys)>,
    invulnerables: Vec<AccountId>,
    // [collator, amount]
    collators: Vec<(AccountId, Balance)>,
    // [delegator, collator, delegation_amount]
    delegations: Vec<(AccountId, AccountId, Balance)>,
    // inflation config
    inflation: InflationInfo<Balance>,
    desired_candidates: u32,
    safe_xcm_version: Option<u32>,
}

impl Default for ExtBuilder {
    fn default() -> ExtBuilder {
        ExtBuilder {
            balances: vec![(ALICE.clone(), INITIAL_BALANCE)],
            authorities: vec![(ALICE.clone(), ALICE_SESSION_KEYS.clone())],
            invulnerables: vec![ALICE.clone()],
            collators: vec![],
            delegations: vec![],
            inflation: InflationInfo {
                expect: Range {
                    min: 700,
                    ideal: 700,
                    max: 700,
                },
                // not used
                annual: Range {
                    min: Perbill::from_percent(50),
                    ideal: Perbill::from_percent(50),
                    max: Perbill::from_percent(50),
                },
                // unrealistically high parameterization, only for testing
                round: Range {
                    min: Perbill::from_percent(5),
                    ideal: Perbill::from_percent(5),
                    max: Perbill::from_percent(5),
                },
            },
            safe_xcm_version: None,
            desired_candidates: 2,
        }
    }
}

impl ExtBuilder {
    pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub fn with_authorities(mut self, authorities: Vec<(AccountId, SessionKeys)>) -> Self {
        self.authorities = authorities;
        self
    }

    pub fn with_invulnerables(mut self, invulnerables: Vec<AccountId>) -> Self {
        self.invulnerables = invulnerables;
        self
    }

    pub fn with_collators(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
        self.collators = collators;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_delegations(
        mut self,
        delegations: Vec<(AccountId, AccountId, Balance)>,
    ) -> Self {
        self.delegations = delegations;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
        self.inflation = inflation;
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self.balances,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        manta_collator_selection::GenesisConfig::<Runtime> {
            invulnerables: self.invulnerables,
            eviction_baseline: manta_collator_selection::GenesisConfig::<Runtime>::default()
                .eviction_baseline,
            eviction_tolerance: manta_collator_selection::GenesisConfig::<Runtime>::default()
                .eviction_tolerance,
            candidacy_bond: EARLY_COLLATOR_MINIMUM_STAKE,
            desired_candidates: self.desired_candidates,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_parachain_staking::GenesisConfig::<Runtime> {
            candidates: self.collators,
            delegations: self.delegations,
            inflation_config: self.inflation,
        }
        .assimilate_storage(&mut t)
        .unwrap();
        pallet_session::GenesisConfig::<Runtime> {
            keys: self
                .authorities
                .iter()
                .cloned()
                .map(|(acc, session_keys)| {
                    (
                        acc.clone(),  // account id
                        acc,          // validator id
                        session_keys, // session keys
                    )
                })
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_asset_manager::GenesisConfig::<Runtime> {
            start_id: <CalamariAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        <pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_xcm::GenesisConfig {
                safe_xcm_version: self.safe_xcm_version,
            },
            &mut t,
        )
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);

        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}

/// Utility function that advances the chain to the desired block number.
pub fn run_to_block(n: u32) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        AuthorInherent::on_initialize(System::block_number());
        Session::on_initialize(System::block_number());
        ParachainStaking::on_initialize(System::block_number());
        Scheduler::on_initialize(System::block_number());
        Democracy::on_initialize(System::block_number());
    }
}
