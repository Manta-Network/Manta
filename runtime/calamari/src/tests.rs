use frame_support::{assert_noop, assert_ok, traits::GenesisBuild};
use hex_literal::hex;
use manta_primitives::{
	time::*, AccountId, AuraId, Balance, BlockNumber, Hash, Header, Index, Signature,
};
use sp_core::crypto::UncheckedInto;
use sp_keyring::sr25519::Keyring;
use sp_runtime::traits::StaticLookup;
use sp_runtime::{traits::AtLeast32BitUnsigned, SaturatedConversion};

const MULTIPLE: Balance = 10_000_000_000;

pub fn calamari_session_keys(keys: AuraId) -> crate::opaque::SessionKeys {
	crate::opaque::SessionKeys { aura: keys }
}

pub struct ExtBuilder {
	existential_deposit: Balance,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: crate::NativeTokenExistentialDeposit::get(),
		}
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap();
		// Deposit tokens to some accounts.
		pallet_balances::GenesisConfig::<crate::Runtime> {
			balances: vec![
				(Keyring::Alice.into(), MULTIPLE * self.existential_deposit),
				(Keyring::Bob.into(), MULTIPLE * self.existential_deposit),
				(Keyring::Eve.into(), MULTIPLE * self.existential_deposit),
			],
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		// Aura configuration
		pallet_collator_selection::GenesisConfig::<crate::Runtime> {
			// 3 block authors
			invulnerables: vec![
				Keyring::Alice.into(),
				Keyring::Bob.into(),
				Keyring::Eve.into(),
			],
			candidacy_bond: crate::currency::KMA * 100,
			..Default::default()
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		// cumulus_pallet_aura_ext::GenesisConfig::default()
		// .assimilate_storage(&mut storage)
		// .unwrap();

		// Session keys configuration
		pallet_session::GenesisConfig::<crate::Runtime> {
			keys: vec![(
				Keyring::Alice.into(),
				Keyring::Alice.into(),
				calamari_session_keys(
					hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"]
						.unchecked_into(),
				),
			)],
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(storage);
		ext
	}
}

pub(crate) fn run_to_block(n: u32) {
	crate::System::set_block_number(n);
}

#[test]
fn deposit_fees_to_block_authors_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		run_to_block(1);
		println!("current author: {:?}", crate::Authorship::author());
		// Transfer
		let amount = 20 * crate::NativeTokenExistentialDeposit::get();
		let bob = Keyring::Bob.into();
		let source_bob = <crate::Runtime as frame_system::Config>::Lookup::unlookup(bob);
		assert_ok!(crate::Balances::transfer(
			crate::Origin::signed(Keyring::Alice.into()),
			source_bob,
			amount
		));
		run_to_block(2);
		println!("current author: {:?}", crate::Authorship::author());
		run_to_block(3);
	});
}
