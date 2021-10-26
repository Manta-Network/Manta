#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
use sp_std::time::Duration;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const VESTING_ID: LockIdentifier = *b"mantavst";

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

		// Store schedules about when and how much tokens will be released.
		type VestingSchedule: Get<[(Percent, Duration); 7]>;
	}

	/// Information regarding the vesting of a given account.
	#[pallet::storage]
	#[pallet::getter(fn vesting)]
	pub type VestingBalances<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The amount vested has been updated. This could indicate more funds are available. The
		/// balance given is the amount which is left unvested (and thus locked).
		/// \[account, unvested\]
		VestingUpdated(T::AccountId, BalanceOf<T>),
		/// An \[account\] has become fully vested. No further vesting can happen.
		VestingCompleted(T::AccountId),
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
		/// Not ready for vesting.
		NotReadyForVesting,
		/// Not enough tokens for vesting.
		BalanceLow,
		/// The size of new schedule is wrong.
		InvalidScheduleLength,
		/// Invalid block number.
		InvalidBlockNumber,
		/// Invalid order of block numbers.
		UnsortedBlockNumbers,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn vest(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::update_lock(&who)
		}

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

			Self::add_vesting_schedule(&who, locked_amount)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// (Re)set pallet's currency lock on `who`'s account in accordance with their
	/// current unvested amount.
	fn update_lock(who: &T::AccountId) -> DispatchResult {
		let vesting = Self::vesting(&who).ok_or(Error::<T>::NotVesting)?;
		let now = T::Timestamp::now();

		// compute the vested portion
		let mut portion = Percent::default();
		for (percentage, block_number) in T::VestingSchedule::get() {
			if now < block_number {
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

	fn add_vesting_schedule(who: &T::AccountId, locked: BalanceOf<T>) -> DispatchResult {
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
