#![allow(dead_code)]

pub use calamari_runtime::{
	currency::KMA, Balances, Call, CollatorSelection, Event, Executive, Origin, Runtime, Session,
	System, TransactionPayment, UncheckedExtrinsic,
};
use manta_primitives::{AccountId, AuraId, Balance};

use cumulus_primitives_parachain_inherent::ParachainInherentData;
use frame_support::{
	assert_ok,
	dispatch::Dispatchable,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
	weights::{DispatchInfo, Weight},
};

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
			balances: vec![],
			authorities: vec![],
			invulnerables: vec![],
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

	pub fn with_safe_xcm_version(mut self, safe_xcm_version: u32) -> Self {
		self.safe_xcm_version = Some(safe_xcm_version);
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

pub fn inherent_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::none()
}

pub fn root_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::root()
}

/// Mock the inherent that sets validation data in ParachainSystem, which
/// contains the `relay_chain_block_number`, which is used in `author-filter` as a
/// source of randomness to filter valid authors at each block.
pub fn set_parachain_inherent_data() {
	use cumulus_primitives_core::PersistedValidationData;
	use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
	let (relay_parent_storage_root, relay_chain_state) =
		RelayStateSproofBuilder::default().into_state_root_and_proof();
	let vfp = PersistedValidationData {
		relay_parent_number: 1u32,
		relay_parent_storage_root,
		..Default::default()
	};
	let parachain_inherent_data = ParachainInherentData {
		validation_data: vfp,
		relay_chain_state: relay_chain_state,
		downward_messages: Default::default(),
		horizontal_messages: Default::default(),
	};
	assert_ok!(Call::ParachainSystem(
		cumulus_pallet_parachain_system::Call::<Runtime>::set_validation_data {
			data: parachain_inherent_data
		}
	)
	.dispatch(inherent_origin()));
}
