#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
#[allow(unused_imports)]
use crate::Pallet as MantaVesting;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn timestamp_setup<T: Config + pallet_timestamp::Config<Moment = u64>>() {
	let now = Duration::from_secs(1660694400)
		.as_millis()
		.saturated_into::<u64>()
		+ 1;
	pallet_timestamp::Pallet::<T>::set_timestamp(now);
}

benchmarks! {
	where_clause {
		where
			T: Config + pallet_timestamp::Config<Moment = u64>,
	}

	vest {
		let caller: T::AccountId = whitelisted_caller();
		// Last period of releasing tokens.
		timestamp_setup::<T>();
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::VestingCompleted(caller).into());
	}

	vested_transfer {
		let caller: T::AccountId = whitelisted_caller();
		let recipient: T::AccountId = account("receiver", 0, SEED);
		let source_recipient = T::Lookup::unlookup(recipient.clone());
		let locked_amount = 100u32.into();
	}: _(RawOrigin::Signed(caller.clone()), source_recipient, locked_amount)
	verify {
		assert_last_event::<T>(Event::VestingUpdated(recipient, locked_amount).into());
	}
}

impl_benchmark_test_suite!(
	MantaVesting,
	crate::mock::ExtBuilder::default()
		.existential_deposit(1)
		.build(),
	crate::mock::Test,
);
