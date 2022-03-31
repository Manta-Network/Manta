// Copyright 2019-2022 Manta Network.
// This file is part of pallet-manta-pay.
//
// pallet-manta-pay is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pallet-manta-pay is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pallet-manta-pay.  If not, see <http://www.gnu.org/licenses/>.

//! # MantaPay Module
//!
//! MantaPay is a Multi-Asset Shielded Payment protocol.
//!
//! _NB_: The design is similar though not the same with MASP (Multi-Asset Shielded Pool).
//!
//! ## Overview
//!
//! The Assets module provides functionality for asset management of fungible asset classes
//! with a fixed supply, including:
//!
//! * Private Asset Mint
//! * Private Asset Transfer
//! * Private Asset Reclaim
//!
//! To use it in your runtime, you need to implement the assets [`Config`](./config.Config.html).
//!
//! The supported dispatchable functions are documented in the [`Call`](./enum.Call.html) enum.
//!
//! ### Terminology
//!
//! * **Private asset mint:** The action of converting certain number of `Asset`s into an UTXO
//!     that holds same number of private assets.
//! * **Private asset transfer:** The action of transferring certain number of private assets from
//!     two UTXOs to another two UTXOs.
//! * **Private asset reclaim:** The action of transferring certain number of private assets from
//!     two UTXOs to another UTXO, and converting the remaining private assets back to public
//!     assets.
//!
//! The assets system in Manta is designed to make the following possible:
//!
//! * Issue a public asset to its creator's account.
//! * Move public assets between accounts.
//! * Converting public assets to private assets, and vice versa.
//! * Move private assets between accounts (in UTXO model).
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `to_private` - Converting an `amount` of units of fungible asset `id` from the caller
//!     to a private UTXO. (The caller does not need to be the owner of this UTXO)
//! * `private_transfer` - Transfer two input UTXOs into two output UTXOs. Require that 1) the input
//!     UTXOs are already in the ledger and are not spend before 2) the sum of private assets in
//!     input UTXOs matches that of the output UTXOs. The requirements are guaranteed via ZK proof.
//! * `to_public` - Transfer two input UTXOs into one output UTXOs, and convert the remaining assets
//!     to the public assets. Require that 1) the input UTXOs are already in the ledger and are not
//!     spend before; 2) the sum of private assets in input UTXOs matches that of the output UTXO +
//!     the reclaimed amount. The requirements are guaranteed via ZK proof.
//!
//! Please refer to the [`Call`](./enum.Call.html) enum and its associated variants for
//! documentation on each function.
//!
//! Please refer to the [`Module`](./struct.Module.html) struct for details on publicly available
//! functions.
//!
//! ## Usage
//!
//! The following example shows how to use the Assets module in your runtime by exposing public
//! functions to:
//!
//! * Initiate the fungible asset for a token distribution event (airdrop).
//! * Query the fungible asset holding balance of an account.
//! * Query the total supply of a fungible asset that has been issued.
//! * Query the total number of private fungible asset that has been minted and not reclaimed.
//!
//! ### Prerequisites
//!
//! Import the Assets module and types and derive your runtime's configuration traits from the
//! Assets module trait.
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::marker::PhantomData;
use frame_support::{transactional, PalletId};
use manta_accounting::{
	asset,
	transfer::{
		canonical::TransferShape, InvalidSinkAccount, InvalidSourceAccount, Proof, ReceiverLedger,
		ReceiverPostError, ReceiverPostingKey, SenderLedger, SenderPostError, SenderPostingKey,
		SinkPostingKey, SourcePostingKey, TransferLedger, TransferLedgerSuperPostingKey,
		TransferPostError,
	},
};
use manta_crypto::{
	constraint::ProofSystem,
	merkle_tree::{self, forest::Configuration as _},
};
use manta_pay::config;
use manta_primitives::{
	assets::{FungibleLedger, FungibleLedgerConsequence},
	types::{AssetId, Balance},
};
use manta_util::codec::Decode as _;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{vec, vec::Vec};
use types::*;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

pub mod types;
pub mod weights;

/// MantaPay Pallet
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use manta_primitives::assets::AssetConfig;
	use sp_runtime::traits::AccountIdConversion;

	/// Pallet
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Asset Configuration
		type AssetConfig: AssetConfig;

		/// Fungible ledger
		type FungibleLedger: FungibleLedger<Self>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Pallet ID
		type PalletId: Get<PalletId>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Shards of the merkle tree of UTXOs
	#[pallet::storage]
	pub(super) type Shards<T: Config> =
		StorageDoubleMap<_, Identity, u8, Identity, u64, (config::Utxo, EncryptedNote), ValueQuery>;

	/// Shard merkle trees
	#[pallet::storage]
	pub(super) type ShardTrees<T: Config> =
		StorageMap<_, Identity, u8, UtxoMerkleTreePath, ValueQuery>;

	/// Outputs of Utxo accumulator
	#[pallet::storage]
	pub(super) type UtxoAccumulatorOutputs<T: Config> =
		StorageMap<_, Identity, config::UtxoAccumulatorOutput, (), ValueQuery>;

	/// Utxo set of MantaPay protocol
	#[pallet::storage]
	pub(super) type UtxoSet<T: Config> = StorageMap<_, Identity, config::Utxo, (), ValueQuery>;

	/// Void number set
	#[pallet::storage]
	pub(super) type VoidNumberSet<T: Config> =
		StorageMap<_, Identity, config::VoidNumber, (), ValueQuery>;

	/// Void number set insertion order
	/// Each element of the key is an `u64` insertion order number of void number.
	#[pallet::storage]
	pub(super) type VoidNumberSetInsertionOrder<T: Config> =
		StorageMap<_, Identity, u64, config::VoidNumber, ValueQuery>;

	/// The size of Void Number Set
	/// FIXME: this should be removed.
	#[pallet::storage]
	pub(super) type VoidNumberSetSize<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Convert the asset encoded in `post` to private asset.
		///
		/// `origin`: the owner of the public asset. `origin` will pay the gas fee for the conversion as well.
		/// `post`: encoded asset to be converted.
		#[pallet::weight(T::WeightInfo::to_private())]
		#[transactional]
		pub fn to_private(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let mut ledger = Self::ledger();
			Self::deposit_event(
				config::TransferPost::try_from(post)
					.map_err(|_| Error::<T>::InvalidSerializedForm)?
					.post(vec![origin], vec![], &(), &mut ledger)
					.map_err(Error::<T>::from)?
					.convert(None),
			);
			Ok(().into())
		}

		/// Transfers private assets encoded in `post`.
		///
		/// # Note
		///
		/// In this transaction, `origin` is just signing the `post` and is not necessarily related
		/// to any of the participants in the transaction itself.
		#[pallet::weight(T::WeightInfo::private_transfer())]
		#[transactional]
		pub fn private_transfer(
			origin: OriginFor<T>,
			post: TransferPost,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let mut ledger = Self::ledger();
			Self::deposit_event(
				config::TransferPost::try_from(post)
					.map_err(|_| Error::<T>::InvalidSerializedForm)?
					.post(vec![], vec![], &(), &mut ledger)
					.map_err(Error::<T>::from)?
					.convert(Some(origin)),
			);
			Ok(().into())
		}

		/// Transforms some private assets into public ones using `post`, sending the public assets
		/// to the `origin` account.
		#[pallet::weight(T::WeightInfo::to_public())]
		#[transactional]
		pub fn to_public(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let mut ledger = Self::ledger();
			Self::deposit_event(
				config::TransferPost::try_from(post)
					.map_err(|_| Error::<T>::InvalidSerializedForm)?
					.post(vec![], vec![origin], &(), &mut ledger)
					.map_err(Error::<T>::from)?
					.convert(None),
			);
			Ok(().into())
		}
	}

	/// Event
	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Mint Event
		ToPrivate {
			/// Asset Minted
			asset: Asset,

			/// Source Account
			source: T::AccountId,
		},

		/// Private Transfer Event
		PrivateTransfer {
			/// Origin Account
			origin: T::AccountId,
		},

		/// Reclaim Event
		ToPublic {
			/// Asset Reclaimed
			asset: Asset,

			/// Sink Account
			sink: T::AccountId,
		},
	}

	/// Error
	#[pallet::error]
	pub enum Error<T> {
		/// Uninitialized Supply
		///
		/// Supply of the given Asset Id has not yet been initialized.
		UninitializedSupply,

		/// Zero Transfer
		///
		/// Public transfers cannot include amounts equal to zero.
		ZeroTransfer,

		/// Balance Low
		///
		/// Attempted to withdraw from balance which was smaller than the withdrawl amount.
		BalanceLow,

		/// Invalid Serialized Form
		///
		/// The transfer could not be interpreted because of an issue during deserialization.
		InvalidSerializedForm,

		/// Invalid Shape
		///
		/// The transfer had an invalid shape.
		InvalidShape,

		/// Asset Spent
		///
		/// An asset present in this transfer has already been spent.
		AssetSpent,

		/// Invalid UTXO Accumulator Output
		///
		/// The sender was constructed on an invalid version of the ledger state.
		InvalidUtxoAccumulatorOutput,

		/// Asset Registered
		///
		/// An asset present in this transfer has already been registered to the ledger.
		AssetRegistered,

		/// Duplicate Spend
		///
		/// There were multiple spend entries for the same underlying asset in this transfer.
		DuplicateSpend,

		/// Duplicate Register
		///
		/// There were multiple register entries for the same underlying asset in this transfer.
		DuplicateRegister,

		/// Invalid Proof
		///
		/// The submitted proof did not pass validation, or errored during validation.
		InvalidProof,

		/// Ledger Internal Error
		///
		/// Internal error caused by ledger internals (Ideally should never happen).
		LedgerUpdateError,

		/// Invalid Source Account
		///
		/// At least one of the source accounts is invalid.
		InvalidSourceAccount,

		/// Invalid Sink Account
		///
		/// At least one of the sink accounts in invalid.
		InvalidSinkAccount,
	}

	impl<T> From<InvalidSourceAccount<T::AccountId>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(_: InvalidSourceAccount<T::AccountId>) -> Self {
			Self::InvalidSourceAccount
		}
	}

	impl<T> From<InvalidSinkAccount<T::AccountId>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(_: InvalidSinkAccount<T::AccountId>) -> Self {
			Self::InvalidSinkAccount
		}
	}

	impl<T> From<SenderPostError> for Error<T> {
		#[inline]
		fn from(err: SenderPostError) -> Self {
			match err {
				SenderPostError::AssetSpent => Self::AssetSpent,
				SenderPostError::InvalidUtxoAccumulatorOutput => Self::InvalidUtxoAccumulatorOutput,
			}
		}
	}

	impl<T> From<ReceiverPostError> for Error<T> {
		#[inline]
		fn from(err: ReceiverPostError) -> Self {
			match err {
				ReceiverPostError::AssetRegistered => Self::AssetRegistered,
			}
		}
	}

	impl<T, Balance> From<TransferPostError<T::AccountId, Balance>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(err: TransferPostError<T::AccountId, Balance>) -> Self {
			match err {
				TransferPostError::InvalidShape => Self::InvalidShape,
				TransferPostError::InvalidSourceAccount(err) => err.into(),
				TransferPostError::InvalidSinkAccount(err) => err.into(),
				TransferPostError::Sender(err) => err.into(),
				TransferPostError::Receiver(err) => err.into(),
				TransferPostError::DuplicateSpend => Self::DuplicateSpend,
				TransferPostError::DuplicateRegister => Self::DuplicateRegister,
				TransferPostError::InvalidProof => Self::InvalidProof,
				TransferPostError::UpdateError(_) => Self::LedgerUpdateError,
			}
		}
	}

	impl<T> Pallet<T>
	where
		T: Config,
	{
		/// Returns the ledger implementation for this pallet.
		#[inline]
		fn ledger() -> Ledger<T> {
			Ledger(PhantomData)
		}

		/// The account ID of AssetManager
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}
	}
}

/// Preprocessed Event
pub enum PreprocessedEvent<T>
where
	T: Config,
{
	/// To Private Event
	ToPrivate {
		/// Asset Minted
		asset: Asset,

		/// Source Account
		source: T::AccountId,
	},

	/// Private Transfer Event
	PrivateTransfer,

	/// To Public Event
	ToPublic {
		/// Asset Reclaimed
		asset: Asset,

		/// Sink Account
		sink: T::AccountId,
	},
}

impl<T> PreprocessedEvent<T>
where
	T: Config,
{
	/// Converts a [`PreprocessedEvent`] with into an [`Event`] using the given `origin` for
	/// [`PreprocessedEvent::PrivateTransfer`].
	#[inline]
	pub fn convert(self, origin: Option<T::AccountId>) -> Event<T> {
		match self {
			Self::ToPrivate { asset, source } => Event::ToPrivate { asset, source },
			Self::PrivateTransfer => Event::PrivateTransfer {
				// FIXME: get rid of unwrap eventually.
				origin: origin.unwrap(),
			},
			Self::ToPublic { asset, sink } => Event::ToPublic { asset, sink },
		}
	}
}

/// Ledger
pub struct Ledger<T>(PhantomData<T>)
where
	T: Config;

/// Wrap Type
#[derive(Clone, Copy)]
pub struct Wrap<T>(T);

impl<T> AsRef<T> for Wrap<T> {
	#[inline]
	fn as_ref(&self) -> &T {
		&self.0
	}
}

/// Wrap Pair Type
#[derive(Clone, Copy)]
pub struct WrapPair<L, R>(L, R);

impl<L, R> AsRef<R> for WrapPair<L, R> {
	#[inline]
	fn as_ref(&self) -> &R {
		&self.1
	}
}

impl<T> SenderLedger<config::Config> for Ledger<T>
where
	T: Config,
{
	type ValidVoidNumber = Wrap<config::VoidNumber>;
	type ValidUtxoAccumulatorOutput = Wrap<config::UtxoAccumulatorOutput>;
	type SuperPostingKey = (Wrap<()>, ());

	#[inline]
	fn is_unspent(&self, void_number: config::VoidNumber) -> Option<Self::ValidVoidNumber> {
		if VoidNumberSet::<T>::contains_key(&void_number) {
			None
		} else {
			Some(Wrap(void_number))
		}
	}

	#[inline]
	fn has_matching_utxo_accumulator_output(
		&self,
		output: config::UtxoAccumulatorOutput,
	) -> Option<Self::ValidUtxoAccumulatorOutput> {
		if UtxoAccumulatorOutputs::<T>::contains_key(output) {
			return Some(Wrap(output));
		}
		None
	}

	#[inline]
	fn spend_all<I>(&mut self, iter: I, super_key: &Self::SuperPostingKey)
	where
		I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidVoidNumber)>,
	{
		let _ = super_key;
		let index = VoidNumberSetSize::<T>::get();
		let mut i = 0;
		for (_, void_number) in iter {
			VoidNumberSet::<T>::insert(void_number.0, ());
			VoidNumberSetInsertionOrder::<T>::insert(index + i, void_number.0);
			i += 1;
		}
		if i != 0 {
			VoidNumberSetSize::<T>::set(index + i);
		}
	}
}

impl<T> ReceiverLedger<config::Config> for Ledger<T>
where
	T: Config,
{
	type ValidUtxo = Wrap<config::Utxo>;
	type SuperPostingKey = (Wrap<()>, ());

	#[inline]
	fn is_not_registered(&self, utxo: config::Utxo) -> Option<Self::ValidUtxo> {
		if UtxoSet::<T>::contains_key(&utxo) {
			None
		} else {
			Some(Wrap(utxo))
		}
	}

	#[inline]
	fn register_all<I>(&mut self, iter: I, super_key: &Self::SuperPostingKey)
	where
		I: IntoIterator<Item = (Self::ValidUtxo, config::EncryptedNote)>,
	{
		let _ = super_key;
		let parameters = config::UtxoAccumulatorModel::decode(
			manta_sdk::pay::testnet::parameters::UtxoAccumulatorModel::get()
				.expect("Checksum did not match."),
		)
		.expect("Unable to decode the Merkle Tree Parameters.");
		let mut shard_indices = iter
			.into_iter()
			.map(move |(utxo, note)| {
				(
					config::MerkleTreeConfiguration::tree_index(&utxo.0),
					utxo.0,
					note,
				)
			})
			.collect::<Vec<_>>();
		shard_indices.sort_by_key(|(s, _, _)| *s);
		let mut shard_insertions = Vec::<(_, Vec<_>)>::new();
		for (shard_index, utxo, note) in shard_indices {
			match shard_insertions.last_mut() {
				Some((index, pairs)) if shard_index == *index => pairs.push((utxo, note)),
				_ => shard_insertions.push((shard_index, vec![(utxo, note)])),
			}
		}
		for (shard_index, insertions) in shard_insertions {
			let mut tree = ShardTrees::<T>::get(shard_index);
			let mut next_root = Option::<config::UtxoAccumulatorOutput>::None;
			let mut current_path = core::mem::take(&mut tree.current_path).into();
			for (utxo, note) in insertions {
				next_root = Some(
					merkle_tree::single_path::raw::insert(
						&parameters,
						&mut tree.leaf_digest,
						&mut current_path,
						utxo,
					)
					.expect("If this errors, then we have run out of Merkle Tree capacity."),
				);
				let next_index = current_path.leaf_index().0 as u64;
				UtxoSet::<T>::insert(utxo, ());
				Shards::<T>::insert(shard_index, next_index, (utxo, EncryptedNote::from(note)));
			}
			tree.current_path = current_path.into();
			if let Some(next_root) = next_root {
				ShardTrees::<T>::insert(shard_index, tree);
				UtxoAccumulatorOutputs::<T>::insert(next_root, ());
			}
		}
	}
}

impl<T> TransferLedger<config::Config> for Ledger<T>
where
	T: Config,
{
	type AccountId = T::AccountId;
	type UpdateError = FungibleLedgerConsequence;
	type Event = PreprocessedEvent<T>;
	type ValidSourceAccount = WrapPair<Self::AccountId, asset::AssetValue>;
	type ValidSinkAccount = WrapPair<Self::AccountId, asset::AssetValue>;
	type ValidProof = Wrap<()>;
	type SuperPostingKey = ();

	#[inline]
	fn check_source_accounts<I>(
		&self,
		asset_id: manta_accounting::asset::AssetId,
		sources: I,
	) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<Self::AccountId>>
	where
		I: Iterator<Item = (Self::AccountId, asset::AssetValue)>,
	{
		sources
			.map(move |(account_id, withdraw)| {
				T::FungibleLedger::can_withdraw(asset_id.0, &account_id, withdraw.0)
					.map(|_| WrapPair(account_id.clone(), withdraw))
					.map_err(|_| InvalidSourceAccount {
						account_id,
						asset_id,
						withdraw,
					})
			})
			.collect()
	}

	#[inline]
	fn check_sink_accounts<I>(
		&self,
		asset_id: manta_accounting::asset::AssetId,
		sinks: I,
	) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<Self::AccountId>>
	where
		I: Iterator<Item = (Self::AccountId, asset::AssetValue)>,
	{
		// NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
		//		 pass the data forward.
		sinks
			.map(move |(account_id, deposit)| {
				T::FungibleLedger::can_deposit(asset_id.0, &account_id, deposit.0)
					.map(|_| WrapPair(account_id.clone(), deposit))
					.map_err(|_| InvalidSinkAccount {
						account_id,
						asset_id,
						deposit,
					})
			})
			.collect()
	}

	#[inline]
	fn is_valid(
		&self,
		asset_id: Option<asset::AssetId>,
		sources: &[SourcePostingKey<config::Config, Self>],
		senders: &[SenderPostingKey<config::Config, Self>],
		receivers: &[ReceiverPostingKey<config::Config, Self>],
		sinks: &[SinkPostingKey<config::Config, Self>],
		proof: Proof<config::Config>,
	) -> Option<(Self::ValidProof, Self::Event)> {
		let (mut verifying_context, event) = match TransferShape::select(
			asset_id.is_some(),
			sources.len(),
			senders.len(),
			receivers.len(),
			sinks.len(),
		)? {
			TransferShape::Mint => (
				manta_sdk::pay::testnet::verifying::Mint::get().expect("Checksum did not match."),
				PreprocessedEvent::<T>::ToPrivate {
					asset: Asset::new(asset_id.unwrap().0, (sources[0].1).0),
					source: sources[0].0.clone(),
				},
			),
			TransferShape::PrivateTransfer => (
				manta_sdk::pay::testnet::verifying::PrivateTransfer::get()
					.expect("Checksum did not match."),
				PreprocessedEvent::<T>::PrivateTransfer,
			),
			TransferShape::Reclaim => (
				manta_sdk::pay::testnet::verifying::Reclaim::get()
					.expect("Checksum did not match."),
				PreprocessedEvent::<T>::ToPublic {
					asset: Asset::new(asset_id.unwrap().0, (sinks[0].1).0),
					sink: sinks[0].0.clone(),
				},
			),
		};
		config::ProofSystem::verify(
			&config::VerifyingContext::decode(&mut verifying_context)
				.expect("Unable to decode the verifying context."),
			&manta_accounting::transfer::TransferPostingKey::generate_proof_input(
				asset_id, sources, senders, receivers, sinks,
			),
			&proof,
		)
		.ok()?
		.then(move || (Wrap(()), event))
	}

	#[inline]
	fn update_public_balances(
		&mut self,
		asset_id: asset::AssetId,
		sources: Vec<SourcePostingKey<config::Config, Self>>,
		sinks: Vec<SinkPostingKey<config::Config, Self>>,
		proof: Self::ValidProof,
		super_key: &TransferLedgerSuperPostingKey<config::Config, Self>,
	) -> Result<(), Self::UpdateError> {
		let _ = (proof, super_key);
		for WrapPair(account_id, withdraw) in sources {
			T::FungibleLedger::transfer(
				asset_id.0,
				&account_id,
				&pallet::Pallet::<T>::account_id(),
				withdraw.0,
			)?;
		}
		for WrapPair(account_id, deposit) in sinks {
			T::FungibleLedger::transfer(
				asset_id.0,
				&pallet::Pallet::<T>::account_id(),
				&account_id,
				deposit.0,
			)?;
		}
		Ok(())
	}
}
