// Copyright 2020-2022 Manta Network.
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
    assets_config::CalamariAssetConfig, currency::KMA, Call, CollatorSelection, Democracy, Runtime,
    Scheduler, Session, System, TransactionPayment,
};
use frame_support::traits::{GenesisBuild, OnFinalize, OnInitialize};
use manta_primitives::{
    assets::AssetConfig,
    types::{AccountId, Balance},
};
use session_key_primitives::util::{unchecked_account_id, unchecked_collator_keys};
use sp_core::sr25519;
pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
    authorities: Vec<(AccountId, SessionKeys)>,
    invulnerables: Vec<AccountId>,
    desired_candidates: u32,
    safe_xcm_version: Option<u32>,
}
use sp_std::marker::PhantomData;

impl Default for ExtBuilder {
    fn default() -> ExtBuilder {
        ExtBuilder {
            balances: vec![(
                unchecked_account_id::<sr25519::Public>("Alice"),
                INITIAL_BALANCE,
            )],
            authorities: vec![(
                unchecked_account_id::<sr25519::Public>("Alice"),
                SessionKeys::new(unchecked_collator_keys("Alice")),
            )],
            invulnerables: vec![unchecked_account_id::<sr25519::Public>("Alice")],
            safe_xcm_version: None,
            desired_candidates: 1,
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

    pub fn with_collators(
        mut self,
        invulnerables: Vec<AccountId>,
        desired_candidates: u32,
    ) -> Self {
        self.invulnerables = invulnerables;
        self.desired_candidates = desired_candidates;
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
            candidacy_bond: BOND_AMOUNT,
            desired_candidates: self.desired_candidates,
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
        Session::on_initialize(System::block_number());
        Scheduler::on_initialize(System::block_number());
        Democracy::on_initialize(System::block_number());
    }
}
