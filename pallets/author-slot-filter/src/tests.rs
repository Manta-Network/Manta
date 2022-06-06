// Copyright 2019-2021 PureStake Inc.
// This file is part of Nimbus.

// Nimbus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Nimbus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Nimbus.  If not, see <http://www.gnu.org/licenses/>.

use super::*;
use crate::mock::*;
use crate::num::NonZeroU32;

use frame_support::assert_ok;
use frame_support::traits::OnRuntimeUpgrade;
use sp_runtime::Percent;

#[test]
fn test_set_eligibility_works() {
	new_test_ext().execute_with(|| {
		let value = num::NonZeroU32::new_unchecked(34);

		assert_ok!(AuthorSlotFilter::set_eligible(
			Origin::root(),
			value.clone()
		));
		assert_eq!(AuthorSlotFilter::eligible_count(), value)
	});
}

#[allow(deprecated)]
#[test]
fn test_migration_works_for_converting_existing_eligible_ratio_to_eligible_count() {
	new_test_ext().execute_with(|| {
		let input_eligible_ratio = Percent::from_percent(50);
		let total_author_count = mock::Authors::get().len();
		let eligible_author_count =
			input_eligible_ratio.clone().mul_ceil(total_author_count) as u32;
		let expected_eligible_count = NonZeroU32::new_unchecked(eligible_author_count);
		let expected_weight = TestDbWeight::get().write + TestDbWeight::get().read;

		<EligibleRatio<Test>>::put(input_eligible_ratio.clone());

		let actual_weight = migration::EligibleRatioToEligiblityCount::<Test>::on_runtime_upgrade();
		assert_eq!(expected_weight, actual_weight);

		let actual_eligible_ratio_after = AuthorSlotFilter::eligible_ratio();
		let actual_eligible_count = AuthorSlotFilter::eligible_count();
		assert_eq!(expected_eligible_count, actual_eligible_count);
		assert_eq!(input_eligible_ratio, actual_eligible_ratio_after);
	});
}

#[allow(deprecated)]
#[test]
fn test_migration_works_for_converting_existing_zero_eligible_ratio_to_default_eligible_count() {
	new_test_ext().execute_with(|| {
		let input_eligible_ratio = Percent::from_percent(0);
		let expected_eligible_count = EligibilityValue::default();
		let expected_weight = TestDbWeight::get().write + TestDbWeight::get().read;

		<EligibleRatio<Test>>::put(input_eligible_ratio.clone());

		let actual_weight = migration::EligibleRatioToEligiblityCount::<Test>::on_runtime_upgrade();
		assert_eq!(expected_weight, actual_weight);

		let actual_eligible_ratio_after = AuthorSlotFilter::eligible_ratio();
		let actual_eligible_count = AuthorSlotFilter::eligible_count();
		assert_eq!(expected_eligible_count, actual_eligible_count);
		assert_eq!(input_eligible_ratio, actual_eligible_ratio_after);
	});
}

#[allow(deprecated)]
#[test]
fn test_migration_inserts_default_value_for_missing_eligible_ratio() {
	new_test_ext().execute_with(|| {
		let default_eligible_ratio = Percent::from_percent(50);
		let expected_default_eligible_count =
			NonZeroU32::new_unchecked(default_eligible_ratio.mul_ceil(Authors::get().len() as u32));
		let expected_weight = TestDbWeight::get().write + TestDbWeight::get().read;

		let actual_weight = migration::EligibleRatioToEligiblityCount::<Test>::on_runtime_upgrade();
		assert_eq!(expected_weight, actual_weight);

		let actual_eligible_count = AuthorSlotFilter::eligible_count();
		assert_eq!(expected_default_eligible_count, actual_eligible_count);
	});
}
