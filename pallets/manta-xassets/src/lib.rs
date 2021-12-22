#![cfg_attr(not(feature = "std"), no_std)]

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
use manta_primitives::{
	currency_id::{CurrencyId, TokenSymbol},
	traits::XCurrency,
};
use sp_runtime::{traits::Member, SaturatedConversion};
use sp_std::{vec, vec::Vec};
use xcm::latest::prelude::*;
use xcm::v1::{
	AssetId, Fungibility, Junction, Junctions, MultiAsset, MultiAssetFilter, MultiAssets,
	MultiLocation, WildMultiAsset,
};
use xcm::v2::{ExecuteXcm, Instruction, WeightLimit, Xcm as XcmV2};
use xcm_executor::traits::{Convert, WeightBounds};

pub use pallet::*;
// Log filter
const MANTA_XASSETS: &str = "manta-xassets";
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Something to execute an XCM message.
		type XcmExecutor: ExecuteXcm<Self::Call>;

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

	// This is an workaround for depositing/withdrawing cross chain tokens
	// Finally, we'll utilize pallet-assets to handle these external tokens.
	#[pallet::storage]
	pub type XTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CurrencyId,
		Blake2_128Concat,
		T::AccountId,
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::generate_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Attempted(Outcome),
		/// Deposit success. [asset, to]
		Deposited(T::AccountId, CurrencyId, BalanceOf<T>),
		/// Withdraw success. [asset, from]
		Withdrawn(T::AccountId, CurrencyId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		BalanceLow,
		SelfChain,
		BadAccountIdToMultiLocation,
		UnweighableMessage,
		NotSupportedToken,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
			currency_id: CurrencyId,
			#[pallet::compact] amount: BalanceOf<T>,
			weight: Weight,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			ensure!(T::SelfParaId::get() != para_id, Error::<T>::SelfChain);

			match currency_id {
				CurrencyId::Token(TokenSymbol::MA) | CurrencyId::Token(TokenSymbol::KMA) => {
					ensure!(
						T::Currency::free_balance(&from) >= amount,
						Error::<T>::BalanceLow
					);
				}
				CurrencyId::Token(TokenSymbol::ACA)
				| CurrencyId::Token(TokenSymbol::KAR)
				| CurrencyId::Token(TokenSymbol::SDN) => {
					ensure!(
						Self::account(currency_id, &from) >= amount,
						Error::<T>::BalanceLow
					);
				}
				_ => return Err(Error::<T>::NotSupportedToken.into()),
			}

			let xcm_origin = T::Conversion::reverse(from)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;

			// create sibling parachain target
			let beneficiary = T::Conversion::reverse(dest)
				.map_err(|_| Error::<T>::BadAccountIdToMultiLocation)?;

			let amount = amount.saturated_into::<u128>();
			let para_id = para_id.saturated_into::<u32>();

			let fungibility = Fungibility::Fungible(amount);
			let junctions = Junctions::X2(
				Junction::Parachain(para_id),
				Junction::GeneralKey(currency_id.encode()),
			);
			let multi_location = MultiLocation::new(0, junctions);
			let asset_id = AssetId::Concrete(multi_location.clone());
			let multi_asset = MultiAsset {
				id: asset_id,
				fun: fungibility,
			};
			// Todo, handle weight_limit

			let mut xcm = XcmV2(vec![
				Instruction::WithdrawAsset(MultiAssets::from(vec![multi_asset.clone()])),
				Instruction::DepositReserveAsset {
					assets: MultiAssetFilter::Wild(WildMultiAsset::All),
					max_assets: 1,
					dest: multi_location.clone(),
					xcm: XcmV2(vec![
						Instruction::BuyExecution {
							fees: multi_asset,
							weight_limit: WeightLimit::Limited(100_000_000_000),
						},
						Instruction::DepositAsset {
							assets: MultiAssetFilter::Wild(WildMultiAsset::All),
							max_assets: 1,
							beneficiary,
						},
					]),
				},
			]);

			log::info!(target: MANTA_XASSETS, "xcm = {:?}", xcm);

			let xcm_weight =
				T::Weigher::weight(&mut xcm).map_err(|()| Error::<T>::UnweighableMessage)?;

			// The last param is the weight we buy on target chain.
			let outcome =
				T::XcmExecutor::execute_xcm_in_credit(xcm_origin, xcm, xcm_weight, xcm_weight);
			log::info!(target: MANTA_XASSETS, "xcm_outcome = {:?}", outcome);

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

	impl<T: Config> XCurrency<T::AccountId> for Pallet<T> {
		type Balance = BalanceOf<T>;
		type CurrencyId = CurrencyId;

		fn account(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
			XTokens::<T>::get(currency_id, who)
		}

		/// Add `amount` to the balance of `who` under `currency_id`
		fn deposit(
			currency_id: Self::CurrencyId,
			who: &T::AccountId,
			amount: Self::Balance,
		) -> DispatchResult {
			XTokens::<T>::mutate(currency_id, who, |balance| {
				// *balance = balance.saturated_add(amount);
				*balance += amount;
			});

			Self::deposit_event(Event::Deposited(who.clone(), currency_id, amount));

			Ok(())
		}

		/// Remove `amount` from the balance of `who` under `currency_id`
		fn withdraw(
			currency_id: Self::CurrencyId,
			who: &T::AccountId,
			amount: Self::Balance,
		) -> DispatchResult {
			XTokens::<T>::mutate(currency_id, who, |balance| {
				// *balance = balance.saturated_add(amount);
				*balance -= amount;
			});

			Self::deposit_event(Event::Withdrawn(who.clone(), currency_id, amount));

			Ok(())
		}
	}
}
