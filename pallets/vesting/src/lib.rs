#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use core::convert::TryFrom;
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		Currency, ExistenceRequirement, Get, LockIdentifier, LockableCurrency, UnixTime,
		WithdrawReasons,
	},
};
use frame_system::{ensure_signed, pallet_prelude::*};
pub use pallet::*;
use sp_runtime::{
	traits::{Saturating, StaticLookup, Zero},
	Percent,
};

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type Schedule = u64;

const VESTING_ID: LockIdentifier = *b"calamvst";

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency trait.
		type Currency: LockableCurrency<Self::AccountId>;

		type Timestamp: UnixTime;

		/// The minimum amount transferred to call `vested_transfer`.
		#[pallet::constant]
		type MinVestedTransfer: Get<BalanceOf<Self>>;

		/// The maximum length of schedule is allowed.
		#[pallet::constant]
		type MaxScheduleLength: Get<u32>;
	}

	/// Information regarding the vesting of a given account.
	#[pallet::storage]
	#[pallet::getter(fn vesting_balance)]
	pub(super) type VestingBalances<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	/// Information regarding the vesting of a given account.
	#[pallet::storage]
	#[pallet::getter(fn vesting_schedule)]
	pub(super) type VestingSchedule<T: Config> = StorageValue<
		_,
		// The schedule is UTC.
		BoundedVec<(Percent, Schedule), T::MaxScheduleLength>,
		ValueQuery,
		DefaultVestingSchedule<T>,
	>;

	#[pallet::type_value]
	pub(super) fn DefaultVestingSchedule<T: Config>(
	) -> BoundedVec<(Percent, Schedule), T::MaxScheduleLength> {
		BoundedVec::try_from(sp_std::vec![
			// 1636329600 = 2021-11-08 00:00:00(UTC)
			(Percent::from_percent(34), 1636329600u64),
			// 1636502400 = 2021-11-10 00:00:00(UTC)
			(Percent::from_percent(11), 1636502400u64),
			// 1641340800 = 2022-01-05 00:00:00(UTC)
			(Percent::from_percent(11), 1641340800u64),
			// 1646179200 = 2022-03-02 00:00:00(UTC)
			(Percent::from_percent(11), 1646179200u64),
			// 1651017600 = 2022-04-27 00:00:00(UTC)
			(Percent::from_percent(11), 1651017600u64),
			// 1655856000 = 2022-06-22 00:00:00(UTC)
			(Percent::from_percent(11), 1655856000u64),
			// 1660694400 = 2022-08-17 00:00:00(UTC)
			(Percent::from_percent(11), 1660694400u64),
		])
		.unwrap_or_default()
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The amount vested has been updated. This could indicate more funds are available.
		/// The balance given is the amount which is left unvested (and thus locked).
		/// \[account, unvested\]
		VestingUpdated(T::AccountId, BalanceOf<T>),
		/// An \[account\] has become fully vested. No further vesting can happen.
		VestingCompleted(T::AccountId),
		/// Update a vesting schedule.
		/// \[new_schedule\]
		VestingScheduleUpdated(BoundedVec<Schedule, T::MaxScheduleLength>),
	}

	/// Error for the vesting pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The account given is not vesting.
		NotVesting,
		/// An existing vesting schedule already exists for this account that cannot be clobbered.
		ExistingVestingSchedule,
		/// Amount being transferred is too low to create a vesting schedule.
		AmountLow,
		/// Not enough tokens for vesting.
		BalanceLow,
		/// Cannot input
		InvalidTimestamp,
		/// The length of new schedule cannot be bigger/smaller than 7.
		InvalidScheduleLength,
		/// The new schedule should be sorted.
		UnsortedSchedule,
		/// The first round of vesting is not done yet.
		ClaimTooEarly,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update vesting schedule.
		///
		/// - `new_schedule`: New schedule for vesting.
		#[pallet::weight(10_000)]
		pub fn update_vesting_schedule(
			origin: OriginFor<T>,
			new_schedule: BoundedVec<Schedule, T::MaxScheduleLength>,
		) -> DispatchResult {
			ensure_root(origin)?;

			// We have only 7 rounds of schedule.
			let old_schedule = VestingSchedule::<T>::get();
			ensure!(
				new_schedule.len() == old_schedule.len(),
				Error::<T>::InvalidScheduleLength
			);

			// Ensure the new schedule should be sorted.
			ensure!(
				new_schedule.as_slice().windows(2).all(|w| w[0] < w[1]),
				Error::<T>::UnsortedSchedule
			);

			// Todo, consider some vesting schedule happen.
			let now = T::Timestamp::now().as_secs();
			ensure!(
				new_schedule.iter().all(|&s| now <= s),
				Error::<T>::InvalidTimestamp
			);

			VestingSchedule::<T>::mutate(|schedule| {
				for (schedule, newer_schedule) in
					schedule.as_mut().iter_mut().zip(new_schedule.iter())
				{
					schedule.1 = *newer_schedule;
				}
			});

			Self::deposit_event(Event::VestingScheduleUpdated(new_schedule));
			Ok(())
		}

		/// Unlock vested balance according to the schedule.
		///
		/// The dispatch origin for this call must be _Signed_ and the sender must have funds still
		/// locked under this pallet.
		///
		/// Emits either `VestingCompleted` or `VestingUpdated`.
		#[pallet::weight(10_000)]
		pub fn vest(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = T::Timestamp::now().as_secs();
			// Ensure signer can claim once time is up to schedule.
			ensure!(
				Some(now) >= VestingSchedule::<T>::get().first().map(|v| v.1),
				Error::<T>::ClaimTooEarly
			);

			Self::update_lock(&who)
		}

		/// Create a vested transfer: send `target` balance with the vesting schedule.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `target`: The account receiving the vested funds.
		/// - `locked_amount`: How much tokens will be transfered.
		#[pallet::weight(10_000)]
		pub fn vested_transfer(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			locked_amount: BalanceOf<T>,
		) -> DispatchResult {
			let transactor = ensure_signed(origin)?;
			ensure!(
				locked_amount >= T::MinVestedTransfer::get(),
				Error::<T>::AmountLow
			);

			ensure!(
				T::Currency::free_balance(&transactor) >= locked_amount,
				Error::<T>::BalanceLow
			);

			let who = T::Lookup::lookup(target)?;
			ensure!(
				!VestingBalances::<T>::contains_key(&who),
				Error::<T>::ExistingVestingSchedule
			);

			T::Currency::transfer(
				&transactor,
				&who,
				locked_amount,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::new_vesting_account(&who, locked_amount)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// (Re)set pallet's currency lock on `who`'s account in accordance with their
	/// current unvested amount.
	fn update_lock(who: &T::AccountId) -> DispatchResult {
		let vesting = Self::vesting_balance(&who).ok_or(Error::<T>::NotVesting)?;
		let now = T::Timestamp::now().as_secs();

		// compute the vested portion
		let mut portion = Percent::default();
		for (percentage, timestamp) in VestingSchedule::<T>::get() {
			if now < timestamp {
				break;
			} else {
				portion = portion.saturating_add(percentage);
			}
		}

		let unvested = (Percent::from_percent(100) - portion) * vesting;

		if unvested.is_zero() {
			T::Currency::remove_lock(VESTING_ID, who);
			VestingBalances::<T>::remove(&who);
			Self::deposit_event(Event::<T>::VestingCompleted(who.clone()));
		} else {
			let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
			T::Currency::set_lock(VESTING_ID, who, unvested, reasons);
			Self::deposit_event(Event::<T>::VestingUpdated(who.clone(), unvested));
		}
		Ok(())
	}

	/// Adds a vesting schedule to a given account.
	fn new_vesting_account(who: &T::AccountId, locked: BalanceOf<T>) -> DispatchResult {
		if locked.is_zero() {
			return Ok(());
		}
		if VestingBalances::<T>::contains_key(&who) {
			return Err(Error::<T>::ExistingVestingSchedule.into());
		}

		VestingBalances::<T>::insert(&who, locked);
		// it can't fail, but even if somehow it did, we don't really care.
		Self::update_lock(who)
	}
}
