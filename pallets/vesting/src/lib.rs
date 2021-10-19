#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		Currency, ExistenceRequirement, Get, LockIdentifier, LockableCurrency, WithdrawReasons,
	},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Saturating, StaticLookup, Zero},
	Permill, RuntimeDebug,
};
use sp_std::prelude::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const VESTING_ID: LockIdentifier = *b"mantavst";
pub type Round = u32;
pub type IsClaimed = bool;

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VestingInfo<T: Config> {
	/// Locked amount at genesis.
	pub total_vesting: BalanceOf<T>,
	/// Record how many rounds user has claimed.
	pub round: Round,
}

impl<T: Config> VestingInfo<T> {
	// 
	pub fn locked_at(&self, who: &T::AccountId, n: T::BlockNumber) -> (BalanceOf<T>, Round) {
		// Calculate how many rounds user can claim.

		let mut current_round = self.round;
		let mut portion = Permill::default();
		for (index, schedule) in T::VestingSchedule::get().iter().enumerate() {
			if n < schedule.1 {
				current_round = index as Round + 1;
				break;
			} else {
				if self.round >= index as Round {
					portion = portion.saturating_add(schedule.0);
				}
			}
		}

		// move this part out if this method to update_lock
		// let _: Result<_, Error<T>> = Vesting::<T>::try_mutate(&who, |info| {
		// 	match info.as_mut() {
		// 		Some(v) => {
		// 			if v.round < current_round {
		// 				v.round = current_round;
		// 			}
		// 		}
		// 		_ => (),
		// 	}

		// 	Ok(())
		// });

		let locked = (Permill::from_percent(100) - portion) * self.locked;

		(locked, current_round)
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency trait.
		type Currency: LockableCurrency<Self::AccountId>;

		/// The minimum amount transferred to call `vested_transfer`.
		#[pallet::constant]
		type MinVestedTransfer: Get<BalanceOf<Self>>;

		type VestingSchedule: Get<Vec<(Permill, BlockNumberFor<Self>)>>;
	}

	/// Information regarding the vesting of a given account.
	#[pallet::storage]
	#[pallet::getter(fn vesting)]
	pub type Vesting<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, VestingInfo<T>>;

	/// Store schedules about when and how much tokens will be released.
	#[pallet::storage]
	#[pallet::getter(fn vesting_schedules)]
	pub type VestingSchedules<T: Config> =
		StorageValue<_, Vec<(Permill, BlockNumberFor<T>)>, ValueQuery>;

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn edit_vesting_info(
			origin: OriginFor<T>,
			timepoint: Vec<BlockNumberFor<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			todo!();
		}

		#[pallet::weight(10_000)]
		pub fn vest(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::update_lock(who);
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
				!Vesting::<T>::contains_key(&who),
				Error::<T>::ExistingVestingSchedule
			);

			T::Currency::transfer(
				&transactor,
				&who,
				locked_amount,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::add_vesting_schedule(&who, locked_amount)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	// /// (Re)set or remove the pallet's currency lock on `who`'s account in accordance with their
	// /// current unvested amount.
	fn update_lock(who: T::AccountId) -> DispatchResult {
		let vesting = Self::vesting(&who).ok_or(Error::<T>::NotVesting)?;
		let now = <frame_system::Pallet<T>>::block_number();
		let locked_now = vesting.locked_at(&who, now);

		// try to remove if, lock 0 amount of token.
		if locked_now.is_zero() {
			T::Currency::remove_lock(VESTING_ID, &who);
			Vesting::<T>::remove(&who);
			Self::deposit_event(Event::<T>::VestingCompleted(who));
		} else {
			let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
			T::Currency::set_lock(VESTING_ID, &who, locked_now, reasons);
			Self::deposit_event(Event::<T>::VestingUpdated(who, locked_now));
		}
		Ok(())
	}

	fn add_vesting_schedule(who: &T::AccountId, locked: BalanceOf<T>) -> DispatchResult {
		if locked.is_zero() {
			return Ok(());
		}
		if Vesting::<T>::contains_key(who) {
			Err(Error::<T>::ExistingVestingSchedule)?
		}

		let vesting_schedule = VestingInfo { locked, round: 0 };
		Vesting::<T>::insert(who, vesting_schedule);
		// it can't fail, but even if somehow it did, we don't really care.
		let _res = Self::update_lock(who.clone());

		Ok(())
	}
}
