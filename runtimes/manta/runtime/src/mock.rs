use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use hex_literal::hex;
use manta_primitives::constants::currency::*;
use sp_core::crypto::UncheckedInto;
use sp_runtime::{AccountId32, BuildStorage, MultiAddress};

pub type AccountId = AccountId32;
pub const ALICE: AccountId = AccountId32::new([0u8; 32]);
pub const BOB: AccountId = AccountId32::new([1u8; 32]);

#[allow(dead_code)]
pub(crate) fn run_to_block(n: u32) {
	while crate::System::block_number() < n {
		crate::System::on_finalize(crate::System::block_number());
		crate::System::set_block_number(crate::System::block_number() + 1);
		crate::System::on_initialize(crate::System::block_number());
	}
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {}
	}
}

impl ExtBuilder {
	pub fn one_thousand_for_alice_n_bob() -> Vec<(AccountId, Balance)> {
		vec![
			// Alice
			(ALICE, 1000 * MA),
			// Bob
			(BOB, 1000 * MA),
		]
	}

	// Create test utility
	pub fn build(self) -> sp_io::TestExternalities {
		let initial_authorities = vec![(
			ALICE,
			ALICE,
			hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"]
				.unchecked_into(),
			hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
				.unchecked_into(),
		)];
		let root_key = ALICE;
		let stash = 100 * MA;

		let manta_genesis_config = manta::chain_spec::manta_testnet_config_genesis(
			initial_authorities,
			Self::one_thousand_for_alice_n_bob(),
			root_key,
			stash,
			false,
		)
		.build_storage()
		.expect("failed to create manta gensis config");

		manta_genesis_config.into()
	}
}

#[test]
fn balances_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		// transfer
		assert_ok!(crate::Balances::transfer(
			crate::Origin::signed(ALICE),
			MultiAddress::Id(BOB),
			20 * MA
		));
		// check balance after transfer
		assert_eq!(crate::System::account(ALICE).data.free, 1000 * MA - 20 * MA);
		assert_eq!(crate::System::account(BOB).data.free, 1000 * MA + 20 * MA);
	});
}

#[test]
#[ignore = "Under implementation"]
fn set_code_should_work() {
	todo!()
}

#[test]
#[ignore = "Under implementation"]
fn genesis_tests() {
	todo!()
}

#[test]
#[ignore = "Under implementation"]
fn manta_pay_should_work() {
	todo!();
}
