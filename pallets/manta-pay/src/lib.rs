// Copyright 2020-2022 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

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
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]

extern crate alloc;

use crate::types::{
    encode, Asset, AssetValue, FullIncomingNote, NullifierCommitment, OutgoingNote, ReceiverChunk,
    SenderChunk, TransferPost, Utxo, UtxoAccumulatorOutput, UtxoMerkleTreePath,
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{traits::tokens::ExistenceRequirement, transactional, PalletId};
use manta_pay::{
    config::{self, utxo::v2::MerkleTreeConfiguration},
    manta_accounting::transfer::{
        self,
        canonical::TransferShape,
        receiver::{ReceiverLedger, ReceiverPostError},
        sender::{SenderLedger, SenderPostError},
        InvalidAuthorizationSignature, InvalidSinkAccount, InvalidSourceAccount, SinkPostingKey,
        SourcePostingKey, TransferLedger, TransferLedgerSuperPostingKey, TransferPostingKeyRef,
    },
    manta_crypto::merkle_tree::{self, forest::Configuration as _},
    manta_parameters::{self, Get as _},
    manta_util::codec::Decode as _,
    parameters::load_transfer_parameters,
};
use manta_primitives::assets::{self, AssetConfig, FungibleLedger as _};
use manta_util::{into_array_unchecked, Array};

pub use crate::types::{Checkpoint, RawCheckpoint};
pub use pallet::*;
pub use types::PullResponse;
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

/// Standard Asset Id
pub type StandardAssetId = u128;

/// Fungible Ledger Error
pub type FungibleLedgerError = assets::FungibleLedgerError<StandardAssetId, AssetValue>;

/// MantaPay Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::StorageVersion};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::AccountIdConversion;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Pallet
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// The module configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Asset Configuration
        type AssetConfig: AssetConfig<Self, AssetId = StandardAssetId, Balance = AssetValue>;

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

    /// UTXO Set
    #[pallet::storage]
    pub(super) type UtxoSet<T: Config> = StorageMap<_, Twox64Concat, Utxo, (), ValueQuery>;

    /// UTXOs and Incoming Notes Grouped by Shard
    #[pallet::storage]
    pub(super) type Shards<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        u8,
        Twox64Concat,
        u64,
        (Utxo, FullIncomingNote),
        ValueQuery,
    >;

    /// Shard Merkle Tree Paths
    #[pallet::storage]
    pub(super) type ShardTrees<T: Config> =
        StorageMap<_, Twox64Concat, u8, UtxoMerkleTreePath, ValueQuery>;

    /// Outputs of Utxo Accumulator
    #[pallet::storage]
    pub(super) type UtxoAccumulatorOutputs<T: Config> =
        StorageMap<_, Twox64Concat, UtxoAccumulatorOutput, (), ValueQuery>;

    /// Nullifier Commitment Set
    #[pallet::storage]
    pub(super) type NullifierCommitmentSet<T: Config> =
        StorageMap<_, Twox64Concat, NullifierCommitment, (), ValueQuery>;

    /// Nullifiers Ordered by Insertion
    #[pallet::storage]
    pub(super) type NullifierSetInsertionOrder<T: Config> =
        StorageMap<_, Twox64Concat, u64, (NullifierCommitment, OutgoingNote), ValueQuery>;

    /// Nullifier Set Size
    #[pallet::storage]
    pub(super) type NullifierSetSize<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Transforms some public assets into private ones using `post`, withdrawing the public
        /// assets from the `origin` account.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::to_private())]
        #[transactional]
        pub fn to_private(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            Self::post_transaction(None, vec![origin], vec![], post)
        }

        /// Transforms some private assets into public ones using `post`, depositing the public
        /// assets in the `origin` account.
        #[pallet::call_index(1)]
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
        #[pallet::call_index(2)]
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
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::public_transfer())]
        #[transactional]
        pub fn public_transfer(
            origin: OriginFor<T>,
            asset: Asset,
            sink: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            FungibleLedger::<T>::transfer(
                Self::id_from_field(asset.id).ok_or(Error::<T>::InvalidAssetId)?,
                &origin,
                &sink,
                asset.value,
                ExistenceRequirement::KeepAlive,
            )
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
        /// To Private Event
        ToPrivate {
            /// Asset Converted
            asset: Asset,

            /// Source Account
            source: T::AccountId,
        },

        /// To Public Event
        ToPublic {
            /// Asset Converted
            asset: Asset,

            /// Sink Account
            sink: T::AccountId,
        },

        /// Private Transfer Event
        PrivateTransfer {
            /// Origin Account
            origin: T::AccountId,
        },

        /// Public Transfer Event
        Transfer {
            /// Asset Transferred
            asset: Asset,

            /// Source Account
            source: T::AccountId,

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

        /// Invalid Asset Id
        ///
        /// The asset id of the transfer could not be converted correctly to the standard format.
        InvalidAssetId,

        /// Invalid Shape
        ///
        /// The transfer had an invalid shape.
        InvalidShape,

        /// Invalid Authorization Signature
        InvalidAuthorizationSignature,

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

        /// [`CannotWithdraw`](FungibleLedgerError::CannotWithdrawMoreThan) from [`FungibleLedgerError`]
        PublicUpdateCannotWithdraw,

        /// [`InvalidMint`](FungibleLedgerError::InvalidMint) from [`FungibleLedgerError`]
        PublicUpdateInvalidMint,

        /// [`InvalidBurn`](FungibleLedgerError::InvalidBurn) from [`FungibleLedgerError`]
        PublicUpdateInvalidBurn,

        /// [`InvalidTransfer`](FungibleLedgerError::InvalidTransfer) from [`FungibleLedgerError`]
        PublicUpdateInvalidTransfer,

        /// Internal Ledger Error
        ///
        /// This is caused by some internal error in the ledger and should never occur.
        InternalLedgerError,
    }

    impl<T> From<InvalidAuthorizationSignature> for Error<T>
    where
        T: Config,
    {
        #[inline]
        fn from(_: InvalidAuthorizationSignature) -> Self {
            Self::InvalidAuthorizationSignature
        }
    }

    impl<T> From<InvalidSourceAccount<config::Config, T::AccountId>> for Error<T>
    where
        T: Config,
    {
        #[inline]
        fn from(_: InvalidSourceAccount<config::Config, T::AccountId>) -> Self {
            Self::InvalidSourceAccount
        }
    }

    impl<T> From<InvalidSinkAccount<config::Config, T::AccountId>> for Error<T>
    where
        T: Config,
    {
        #[inline]
        fn from(_: InvalidSinkAccount<config::Config, T::AccountId>) -> Self {
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
                FungibleLedgerError::InvalidAssetId(_) => Self::PublicUpdateInvalidAssetId,
                FungibleLedgerError::BelowMinimum => Self::PublicUpdateBelowMinimum,
                FungibleLedgerError::CannotCreate => Self::PublicUpdateCannotCreate,
                FungibleLedgerError::UnknownAsset => Self::PublicUpdateUnknownAsset,
                FungibleLedgerError::Overflow => Self::PublicUpdateOverflow,
                FungibleLedgerError::CannotWithdrawMoreThan(_) => Self::PublicUpdateCannotWithdraw,
                FungibleLedgerError::InvalidMint(_) => Self::PublicUpdateInvalidMint,
                FungibleLedgerError::InvalidBurn(_) => Self::PublicUpdateInvalidBurn,
                FungibleLedgerError::InvalidTransfer(_) => Self::PublicUpdateInvalidTransfer,
            }
        }
    }

    /// Transfer Post Error
    pub type TransferPostError<T> = transfer::TransferPostError<
        config::Config,
        <T as frame_system::Config>::AccountId,
        FungibleLedgerError,
    >;

    impl<T> From<TransferPostError<T>> for Error<T>
    where
        T: Config,
    {
        #[inline]
        fn from(err: TransferPostError<T>) -> Self {
            match err {
                TransferPostError::<T>::InvalidShape => Self::InvalidShape,
                TransferPostError::<T>::InvalidAuthorizationSignature(err) => err.into(),
                TransferPostError::<T>::InvalidSourceAccount(err) => err.into(),
                TransferPostError::<T>::InvalidSinkAccount(err) => err.into(),
                TransferPostError::<T>::Sender(err) => err.into(),
                TransferPostError::<T>::Receiver(err) => err.into(),
                TransferPostError::<T>::DuplicateMint => Self::DuplicateRegister,
                TransferPostError::<T>::DuplicateSpend => Self::DuplicateSpend,
                TransferPostError::<T>::InvalidProof => Self::InvalidProof,
                TransferPostError::<T>::UpdateError(err) => err.into(),
            }
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        /// Maximum Number of Updates per Shard (based on benchmark result)
        const PULL_MAX_RECEIVER_UPDATE_SIZE: u64 = 32768;

        /// Maximum Size of Sender Data Update (based on benchmark result)
        const PULL_MAX_SENDER_UPDATE_SIZE: u64 = 32768;

        /// Pulls receiver data from the ledger starting at the `receiver_indices`.
        /// The pull algorithm is greedy. It tries to pull as many as possible from each shard
        /// before moving to the next shard.
        #[inline]
        fn pull_receivers(
            receiver_indices: [usize; MerkleTreeConfiguration::FOREST_WIDTH],
            max_update_request: u64,
        ) -> (bool, ReceiverChunk) {
            let mut more_receivers = false;
            let mut receivers = Vec::new();
            let mut receivers_pulled: u64 = 0;
            let max_update = if max_update_request > Self::PULL_MAX_RECEIVER_UPDATE_SIZE {
                Self::PULL_MAX_RECEIVER_UPDATE_SIZE
            } else {
                max_update_request
            };

            for (shard_index, utxo_index) in receiver_indices.into_iter().enumerate() {
                more_receivers |= Self::pull_receivers_for_shard(
                    shard_index as u8,
                    utxo_index,
                    max_update,
                    &mut receivers,
                    &mut receivers_pulled,
                );
                // NOTE: If max capacity is reached and there is more to pull, then we return.
                if receivers_pulled == max_update && more_receivers {
                    break;
                }
            }
            (more_receivers, receivers)
        }

        /// Pulls receiver data from the shard at `shard_index` starting at the `receiver_index`,
        /// pushing the results back to `receivers`.
        #[inline]
        fn pull_receivers_for_shard(
            shard_index: u8,
            receiver_index: usize,
            max_update: u64,
            receivers: &mut ReceiverChunk,
            receivers_pulled: &mut u64,
        ) -> bool {
            let max_receiver_index = (receiver_index as u64) + max_update;
            for idx in (receiver_index as u64)..max_receiver_index {
                if *receivers_pulled == max_update {
                    return Shards::<T>::contains_key(shard_index, idx);
                }
                match Shards::<T>::try_get(shard_index, idx) {
                    Ok(next) => {
                        *receivers_pulled += 1;
                        receivers.push(next);
                    }
                    _ => return false,
                }
            }
            Shards::<T>::contains_key(shard_index, max_receiver_index)
        }

        /// Pulls sender data from the ledger starting at the `sender_index`.
        #[inline]
        fn pull_senders(sender_index: usize, max_update_request: u64) -> (bool, SenderChunk) {
            let mut senders = Vec::new();
            let max_sender_index = if max_update_request > Self::PULL_MAX_SENDER_UPDATE_SIZE {
                (sender_index as u64) + Self::PULL_MAX_SENDER_UPDATE_SIZE
            } else {
                (sender_index as u64) + max_update_request
            };
            for idx in (sender_index as u64)..max_sender_index {
                match NullifierSetInsertionOrder::<T>::try_get(idx) {
                    Ok(next) => senders.push(next),
                    _ => return (false, senders),
                }
            }
            (
                NullifierSetInsertionOrder::<T>::contains_key(max_sender_index as u64),
                senders,
            )
        }

        /// Returns the diff of ledger state since the given `checkpoint`, `max_receivers`, and
        /// `max_senders`.
        #[inline]
        pub fn pull_ledger_diff(
            checkpoint: Checkpoint,
            max_receivers: u64,
            max_senders: u64,
        ) -> PullResponse {
            let (more_receivers, receivers) =
                Self::pull_receivers(*checkpoint.receiver_index, max_receivers);
            let (more_senders, senders) = Self::pull_senders(checkpoint.sender_index, max_senders);
            let senders_receivers_total = (0..=255)
                .map(|i| ShardTrees::<T>::get(i).current_path.leaf_index as u128)
                .sum::<u128>()
                + NullifierSetSize::<T>::get() as u128;
            PullResponse {
                should_continue: more_receivers || more_senders,
                receivers,
                senders,
                senders_receivers_total,
            }
        }

        /// Returns the account ID of this pallet.
        #[inline]
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
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
                    .post(
                        &load_transfer_parameters(),
                        &mut Ledger(PhantomData),
                        &(),
                        sources,
                        sinks,
                    )
                    .map_err(Error::<T>::from)?
                    .convert(origin),
            );
            Ok(().into())
        }

        ///
        #[inline]
        pub fn id_from_field(id: [u8; 32]) -> Option<StandardAssetId> {
            if u128::from_le_bytes(Array::from_iter(id[16..32].iter().copied()).into()) == 0 {
                Some(u128::from_le_bytes(
                    Array::from_iter(id[0..16].iter().copied()).into(),
                ))
            } else {
                None
            }
        }

        ///
        #[inline]
        pub fn field_from_id(id: StandardAssetId) -> [u8; 32] {
            into_array_unchecked([id.to_le_bytes(), [0; 16]].concat())
        }
    }
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

impl<T> SenderLedger<config::Parameters> for Ledger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxoAccumulatorOutput = Wrap<config::UtxoAccumulatorOutput>;
    type ValidNullifier = Wrap<config::Nullifier>;

    #[inline]
    fn is_unspent(&self, nullifier: config::Nullifier) -> Option<Self::ValidNullifier> {
        if NullifierCommitmentSet::<T>::contains_key(encode(nullifier.commitment)) {
            None
        } else {
            Some(Wrap(nullifier))
        }
    }

    #[inline]
    fn has_matching_utxo_accumulator_output(
        &self,
        output: config::UtxoAccumulatorOutput,
    ) -> Option<Self::ValidUtxoAccumulatorOutput> {
        if UtxoAccumulatorOutputs::<T>::contains_key(encode(output)) {
            Some(Wrap(output))
        }
        None
    }

    #[inline]
    fn spend_all<I>(&mut self, super_key: &Self::SuperPostingKey, iter: I)
    where
        I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidNullifier)>,
    {
        let _ = super_key;
        let index = NullifierSetSize::<T>::get();
        let mut i = 0;
        for (_, nullifier) in iter {
            let nullifier_commitment = encode(nullifier.0.commitment);
            NullifierCommitmentSet::<T>::insert(nullifier_commitment, ());
            NullifierSetInsertionOrder::<T>::insert(
                index + i,
                (
                    nullifier_commitment,
                    OutgoingNote::from(nullifier.0.outgoing_note),
                ),
            );
            i += 1;
        }
        if i != 0 {
            NullifierSetSize::<T>::set(index + i);
        }
    }
}

impl<T> ReceiverLedger<config::Parameters> for Ledger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxo = Wrap<config::Utxo>;

    #[inline]
    fn is_not_registered(&self, utxo: config::Utxo) -> Option<Self::ValidUtxo> {
        if UtxoSet::<T>::contains_key(Utxo::from(utxo)) {
            None
        } else {
            Some(Wrap(utxo))
        }
    }

    #[inline]
    fn register_all<I>(&mut self, super_key: &Self::SuperPostingKey, iter: I)
    where
        I: IntoIterator<Item = (Self::ValidUtxo, config::Note)>,
    {
        let _ = super_key;
        let utxo_accumulator_model = config::UtxoAccumulatorModel::decode(
            manta_parameters::pay::testnet::parameters::UtxoAccumulatorModel::get()
                .expect("Checksum did not match."),
        )
        .expect("Unable to decode the Merkle Tree Parameters.");
        let utxo_accumulator_item_hash = config::utxo::v2::UtxoAccumulatorItemHash::decode(
            manta_parameters::pay::testnet::parameters::UtxoAccumulatorItemHash::get()
                .expect("Checksum did not match."),
        )
        .expect("Unable to decode the Merkle Tree Item Hash.");
        let mut shard_indices = iter
            .into_iter()
            .map(|(utxo, note)| {
                (
                    MerkleTreeConfiguration::tree_index(
                        &utxo.0.item_hash(&utxo_accumulator_item_hash, &mut ()),
                    ),
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
            let cloned_tree = tree.clone();
            let mut next_root = Option::<config::UtxoAccumulatorOutput>::None;
            let mut current_path = cloned_tree.current_path.into();
            println!("Tree index {shard_index}");
            if cloned_tree.leaf_digest.is_some() {
                let old_root = merkle_tree::path::CurrentPath::root(
                    &current_path,
                    &utxo_accumulator_model,
                    &cloned_tree.leaf_digest.unwrap(),
                );
                println!("Old Root: {:#?}", old_root);
            } else {
                println!("Old root is None");
            }
            for (utxo, note) in insertions {
                next_root = Some(
                    merkle_tree::single_path::raw::insert(
                        &utxo_accumulator_model,
                        &mut tree.leaf_digest,
                        &mut current_path,
                        utxo.item_hash(&utxo_accumulator_item_hash, &mut ()),
                    )
                    .expect("If this errors, then we have run out of Merkle Tree capacity."),
                );
                println!("Root: {:#?}", next_root.unwrap());
                let next_index = current_path.leaf_index().0 as u64;
                let utxo = Utxo::from(utxo);
                UtxoSet::<T>::insert(utxo, ());
                Shards::<T>::insert(
                    shard_index,
                    next_index,
                    (utxo, FullIncomingNote::from(note)),
                );
            }
            tree.current_path = current_path.into();
            if let Some(next_root) = next_root {
                ShardTrees::<T>::insert(shard_index, tree);
                UtxoAccumulatorOutputs::<T>::insert(encode(next_root), ());
            }
        }
    }
}

impl<T> TransferLedger<config::Config> for Ledger<T>
where
    T: Config,
{
    type SuperPostingKey = ();
    type AccountId = T::AccountId;
    type Event = PreprocessedEvent<T>;
    type UpdateError = FungibleLedgerError;
    type ValidSourceAccount = WrapPair<Self::AccountId, AssetValue>;
    type ValidSinkAccount = WrapPair<Self::AccountId, AssetValue>;
    type ValidProof = Wrap<()>;

    #[inline]
    fn check_source_accounts<I>(
        &self,
        asset_id: &config::AssetId,
        sources: I,
    ) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<config::Config, Self::AccountId>>
    where
        I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        sources
            .map(move |(account_id, withdraw)| {
                FungibleLedger::<T>::can_withdraw(
                    Pallet::<T>::id_from_field(encode(asset_id)).expect("FIXME"),
                    &account_id,
                    &withdraw,
                    ExistenceRequirement::KeepAlive,
                )
                .map(|_| WrapPair(account_id.clone(), withdraw))
                .map_err(|_| InvalidSourceAccount {
                    account_id,
                    asset_id: *asset_id,
                    withdraw,
                })
            })
            .collect()
    }

    #[inline]
    fn check_sink_accounts<I>(
        &self,
        asset_id: &config::AssetId,
        sinks: I,
    ) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<config::Config, Self::AccountId>>
    where
        I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        // NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
        // pass the data forward.
        sinks
            .map(move |(account_id, deposit)| {
                FungibleLedger::<T>::can_deposit(
                    Pallet::<T>::id_from_field(encode(asset_id)).expect("FIXME"),
                    &account_id,
                    deposit,
                    false,
                )
                .map(|_| WrapPair(account_id.clone(), deposit))
                .map_err(|_| InvalidSinkAccount {
                    account_id,
                    asset_id: *asset_id,
                    deposit,
                })
            })
            .collect()
    }

    #[inline]
    fn is_valid(
        &self,
        posting_key: TransferPostingKeyRef<config::Config, Self>,
    ) -> Option<(Self::ValidProof, Self::Event)> {
        let (mut verifying_context, event) =
            match TransferShape::from_posting_key_ref(&posting_key)? {
                TransferShape::ToPrivate => (
                    manta_parameters::pay::testnet::verifying::ToPrivate::get()
                        .expect("Checksum did not match."),
                    PreprocessedEvent::<T>::ToPrivate {
                        asset: Asset::new(
                            encode(posting_key.asset_id.unwrap()),
                            posting_key.sources[0].1,
                        ),
                        source: posting_key.sources[0].0.clone(),
                    },
                ),
                TransferShape::PrivateTransfer => (
                    manta_parameters::pay::testnet::verifying::PrivateTransfer::get()
                        .expect("Checksum did not match."),
                    PreprocessedEvent::<T>::PrivateTransfer,
                ),
                TransferShape::ToPublic => (
                    manta_parameters::pay::testnet::verifying::ToPublic::get()
                        .expect("Checksum did not match."),
                    PreprocessedEvent::<T>::ToPublic {
                        asset: Asset::new(
                            encode(posting_key.asset_id.unwrap()),
                            posting_key.sinks[0].1,
                        ),
                        sink: posting_key.sinks[0].0.clone(),
                    },
                ),
            };
        posting_key
            .has_valid_proof(
                &config::VerifyingContext::decode(&mut verifying_context)
                    .expect("Unable to decode the verifying context."),
            )
            .ok()?
            .then_some((Wrap(()), event))
    }

    #[inline]
    fn update_public_balances(
        &mut self,
        super_key: &TransferLedgerSuperPostingKey<config::Config, Self>,
        asset_id: config::AssetId,
        sources: Vec<SourcePostingKey<config::Config, Self>>,
        sinks: Vec<SinkPostingKey<config::Config, Self>>,
        proof: Self::ValidProof,
    ) -> Result<(), Self::UpdateError> {
        let _ = (proof, super_key);
        for WrapPair(account_id, withdraw) in sources {
            FungibleLedger::<T>::transfer(
                Pallet::<T>::id_from_field(encode(asset_id)).expect("FIXME"),
                &account_id,
                &Pallet::<T>::account_id(),
                withdraw,
                ExistenceRequirement::KeepAlive,
            )?;
        }
        for WrapPair(account_id, deposit) in sinks {
            FungibleLedger::<T>::transfer(
                Pallet::<T>::id_from_field(encode(asset_id)).expect("FIXME"),
                &Pallet::<T>::account_id(),
                &account_id,
                deposit,
                ExistenceRequirement::KeepAlive,
            )?;
        }
        Ok(())
    }
}
