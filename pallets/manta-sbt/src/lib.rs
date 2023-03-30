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

//! # MantaSBT Module
//!
//! MantaSBT creates non-transferable nfts (soul-bound) as unspendable UTXOs
//!
//! ## Overview
//!
//! There are two calls `reserve_sbt` and `to_private`.
//!
//! `reserve_sbt`: Reserves unique `AssetIds` for user to later mint into sbt.
//!
//! `to_private`: Mints SBT with signer generated `TransferPost` using previously reserved `AssetId`. Stores relevant metadata with associated `AssetId`
//!
//! Ownership of SBT is recorded as a corresponding UTXO. You can prove you own SBT using `TransactionData` which can reconstruct UTXO

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency, StorageVersion},
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use manta_support::manta_pay::{
    asset_value_encode, fp_decode, fp_encode, id_from_field, AccountId, AssetValue, Checkpoint,
    FullIncomingNote, MTParametersError, Proof, PullResponse, ReceiverChunk, StandardAssetId,
    TransferPost, Utxo, UtxoAccumulatorOutput, UtxoItemHashError, UtxoMerkleTreePath,
    VerifyingContextError, Wrap, WrapPair,
};
use sha3::{Digest, Keccak256};
use sp_core::{H160, H256, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    ArithmeticError,
};

use errors::{ReceiverLedgerError, SenderLedgerError, TransferLedgerError};
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
use manta_util::codec::Encode;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod errors;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmark;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "runtime")]
pub mod runtime;

// One in encoded form, used to check that value input in `ToPrivate` post is one
const ENCODED_ONE: [u8; 16] = 1u128.to_le_bytes();

/// Type alias for currency balance.
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Eth based address
type EvmAddress = H160;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Eip712Signature = [u8; 65];

/// Enum with each possible type of whitelisted Eth Address
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EvmAddressType {
    /// BAB holder
    Bab(EvmAddress),
    /// Galxe holder
    Galxe(EvmAddress),
}

/// Different mint types
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MintType {
    Bab,
    Galxe,
}

impl From<EvmAddressType> for MintType {
    fn from(address_type: EvmAddressType) -> Self {
        match address_type {
            EvmAddressType::Bab(_) => MintType::Bab,
            EvmAddressType::Galxe(_) => MintType::Galxe,
        }
    }
}

/// Status of
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MintStatus {
    Available(StandardAssetId),
    AlreadyMinted,
}

/// MantaSBT Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Pallet
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// The module configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The origin which can change the privileged whitelist account and set time range for mints
        type AdminOrigin: EnsureOrigin<Self::Origin>;

        /// Pallet ID
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Number of unique Asset Ids reserved per `reserve_sbt` call, is the amount of SBTs allowed to be minted
        #[pallet::constant]
        type MintsPerReserve: Get<u16>;

        /// Price to reserve Asset Ids
        #[pallet::constant]
        type ReservePrice: Get<BalanceOf<Self>>;

        /// Max size in bytes of stored metadata
        #[pallet::constant]
        type SbtMetadataBound: Get<u32>;

        /// ChainId for `Eip712Signature`
        #[pallet::constant]
        type ChainId: Get<u64>;
    }

    /// Counter for SBT AssetId. Increments by one everytime a new asset id is requested.
    ///
    /// Should only ever be modified by `next_sbt_id_and_increment()`
    #[pallet::storage]
    pub(super) type NextSbtId<T: Config> = StorageValue<_, StandardAssetId, OptionQuery>;

    /// Account that can add evm accounts to whitelist
    #[pallet::storage]
    pub(super) type WhitelistAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Whitelist for Evm Accounts
    #[pallet::storage]
    pub(super) type EvmAddressWhitelist<T: Config> =
        StorageMap<_, Blake2_128Concat, EvmAddressType, MintStatus, OptionQuery>;

    /// Range of time at which evm mints are possible.
    #[pallet::storage]
    pub(super) type MintTimeRange<T: Config> =
        StorageMap<_, Blake2_128Concat, MintType, (T::Moment, Option<T::Moment>), OptionQuery>;

    /// SBT Metadata maps `StandardAsset` to the correstonding SBT metadata
    ///
    /// Metadata is raw bytes that correspond to an image
    #[pallet::storage]
    pub(super) type SbtMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        StandardAssetId,
        BoundedVec<u8, T::SbtMetadataBound>,
        OptionQuery,
    >;

    /// Whitelists accounts to be able to mint SBTs with designated `StandardAssetId`
    #[pallet::storage]
    pub(super) type ReservedIds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (StandardAssetId, StandardAssetId),
        OptionQuery,
    >;

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

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: From<AccountId> + Into<AccountId>,
    {
        /// Mints a zkSBT
        ///
        /// `TransferPost` is posted to private ledger and SBT metadata is stored onchain.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::to_private())]
        #[transactional]
        pub fn to_private(
            who: OriginFor<T>,
            post: Box<TransferPost>,
            metadata: BoundedVec<u8, T::SbtMetadataBound>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

            let (start_id, end_id) = ReservedIds::<T>::get(&who).ok_or(Error::<T>::NotReserved)?;

            // Checks that it is indeed a to_private post with a value of 1 and has correct asset_id
            Self::check_post_shape(&post, start_id)?;

            SbtMetadata::<T>::insert(start_id, metadata);
            let increment_start_id = start_id
                .checked_add(One::one())
                .ok_or(ArithmeticError::Overflow)?;

            // If `ReservedIds` are all used remove from storage, otherwise increment the next `AssetId` to be used next time for minting SBT
            if increment_start_id > end_id {
                ReservedIds::<T>::remove(&who)
            } else {
                ReservedIds::<T>::insert(&who, (increment_start_id, end_id))
            }

            Self::post_transaction(vec![who], *post)?;
            Ok(().into())
        }

        /// Reserves AssetIds to be used subsequently in `to_private` above.
        ///
        /// Increments AssetManager's AssetId counter.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::reserve_sbt())]
        #[transactional]
        pub fn reserve_sbt(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Charges fee to reserve AssetIds
            <T as pallet::Config>::Currency::transfer(
                &who,
                &Self::account_id(),
                T::ReservePrice::get(),
                ExistenceRequirement::KeepAlive,
            )?;

            // Reserves uniques AssetIds to be used later to mint SBTs
            let asset_id_range: Vec<StandardAssetId> = (0..T::MintsPerReserve::get())
                .map(|_| Self::next_sbt_id_and_increment())
                .collect::<Result<Vec<StandardAssetId>, _>>()?;

            // The range of `AssetIds` that are reserved as SBTs
            let start_id: StandardAssetId = *asset_id_range.first().ok_or(Error::<T>::ZeroMints)?;
            let stop_id: StandardAssetId = *asset_id_range.last().ok_or(Error::<T>::ZeroMints)?;

            ReservedIds::<T>::insert(&who, (start_id, stop_id));
            Self::deposit_event(Event::<T>::SBTReserved {
                who,
                start_id,
                stop_id,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        #[transactional]
        pub fn whitelist_evm_account(
            origin: OriginFor<T>,
            evm_address: EvmAddressType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let whitelist_account =
                WhitelistAccount::<T>::get().ok_or(Error::<T>::NotWhitelistAccount)?;
            ensure!(who == whitelist_account, Error::<T>::NotWhitelistAccount);

            let asset_id = Self::next_sbt_id_and_increment()?;
            let mint_status = MintStatus::Available(asset_id);
            EvmAddressWhitelist::<T>::insert(evm_address, mint_status);

            Self::deposit_event(Event::<T>::WhitelistEvmAddress {
                address: evm_address,
                asset_id,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        #[transactional]
        pub fn mint_sbt_eth(
            origin: OriginFor<T>,
            post: Box<TransferPost>,
            eth_signature: Eip712Signature,
            address_type: EvmAddressType,
            metadata: BoundedVec<u8, T::SbtMetadataBound>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // check that mint type is within time window
            Self::check_mint_time(address_type.into())?;

            let address = Self::verify_eip712_signature(&post.proof, &eth_signature)
                .ok_or(Error::<T>::BadSignature)?;
            // Check signature
            match address_type {
                EvmAddressType::Bab(eth_address) | EvmAddressType::Galxe(eth_address) => {
                    ensure!(eth_address == address, Error::<T>::BadSignature);
                }
            }

            let mint_status =
                EvmAddressWhitelist::<T>::get(address_type).ok_or(Error::<T>::NotWhitelisted)?;
            let asset_id = match mint_status {
                MintStatus::Available(asset) => asset,
                MintStatus::AlreadyMinted => return Err(Error::<T>::NotWhitelisted.into()),
            };
            // Change status to minted
            EvmAddressWhitelist::<T>::insert(address_type, MintStatus::AlreadyMinted);

            Self::check_post_shape(&post, asset_id)?;
            SbtMetadata::<T>::insert(asset_id, metadata);

            Self::post_transaction(vec![who], *post)?;
            Self::deposit_event(Event::<T>::MintSbtEvm {
                asset_id,
                address: address_type,
            });
            Ok(().into())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(0)]
        #[transactional]
        pub fn change_whitelist_account(
            origin: OriginFor<T>,
            account: Option<T::AccountId>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            WhitelistAccount::<T>::set(account.clone());
            Self::deposit_event(Event::<T>::ChangeWhitelistAccount { account });
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(0)]
        #[transactional]
        pub fn set_mint_time(
            origin: OriginFor<T>,
            mint_type: MintType,
            start_time: T::Moment,
            end_time: Option<T::Moment>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            MintTimeRange::<T>::insert(mint_type, (start_time, end_time));
            Self::deposit_event(Event::<T>::SetTimeRange {
                mint_type,
                start_time,
                end_time,
            });
            Ok(())
        }
    }

    /// Event
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Mints SBTs to private ledger
        MintSbt {
            /// AssetId on private leger
            asset: StandardAssetId,
            /// Source Account
            source: T::AccountId,
        },
        /// Reserve `AssetIds` as SBT
        SBTReserved {
            /// Public Account reserving SBT mints
            who: T::AccountId,
            /// Start of `AssetIds` reserved for use on private ledger
            start_id: StandardAssetId,
            /// End of `AssetIds` reserved for use private ledger, does not include this value
            stop_id: StandardAssetId,
        },
        /// Evm Address is Whitelisted
        WhitelistEvmAddress {
            /// Eth Address that is now whitelisted to mint an SBT
            address: EvmAddressType,
            /// AssetId that is reserved for above Eth address
            asset_id: StandardAssetId,
        },
        /// Sbt is minted using Whitelisted Eth account
        MintSbtEvm {
            /// Eth Address that is used to mint sbt
            address: EvmAddressType,
            /// AssetId of minted SBT
            asset_id: StandardAssetId,
        },
        /// Privileged `WhitelistAccount` is changed
        ChangeWhitelistAccount {
            /// Account that is now the new privileged whitelist account
            account: Option<T::AccountId>,
        },
        SetTimeRange {
            /// The whitelist that time range is set for (ex: BAB)
            mint_type: MintType,
            /// Start time at which minting is valid
            start_time: T::Moment,
            /// End time at which minting will no longer be valid, None represents no end time.
            end_time: Option<T::Moment>,
        },
    }

    /// Error
    #[pallet::error]
    pub enum Error<T> {
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

        /// Pallet is configured to allow no SBT mints, only happens when `MintsPerReserve` is zero
        ZeroMints,

        /// Value of `ToPrivate` post must be one.
        ValueNotOne,

        /// `AssetId` was not reserved by this account to mint
        NotReserved,

        /// SBT only allows `ToPrivate` Transactions
        NoSenderLedger,

        /// Incorrect EVM based signature
        BadSignature,

        /// Eth account is not whitelisted for free mint
        NotWhitelisted,

        /// Account is not the privileged account able to whitelist eth addresses
        NotWhitelistAccount,

        /// Minting SBT is outside defined time range
        MintNotAvailable,
    }
}

impl<T: Config> Pallet<T>
where
    T::AccountId: From<AccountId> + Into<AccountId>,
{
    /// Maximum Number of Updates per Shard (based on benchmark result)
    const PULL_MAX_RECEIVER_UPDATE_SIZE: u64 = 32768;

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

    /// Returns the diff of ledger state since the given `checkpoint` and `max_receivers`.
    /// This `Ledger` implementaion has no senders by definition, cannot transfer SBTs.
    #[inline]
    pub fn pull_ledger_diff(
        checkpoint: Checkpoint,
        max_receivers: u64,
        _max_senders: u64,
    ) -> PullResponse {
        let (more_receivers, receivers) =
            Self::pull_receivers(*checkpoint.receiver_index, max_receivers);
        let senders_receivers_total = (0..=255)
            .map(|i| ShardTrees::<T>::get(i).current_path.leaf_index as u128)
            .sum::<u128>();
        PullResponse {
            should_continue: more_receivers,
            receivers,
            senders: vec![],
            senders_receivers_total: asset_value_encode(senders_receivers_total),
        }
    }

    /// Posts the transaction encoded in `post` to the ledger, using `sources` and `sinks` as
    /// the public deposit and public withdraw accounts respectively.
    #[inline]
    fn post_transaction(
        sources: Vec<T::AccountId>,
        post: TransferPost,
    ) -> DispatchResultWithPostInfo {
        Self::deposit_event(
            config::TransferPost::try_from(post)
                .map_err(|_| Error::<T>::InvalidSerializedForm)?
                .post(
                    &load_transfer_parameters(),
                    &mut SBTLedger(PhantomData),
                    &(),
                    sources.into_iter().map(Into::into).collect(),
                    Vec::new(),
                )
                .map_err(Error::<T>::from)?
                .convert(),
        );
        Ok(().into())
    }

    /// Returns the account ID of this pallet.
    #[inline]
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    /// Returns and increments the [`NextAssetId`] by one.
    #[inline]
    fn next_sbt_id_and_increment() -> Result<StandardAssetId, DispatchError> {
        NextSbtId::<T>::try_mutate(|maybe_val| {
            match maybe_val {
                Some(current) => {
                    let id = *current;
                    *maybe_val = Some(
                        current
                            .checked_add(One::one())
                            .ok_or(ArithmeticError::Overflow)?,
                    );
                    Ok(id)
                }
                // If storage is empty, starts at value of one (Field cannot be zero)
                None => {
                    *maybe_val = Some(2);
                    Ok(One::one())
                }
            }
        })
    }

    /// Checks that post is `ToPrivate` with a value of one
    #[inline]
    fn check_post_shape(post: &TransferPost, asset_id: StandardAssetId) -> DispatchResult {
        // Checks that it is indeed a to_private post with a value of 1
        ensure!(
            post.sources.len() == 1
                && post.sender_posts.is_empty()
                && post.receiver_posts.len() == 1
                && post.sinks.is_empty(),
            Error::<T>::InvalidShape
        );
        // Checks that value is one, this is defensive as value is not used for SBT.
        ensure!(
            post.sources.first() == Some(&ENCODED_ONE),
            Error::<T>::ValueNotOne
        );

        let post_id: StandardAssetId = post
            .asset_id
            .and_then(id_from_field)
            .ok_or(Error::<T>::InvalidAssetId)?;
        // Ensure asset id is correct, only a single unique asset_id mapped to account is valid
        ensure!(asset_id == post_id, Error::<T>::InvalidAssetId);
        Ok(())
    }

    /// Checks that signature was generated using the `TransferPost` proof field as payload
    #[inline]
    fn verify_eip712_signature(proof: &Proof, sig: &[u8; 65]) -> Option<EvmAddress> {
        let msg = Self::eip712_signable_message(proof);
        let msg_hash = keccak_256(msg.as_slice());

        recover_signer(sig, &msg_hash)
    }

    // Eip-712 message to be signed
    #[inline]
    fn eip712_signable_message(proof: &Proof) -> Vec<u8> {
        let domain_separator = Self::evm_account_domain_separator();
        let payload_hash = Self::evm_account_payload_hash(proof);

        let mut msg = b"\x19\x01".to_vec();
        msg.extend_from_slice(&domain_separator);
        msg.extend_from_slice(&payload_hash);
        msg
    }

    #[inline]
    fn evm_account_payload_hash(proof: &Proof) -> [u8; 32] {
        let tx_type_hash = &sha3_256("Transaction(bytes proof)");
        let mut tx_msg = tx_type_hash.to_vec();
        tx_msg.extend_from_slice(&keccak_256(proof.as_slice()));
        keccak_256(tx_msg.as_slice())
    }

    #[inline]
    fn evm_account_domain_separator() -> [u8; 32] {
        let domain_hash =
            &sha3_256("EIP712Domain(string name,string version,uint256 chainId,bytes32 salt)");
        let mut domain_seperator_msg = domain_hash.to_vec();
        domain_seperator_msg.extend_from_slice(&sha3_256("Claim Free SBT")); // name
        domain_seperator_msg.extend_from_slice(&sha3_256("1")); // version
        domain_seperator_msg.extend_from_slice(&to_bytes(T::ChainId::get())); // chain id
        domain_seperator_msg.extend_from_slice(
            frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()).as_ref(),
        ); // genesis block hash
        keccak_256(domain_seperator_msg.as_slice())
    }

    /// Checks that mint type is available to mint within time window defined in `MintTimeRange`
    #[inline]
    fn check_mint_time(mint_type: MintType) -> DispatchResult {
        let current_time = pallet_timestamp::Pallet::<T>::now();
        let (start_time, end_time) =
            MintTimeRange::<T>::get(mint_type).ok_or(Error::<T>::MintNotAvailable)?;

        // checks that current time falls within bounds
        if start_time > current_time {
            return Err(Error::<T>::MintNotAvailable.into());
        } else {
            // Checks if end time is Some and then compares it to current time. A value of None corresponds to no ending time
            if let Some(time) = end_time {
                if time < current_time {
                    return Err(Error::<T>::MintNotAvailable.into());
                }
            }
        }
        Ok(())
    }

    /// Returns an Etherum public key derived from an Ethereum secret key.
    #[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
    pub fn eth_public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
        libsecp256k1::PublicKey::from_secret_key(secret)
    }

    /// Returns an Etherum address derived from an Ethereum secret key.
    /// Only for tests
    #[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
    pub fn eth_address(secret: &libsecp256k1::SecretKey) -> EvmAddress {
        EvmAddress::from_slice(&keccak_256(&Self::eth_public(secret).serialize()[1..65])[12..])
    }

    /// Constructs a message and signs it.
    #[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
    pub fn eth_sign(secret: &libsecp256k1::SecretKey, proof: &Proof) -> Eip712Signature {
        let msg = keccak_256(&Self::eip712_signable_message(proof));
        let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        r
    }
}

#[inline]
fn recover_signer(sig: &[u8; 65], msg_hash: &[u8; 32]) -> Option<H160> {
    secp256k1_ecdsa_recover(sig, msg_hash)
        .map(|pubkey| H160::from(H256::from_slice(&keccak_256(&pubkey))))
        .ok()
}

#[inline]
pub fn sha3_256(s: &str) -> [u8; 32] {
    let mut result = [0u8; 32];

    // create a SHA3-256 object
    let mut hasher = Keccak256::new();
    // write input message
    hasher.update(s);
    // read hash digest
    result.copy_from_slice(&hasher.finalize()[..32]);

    result
}

/// Convert any type that implements Into<U256> into byte representation ([u8, 32])
#[inline]
pub fn to_bytes<T: Into<U256>>(value: T) -> [u8; 32] {
    Into::<[u8; 32]>::into(value.into())
}

/// Preprocessed Event
enum PreprocessedEvent<T>
where
    T: Config,
{
    /// To Private Event
    ToPrivate {
        /// Asset Minted
        asset: StandardAssetId,

        /// Source Account
        source: T::AccountId,
    },
}

impl<T> PreprocessedEvent<T>
where
    T: Config,
{
    /// Converts a [`PreprocessedEvent`] with into an [`Event`] using the given `origin` for
    /// [`PreprocessedEvent::PrivateTransfer`].
    #[inline]
    fn convert(self) -> Event<T> {
        match self {
            Self::ToPrivate { asset, source } => Event::MintSbt { asset, source },
        }
    }
}

/// Only allows `ToPrivate`. There are no checks on the source accounts. `ToPublic` and `PrivateTransfer` will fail.
struct SBTLedger<T>(PhantomData<T>)
where
    T: Config;

impl<T> SenderLedger<config::Parameters> for SBTLedger<T>
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
        Ok(Wrap(nullifier))
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
        _super_key: &Self::SuperPostingKey,
        _iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidNullifier)>,
    {
        Ok(())
    }
}

impl<T> ReceiverLedger<config::Parameters> for SBTLedger<T>
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

impl<T> TransferLedger<config::Config> for SBTLedger<T>
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
        _asset_id: &config::AssetId,
        sources: I,
    ) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<config::Config, AccountId>>
    where
        I: Iterator<Item = (AccountId, config::AssetValue)>,
    {
        Ok(sources
            .map(move |(account_id, withdraw)| WrapPair(account_id, withdraw))
            .collect())
    }

    #[inline]
    fn check_sink_accounts<I>(
        &self,
        _asset_id: &config::AssetId,
        _sinks: I,
    ) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<config::Config, AccountId>>
    where
        I: Iterator<Item = (AccountId, config::AssetValue)>,
    {
        // No Sinks for SBT
        Ok(Vec::new())
    }

    #[inline]
    fn is_valid(
        &self,
        posting_key: TransferPostingKeyRef<config::Config, Self>,
    ) -> Result<(Self::ValidProof, Self::Event), TransferLedgerError<T>> {
        let transfer_shape = TransferShape::from_posting_key_ref(&posting_key);
        let (mut verifying_context, event) =
            match transfer_shape.ok_or(TransferLedgerError::InvalidTransferShape)? {
                TransferShape::ToPrivate => {
                    if let Some(asset_id) = posting_key.asset_id.or(None) {
                        let asset_id = id_from_field(
                            fp_encode(asset_id).map_err(TransferLedgerError::FpEncodeError)?,
                        )
                        .ok_or(TransferLedgerError::UnknownAsset)?;
                        (
                            manta_parameters::pay::verifying::ToPrivate::get()
                                .ok_or(TransferLedgerError::ChecksumError)?,
                            PreprocessedEvent::<T>::ToPrivate {
                                asset: asset_id,
                                source: posting_key.sources[0].0.into(),
                            },
                        )
                    } else {
                        return Err(TransferLedgerError::UnknownAsset);
                    }
                }
                TransferShape::PrivateTransfer => {
                    return Err(TransferLedgerError::SenderLedgerError(
                        SenderLedgerError::NoSenderLedger,
                    ))
                }
                TransferShape::ToPublic => {
                    return Err(TransferLedgerError::SenderLedgerError(
                        SenderLedgerError::NoSenderLedger,
                    ))
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
        _super_key: &TransferLedgerSuperPostingKey<config::Config, Self>,
        _asset_id: config::AssetId,
        _sources: Vec<SourcePostingKey<config::Config, Self>>,
        _sinks: Vec<SinkPostingKey<config::Config, Self>>,
        _proof: Self::ValidProof,
    ) -> Result<(), TransferLedgerError<T>> {
        Ok(())
    }
}
