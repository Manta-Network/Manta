#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use cumulus_primitives_core::ParaId;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, Get, Hooks, IsType, ReservableCurrency},
	PalletId,
};
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
use sp_runtime::SaturatedConversion;
use sp_std::{vec, vec::Vec};
use xcm::v0::{ExecuteXcm, Junction, MultiAsset, MultiLocation, Order, Outcome, Xcm};
use xcm_executor::traits::{Convert, WeightBounds};

pub use pallet::*;
// Log filter
const MANTA_XASSETS: &str = "manta-xassets";
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Something to execute an XCM message.
		type XcmExecutor: ExecuteXcm<Self::Call>;

		/// Some parachains manta trusts.
		type TrustedChains: Get<Vec<(MultiLocation, u128)>>;

		/// Convert AccountId to MultiLocation.
		type Conversion: Convert<MultiLocation, Self::AccountId>;

		/// This pallet id.
		type PalletId: Get<PalletId>;

		type Currency: ReservableCurrency<Self::AccountId>;

		/// Manta's parachain id.
		type SelfParaId: Get<ParaId>;

		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Attempted(Outcome),
	}

	#[pallet::error]
	pub enum Error<T> {
		BalanceLow,
		SelfChain,
		BadAccountIdToMultiLocation,
		UnweighableMessage,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Teleport manta tokens to sibling parachain.
		///
		/// - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
		/// - `para_id`: Sibling parachain id.
		/// - `dest`: Who will receive foreign tokens on sibling parachain.
		/// - `amount`: How many tokens will be transferred.
		#[pallet::weight(10000)]
		pub fn teleport_to_parachain(
			origin: OriginFor<T>,
			para_id: ParaId,
			dest: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			ensure!(T::SelfParaId::get() != para_id, Error::<T>::SelfChain);
			ensure!(
				T::Currency::free_balance(&from) >= amount,
				Error::<T>::BalanceLow
			);
			let xcm_origin = T::Conversion::reverse(from)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;

			// create sibling parachain target
			let xcm_target = T::Conversion::reverse(dest)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;

			// target chain location
			let receiver_chain =
				MultiLocation::X2(Junction::Parent, Junction::Parachain(para_id.into()));

			let amount = amount.saturated_into::<u128>();

			// create friend parachain xcm
			let xcm = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain(T::SelfParaId::get().into()),
					),
					amount,
				}],
				effects: vec![Order::InitiateTeleport {
					assets: vec![MultiAsset::All],
					dest: receiver_chain,
					effects: vec![
						// Todo, just disable this order, it doesn't work for now.
						// Order::BuyExecution {
						// 	fees: MultiAsset::All,
						// 	weight: 0,
						// 	debt: 3000_000_000,
						// 	halt_on_error: false,
						// 	xcm: vec![],
						// },
						Order::DepositAsset {
							assets: vec![MultiAsset::All],
							dest: xcm_target,
						},
					],
				}],
			};

			// Todo, just disable this line, it doesn't work for now.
			// let weight =
			// 	T::Weigher::weight(&mut friend_xcm).map_err(|()| Error::<T>::UnweighableMessage)?;

			// The last param is the weight we buy on target chain.
			let outcome =
				T::XcmExecutor::execute_xcm_in_credit(xcm_origin, xcm, 3000000000, 3000000000);
			log::info!(target: MANTA_XASSETS, "xcm_outcome = {:?}", outcome);

			Self::deposit_event(Event::Attempted(outcome));

			Ok(())
		}

		/// Transfer manta tokens to sibling parachain.
		///
		/// - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
		/// - `para_id`: Sibling parachain id.
		/// - `dest`: Who will receive foreign tokens on sibling parachain.
		/// - `amount`: How many tokens will be transferred.
		/// - `weight`: Specify the weight of xcm.
		#[pallet::weight(10000)]
		pub fn transfer_to_parachain(
			origin: OriginFor<T>,
			para_id: ParaId,
			dest: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
			weight: Weight,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			ensure!(T::SelfParaId::get() != para_id, Error::<T>::SelfChain);
			ensure!(
				T::Currency::free_balance(&from) >= amount,
				Error::<T>::BalanceLow
			);
			let xcm_origin = T::Conversion::reverse(from)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;

			// create sibling parachain target
			let xcm_target = T::Conversion::reverse(dest)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;

			// target chain location
			let receiver_chain =
				MultiLocation::X2(Junction::Parent, Junction::Parachain(para_id.into()));

			let amount = amount.saturated_into::<u128>();

			// create friend parachain xcm
			let xcm = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain(T::SelfParaId::get().into()),
					),
					amount,
				}],
				effects: vec![Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: receiver_chain,
					effects: vec![
						// Todo, just disable this order, it doesn't work for now.
						// Order::BuyExecution {
						// 	fees: MultiAsset::All,
						// 	weight: 0,
						// 	debt: 3000_000_000,
						// 	halt_on_error: false,
						// 	xcm: vec![],
						// },
						Order::DepositAsset {
							assets: vec![MultiAsset::All],
							dest: xcm_target,
						},
					],
				}],
			};

			// Todo, just disable this line, it doesn't work for now.
			// let weight =
			// 	T::Weigher::weight(&mut friend_xcm).map_err(|()| Error::<T>::UnweighableMessage)?;

			// The last param is the weight we buy on target chain.
			let outcome = T::XcmExecutor::execute_xcm_in_credit(xcm_origin, xcm, weight, weight);
			log::info!(target: MANTA_XASSETS, "xcm_outcome = {:?}", outcome);

			Self::deposit_event(Event::Attempted(outcome));

			Ok(())
		}

		/// Transfer DOT to relaychain.
		///
		/// - `origin`: Must be capable of withdrawing the `assets` and executing UP.
		/// - `dest`: Who will receive DOT on relaychain.
		/// - `amount`: How many tokens will be transferred.
		/// - `weight`: Specify the weight of um.
		#[pallet::weight(10000)]
		pub fn transfer_to_relaychain(
			origin: OriginFor<T>,
			dest: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
			weight: Weight,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let um_origin = T::Conversion::reverse(from)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;
			// Todo, ensure this caller has enough DOT to transfer.

			// create relaychain target
			let um_target = T::Conversion::reverse(dest)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;
			let amount = amount.saturated_into::<u128>();

			// create friend relaychain xcm
			let um = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Junction::Parent),
					amount,
				}],
				effects: vec![Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain(T::SelfParaId::get().into()),
					),
					effects: vec![Order::DepositAsset {
						assets: vec![MultiAsset::All],
						dest: um_target,
					}],
				}],
			};

			let outcome = T::XcmExecutor::execute_xcm_in_credit(um_origin, um, weight, weight);
			log::info!(target: MANTA_XASSETS, "um_outcome = {:?}", outcome);

			Self::deposit_event(Event::Attempted(outcome));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub place_holder: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			Self {
				place_holder: PhantomData,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {}
	}
}
