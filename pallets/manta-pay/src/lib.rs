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
//! ## Overview
//!
//! The Assets module provides functionality for asset management of fungible asset classes with
//! a fixed supply, including:
//!
//! * To Private Asset Conversion (see [`to_private`])
//! * To Public Asset Conversion (see [`to_public`])
//! * Private Asset Transfer (see [`private_transfer`]
//! * Public Asset Transfer (see [`public_transfer`])
//!
//! To use it in your runtime, you need to implement the assets [`Config`].
//!
//! The supported dispatchable functions are documented in the [`Call`] enum.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * [`to_public`]: Converts a public asset into a private one.
//! * [`to_private`]: Converts a private asset back into a public one.
//! * [`private_transfer`]: Transfers assets between two private accounts.
//! * [`public_transfer`]: Transfers assets between two public accounts.
//!
//! Please refer to the [`Call`] enum and its associated variants for documentation on each
//! function.
//!
//! Please refer to the [`Module`] struct for details on publicly available functions.
//!
//! [`to_private`]: Pallet::to_private
//! [`to_public`]: Pallet::to_public
//! [`private_transfer`]: Pallet::private_transfer
//! [`public_transfer`]: Pallet::public_transfer

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{transactional, PalletId};
use manta_accounting::{
	asset,
	transfer::{
		canonical::TransferShape, InvalidSinkAccount, InvalidSourceAccount, Proof, ReceiverLedger,
		ReceiverPostError, ReceiverPostingKey, SenderLedger, SenderPostError, SenderPostingKey,
		SinkPostingKey, SourcePostingKey, TransferLedger, TransferLedgerSuperPostingKey,
		TransferPostError, TransferPostingKey,
	},
};
use manta_crypto::{
	constraint::ProofSystem,
	merkle_tree::{self, forest::Configuration as _},
};
use manta_pay::config;
use manta_primitives::{
	assets::{AssetConfig, FungibleLedger as _, FungibleLedgerError},
	types::{AssetId, Balance},
};
use manta_util::codec::Decode as _;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use types::*;

#[cfg(feature = "serde")]
use manta_util::serde::{Deserialize, Serialize};

pub use manta_pay::signer::{Checkpoint, RawCheckpoint};
pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

pub mod types;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "runtime")]
pub mod runtime;

/// MantaPay Pallet
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
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
		type AssetConfig: AssetConfig<Self>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Pallet ID
		type PalletId: Get<PalletId>;
	}

	/// Fungible Ledger Implementation for [`Config`]
	pub(crate) type FungibleLedger<T> =
		<<T as Config>::AssetConfig as AssetConfig<T>>::FungibleLedger;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// UTXOs and Encrypted Notes Grouped by Shard
	#[pallet::storage]
	pub(super) type Shards<T: Config> =
		StorageDoubleMap<_, Identity, u8, Identity, u64, (Utxo, EncryptedNote), ValueQuery>;

	/// Shard Merkle Tree Paths
	#[pallet::storage]
	pub(super) type ShardTrees<T: Config> =
		StorageMap<_, Identity, u8, UtxoMerkleTreePath, ValueQuery>;

	/// Outputs of Utxo Accumulator
	#[pallet::storage]
	pub(super) type UtxoAccumulatorOutputs<T: Config> =
		StorageMap<_, Identity, UtxoAccumulatorOutput, (), ValueQuery>;

	/// UTXO Set
	#[pallet::storage]
	pub(super) type UtxoSet<T: Config> = StorageMap<_, Identity, Utxo, (), ValueQuery>;

	/// Void Number Set
	#[pallet::storage]
	pub(super) type VoidNumberSet<T: Config> = StorageMap<_, Identity, VoidNumber, (), ValueQuery>;

	/// Void Number Ordered by Insertion
	#[pallet::storage]
	pub(super) type VoidNumberSetInsertionOrder<T: Config> =
		StorageMap<_, Identity, u64, VoidNumber, ValueQuery>;

	/// The size of Void Number Set
	#[pallet::storage]
	pub(super) type VoidNumberSetSize<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transforms some public assets into private ones using `post`, withdrawing the public
		/// assets from the `origin` account.
		#[pallet::weight(T::WeightInfo::to_private())]
		#[transactional]
		pub fn to_private(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			Self::post_transaction(None, vec![origin], vec![], post)
		}

		/// Transforms some private assets into public ones using `post`, depositing the public
		/// assets in the `origin` account.
		#[pallet::weight(T::WeightInfo::to_public())]
		#[transactional]
		pub fn to_public(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			Self::post_transaction(None, vec![], vec![origin], post)
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
			Self::post_transaction(Some(origin), vec![], vec![], post)
		}

		/// Transfers public `asset` from `origin` to the `sink` account.
		#[pallet::weight(T::WeightInfo::public_transfer())]
		#[transactional]
		pub fn public_transfer(
			origin: OriginFor<T>,
			asset: Asset,
			sink: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			FungibleLedger::<T>::transfer(asset.id, &origin, &sink, asset.value)
				.map_err(Error::<T>::from)?;
			Self::deposit_event(Event::Transfer {
				asset,
				source: origin,
				sink,
			});
			Ok(().into())
		}
	}

	/// Event
	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Public Transfer Event
		Transfer {
			/// Asset Transferred
			asset: Asset,

			/// Source Account
			source: T::AccountId,

			/// Sink Account
			sink: T::AccountId,
		},

		/// To Private Event
		ToPrivate {
			/// Asset Converted
			asset: Asset,

			/// Source Account
			source: T::AccountId,
		},

		/// Private Transfer Event
		PrivateTransfer {
			/// Origin Account
			origin: T::AccountId,
		},

		/// To Public Event
		ToPublic {
			/// Asset Converted
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
		/// Attempted to withdraw from balance which was smaller than the withdrawal amount.
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

		/// Invalid Source Account
		///
		/// At least one of the source accounts is invalid.
		InvalidSourceAccount,

		/// Invalid Sink Account
		///
		/// At least one of the sink accounts in invalid.
		InvalidSinkAccount,

		/// [`InvalidAssetId`](FungibleLedgerError::InvalidAssetId) from [`FungibleLedgerError`]
		PublicUpdateInvalidAssetId,

		/// [`BelowMinimum`](FungibleLedgerError::BelowMinimum) from [`FungibleLedgerError`]
		PublicUpdateBelowMinimum,

		/// [`CannotCreate`](FungibleLedgerError::CannotCreate) from [`FungibleLedgerError`]
		PublicUpdateCannotCreate,

		/// [`UnknownAsset`](FungibleLedgerError::UnknownAsset) from [`FungibleLedgerError`]
		PublicUpdateUnknownAsset,

		/// [`Overflow`](FungibleLedgerError::Overflow) from [`FungibleLedgerError`]
		PublicUpdateOverflow,

		/// [`Underflow`](FungibleLedgerError::Underflow) from [`FungibleLedgerError`]
		PublicUpdateUnderflow,

		/// [`Frozen`](FungibleLedgerError::Frozen) from [`FungibleLedgerError`]
		PublicUpdateFrozen,

		/// [`ReducedToZero`](FungibleLedgerError::ReducedToZero) from [`FungibleLedgerError`]
		PublicUpdateReducedToZero,

		/// [`NoFunds`](FungibleLedgerError::NoFunds) from [`FungibleLedgerError`]
		PublicUpdateNoFunds,

		/// [`WouldDie`](FungibleLedgerError::WouldDie) from [`FungibleLedgerError`]
		PublicUpdateWouldDie,

		/// [`InvalidTransfer`](FungibleLedgerError::InvalidTransfer) from [`FungibleLedgerError`]
		PublicUpdateInvalidTransfer,

		/// Internal Ledger Error
		///
		/// This is caused by some internal error in the ledger and should never occur.
		InternalLedgerError,
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

	impl<T> From<FungibleLedgerError> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(err: FungibleLedgerError) -> Self {
			match err {
				FungibleLedgerError::InvalidAssetId => Self::PublicUpdateInvalidAssetId,
				FungibleLedgerError::BelowMinimum => Self::PublicUpdateBelowMinimum,
				FungibleLedgerError::CannotCreate => Self::PublicUpdateCannotCreate,
				FungibleLedgerError::UnknownAsset => Self::PublicUpdateUnknownAsset,
				FungibleLedgerError::Overflow => Self::PublicUpdateOverflow,
				FungibleLedgerError::Underflow => Self::PublicUpdateUnderflow,
				FungibleLedgerError::Frozen => Self::PublicUpdateFrozen,
				FungibleLedgerError::ReducedToZero(_) => Self::PublicUpdateReducedToZero,
				FungibleLedgerError::NoFunds => Self::PublicUpdateNoFunds,
				FungibleLedgerError::WouldDie => Self::PublicUpdateWouldDie,
				FungibleLedgerError::InvalidTransfer(_e) => Self::PublicUpdateInvalidTransfer,
				_ => Self::InternalLedgerError,
			}
		}
	}

	impl<T> From<TransferPostError<T::AccountId, FungibleLedgerError>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(err: TransferPostError<T::AccountId, FungibleLedgerError>) -> Self {
			match err {
				TransferPostError::InvalidShape => Self::InvalidShape,
				TransferPostError::InvalidSourceAccount(err) => err.into(),
				TransferPostError::InvalidSinkAccount(err) => err.into(),
				TransferPostError::Sender(err) => err.into(),
				TransferPostError::Receiver(err) => err.into(),
				TransferPostError::DuplicateSpend => Self::DuplicateSpend,
				TransferPostError::DuplicateRegister => Self::DuplicateRegister,
				TransferPostError::InvalidProof => Self::InvalidProof,
				TransferPostError::UpdateError(err) => err.into(),
			}
		}
	}

	impl<T> Pallet<T>
	where
		T: Config,
	{
		/// Maximum Number of Updates per Shard
		const PULL_MAX_PER_SHARD_UPDATE_SIZE: usize = 128;

		/// Maximum Size of Sender Data Update
		const PULL_MAX_SENDER_UPDATE_SIZE: usize = 1024;

		/// Pulls receiver data from the ledger starting at the `receiver_index`.
		#[inline]
		fn pull_receivers(receiver_index: [usize; 256]) -> (bool, ReceiverChunk) {
			let mut more_receivers = false;
			let mut receivers = Vec::new();
			for (i, index) in receiver_index.into_iter().enumerate() {
				more_receivers |= Self::pull_receivers_for_shard(i as u8, index, &mut receivers);
			}
			(more_receivers, receivers)
		}

		/// Pulls receiver data from the shard at `shard_index` starting at the `receiver_index`,
		/// pushing the results back to `receivers`.
		#[inline]
		fn pull_receivers_for_shard(
			shard_index: u8,
			receiver_index: usize,
			receivers: &mut ReceiverChunk,
		) -> bool {
			let mut iter = if receiver_index == 0 {
				Shards::<T>::iter_prefix(shard_index)
			} else {
				let raw_key = Shards::<T>::hashed_key_for(shard_index, receiver_index as u64 - 1);
				Shards::<T>::iter_prefix_from(shard_index, raw_key)
			};
			for _ in 0..Self::PULL_MAX_PER_SHARD_UPDATE_SIZE {
				match iter.next() {
					Some((_, next)) => receivers.push(next),
					_ => return false,
				}
			}
			iter.next().is_some()
		}

		/// Pulls sender data from the ledger starting at the `sender_index`.
		#[inline]
		fn pull_senders(sender_index: usize) -> (bool, SenderChunk) {
			let mut senders = Vec::new();
			let mut iter = VoidNumberSetInsertionOrder::<T>::iter().skip(sender_index);
			for _ in 0..Self::PULL_MAX_SENDER_UPDATE_SIZE {
				match iter.next() {
					Some((_, next)) => senders.push(next),
					_ => return (false, senders),
				}
			}
			(iter.next().is_some(), senders)
		}

		/// Returns the diff of ledger state since the given `checkpoint`.
		#[inline]
		pub fn pull_ledger_diff(checkpoint: Checkpoint) -> PullResponse {
			let (more_receivers, receivers) = Self::pull_receivers(*checkpoint.receiver_index);
			let (more_senders, senders) = Self::pull_senders(checkpoint.sender_index);
			PullResponse {
				should_continue: more_receivers || more_senders,
				receivers,
				senders,
			}
		}

		/// Returns the account ID of this pallet.
		#[inline]
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}

		/// Posts the transaction encoded in `post` to the ledger, using `sources` and `sinks` as
		/// the public deposit and public withdraw accounts respectively.
		#[inline]
		fn post_transaction(
			origin: Option<T::AccountId>,
			sources: Vec<T::AccountId>,
			sinks: Vec<T::AccountId>,
			post: TransferPost,
		) -> DispatchResultWithPostInfo {
			Self::deposit_event(
				config::TransferPost::try_from(post)
					.map_err(|_| Error::<T>::InvalidSerializedForm)?
					.post(sources, sinks, &(), &mut Ledger(PhantomData))
					.map_err(Error::<T>::from)?
					.convert(origin),
			);
			Ok(().into())
		}
	}
}

/// Receiver Chunk Data Type
pub type ReceiverChunk = Vec<(Utxo, EncryptedNote)>;

/// Sender Chunk Data Type
pub type SenderChunk = Vec<VoidNumber>;

/// Ledger Source Pull Response
#[cfg_attr(
	feature = "serde",
	derive(Deserialize, Serialize),
	serde(crate = "manta_util::serde", deny_unknown_fields)
)]
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Hash, PartialEq, TypeInfo)]
pub struct PullResponse {
	/// Pull Continuation Flag
	///
	/// The `should_continue` flag is set to `true` if the client should request more data from the
	/// ledger to finish the pull.
	pub should_continue: bool,

	/// Ledger Receiver Chunk
	pub receivers: ReceiverChunk,

	/// Ledger Sender Chunk
	pub senders: SenderChunk,
}

/// Preprocessed Event
enum PreprocessedEvent<T>
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
	fn convert(self, origin: Option<T::AccountId>) -> Event<T> {
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
struct Ledger<T>(PhantomData<T>)
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
		if VoidNumberSet::<T>::contains_key(encode(&void_number)) {
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
		if UtxoAccumulatorOutputs::<T>::contains_key(encode(&output)) {
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
			let void_number = encode(&void_number.0);
			VoidNumberSet::<T>::insert(void_number, ());
			VoidNumberSetInsertionOrder::<T>::insert(index + i, void_number);
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
		if UtxoSet::<T>::contains_key(encode(&utxo)) {
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
				let utxo = encode(&utxo);
				UtxoSet::<T>::insert(utxo, ());
				Shards::<T>::insert(shard_index, next_index, (utxo, EncryptedNote::from(note)));
			}
			tree.current_path = current_path.into();
			if let Some(next_root) = next_root {
				ShardTrees::<T>::insert(shard_index, tree);
				UtxoAccumulatorOutputs::<T>::insert(encode(&next_root), ());
			}
		}
	}
}

impl<T> TransferLedger<config::Config> for Ledger<T>
where
	T: Config,
{
	type AccountId = T::AccountId;
	type UpdateError = FungibleLedgerError;
	type Event = PreprocessedEvent<T>;
	type ValidSourceAccount = WrapPair<Self::AccountId, asset::AssetValue>;
	type ValidSinkAccount = WrapPair<Self::AccountId, asset::AssetValue>;
	type ValidProof = Wrap<()>;
	type SuperPostingKey = ();

	#[inline]
	fn check_source_accounts<I>(
		&self,
		asset_id: asset::AssetId,
		sources: I,
	) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<Self::AccountId>>
	where
		I: Iterator<Item = (Self::AccountId, asset::AssetValue)>,
	{
		sources
			.map(move |(account_id, withdraw)| {
				FungibleLedger::<T>::can_withdraw(asset_id.0, &account_id, withdraw.0)
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
		asset_id: asset::AssetId,
		sinks: I,
	) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<Self::AccountId>>
	where
		I: Iterator<Item = (Self::AccountId, asset::AssetValue)>,
	{
		// NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
		//		 pass the data forward.
		sinks
			.map(move |(account_id, deposit)| {
				FungibleLedger::<T>::can_deposit(asset_id.0, &account_id, deposit.0)
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
			&TransferPostingKey::generate_proof_input(asset_id, sources, senders, receivers, sinks),
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
			FungibleLedger::<T>::transfer(
				asset_id.0,
				&account_id,
				&Pallet::<T>::account_id(),
				withdraw.0,
			)?;
		}
		for WrapPair(account_id, deposit) in sinks {
			FungibleLedger::<T>::transfer(
				asset_id.0,
				&Pallet::<T>::account_id(),
				&account_id,
				deposit.0,
			)?;
		}
		Ok(())
	}
}
