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

#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
use crate::Pallet;
use core::{ops::Div, time::Duration};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_runtime::{traits::AtLeast32BitUnsigned, SaturatedConversion};

const SEED: u32 = 0;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 100;

fn assert_has_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn init_setup<
	T: Config + pallet_timestamp::Config<Moment = u64> + pallet_balances::Config<I>,
	I: 'static,
>(
	caller: &T::AccountId,
) where
	<T as pallet_balances::Config>::Balance: TryFrom<u128>,
	T: pallet_balances::Config,
{
	let now = Duration::from_secs(1636329600 - 1)
		.as_millis()
		.saturated_into::<u64>()
		+ 1;
	pallet_timestamp::Pallet::<T>::set_timestamp(now);
	let existential_deposit = <T as pallet_balances::Config<I>>::ExistentialDeposit::get();
	let amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
	let source_caller = T::Lookup::unlookup(caller.clone());
	let _ = pallet_balances::Pallet::<T, I>::make_free_balance_be(caller, amount);
	assert_ok!(pallet_balances::Pallet::<T, I>::set_balance(
		RawOrigin::Root.into(),
		source_caller,
		amount,
		amount
	));
	assert_eq!(
		pallet_balances::Pallet::<T, I>::free_balance(caller),
		amount
	);
}

benchmarks! {
	where_clause {
		where
			T: Config + pallet_timestamp::Config<Moment = u64> + pallet_balances::Config,
			BalanceOf<T>: AtLeast32BitUnsigned,
			<T as pallet_balances::Config>::Balance: From<u128>,
	}

	update_vesting_schedule {
		let new_schedule = BoundedVec::try_from(
			crate::Pallet::<T>::vesting_schedule()
				.iter()
				.map(|(_, s)| s + 1)
				.collect::<sp_std::vec::Vec<u64>>(),
		)
		.unwrap_or_default();
	}: _(RawOrigin::Root, new_schedule.clone())
	verify {
		assert_has_event::<T>(Event::VestingScheduleUpdated(new_schedule).into());
	}

	vest {
		let caller: T::AccountId = whitelisted_caller();
		let recipient: T::AccountId = account("receiver", 0, SEED);
		let source_recipient = T::Lookup::unlookup(recipient.clone());
		init_setup::<T, ()>(&caller);
		let existential_deposit = <T as pallet_balances::Config<()>>::ExistentialDeposit::get();
		let unvested = existential_deposit
			.saturating_mul(ED_MULTIPLIER.div(10u32).into())
			.saturated_into::<u128>()
			.try_into()
			.ok()
			.unwrap();
		assert_ok!(
			crate::Pallet::<T>::vested_transfer(
				RawOrigin::Signed(caller).into(), source_recipient, unvested
			)
		);
		assert!(crate::Pallet::<T>::vesting_balance(&recipient).is_some());
		let now = Duration::from_secs(1660694400)
			.as_millis()
			.saturated_into::<u64>()
			+ 1;
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
	}: _(RawOrigin::Signed(recipient.clone()))
	verify {
		assert_has_event::<T>(Event::VestingCompleted(recipient).into());
	}

	vested_transfer {
		let caller: T::AccountId = whitelisted_caller();
		init_setup::<T, ()>(&caller);
		let existential_deposit = <T as pallet_balances::Config<()>>::ExistentialDeposit::get();
		let unvested = existential_deposit
			.saturating_mul(ED_MULTIPLIER.div(10u32).into())
			.saturated_into::<u128>()
			.try_into()
			.ok()
			.unwrap();
		let recipient: T::AccountId = account("receiver", 0, SEED);
		let source_recipient = T::Lookup::unlookup(recipient.clone());
	}: _(RawOrigin::Signed(caller.clone()), source_recipient, unvested)
	verify {
		assert_eq!(crate::Pallet::<T>::vesting_balance(&recipient), Some(unvested));
		assert_has_event::<T>(Event::VestingUpdated(recipient, unvested).into());
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::ExtBuilder::default()
		.existential_deposit(1)
		.build(),
	crate::mock::Test,
);
