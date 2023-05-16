// Copyright 2020-2023 Manta Network.
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

use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use errors::{ReceiverLedgerError, SenderLedgerError, TransferLedgerError};
use frame_support::{
    pallet_prelude::*, traits::tokens::ExistenceRequirement, transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use manta_pay::{
    config::{self, utxo::MerkleTreeConfiguration},
    manta_accounting::transfer::{
        self,
        canonical::TransferShape,
        receiver::{ReceiverLedger, ReceiverPostError},
        sender::{SenderLedger, SenderPostError},
        InvalidAuthorizationSignature, InvalidSinkAccount, InvalidSourceAccount, ProofSystemError,
        SinkPostingKey, SourcePostingKey, TransferLedger, TransferLedgerSuperPostingKey,
        TransferPostingKeyRef,
    },
    manta_crypto::merkle_tree::{self, forest::Configuration as _},
    manta_parameters::{self, Get as _},
    manta_util::codec::Decode as _,
    parameters::load_transfer_parameters,
};
use manta_primitives::assets::{self, AssetConfig, FungibleLedger as _};
use manta_support::manta_pay::{
    asset_value_decode, asset_value_encode, fp_decode, fp_encode, id_from_field, AccountId, Asset,
    AssetValue, Checkpoint, FullIncomingNote, InitialSyncResponse, MTParametersError,
    NullifierCommitment, OutgoingNote, PullResponse, ReceiverChunk, SenderChunk, StandardAssetId,
    TransferPost, Utxo, UtxoAccumulatorOutput, UtxoItemHashError, UtxoMerkleTreePath,
    VerifyingContextError, Wrap, WrapPair,
};
use manta_util::codec::Encode;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

pub mod errors;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "runtime")]
pub mod runtime;

/// Fungible Ledger Error
pub type FungibleLedgerError = assets::FungibleLedgerError<StandardAssetId, AssetValue>;

/// MantaPay Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;
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
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
    impl<T: Config> Pallet<T>
    where
        T::AccountId: From<AccountId> + Into<AccountId>,
    {
        /// Transforms some public assets into private ones using `post`, withdrawing the public
        /// assets from the `origin` account.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::to_private())]
        #[transactional]
        pub fn to_private(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            ensure!(
                post.sources.len() == 1
                    && post.sender_posts.is_empty()
                    && post.receiver_posts.len() == 1
                    && post.sinks.is_empty()
                    && post.sink_accounts.is_empty(),
                Error::<T>::InvalidShape
            );
            // Prevent ledger bloat from zero value transactions
            for source in post.sources.iter() {
                ensure!(
                    asset_value_decode(*source) > 0u128,
                    Error::<T>::ZeroTransfer
                );
            }
            Self::post_transaction(None, vec![origin], vec![], post)
        }

        /// Transforms some private assets into public ones using `post`, depositing the public
        /// assets in the `origin` account.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::to_public())]
        #[transactional]
        pub fn to_public(origin: OriginFor<T>, post: TransferPost) -> DispatchResultWithPostInfo {
            let _origin = ensure_signed(origin)?;
            ensure!(
                post.sources.is_empty()
                    && post.sender_posts.len() == 2
                    && post.receiver_posts.len() == 1
                    && post.sinks.len() == 1
                    && post.sink_accounts.len() == 1,
                Error::<T>::InvalidShape
            );
            for sink in post.sinks.iter() {
                ensure!(asset_value_decode(*sink) > 0u128, Error::<T>::ZeroTransfer);
            }
            Self::post_transaction(
                None,
                vec![],
                // Possibility to panic must be prevented by preceding shape check
                vec![T::AccountId::from(post.sink_accounts[0])],
                post,
            )
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
            ensure!(
                post.sources.is_empty()
                    && post.sender_posts.len() == 2
                    && post.receiver_posts.len() == 2
                    && post.sinks.is_empty()
                    && post.sink_accounts.is_empty(),
                Error::<T>::InvalidShape
            );
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
                id_from_field(asset.id).ok_or(Error::<T>::InvalidAssetId)?,
                &origin,
                &sink,
                asset_value_decode(asset.value),
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
            origin: Option<T::AccountId>,
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
        /// Zero Transfer
        ///
        /// Transfers cannot include amounts equal to zero.
        ZeroTransfer,

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

        /// Fungible Ledger Encode Error
        FungibleLedgerEncodeError,

        /// Sender Ledger Fp Encoding failed.
        SenderLedgerFpEncodeError,

        /// Sender Ledger [`OutgoingNote`] failed to decode
        SenderLedgerOutgoingNodeDecodeFailed,

        /// Reciever Ledger Utxo decode failed
        ReceiverLedgerUtxoDecodeFailed,

        /// Receiver Ledger Wrong Checksum Error
        ReceiverLedgerChecksumError,

        /// Receiver Ledger Merkle Tree Parameters Decoding Error
        ReceiverLedgerMTParametersDecodeError,

        /// Receiver Ledger Utxo Accumulator Item Hash Decoding Error
        ReceiverLedgerUtxoAccumulatorItemHashDecodeError,

        /// Receiver Ledger Merkle Tree Out of Capacity Error
        ReceiverLedgerMerkleTreeCapacityError,

        /// Receiver Ledger Field Element Encoding Error
        ReceiverLedgerFpEncodeError,

        /// Receiver Ledger Field Element Decoding Error
        ReceiverLedgerFpDecodeError,

        /// Receiver Ledger Path Decoding Error
        ReceiverLedgerPathDecodeError,

        /// Receiver Ledger Full Incoming Note Decoding Error
        ReceiverLedgerFullNoteDecodeError,

        /// Transfer Ledger Wrong Checksum Error
        TransferLedgerChecksumError,

        /// Transfer Ledger `VerifyingContext` cannont be decoded
        TransferLedgerVerifyingContextDecodeError,

        /// Transer Ledger Field Element Encoding Error
        TransferLedgerFpEncodeError,

        /// Transfer Ledger Unknown Asset
        TransferLedgerUnknownAsset,

        /// Transfer Ledger Proof Error
        TransferLedgerProofSystemFailed,

        /// Marker Error, this error exists for `PhantomData` should never happen
        Marker,
    }

    impl<T> Pallet<T>
    where
        T: Config,
        T::AccountId: From<AccountId> + Into<AccountId>,
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

        /// Returns the diff of ledger state since the given `checkpoint` and `max_receivers` to
        /// perform the initial synchronization.
        #[inline]
        pub fn initial_pull(checkpoint: Checkpoint, max_receivers: u64) -> InitialSyncResponse {
            let (should_continue, receivers) =
                Self::pull_receivers(*checkpoint.receiver_index, max_receivers);
            let utxo_data = receivers.into_iter().map(|receiver| receiver.0).collect();
            let membership_proof_data = (0..=255)
                .map(|i| ShardTrees::<T>::get(i).current_path)
                .collect();
            let nullifier_count = NullifierSetSize::<T>::get() as u128;
            InitialSyncResponse {
                should_continue,
                utxo_data,
                membership_proof_data,
                nullifier_count,
            }
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
                NullifierSetInsertionOrder::<T>::contains_key(max_sender_index),
                senders,
            )
        }

        /// Returns ledger total count
        /// In the initial state of the ledger, the total value will be 256;
        /// if we want to get an accurate value, we need to request `pull_receivers` to fix this value;
        /// but a simple total count interface does not need to add more complicated logic.
        #[inline]
        pub fn pull_ledger_total_count() -> [u8; 16] {
            let receivers_total = (0..=255)
                .map(|i| ShardTrees::<T>::get(i).current_path.leaf_index as u128)
                .sum::<u128>()
                + 256u128;
            let senders_total = NullifierSetSize::<T>::get() as u128;
            asset_value_encode(receivers_total + senders_total)
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

            PullResponse {
                should_continue: more_receivers || more_senders,
                receivers,
                senders,
                senders_receivers_total: Self::pull_ledger_total_count(),
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
                        sources.into_iter().map(Into::into).collect(),
                        sinks.into_iter().map(Into::into).collect(),
                    )
                    .map_err(Error::<T>::from)?
                    .convert(origin),
            );
            Ok(().into())
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
            Self::PrivateTransfer => Event::PrivateTransfer { origin },
            Self::ToPublic { asset, sink } => Event::ToPublic { asset, sink },
        }
    }
}

/// Ledger
struct Ledger<T>(PhantomData<T>)
where
    T: Config;

impl<T> SenderLedger<config::Parameters> for Ledger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxoAccumulatorOutput = Wrap<config::UtxoAccumulatorOutput>;
    type ValidNullifier = Wrap<config::Nullifier>;
    type Error = SenderLedgerError;

    #[inline]
    fn is_unspent(
        &self,
        nullifier: config::Nullifier,
    ) -> Result<Self::ValidNullifier, Self::Error> {
        if NullifierCommitmentSet::<T>::contains_key(
            fp_encode(nullifier.nullifier.commitment).map_err(SenderLedgerError::FpEncodeError)?,
        ) {
            Err(SenderLedgerError::AssetSpent)
        } else {
            Ok(Wrap(nullifier))
        }
    }

    #[inline]
    fn has_matching_utxo_accumulator_output(
        &self,
        output: config::UtxoAccumulatorOutput,
    ) -> Result<Self::ValidUtxoAccumulatorOutput, Self::Error> {
        let accumulator_output = fp_encode(output).map_err(SenderLedgerError::FpEncodeError)?;
        // NOTE: Checking for an empty(zeroed) byte array. This happens for UTXOs with `value = 0`,
        // for which you dont need a membership proof, but you still need a root (in this case
        // zeroed).
        if accumulator_output == [0u8; 32]
            || UtxoAccumulatorOutputs::<T>::contains_key(accumulator_output)
        {
            return Ok(Wrap(output));
        }
        Err(SenderLedgerError::InvalidUtxoAccumulatorOutput)
    }

    #[inline]
    fn spend_all<I>(
        &mut self,
        super_key: &Self::SuperPostingKey,
        iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidNullifier)>,
    {
        let _ = super_key;
        let index = NullifierSetSize::<T>::get();
        let mut i = 0;
        for (_, nullifier) in iter {
            let nullifier_commitment = fp_encode(nullifier.0.nullifier.commitment)
                .map_err(SenderLedgerError::FpEncodeError)?;
            NullifierCommitmentSet::<T>::insert(nullifier_commitment, ());
            NullifierSetInsertionOrder::<T>::insert(
                index + i,
                (
                    nullifier_commitment,
                    OutgoingNote::try_from(nullifier.0.outgoing_note)
                        .map_err(SenderLedgerError::OutgoingNoteDecodeError)?,
                ),
            );
            i += 1;
        }
        if i != 0 {
            NullifierSetSize::<T>::set(index + i);
        }
        Ok(())
    }
}

impl<T> ReceiverLedger<config::Parameters> for Ledger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxo = Wrap<config::Utxo>;
    type Error = ReceiverLedgerError<T>;

    #[inline]
    fn is_not_registered(&self, utxo: config::Utxo) -> Result<Self::ValidUtxo, Self::Error> {
        if UtxoSet::<T>::contains_key(
            Utxo::try_from(utxo).map_err(ReceiverLedgerError::UtxoDecodeError)?,
        ) {
            Err(ReceiverLedgerError::AssetRegistered)
        } else {
            Ok(Wrap(utxo))
        }
    }

    #[inline]
    fn register_all<I>(
        &mut self,
        super_key: &Self::SuperPostingKey,
        iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxo, config::Note)>,
    {
        let _ = super_key;
        let utxo_accumulator_model = config::UtxoAccumulatorModel::decode(
            manta_parameters::pay::parameters::UtxoAccumulatorModel::get()
                .ok_or(ReceiverLedgerError::ChecksumError)?,
        )
        .map_err(ReceiverLedgerError::MTParametersDecodeError)?;
        let utxo_accumulator_item_hash = config::utxo::UtxoAccumulatorItemHash::decode(
            manta_parameters::pay::parameters::UtxoAccumulatorItemHash::get()
                .ok_or(ReceiverLedgerError::ChecksumError)?,
        )
        .map_err(ReceiverLedgerError::UtxoAccumulatorItemHashDecodeError)?;
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
            let mut current_path = cloned_tree
                .current_path
                .try_into()
                .map_err(ReceiverLedgerError::PathDecodeError)?;
            let mut leaf_digest = tree
                .leaf_digest
                .map(|x| fp_decode(x.to_vec()).map_err(ReceiverLedgerError::FpDecodeError))
                .map_or(Ok(None), |r| r.map(Some))?;
            for (utxo, note) in insertions {
                next_root = Some(
                    merkle_tree::single_path::raw::insert(
                        &utxo_accumulator_model,
                        &mut leaf_digest,
                        &mut current_path,
                        utxo.item_hash(&utxo_accumulator_item_hash, &mut ()),
                    )
                    .ok_or(ReceiverLedgerError::MerkleTreeCapacityError)?,
                );
                let next_index = current_path.leaf_index().0 as u64;
                let utxo = Utxo::try_from(utxo).map_err(ReceiverLedgerError::UtxoDecodeError)?;
                UtxoSet::<T>::insert(utxo, ());
                Shards::<T>::insert(
                    shard_index,
                    next_index,
                    (
                        utxo,
                        FullIncomingNote::try_from(note)
                            .map_err(ReceiverLedgerError::FullNoteDecodeError)?,
                    ),
                );
            }
            tree.current_path = current_path
                .try_into()
                .map_err(ReceiverLedgerError::PathDecodeError)?;
            tree.leaf_digest = leaf_digest
                .map(|x| fp_encode(x).map_err(ReceiverLedgerError::FpEncodeError))
                .map_or(Ok(None), |r| r.map(Some))?;
            if let Some(next_root) = next_root {
                ShardTrees::<T>::insert(shard_index, tree);
                UtxoAccumulatorOutputs::<T>::insert(
                    fp_encode(next_root).map_err(ReceiverLedgerError::FpEncodeError)?,
                    (),
                );
            }
        }
        Ok(())
    }
}

impl<T> TransferLedger<config::Config> for Ledger<T>
where
    T: Config,
    T::AccountId: From<AccountId> + Into<AccountId>,
{
    type SuperPostingKey = ();
    type Event = PreprocessedEvent<T>;
    type ValidSourceAccount = WrapPair<AccountId, AssetValue>;
    type ValidSinkAccount = WrapPair<AccountId, AssetValue>;
    type ValidProof = Wrap<()>;
    type Error = TransferLedgerError<T>;

    #[inline]
    fn check_source_accounts<I>(
        &self,
        asset_id: &config::AssetId,
        sources: I,
    ) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<config::Config, AccountId>>
    where
        I: Iterator<Item = (AccountId, config::AssetValue)>,
    {
        sources
            .map(move |(account_id, withdraw)| {
                FungibleLedger::<T>::can_withdraw(
                    id_from_field(fp_encode(*asset_id).map_err(|_e| InvalidSourceAccount {
                        account_id,
                        asset_id: *asset_id,
                        withdraw,
                    })?)
                    .ok_or(InvalidSourceAccount {
                        account_id,
                        asset_id: *asset_id,
                        withdraw,
                    })?,
                    &account_id.into(),
                    &withdraw,
                    ExistenceRequirement::KeepAlive,
                )
                .map(|_| WrapPair(account_id, withdraw))
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
    ) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<config::Config, AccountId>>
    where
        I: Iterator<Item = (AccountId, config::AssetValue)>,
    {
        // NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
        // pass the data forward.
        sinks
            .map(move |(account_id, deposit)| {
                FungibleLedger::<T>::can_deposit(
                    id_from_field(fp_encode(*asset_id).map_err(|_e| InvalidSinkAccount {
                        account_id,
                        asset_id: *asset_id,
                        deposit,
                    })?)
                    .ok_or(InvalidSinkAccount {
                        account_id,
                        asset_id: *asset_id,
                        deposit,
                    })?,
                    &account_id.into(),
                    deposit,
                    false,
                )
                .map(|_| WrapPair(account_id, deposit))
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
    ) -> Result<(Self::ValidProof, Self::Event), TransferLedgerError<T>> {
        let transfer_shape = TransferShape::from_posting_key_ref(&posting_key);
        let (mut verifying_context, event) = match transfer_shape
            .ok_or(TransferLedgerError::InvalidTransferShape)?
        {
            TransferShape::ToPrivate => {
                if let Some(asset_id) = posting_key.asset_id.or(None) {
                    let asset_id =
                        fp_encode(asset_id).map_err(TransferLedgerError::FpEncodeError)?;
                    (
                        manta_parameters::pay::verifying::ToPrivate::get()
                            .ok_or(TransferLedgerError::ChecksumError)?,
                        PreprocessedEvent::<T>::ToPrivate {
                            asset: Asset::new(
                                asset_id,
                                asset_value_encode(posting_key.sources[0].1),
                            ),
                            source: posting_key.sources[0].0.into(),
                        },
                    )
                } else {
                    return Err(TransferLedgerError::UnknownAsset);
                }
            }
            TransferShape::PrivateTransfer => (
                manta_parameters::pay::verifying::PrivateTransfer::get()
                    .ok_or(TransferLedgerError::ChecksumError)?,
                PreprocessedEvent::<T>::PrivateTransfer,
            ),
            TransferShape::ToPublic => {
                if let Some(asset_id) = posting_key.asset_id.or(None) {
                    let asset_id =
                        fp_encode(asset_id).map_err(TransferLedgerError::FpEncodeError)?;
                    (
                        manta_parameters::pay::verifying::ToPublic::get()
                            .ok_or(TransferLedgerError::ChecksumError)?,
                        PreprocessedEvent::<T>::ToPublic {
                            asset: Asset::new(asset_id, asset_value_encode(posting_key.sinks[0].1)),
                            sink: posting_key.sinks[0].0.into(),
                        },
                    )
                } else {
                    return Err(TransferLedgerError::UnknownAsset);
                }
            }
        };
        let verification = posting_key
            .has_valid_proof(
                &config::VerifyingContext::decode(&mut verifying_context)
                    .map_err(TransferLedgerError::VerifyingContextDecodeError)?,
            )
            .map_err(TransferLedgerError::ProofSystemError)?;
        if verification {
            Ok((Wrap(()), event))
        } else {
            Err(TransferLedgerError::InvalidProof)
        }
    }

    #[inline]
    fn update_public_balances(
        &mut self,
        super_key: &TransferLedgerSuperPostingKey<config::Config, Self>,
        asset_id: config::AssetId,
        sources: Vec<SourcePostingKey<config::Config, Self>>,
        sinks: Vec<SinkPostingKey<config::Config, Self>>,
        proof: Self::ValidProof,
    ) -> Result<(), TransferLedgerError<T>> {
        let _ = (proof, super_key);
        let asset_id_type =
            id_from_field(fp_encode(asset_id).map_err(TransferLedgerError::FpEncodeError)?)
                .ok_or(TransferLedgerError::UnknownAsset)?;
        for WrapPair(account_id, withdraw) in sources {
            FungibleLedger::<T>::transfer(
                asset_id_type,
                &account_id.into(),
                &Pallet::<T>::account_id(),
                withdraw,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(TransferLedgerError::FungibleLedgerError)?;
        }
        for WrapPair(account_id, deposit) in sinks {
            FungibleLedger::<T>::transfer(
                asset_id_type,
                &Pallet::<T>::account_id(),
                &account_id.into(),
                deposit,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(TransferLedgerError::FungibleLedgerError)?;
        }
        Ok(())
    }
}
