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

use super::{Event as PalletEvent, *};
use chrono::prelude::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event as MockEvent, *};

#[test]
fn alice_vesting_for_bob_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			// Cannot vest tokens that is less than expected.
			assert_noop!(
				CalamariVesting::vested_transfer(
					Origin::signed(ALICE),
					BOB,
					MinVestedTransfer::get() - 1
				),
				Error::<Test>::AmountLow
			);

			// Signer cannot vest tokens that exceeds all he has.
			assert_noop!(
				CalamariVesting::vested_transfer(Origin::signed(ALICE), BOB, ALICE_DEPOSIT + 1),
				Error::<Test>::BalanceLow
			);

			let unvested = 100;
			assert_ok!(CalamariVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				unvested
			));

			// Cannot vest tokens the same user more than twice.
			assert_noop!(
				CalamariVesting::vested_transfer(Origin::signed(ALICE), BOB, unvested),
				Error::<Test>::ExistingVestingSchedule
			);

			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT - unvested);
			assert_eq!(Balances::free_balance(BOB), unvested);
			assert_eq!(VestingBalances::<Test>::get(BOB), Some(unvested));

			// Now Bob cannot claim any token.
			assert_noop!(
				CalamariVesting::vest(Origin::signed(BOB)),
				Error::<Test>::ClaimTooEarly,
			);

			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingUpdated(
				BOB, unvested,
			)));

			run_to_block(3);
			// Ensure current timestamp is bigger than the 1th round of schedule.
			// Now Bob can claim 1th round vested tokens.
			let first_round = 0;
			let now = VestingSchedule::<Test>::get()[first_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			assert_ok!(CalamariVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), unvested);

			// BOB cannot transfer more than 1th round of vested tokens.
			// Bacause the rest of tokens are locked.
			let vested = VestingSchedule::<Test>::get()[first_round].0 * unvested;
			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingUpdated(
				BOB,
				unvested - vested,
			)));

			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, vested + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, vested));
			assert_eq!(
				Balances::free_balance(ALICE),
				ALICE_DEPOSIT - unvested + vested
			);
			assert_eq!(Balances::free_balance(BOB), unvested - vested);

			// Ensure current timestamp is bigger than the 6th round of schedule.
			// Now Bob can claim 6th round vested tokens.
			let last_round = 5;
			let now = VestingSchedule::<Test>::get()[last_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			assert_ok!(CalamariVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), unvested - vested);

			// Check vested done event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingCompleted(
				BOB,
			)));

			// Now, Bob can transfer all his tokens.
			assert_ok!(Balances::transfer(
				Origin::signed(BOB),
				ALICE,
				unvested - vested
			));
			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT);
			assert_eq!(Balances::free_balance(BOB), 0);

			// Ensure vesting info is removed once vesting is done.
			assert_eq!(VestingBalances::<Test>::get(BOB), None);
		});
}

#[test]
fn alice_vesting_for_bob_claim_slowly_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			let unvested = 100;
			assert_ok!(CalamariVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				unvested
			));
			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT - unvested);
			assert_eq!(Balances::free_balance(BOB), unvested);
			assert_eq!(VestingBalances::<Test>::get(BOB), Some(unvested));

			// Now Bob cannot claim any token.
			assert_noop!(
				CalamariVesting::vest(Origin::signed(BOB)),
				Error::<Test>::ClaimTooEarly,
			);

			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingUpdated(
				BOB, unvested,
			)));

			// Ensure current timestamp is bigger than the 4th round of schedule.
			// Now Bob can claim 4th round vested tokens.
			let fourth_round = 3;
			let now = VestingSchedule::<Test>::get()[fourth_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			assert_ok!(CalamariVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), unvested);

			// Calculate how many tokens that have been vested.
			let vested = VestingSchedule::<Test>::get()[..=fourth_round]
				.iter()
				.map(|s| s.0)
				.fold(Percent::from_percent(0), |acc, p| acc.saturating_add(p))
				* unvested;
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, vested + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, vested));
			assert_eq!(
				Balances::free_balance(ALICE),
				ALICE_DEPOSIT - unvested + vested
			);
			assert_eq!(Balances::free_balance(BOB), unvested - vested);
		});
}

#[test]
fn alice_vesting_for_bob_claim_arbitrarily_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			let unvested = 100;
			assert_ok!(CalamariVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				unvested
			));
			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT - unvested);
			assert_eq!(Balances::free_balance(BOB), unvested);
			assert_eq!(VestingBalances::<Test>::get(BOB), Some(unvested));

			run_to_block(3);
			// Ensure current timestamp is bigger than the 1th round of schedule.
			// Now Bob can claim 1th round vested tokens.
			let first_round = 0;
			let now = VestingSchedule::<Test>::get()[first_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			assert_ok!(CalamariVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), unvested);

			// BOB cannot transfer more than 1th round of vested tokens.
			// Bacause the rest of tokens are locked.
			let vested_1 = VestingSchedule::<Test>::get()[first_round].0 * unvested;
			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingUpdated(
				BOB,
				unvested - vested_1,
			)));

			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, vested_1 + 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, vested_1));
			assert_eq!(
				Balances::free_balance(ALICE),
				ALICE_DEPOSIT - unvested + vested_1
			);
			assert_eq!(Balances::free_balance(BOB), unvested - vested_1);

			// Ensure current timestamp is bigger than the 5th round of schedule.
			// Now Bob can claim 5th round vested tokens.
			let sixth_round = 4;
			let now = VestingSchedule::<Test>::get()[sixth_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			assert_ok!(CalamariVesting::vest(Origin::signed(BOB)));

			// All vested for 5th round.
			let vested_0_to_4 = VestingSchedule::<Test>::get()[..=sixth_round]
				.iter()
				.map(|s| s.0)
				.fold(Percent::from_percent(0), |acc, p| acc.saturating_add(p))
				* unvested;
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, vested_0_to_4 + 1 - vested_1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			// Vested only 6th round.
			let vested_5 = VestingSchedule::<Test>::get()[sixth_round].0 * unvested;

			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingUpdated(
				BOB, 11,
			)));
			assert_eq!(
				Balances::free_balance(BOB),
				vested_0_to_4 + vested_5 - vested_1
			);

			assert_ok!(Balances::transfer(
				Origin::signed(BOB),
				ALICE,
				vested_0_to_4 - vested_1
			));
			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT - vested_5);
			assert_eq!(Balances::free_balance(BOB), vested_5);
		});
}

#[test]
fn vesting_complete_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			let unvested = 100;
			assert_ok!(CalamariVesting::vested_transfer(
				Origin::signed(ALICE),
				BOB,
				unvested
			));
			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT - unvested);
			assert_eq!(VestingBalances::<Test>::get(BOB), Some(unvested));

			// Now Bob cannot claim any token.
			assert_noop!(
				CalamariVesting::vest(Origin::signed(BOB)),
				Error::<Test>::ClaimTooEarly,
			);

			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingUpdated(
				BOB, unvested,
			)));

			// Now Bob cannot transfer locked tokens.
			assert_noop!(
				Balances::transfer(Origin::signed(BOB), ALICE, 1),
				pallet_balances::Error::<Test, _>::LiquidityRestrictions,
			);

			// Ensure current timestamp is bigger than the 6th round of schedule.
			// Now Bob can claim 6th round vested tokens.
			let last_round = 5;
			let now = VestingSchedule::<Test>::get()[last_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			assert_ok!(CalamariVesting::vest(Origin::signed(BOB)));
			assert_eq!(Balances::free_balance(BOB), unvested);

			// Check vested done event
			System::assert_has_event(MockEvent::CalamariVesting(PalletEvent::VestingCompleted(
				BOB,
			)));
			let vested = unvested;

			// Now, Bob can transfer all his tokens.
			assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, vested));
			assert_eq!(Balances::free_balance(ALICE), ALICE_DEPOSIT);
			assert_eq!(Balances::free_balance(BOB), 0);

			// Ensure vesting info is removed once vesting is done.
			assert_eq!(VestingBalances::<Test>::get(BOB), None);
		});
}

#[test]
fn partially_update_vesting_schedule_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			// Ensure current timestamp is bigger than the 1th round of schedule.
			// Now Bob can claim 1th round vested tokens.
			let frist_round = 0;
			let now = VestingSchedule::<Test>::get()[frist_round].1 * 1000 + 1;
			Timestamp::set_timestamp(now);

			// skip 2 round of old schedule.
			let skipped_count = 2;
			let new_schedule = BoundedVec::try_from({
				let mut new_schedule = vec![];
				for (index, (_, schedule)) in VestingSchedule::<Test>::get().iter().enumerate() {
					if index < skipped_count {
						// Do not change old schedule
						new_schedule.push(*schedule);
						continue;
					}
					// odd means more early than old schedle but still later than now.
					// even means more late than old schedle but still later than now.
					if index % 2 == 0 {
						new_schedule.push(*schedule + 1);
					} else {
						new_schedule.push(*schedule - 1);
					}
				}
				new_schedule
			})
			.unwrap_or_default();

			assert_ok!(CalamariVesting::update_vesting_schedule(
				Origin::root(),
				new_schedule.clone()
			));
			// Check storage
			assert_eq!(
				VestingSchedule::<Test>::get()
					.iter()
					.map(|(_, s)| *s)
					.collect::<Vec<u64>>(),
				*new_schedule
			);
			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(
				PalletEvent::VestingScheduleUpdated(new_schedule),
			));
		});
}

#[test]
fn update_brand_new_vesting_schedule_should_work() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			// Ensure current timestamp is bigger than the 1th round of schedule.
			// Now Bob can claim 1th round vested tokens.
			let frist_round = 0;
			let now = VestingSchedule::<Test>::get()[frist_round].1 * 1000 - 1;
			Timestamp::set_timestamp(now);

			let new_schedule = BoundedVec::try_from(
				VestingSchedule::<Test>::get()
					.iter()
					.map(|(_, s)| s + 1)
					.collect::<Vec<u64>>(),
			)
			.unwrap_or_default();
			assert_ok!(CalamariVesting::update_vesting_schedule(
				Origin::root(),
				new_schedule.clone()
			));
			// Check storage
			assert_eq!(
				VestingSchedule::<Test>::get()
					.iter()
					.map(|(_, s)| *s)
					.collect::<Vec<u64>>(),
				*new_schedule
			);
			// Check event
			System::assert_has_event(MockEvent::CalamariVesting(
				PalletEvent::VestingScheduleUpdated(new_schedule),
			));
		});
}

#[test]
fn invalid_schedule_should_not_be_updated() {
	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			// Cannot update the length of schedule is bigger than 6 or smaller than 6.
			let wrong_length_schedule: BoundedVec<u64, <Test as Config>::MaxScheduleLength> =
				BoundedVec::try_from(vec![1, 2, 3, 4, 5, 6, 7]).unwrap_or_default();
			assert_noop!(
				CalamariVesting::update_vesting_schedule(Origin::root(), wrong_length_schedule),
				Error::<Test>::InvalidScheduleLength,
			);

			// We have only 6 rounds of schedule.
			let wrong_length_schedule: BoundedVec<u64, <Test as Config>::MaxScheduleLength> =
				BoundedVec::try_from(vec![1, 2, 3, 4, 5]).unwrap_or_default();
			assert_noop!(
				CalamariVesting::update_vesting_schedule(Origin::root(), wrong_length_schedule),
				Error::<Test>::InvalidScheduleLength,
			);

			// The new schedule should be a sorted array.
			let invalid_schedule: BoundedVec<u64, <Test as Config>::MaxScheduleLength> =
				BoundedVec::try_from(vec![1, 2, 9, 4, 8, 6]).unwrap_or_default();
			assert_noop!(
				CalamariVesting::update_vesting_schedule(Origin::root(), invalid_schedule),
				Error::<Test>::UnsortedSchedule,
			);

			// Check updating invalid partial schedule should not work.
			let next_round = 3;
			// now is between 3th round and 4th round.
			let now = (VestingSchedule::<Test>::get()[next_round].1 - 1) * 1000;
			Timestamp::set_timestamp(now);

			let invalid_schedule = BoundedVec::try_from({
				let mut new_schedule = vec![];
				for (index, (_, schedule)) in VestingSchedule::<Test>::get().iter().enumerate() {
					if index < next_round {
						// Do not change old schedule
						new_schedule.push(*schedule);
						continue;
					}
					// Set one schedule that is past time.
					// This schedule is earlier than now.
					if index == next_round {
						new_schedule.push((now - 2) / 1000);
						continue;
					}
					// Do not change the rest of future schedule;
					new_schedule.push(*schedule);
				}
				new_schedule
			})
			.unwrap_or_default();

			assert_noop!(
				CalamariVesting::update_vesting_schedule(Origin::root(), invalid_schedule),
				Error::<Test>::InvalidSchedule,
			);
		});
}

#[test]
fn check_vesting_schedule() {
	#[rustfmt::skip]
	let default_schedule: [(Percent, (i32, u32, u32, u32, u32, u32), &'static str); 6] = [
		// (Percentage, (timestamp), date)
		(Percent::from_percent(45), (2021, 12, 10, 0, 0, 0), "2021-12-10 00:00:00"),
		(Percent::from_percent(11), (2022, 01, 05, 0, 0, 0), "2022-01-05 00:00:00"),
		(Percent::from_percent(11), (2022, 03, 02, 0, 0, 0), "2022-03-02 00:00:00"),
		(Percent::from_percent(11), (2022, 04, 27, 0, 0, 0), "2022-04-27 00:00:00"),
		(Percent::from_percent(11), (2022, 06, 22, 0, 0, 0), "2022-06-22 00:00:00"),
		(Percent::from_percent(11), (2022, 08, 17, 0, 0, 0), "2022-08-17 00:00:00"),
	];

	ExtBuilder::default()
		.existential_deposit(1)
		.build()
		.execute_with(|| {
			// Check current schedule.
			let schedule = VestingSchedule::<Test>::get();
			let schedule_len: u32 = <Test as Config>::MaxScheduleLength::get();
			assert_eq!(schedule.len(), schedule_len as usize);

			//Check percentage.
			assert_eq!(
				schedule
					.iter()
					.map(|(p, _)| p)
					.fold(Percent::from_percent(0), |acc, p| acc.saturating_add(*p)),
				Percent::from_percent(100)
			);

			for ((p, s), ds) in schedule.iter().zip(default_schedule.iter()) {
				let dt = Utc
					.ymd(ds.1 .0, ds.1 .1, ds.1 .2)
					.and_hms(ds.1 .3, ds.1 .4, ds.1 .5);

				// Check each percentage is correct.
				assert_eq!(ds.0, *p);
				// Check datetime is correct.
				assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), ds.2);
				// Check timestamp is correct.
				assert_eq!(dt.timestamp() as u64, *s);
			}
		});
}
