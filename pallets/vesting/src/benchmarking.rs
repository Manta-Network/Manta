#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
#[allow(unused_imports)]
use crate::Pallet as MantaVesting;
use core::{convert::TryInto, time::Duration};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_runtime::{traits::AtLeast32BitUnsigned, SaturatedConversion};

const SEED: u32 = 0;

fn assert_has_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn init_setup<
	T: Config + pallet_timestamp::Config<Moment = u64> + pallet_balances::Config<I>,
	I: 'static,
>(
	caller: &T::AccountId,
	amount: BalanceOf<T>,
) where
	<T as pallet_balances::Config>::Balance: TryFrom<u128>,
	T: pallet_balances::Config,
{
	let now = Duration::from_secs(1636329600 - 1)
		.as_millis()
		.saturated_into::<u64>()
		+ 1;
	pallet_timestamp::Pallet::<T>::set_timestamp(now);

	let amount = amount.saturated_into::<u128>();
	let _ = pallet_balances::Pallet::<T, I>::deposit_creating(
		&caller,
		amount.try_into().map_err(|_| "").unwrap(),
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
				.collect::<Vec<u64>>(),
		)
		.unwrap_or_default();
		dbg!(&new_schedule);
	}: _(RawOrigin::Root, new_schedule.clone())
	verify {
		assert_has_event::<T>(Event::VestingScheduleUpdated(new_schedule).into());
	}

	vest {
		let caller: T::AccountId = whitelisted_caller();
		let recipient: T::AccountId = account("receiver", 0, SEED);
		let source_recipient = T::Lookup::unlookup(recipient.clone());

		let unvested: BalanceOf<T> = 1000u32.into();
		init_setup::<T, ()>(&caller, unvested);
		assert_ok!(crate::Pallet::<T>::vested_transfer(RawOrigin::Signed(caller.clone()).into(), source_recipient, 100u32.into()));
		assert_eq!(crate::Pallet::<T>::vesting_balance(&recipient).is_some(), true);

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
		let unvested: BalanceOf<T> = 100u32.into();
		init_setup::<T, ()>(&caller, unvested);
		let recipient: T::AccountId = account("receiver", 0, SEED);
		let source_recipient = T::Lookup::unlookup(recipient.clone());
	}: _(RawOrigin::Signed(caller.clone()), source_recipient, unvested)
	verify {
		assert_eq!(crate::Pallet::<T>::vesting_balance(&recipient), Some(unvested));
		assert_has_event::<T>(Event::VestingUpdated(recipient, unvested).into());
	}
}

impl_benchmark_test_suite!(
	MantaVesting,
	crate::mock::ExtBuilder::default()
		.existential_deposit(1)
		.build(),
	crate::mock::Test,
);
