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

//! Calamari Parachain Integration Tests.

pub use calamari_runtime::{
	currency::KMA, Call, CollatorSelection, Democracy, Event, Origin, Runtime, Scheduler, Session,
	System, TransactionPayment,
};
use frame_support::{
	traits::{GenesisBuild, OnFinalize, OnInitialize},
	weights::{DispatchInfo, Weight},
};
use manta_primitives::{
	helpers::{get_account_id_from_seed, get_collator_keys_from_seed},
	AccountId, AuraId, Balance,
};
use sp_core::sr25519;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo {
		weight: w,
		..Default::default()
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

pub fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
	authorities: Vec<(AccountId, AuraId)>,
	invulnerables: Vec<AccountId>,
	safe_xcm_version: Option<u32>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				1_000_000_000_000 * KMA,
			)],
			authorities: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			)],
			invulnerables: vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
			safe_xcm_version: None,
		}
	}
}

impl ExtBuilder {
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn with_authorities(mut self, authorities: Vec<(AccountId, AuraId)>) -> Self {
		self.authorities = authorities;
		self
	}

	pub fn with_collators(mut self, invulnerables: Vec<AccountId>) -> Self {
		self.invulnerables = invulnerables;
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
			candidacy_bond: KMA * 1000, // How many tokens will be reserved as collator
			desired_candidates: 1,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_session::GenesisConfig::<Runtime> {
			keys: self
				.authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                    // account id
						acc,                                            // validator id
						calamari_runtime::opaque::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
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

pub fn root_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::root()
}
