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
//! * Asset Issuance
//! * Asset Transfer
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
//! * **Asset issuance:** The creation of the asset (note: this asset can only be created once)
//! * **Asset transfer:** The action of transferring assets from one account to another.
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
//! * `transfer_asset` - Transfers an `amount` of units of fungible asset `id` from the balance of
//!     the function caller's account (`origin`) to a `target` account.
//! * `mint` - Converting an `amount` of units of fungible asset `id` from the caller
//!     to a private UTXO. (The caller does not need to be the owner of this UTXO)
//! * `private_transfer` - Transfer two input UTXOs into two output UTXOs. Require that 1) the input
//!     UTXOs are already in the ledger and are not spend before 2) the sum of private assets in
//!     input UTXOs matches that of the output UTXOs. The requirements are guaranteed via ZK proof.
//! * `reclaim` - Transfer two input UTXOs into one output UTXOs, and convert the remaining assets
//!     to the public assets. Require that 1) the input UTXOs are already in the ledger and are not
//!     spend before; 2) the sum of private assets in input UTXOs matches that of the output UTXO +
//!     the reclaimed amount. The requirements are guaranteed via ZK proof.
//!
//! Please refer to the [`Call`](./enum.Call.html) enum and its associated variants for
//! documentation on each function.
//!
//! ### Public Functions
//!
//! * `balance` - Get the asset balance of `who`.
//! * `total_supply` - Get the total supply of an asset `id`.
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
use frame_support::{ensure, require_transactional};
use manta_accounting::{
	asset,
	transfer::{
		canonical::TransferShape, AccountBalance, InvalidSinkAccount, InvalidSourceAccount, Proof,
		ReceiverLedger, ReceiverPostError, ReceiverPostingKey, SenderLedger, SenderPostError,
		SenderPostingKey, SinkPostingKey, SourcePostingKey, TransferLedger,
		TransferLedgerSuperPostingKey, TransferPostError,
	},
};
use manta_crypto::{
	constraint::ProofSystem,
	merkle_tree::{self, forest::Configuration as _},
};
use manta_pay::config;
use manta_util::codec::Decode as _;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

pub mod weights;

pub use pallet::*;

/// Type Definitions for Protocol Structures
pub mod types {
	use super::*;

	/// Asset Id Type
	pub type AssetId = asset::AssetIdType;

	/// Asset Value Type
	pub type AssetValue = asset::AssetValueType;

	/// Asset
	#[derive(
		Clone,
		Copy,
		Debug,
		Decode,
		Default,
		Encode,
		Eq,
		Hash,
		MaxEncodedLen,
		Ord,
		PartialEq,
		PartialOrd,
		TypeInfo,
	)]
	pub struct Asset {
		/// Asset Id
		pub id: AssetId,

		/// Asset Value
		pub value: AssetValue,
	}

	impl Asset {
		/// Builds a new [`Asset`] from `id` and `value`.
		#[inline]
		pub fn new(id: AssetId, value: AssetValue) -> Self {
			Self { id, value }
		}
	}

	/// Encrypted Note
	#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
	pub struct EncryptedNote {
		/// Ciphertext
		pub ciphertext: [u8; 36],

		/// Ephemeral Public Key
		pub ephemeral_public_key: config::PublicKey,
	}

	impl Default for EncryptedNote {
		#[inline]
		fn default() -> Self {
			Self {
				ciphertext: [0; 36],
				ephemeral_public_key: Default::default(),
			}
		}
	}

	impl From<config::EncryptedNote> for EncryptedNote {
		#[inline]
		fn from(note: config::EncryptedNote) -> Self {
			Self {
				ciphertext: note.ciphertext.into(),
				ephemeral_public_key: note.ephemeral_public_key,
			}
		}
	}

	impl From<EncryptedNote> for config::EncryptedNote {
		#[inline]
		fn from(note: EncryptedNote) -> Self {
			Self {
				ciphertext: note.ciphertext.into(),
				ephemeral_public_key: note.ephemeral_public_key,
			}
		}
	}

	/// Sender Post
	#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
	pub struct SenderPost {
		/// UTXO Accumulator Output
		pub utxo_accumulator_output: config::UtxoAccumulatorOutput,

		/// Void Number
		pub void_number: config::VoidNumber,
	}

	impl From<config::SenderPost> for SenderPost {
		#[inline]
		fn from(post: config::SenderPost) -> Self {
			Self {
				utxo_accumulator_output: post.utxo_accumulator_output,
				void_number: post.void_number,
			}
		}
	}

	impl From<SenderPost> for config::SenderPost {
		#[inline]
		fn from(post: SenderPost) -> Self {
			Self {
				utxo_accumulator_output: post.utxo_accumulator_output,
				void_number: post.void_number,
			}
		}
	}

	/// Receiver Post
	#[derive(Clone, Debug, Decode, Encode, Eq, Hash, MaxEncodedLen, PartialEq, TypeInfo)]
	pub struct ReceiverPost {
		/// Unspent Transaction Output
		pub utxo: config::Utxo,

		/// Encrypted Note
		pub note: EncryptedNote,
	}

	impl From<config::ReceiverPost> for ReceiverPost {
		#[inline]
		fn from(post: config::ReceiverPost) -> Self {
			Self {
				utxo: post.utxo,
				note: post.note.into(),
			}
		}
	}

	impl From<ReceiverPost> for config::ReceiverPost {
		#[inline]
		fn from(post: ReceiverPost) -> Self {
			Self {
				utxo: post.utxo,
				note: post.note.into(),
			}
		}
	}

	/// Transfer Post
	#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
	pub struct TransferPost {
		/// Asset Id
		pub asset_id: Option<AssetId>,

		/// Sources
		pub sources: Vec<AssetValue>,

		/// Sender Posts
		pub sender_posts: Vec<SenderPost>,

		/// Receiver Posts
		pub receiver_posts: Vec<ReceiverPost>,

		/// Sinks
		pub sinks: Vec<AssetValue>,

		/// Validity Proof
		pub validity_proof: config::Proof,
	}

	impl From<config::TransferPost> for TransferPost {
		#[inline]
		fn from(post: config::TransferPost) -> Self {
			Self {
				asset_id: post.asset_id.map(|id| id.0),
				sources: post.sources.into_iter().map(|s| s.0).collect(),
				sender_posts: post.sender_posts.into_iter().map(Into::into).collect(),
				receiver_posts: post.receiver_posts.into_iter().map(Into::into).collect(),
				sinks: post.sinks.into_iter().map(|s| s.0).collect(),
				validity_proof: post.validity_proof,
			}
		}
	}

	impl From<TransferPost> for config::TransferPost {
		#[inline]
		fn from(post: TransferPost) -> Self {
			Self {
				asset_id: post.asset_id.map(asset::AssetId),
				sources: post.sources.into_iter().map(asset::AssetValue).collect(),
				sender_posts: post.sender_posts.into_iter().map(Into::into).collect(),
				receiver_posts: post.receiver_posts.into_iter().map(Into::into).collect(),
				sinks: post.sinks.into_iter().map(asset::AssetValue).collect(),
				validity_proof: post.validity_proof,
			}
		}
	}

	/// Leaf Digest Type
	pub type LeafDigest = merkle_tree::LeafDigest<config::MerkleTreeConfiguration>;

	/// Inner Digest Type
	pub type InnerDigest = merkle_tree::InnerDigest<config::MerkleTreeConfiguration>;

	/// Merkle Tree Current Path
	#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
	pub struct CurrentPath {
		/// Sibling Digest
		pub sibling_digest: LeafDigest,

		/// Leaf Index
		pub leaf_index: u32,

		/// Inner Path
		pub inner_path: Vec<InnerDigest>,
	}

	impl MaxEncodedLen for CurrentPath {
		#[inline]
		fn max_encoded_len() -> usize {
			0_usize
				.saturating_add(LeafDigest::max_encoded_len())
				.saturating_add(u32::max_encoded_len())
				.saturating_add(
					// NOTE: We know that these paths don't exceed the path length.
					InnerDigest::max_encoded_len().saturating_mul(
						manta_crypto::merkle_tree::path_length::<config::MerkleTreeConfiguration>(),
					),
				)
		}
	}

	impl From<merkle_tree::CurrentPath<config::MerkleTreeConfiguration>> for CurrentPath {
		#[inline]
		fn from(path: merkle_tree::CurrentPath<config::MerkleTreeConfiguration>) -> Self {
			Self {
				sibling_digest: path.sibling_digest,
				leaf_index: path.inner_path.leaf_index.0 as u32,
				inner_path: path.inner_path.path,
			}
		}
	}

	impl From<CurrentPath> for merkle_tree::CurrentPath<config::MerkleTreeConfiguration> {
		#[inline]
		fn from(path: CurrentPath) -> Self {
			Self::new(
				path.sibling_digest,
				(path.leaf_index as usize).into(),
				path.inner_path,
			)
		}
	}

	/// UTXO Merkle Tree Path
	#[derive(Clone, Debug, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
	pub struct UtxoMerkleTreePath {
		/// Current Leaf Digest
		pub leaf_digest: Option<LeafDigest>,

		/// Current Path
		pub current_path: CurrentPath,
	}
}

/// MantaPay Pallet
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::StaticLookup;

	/// Extrinsic Weight Info
	pub trait WeightInfo {
		/// Returns the [`Weight`] of the [`Pallet::transfer`] extrinsic.
		fn transfer() -> Weight;

		/// Returns the [`Weight`] of the [`Pallet::mint`] extrinsic.
		fn mint() -> Weight;

		/// Returns the [`Weight`] of the [`Pallet::private_transfer`] extrinsic.
		fn private_transfer() -> Weight;

		/// Returns the [`Weight`] of the [`Pallet::reclaim`] extrinsic.
		fn reclaim() -> Weight;
	}

	/// Pallet
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Public Balance State
	#[pallet::storage]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		AssetId,
		AssetValue,
		ValueQuery,
	>;

	/// Total Supply per AssetId
	#[pallet::storage]
	pub(super) type TotalSupply<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, AssetValue, ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type Shards<T: Config> =
		StorageDoubleMap<_, Identity, u8, Identity, u64, (config::Utxo, EncryptedNote), ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type ShardTrees<T: Config> =
		StorageMap<_, Identity, u8, UtxoMerkleTreePath, ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type UtxoAccumulatorOutputs<T: Config> =
		StorageMap<_, Identity, config::UtxoAccumulatorOutput, (), ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type UtxoSet<T: Config> = StorageMap<_, Identity, config::Utxo, (), ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type VoidNumberSet<T: Config> =
		StorageMap<_, Identity, config::VoidNumber, (), ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type VoidNumberSetInsertionOrder<T: Config> =
		StorageMap<_, Identity, u64, config::VoidNumber, ValueQuery>;

	///
	#[pallet::storage]
	pub(super) type VoidNumberSetSize<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Genesis Configuration
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub owner: T::AccountId,
		pub assets: alloc::collections::btree_set::BTreeSet<(AssetId, AssetValue)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		#[inline]
		fn default() -> Self {
			/* FIXME: `AccountId` does not implement default!
			GenesisConfig {
				owner: Default::default(),
				assets: Default::default(),
			}
			*/
			todo!()
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		#[inline]
		fn build(&self) {
			for (id, value) in &self.assets {
				Pallet::<T>::init_asset(&self.owner, *id, *value);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers public `asset` from `origin` to `target`.
		#[pallet::weight(T::WeightInfo::transfer())]
		#[require_transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			asset: Asset,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;
			ensure!(
				TotalSupply::<T>::contains_key(&asset.id),
				Error::<T>::UninitializedSupply
			);
			let origin_balance = Balances::<T>::get(&origin, asset.id);
			ensure!(asset.value > 0, Error::<T>::ZeroTransfer);
			ensure!(origin_balance >= asset.value, Error::<T>::BalanceLow);
			Balances::<T>::mutate(&origin, asset.id, |balance| *balance -= asset.value);
			Balances::<T>::mutate(&target, asset.id, |balance| *balance += asset.value);
			Self::deposit_event(Event::Transfer {
				asset,
				source: origin,
				sink: target,
			});
			Ok(().into())
		}

		/// Mints some assets encoded in `post` to the `origin` account.
		#[pallet::weight(T::WeightInfo::mint())]
		#[require_transactional]
		pub fn mint(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let mut ledger = Self::ledger();
			Self::deposit_event(
				config::TransferPost::from(post)
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
		#[require_transactional]
		pub fn private_transfer(
			origin: OriginFor<T>,
			post: TransferPost,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let mut ledger = Self::ledger();
			Self::deposit_event(
				config::TransferPost::from(post)
					.post(vec![], vec![], &(), &mut ledger)
					.map_err(Error::<T>::from)?
					.convert(Some(origin)),
			);
			Ok(().into())
		}

		/// Transforms some private assets into public ones using `post`, sending the public assets
		/// to the `origin` account.
		#[pallet::weight(T::WeightInfo::reclaim())]
		#[require_transactional]
		pub fn reclaim(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let mut ledger = Self::ledger();
			Self::deposit_event(
				config::TransferPost::from(post)
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
		/// Transfer Event
		Transfer {
			/// Asset Transfered
			asset: Asset,

			/// Source Account
			source: T::AccountId,

			/// Sink Account
			sink: T::AccountId,
		},

		/// Mint Event
		Mint {
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
		Reclaim {
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
	}

	impl<T> From<InvalidSourceAccount<T::AccountId>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(err: InvalidSourceAccount<T::AccountId>) -> Self {
			match err.balance {
				AccountBalance::Known(_) => Self::BalanceLow,
				AccountBalance::UnknownAccount => {
					unreachable!("Accounts are checked before reaching this point.")
				}
			}
		}
	}

	impl<T> From<InvalidSinkAccount<T::AccountId>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(err: InvalidSinkAccount<T::AccountId>) -> Self {
			let _ = err;
			unimplemented!("Accounts are checked before reaching this point.")
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

	impl<T> From<TransferPostError<T::AccountId>> for Error<T>
	where
		T: Config,
	{
		#[inline]
		fn from(err: TransferPostError<T::AccountId>) -> Self {
			match err {
				TransferPostError::InvalidShape => Self::InvalidShape,
				TransferPostError::InvalidSourceAccount(err) => err.into(),
				TransferPostError::InvalidSinkAccount(err) => err.into(),
				TransferPostError::Sender(err) => err.into(),
				TransferPostError::Receiver(err) => err.into(),
				TransferPostError::DuplicateSpend => Self::DuplicateSpend,
				TransferPostError::DuplicateRegister => Self::DuplicateRegister,
				TransferPostError::InvalidProof => Self::InvalidProof,
			}
		}
	}
}

impl<T> Pallet<T>
where
	T: Config,
{
	/// Initializes `asset_id` with a supply of `total`, giving control to `owner`.
	#[inline]
	fn init_asset(owner: &T::AccountId, asset_id: AssetId, total: AssetValue) {
		TotalSupply::<T>::insert(asset_id, total);
		Balances::<T>::insert(owner, asset_id, total);
	}

	/// Returns the balance of `account` for the asset with the given `id`.
	#[inline]
	pub fn balance(account: T::AccountId, id: AssetId) -> AssetValue {
		Balances::<T>::get(account, id)
	}

	/// Returns the total supply of the asset with the given `id`.
	#[inline]
	pub fn total_supply(id: AssetId) -> AssetValue {
		TotalSupply::<T>::get(id)
	}

	/// Returns the ledger implementation for this pallet.
	#[inline]
	fn ledger() -> Ledger<T> {
		Ledger(PhantomData)
	}
}

/// Preprocessed Event
pub enum PreprocessedEvent<T>
where
	T: Config,
{
	/// Mint Event
	Mint {
		/// Asset Minted
		asset: Asset,

		/// Source Account
		source: T::AccountId,
	},

	/// Private Transfer Event
	PrivateTransfer,

	/// Reclaim Event
	Reclaim {
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
			Self::Mint { asset, source } => Event::Mint { asset, source },
			Self::PrivateTransfer => Event::PrivateTransfer {
				origin: origin.unwrap(),
			},
			Self::Reclaim { asset, sink } => Event::Reclaim { asset, sink },
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
		// NOTE: Existence of accounts is type-checked so we only need check account balances.
		sources
			.map(move |(account_id, withdraw)| {
				match Balances::<T>::try_get(&account_id, asset_id.0) {
					Ok(balance) => {
						// FIXME: Check if balance would withdraw more than existential deposit.
						if balance >= withdraw.0 {
							Ok(WrapPair(account_id, withdraw))
						} else {
							Err(InvalidSourceAccount {
								account_id,
								balance: AccountBalance::Known(asset::AssetValue(balance)),
								withdraw,
							})
						}
					}
					_ => Err(InvalidSourceAccount {
						account_id,
						balance: AccountBalance::Known(asset::AssetValue(0)),
						withdraw,
					}),
				}
			})
			.collect()
	}

	#[inline]
	fn check_sink_accounts<I>(
		&self,
		sinks: I,
	) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<Self::AccountId>>
	where
		I: Iterator<Item = (Self::AccountId, asset::AssetValue)>,
	{
		// NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
		//		 pass the data forward.
		Ok(sinks
			.map(move |(account_id, deposit)| WrapPair(account_id, deposit))
			.collect())
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
				PreprocessedEvent::<T>::Mint {
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
				PreprocessedEvent::<T>::Reclaim {
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
	) {
		let _ = (proof, super_key);
		for WrapPair(account_id, withdraw) in sources {
			Balances::<T>::mutate(&account_id, asset_id.0, |balance| *balance -= withdraw.0);
		}
		for WrapPair(account_id, deposit) in sinks {
			Balances::<T>::mutate(&account_id, asset_id.0, |balance| *balance += deposit.0);
		}
	}
}
