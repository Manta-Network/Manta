//! Tests for the Manta Runtime Configuration

use codec::Encode;
use frame_support::{
	assert_ok,
	storage::StorageValue,
	traits::{OnFinalize, OnInitialize},
	weights::{constants::*, GetDispatchInfo, WeightToFeePolynomial},
};
use hex_literal::hex;
use manta_primitives::constants::currency::*;
use pallet_transaction_payment::Multiplier;
use separator::Separatable;
use sp_core::crypto::UncheckedInto;
use sp_runtime::{AccountId32, BuildStorage, FixedPointNumber, MultiAddress};

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

pub struct ExtBuilder {
	pub init_balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			init_balances: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn one_thousand_for_alice_n_bob(mut self) -> Self {
		self.init_balances = vec![(ALICE, 1000 * MA), (BOB, 1000 * MA)];
		self
	}

	// Create test utility, runtime mock
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
			self.init_balances,
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
	ExtBuilder::default()
		.one_thousand_for_alice_n_bob()
		.build()
		.execute_with(|| {
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
#[ignore = "It looks no way to remove panic while building manta_genesis_config."]
// https://github.com/paritytech/substrate/blob/v3.0.0/frame/balances/src/lib.rs#L481
fn check_existential_deposit() {
	let initial_authorities = vec![(
		ALICE,
		ALICE,
		hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"].unchecked_into(),
		hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"].unchecked_into(),
	)];
	let root_key = ALICE;
	let stash = crate::NativeTokenExistentialDeposit::get() - 1 * cMA;

	let init_balances: Vec<(AccountId, Balance)> = vec![
		(ALICE, crate::NativeTokenExistentialDeposit::get() - 1 * cMA),
		(BOB, crate::NativeTokenExistentialDeposit::get() - 1 * cMA),
	];

	let manta_genesis_config = manta::chain_spec::manta_testnet_config_genesis(
		initial_authorities,
		init_balances,
		root_key,
		stash,
		false,
	)
	.build_storage();

	assert!(manta_genesis_config.is_err());
}

#[test]
fn authoring_blocks_in_mock_runtime_should_work() {
	ExtBuilder::default()
		.one_thousand_for_alice_n_bob()
		.build()
		.execute_with(|| {
			run_to_block(20);
			assert_eq!(crate::System::block_number(), 20);
		});
}

// Follow kusama runtime configuration tests
// https://github.com/paritytech/polkadot/blob/master/runtime/kusama/src/tests.rs
#[test]
fn remove_keys_weight_is_sensible() {
	use pallet_manta_pay::WeightInfo;
	// mint_private_asset has the max weights in manta-pay.
	let max_weight = <crate::Runtime as pallet_manta_pay::Config>::WeightInfo::mint_private_asset();
	// Max remove keys limit should be no more than half the total block weight.
	assert!(max_weight * 2 < crate::BlockWeights::get().max_block);
}

#[test]
fn sample_size_is_sensible() {
	use frame_support::weights::{constants::RocksDbWeight, Weight};
	use pallet_manta_pay::WeightInfo;
	let max_weight: Weight = RocksDbWeight::get().reads_writes(8, 5);
	// Max sample cleanup should be no more than half the total block weight.
	assert!(max_weight * 2 < crate::BlockWeights::get().max_block);
	assert!(
		<crate::Runtime as pallet_manta_pay::Config>::WeightInfo::reclaim() * 2
			< crate::BlockWeights::get().max_block
	);
}

#[test]
fn payout_weight_portion() {
	use pallet_staking::WeightInfo;
	let payout_weight =
		<crate::Runtime as pallet_staking::Config>::WeightInfo::payout_stakers_alive_staked(
			crate::MaxNominatorRewardedPerValidator::get(),
		) as f64;
	let block_weight = crate::BlockWeights::get().max_block as f64;

	println!(
		"a full payout takes {:.2} of the block weight [{} / {}]",
		payout_weight / block_weight,
		payout_weight,
		block_weight
	);
	assert!(payout_weight * 2f64 < block_weight);
}

#[test]
fn block_cost() {
	let max_block_weight = crate::BlockWeights::get().max_block;
	let raw_fee: u128 = crate::IdentityFee::calc(&max_block_weight);

	println!(
		"Full Block weight == {} // WeightToFee(full_block) == {} plank",
		max_block_weight,
		raw_fee.separated_string(),
	);
}

#[test]
fn transfer_cost_min_multiplier() {
	let min_multiplier = crate::MinimumMultiplier::get();
	let call = <pallet_balances::Call<crate::Runtime>>::transfer_keep_alive(
		Default::default(),
		Default::default(),
	);
	let info = call.get_dispatch_info();
	// convert to outer call.
	let call = crate::Call::Balances(call);
	let len = call.using_encoded(|e| e.len()) as u32;

	let mut ext = ExtBuilder::default().one_thousand_for_alice_n_bob().build();
	let mut test_with_multiplier = |m| {
		ext.execute_with(|| {
			pallet_transaction_payment::NextFeeMultiplier::put(m);
			let fee = crate::TransactionPayment::compute_fee(len, &info, 0);
			println!(
				"weight = {:?} // multiplier = {:?} // full transfer fee = {:?}",
				info.weight.separated_string(),
				pallet_transaction_payment::NextFeeMultiplier::get(),
				fee.separated_string(),
			);
		});
	};

	test_with_multiplier(min_multiplier);
	test_with_multiplier(Multiplier::saturating_from_rational(1, 1u128));
	test_with_multiplier(Multiplier::saturating_from_rational(1, 1_000u128));
	test_with_multiplier(Multiplier::saturating_from_rational(1, 1_000_000u128));
	test_with_multiplier(Multiplier::saturating_from_rational(1, 1_000_000_000u128));
}

#[test]
#[ignore = "we don't support election right now."]
fn nominator_limit() {
	todo!("https://github.com/paritytech/polkadot/blob/v0.9.2/runtime/kusama/src/tests.rs#L110");
}

#[test]
#[ignore = "substrate 3.0 doesn't have pallet_staking_reward_fn"]
fn compute_inflation_should_give_sensible_results() {
	todo!("https://github.com/paritytech/polkadot/blob/v0.9.2/runtime/kusama/src/tests.rs#L136")
}

#[test]
#[ignore = "we might need to impl era_payout"]
fn era_payout_should_give_sensible_results() {
	todo!("https://github.com/paritytech/polkadot/blob/v0.9.2/runtime/kusama/src/tests.rs#L155");
}

// Tests to make sure that Manta's weights and fees match what we
// expect from Substrate or ORML.
//
// These test are not meant to be exhaustive, as it is inevitable that
// weights in Substrate will change. Instead they are supposed to provide
// some sort of indicator that calls we consider important (e.g
// Balances::transfer) have not suddenly changed from under us.
#[test]
fn sanity_check_weight_per_time_constants_are_as_expected() {
	// These values comes from Substrate, we want to make sure that if it
	// ever changes we don't accidently break Polkadot
	assert_eq!(WEIGHT_PER_SECOND, 1_000_000_000_000);
	assert_eq!(WEIGHT_PER_MILLIS, WEIGHT_PER_SECOND / 1000);
	assert_eq!(WEIGHT_PER_MICROS, WEIGHT_PER_MILLIS / 1000);
	assert_eq!(WEIGHT_PER_NANOS, WEIGHT_PER_MICROS / 1000);
}
