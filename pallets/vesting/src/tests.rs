use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_runtime::SaturatedConversion;

#[test]
fn alice_vesting_for_bob_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			let locked = 100;
			assert_ok!(MantaVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				locked
			));
			assert_eq!(Balances::free_balance(ALICE), 10_000 - locked);

			run_to_block(3);
			let now = VestingSchedule::get()[0]
				.1
				.as_millis()
				.saturated_into::<u64>()
				+ 1;
			Timestamp::set_timestamp(now);

			assert_ok!(MantaVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), locked);

			// BOB cannot transfer more than 34 tokens.
			// Bacause rest of 66 is locked now.
			let amount = 34;
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, amount + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, amount));
			assert_eq!(Balances::free_balance(ALICE), 10_000 - locked + amount);
			assert_eq!(Balances::free_balance(BOB), locked - amount);
		});
}

#[test]
fn alice_vesting_for_bob_claim_slowly_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			let locked = 100;
			assert_ok!(MantaVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				locked
			));
			assert_eq!(Balances::free_balance(ALICE), 10_000 - locked);

			run_to_block(5);
			let now = VestingSchedule::get()[1]
				.1
				.as_millis()
				.saturated_into::<u64>()
				+ 1;
			Timestamp::set_timestamp(now);

			assert_ok!(MantaVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), locked);

			// BOB cannot transfer more than 45 tokens.
			// Bacause rest of 55 is locked now.
			let amount = 45;
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, amount + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, amount));
			assert_eq!(Balances::free_balance(ALICE), 10_000 - locked + amount);
			assert_eq!(Balances::free_balance(BOB), locked - amount);
		});
}

#[test]
fn alice_vesting_for_bob_claim_arbitrarily_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			let locked = 100;
			assert_ok!(MantaVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				locked
			));
			assert_eq!(Balances::free_balance(ALICE), 10_000 - locked);

			run_to_block(3);
			assert_ok!(MantaVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), locked);

			run_to_block(7);
			let now = VestingSchedule::get()[2]
				.1
				.as_millis()
				.saturated_into::<u64>()
				+ 1;
			Timestamp::set_timestamp(now);

			// BOB cannot transfer more than 34 tokens.
			// Bacause rest of 66 is locked now.
			let amount = 34;
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, amount + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(MantaVesting::vest(Origin::signed(BOB)));

			// BOB cannot transfer more than 56 tokens.
			// Bacause rest of 44 is locked now.
			let amount = 56;
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, amount + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, amount));
			assert_eq!(Balances::free_balance(ALICE), 10_000 - locked + amount);
			assert_eq!(Balances::free_balance(BOB), locked - amount);
		});
}
