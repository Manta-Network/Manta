#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use cumulus_primitives_core::ParaId;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Get, Hooks, IsType, Randomness, ReservableCurrency},
	transactional, PalletId,
};
use frame_system::{
	ensure_root, ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
use sp_runtime::{
	traits::{Saturating, Zero},
	DispatchResult, Permill, SaturatedConversion,
};
use sp_std::{vec, vec::Vec};
use xcm::v0::{
	Error as XcmError, ExecuteXcm, Junction, MultiAsset, MultiLocation, Order, Outcome,
	Result as XcmResult, Xcm,
};
use xcm_executor::{
	traits::{Convert, FilterAssetLocation, TransactAsset, WeightBounds},
	Assets,
};

pub use pallet::*;
// Log filter
const MANTA_XASSETS: &str = "manta-xassets";
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type XcmExecutor: ExecuteXcm<Self::Call>;
		type FriendChains: Get<Vec<(MultiLocation, u128)>>;
		type Conversion: Convert<MultiLocation, Self::AccountId>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type SelfParaId: Get<ParaId>;
		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// pub struct Pallet<T>(PhantomData<T>);
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {}
	
	#[pallet::error]
	pub enum Error<T> {
		BalanceLow,
		SelfChain,
		UnweighableMessage,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// transfer to parachain
		#[pallet::weight(10000)]
		fn transfer_to_parachain(
			origin: OriginFor<T>,
			para_id: ParaId,
			dest: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(T::SelfParaId::get() != para_id, Error::<T>::SelfChain);

			// create friend parachain target
			let friend_chain_target = T::Conversion::reverse(dest.clone())
				.expect("failed to create friend chain target origin");

			// friend chain location
			let asset_location =
				MultiLocation::X2(Junction::Parent, Junction::Parachain(u32::from(para_id)));

			log::info! {target: MANTA_XASSETS, "amount = {:?}", amount};
			log::info! {target: MANTA_XASSETS, "asset_location = {:?}", asset_location};

			let amount = amount.saturated_into::<u128>();

			// create friend parachain xcm
			let mut friend_xcm = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: asset_location.clone(),
					amount,
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: asset_location.clone(),
					effects: vec![
						Order::BuyExecution {
							fees: MultiAsset::All,
							weight: 0,
							debt: 300_000_000_000,
							halt_on_error: false,
							xcm: vec![],
						},
						Order::DepositReserveAsset {
							assets: vec![MultiAsset::All],
							dest: asset_location.clone(),
							effects: vec![Order::DepositAsset {
								assets: vec![MultiAsset::All],
								dest: asset_location,
							}],
						}],
				}],
			};

			log::info! {target: MANTA_XASSETS, "friend_xcm = {:?}", friend_xcm};

			let weight = T::Weigher::weight(&mut friend_xcm).map_err(|()| Error::<T>::UnweighableMessage)?;

			// The last param is the weight we buy on target chain.
			let xcm_outcome = T::XcmExecutor::execute_xcm_in_credit(
				friend_chain_target,
				friend_xcm.into(),
				weight,
				weight,
			);
			log::info! {target: MANTA_XASSETS, "xcm_outcome = {:?}", xcm_outcome};

			Ok(().into())
		}

		/// transfer to relaychain
		#[pallet::weight(10000)]
		fn transfer_to_relaychain(
			origin: OriginFor<T>,
			dest: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			// create friend relaychain target
			let friend_chain_target = T::Conversion::reverse(dest.clone())
				.expect("failed to create friend chain target origin");

			// friend chain location
			let asset_location = MultiLocation::X1(Junction::Parent);

			log::info! {target: MANTA_XASSETS, "amount = {:?}", amount};
			log::info! {target: MANTA_XASSETS, "asset_location = {:?}", asset_location};

			let amount = amount.saturated_into::<u128>();

			// create friend relaychain xcm
			let friend_xcm = Xcm::<T>::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: asset_location.clone(),
					amount,
				}],
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: asset_location,
				}],
			};

			log::info! {target: MANTA_XASSETS, "friend_xcm = {:?}", friend_xcm};

			let xcm_outcome =
				T::XcmExecutor::execute_xcm(friend_chain_target, friend_xcm.into(), 300_0000);

			log::info! {target: MANTA_XASSETS, "xcm_outcome = {:?}", xcm_outcome};

			Ok(().into())
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
