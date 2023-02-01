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

//! # Zknft Module
//!
//! Zknft is a Multi-Asset Shielded Payment protocol.
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
    asset_value_decode, Asset, AssetValue,
    FullIncomingNote, NullifierCommitment, OutgoingNote, TransferPost,
    Utxo, UtxoAccumulatorOutput, UtxoMerkleTreePath,
};
use alloc::{vec};
use frame_support::{traits::tokens::ExistenceRequirement, transactional, PalletId};
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use manta_primitives::{
    assets::{AssetConfig, AssetRegistry, FungibleLedger as _, IsFungible},
    types::StandardAssetId,
    nft::NonFungibleLedger as _,
};
use sp_runtime::traits::{Get};

// pub use crate::ledger::Ledger;
pub use crate::types::{Checkpoint, RawCheckpoint, PullResponse};
pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

pub mod types;
pub mod weights;
pub mod ledger;
pub mod ledgers;
pub mod errors;
pub mod impl_pay;
pub mod common;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "runtime")]
pub mod runtime;

/// Zknft Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::StorageVersion};
    use frame_system::pallet_prelude::*;
    use crate::types::AssetType;

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

    pub(crate) type NonFungibleLedger<T> =
        <<T as Config>::AssetConfig as AssetConfig<T>>::NonFungibleLedger;

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
        pub fn to_private(origin: OriginFor<T>, asset_type: AssetType, post: TransferPost) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            Self::post_transaction(None, vec![origin], vec![], post, asset_type)
        }

        /// Transforms some private assets into public ones using `post`, depositing the public
        /// assets in the `origin` account.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::to_public())]
        #[transactional]
        pub fn to_public(origin: OriginFor<T>, asset_type: AssetType, post: TransferPost) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            Self::post_transaction(None, vec![], vec![origin], post, asset_type)
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
            asset_type: AssetType,
            post: TransferPost,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            Self::post_transaction(Some(origin), vec![], vec![], post, asset_type)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::private_transfer())]
        #[transactional]
        pub fn private_transfer_asset(
            origin: OriginFor<T>,
            asset_id: StandardAssetId,
            asset_type: AssetType,
            origin_zk_address: [u8; 32],
            post: TransferPost,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            // this method is used for sbt private transfer.
            // only support private transfer by project manager.
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistry::get_metadata(&asset_id)
                .ok_or(Error::<T>::PublicUpdateInvalidMint)?;

            let zk_address = metadata.get_origin_zkaddress().ok_or(Error::<T>::PrivateTransferZkAddressNotExist)?;
            ensure!(origin_zk_address == zk_address, Error::<T>::PrivateTransferZkAddressNotMatch);

            // TODO: account compare
            metadata.get_origin_account().ok_or(Error::<T>::PrivateTransferAccountNotExist)?;
            // ensure!(origin.clone() == origin_account, Error::<T>::PrivateTransferAccountNotMatch);

            Self::post_transaction(Some(origin), vec![], vec![], post, asset_type)
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
            let id = Self::id_from_field(asset.id).ok_or(Error::<T>::InvalidAssetId)?;
            let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistry::get_metadata(&id)
                .ok_or(Error::<T>::PublicUpdateUnknownAsset)?;
            if metadata.is_fungible() {
                FungibleLedger::<T>::transfer(
                    id,
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
            } else {
                let (collection_id, item_id) = metadata
                    .get_non_fungible_id()
                    .ok_or(Error::<T>::PublicUpdateUnknownAsset)?;
                let owner = NonFungibleLedger::<T>::owner(collection_id, item_id)
                    .ok_or(Error::<T>::PublicUpdateUnknownAsset)?;
                ensure!(owner == origin, Error::<T>::PublicUpdateInvalidTransfer);

                NonFungibleLedger::<T>::transfer(collection_id, item_id, &sink)
                    .map_err(Error::<T>::from)?;

                Self::deposit_event(Event::Transfer {
                    asset,
                    source: origin,
                    sink,
                });
                Ok(().into())
            }
        }
    }

    /// Event
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
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

        /// Encode Error
        EncodeError,

        ///
        PrivateTransferNotAllowed,

        ///
        PrivateTransferZkAddressNotExist,
        ///
        PrivateTransferZkAddressNotMatch,

        ///
        PrivateTransferAccountNotExist,

        ///
        PrivateTransferAccountNotMatch,
    }
}

