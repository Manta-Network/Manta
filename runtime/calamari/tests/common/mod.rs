#![allow(dead_code)]

pub use calamari_runtime::{
	currency::KMA, AllPalletsReversedWithSystemFirst, Aura, AuraExt, Authorship, Balances, Call,
	CollatorSelection, Event, Executive, Origin, Runtime, Session, System, TransactionPayment,
	UncheckedExtrinsic,
};
use manta_primitives::{
	time::*, AccountId, AuraId, Balance, BlockNumber, Hash, Header, Index, Signature,
};
use pallet_transaction_payment::{ChargeTransactionPayment, Multiplier};

use cumulus_primitives_parachain_inherent::ParachainInherentData;
use frame_support::{
	assert_ok,
	dispatch::Dispatchable,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
	weights::{DispatchClass, DispatchInfo, Weight},
};
use frame_system::InitKind;
use sp_core::{sr25519, Encode, Pair, Public, H160};
use sp_runtime::{
	traits::{
		Applyable, Convert, IdentifyAccount, One, SignedExtension, TrailingZeroInput, Verify,
	},
	Digest, DigestItem, Perbill,
};

use std::collections::BTreeMap;

// A valid signed Alice transfer.
pub const VALID_ETH_TX: &str =
	"f86880843b9aca0083b71b0094111111111111111111111111111111111111111182020080820a26a\
	08c69faf613b9f72dbb029bb5d5acf42742d214c79743507e75fdc8adecdee928a001be4f58ff278ac\
	61125a81a582a717d9c5d6554326c01b878297c6522b12282";

// An invalid signed Alice transfer with a gas limit artifically set to 0.
pub const INVALID_ETH_TX: &str =
	"f86180843b9aca00809412cb274aad8251c875c0bf6872b67d9983e53fdd01801ca00e28ba2dd3c5a\
	3fd467d4afd7aefb4a34b373314fff470bb9db743a84d674a0aa06e5994f2d07eafe1c37b4ce5471ca\
	ecec29011f6f5bf0b1a552c55ea348df35f";

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo {
		weight: w,
		..Default::default()
	}
}

/// Utility function that advances the chain to the desired block number.
/// If an author is provided, that author information is injected to all the blocks in the meantime.
pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		Authorship::on_finalize(System::block_number());
		Session::on_finalize(System::block_number());
		CollatorSelection::on_finalize(System::block_number());
		TransactionPayment::on_finalize(System::block_number());
		Aura::on_finalize(System::block_number());
		AuraExt::on_finalize(System::block_number());

		// let call = Call::Balances(pallet_balances::Call::transfer {
		// 	dest: sp_runtime::MultiAddress::Id(AccountId::from(CHARLIE)),
		// 	value: 10 * KMA,
		// });

		// let len = 10;
		// let info = info_from_weight(5);
		// let maybe_pre = ChargeTransactionPayment::<Runtime>::from(0)
		// 	.pre_dispatch(&AccountId::from(BOB), &call, &info, len)
		// 	.unwrap();

		// let res = call.clone().dispatch(Origin::signed(AccountId::from(BOB)));

		// let post_info = match res {
		// 	Ok(info) => info,
		// 	Err(err) => err.post_info,
		// };

		// ChargeTransactionPayment::<Runtime>::post_dispatch(
		// 	maybe_pre,
		// 	&info,
		// 	&post_info,
		// 	len,
		// 	&res.map(|_| ()).map_err(|e| e.error),
		// );

		// println!("\n session QueuedKeys are: {:?} \n", Session::queued_keys());

		System::set_block_number(System::block_number() + 1);
		Authorship::on_initialize(System::block_number());
		CollatorSelection::on_initialize(System::block_number());
		Session::on_initialize(System::block_number());
		Aura::on_initialize(System::block_number());
		AuraExt::on_initialize(System::block_number());
		TransactionPayment::on_initialize(System::block_number());

		// println!(
		// 	"\n ALICE's last authored block: {:?} \n",
		// 	CollatorSelection::last_authored_block(AccountId::from(ALICE))
		// );

		// println!(
		// 	"\n BOB's last authored block: {:?} \n",
		// 	CollatorSelection::last_authored_block(AccountId::from(BOB))
		// );
	}
}

pub fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

pub struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	invulnerables: Vec<AccountId>,
	safe_xcm_version: Option<u32>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			invulnerables: vec![],
			safe_xcm_version: None,
		}
	}
}

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

type AccountPublic = <Signature as Verify>::Signer;
/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

impl ExtBuilder {
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
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

		let authorities = vec![(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_collator_keys_from_seed("Alice"),
		)];
		pallet_session::GenesisConfig::<Runtime> {
			keys: authorities
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

pub const CHAIN_ID: u64 = 1281;
// pub const ALICE: [u8; 32] = [4u8; 32];
// pub const BOB: [u8; 32] = [5u8; 32];
// pub const CHARLIE: [u8; 32] = [6u8; 32];
// pub const DAVE: [u8; 32] = [7u8; 32];

pub fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
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
