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
//! MantaSBT creates non-transferable nfts (soul-bound) as non spendable UTXOs
//!
//! ## Overview
//!
//! There are two paths in which an account can mint a zkSBT.
//! One is by using the native token (KMA/MANTA) to reserve the right to mint and subsequently minting the zkSBT.
//! The other is by having a `EvmAddress` added to an allowlist which gives user one free zkSBT to mint (still costs tx fee).
//! Ownership of SBT is recorded as a corresponding UTXO.
//! User can prove ownership of SBT using `TransactionData` which can reconstruct UTXO and check its existence on-chain.
//!
//! ### Minting zkSBT using native token to pay
//!
//! There are two calls `reserve_sbt` and `to_private`.
//!
//! `reserve_sbt`: Reserves unique `AssetIds` for user to later mint into sbt.
//!
//! `to_private`: Mints SBT with signer generated `TransferPost` using previously reserved `AssetId`.
//! Stores relevant metadata with associated `AssetId`
//!
//! ### Minting zkSBT using `EvmAddress` allowlist
//!
//! First some `AdminOrigin` must setup the allowlist, the following must be called to setup allowlist:
//!
//! `change_allowlist_account`: `AdminOrigin` must set a privileged account to have power to allowlist `EvmAddress`
//! `set_mint_chain_info`: `AdminOrigin` must set a time range for a particular `MintType` to be valid.
//! `allowlist_evm_account`: Account set in `change_allowlist_account` can allow a particular `EvmAddress` one free mint of zkSBT.
//!
//! Second step a user that has been added to `EvmAccountAllowlist` can now mint their zkSBT.
//!
//! `mint_sbt_eth`: User must generate a zkp corresponding to the reserved `AssetId` mapped to their `EvmAddress`.
//! Subsequently user must generate signature by signing zkp with their eth private key.
//! If their `EvmAddress` has been allowlisted then user will have a zkSBT for free (minus tx fee cost)!

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency, StorageVersion, Time},
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use manta_support::manta_pay::{
    asset_value_encode, fp_decode, fp_encode, id_from_field, AccountId, AssetValue, Checkpoint,
    FullIncomingNote, MTParametersError, Proof, PullResponse, ReceiverChunk, StandardAssetId,
    TransferPost, Utxo, UtxoItemHashError, UtxoMerkleTreePath, VerifyingContextError, Wrap,
    WrapPair,
};
use sha3::{Digest, Keccak256};
use sp_core::{H160, H256, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::{
    traits::{AccountIdConversion, IdentifyAccount, One, Verify, Zero},
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

/// Permissionless mint id
const MANTA_MINT_ID: MintId = 0;

/// Type alias for currency balance.
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Eth based address
type EvmAddress = H160;

/// A 512-bit value, plus 8 bits for recovery ID).
pub type Eip712Signature = [u8; 65];

/// Each mint type shall have a unique id
pub type MintId = u32;

/// zkSBT mint Status of `EvmAddressType`. This has flag `AlreadyMinted` to put into storage after successful mint
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MintStatus {
    Available(StandardAssetId),
    AlreadyMinted,
}

/// Mint metadata that corresponds to an assigned `MintId`
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(Bound))]
pub struct RegisteredMint<Moment, Bound: Get<u32>> {
    pub mint_name: BoundedVec<u8, Bound>,
    pub start_time: Moment,
    pub end_time: Option<Moment>,
}

/// Mint Metadata stored for a minted zkSBT
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(Bound))]
pub struct MetadataV2<Bound: Get<u32>> {
    pub mint_id: MintId,
    pub collection_id: Option<u128>,
    pub item_id: Option<u128>,
    pub extra: Option<BoundedVec<u8, Bound>>,
}

/// Signature and Public key used for verification
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SignatureInfo<S, P> {
    sig: S,
    pub_key: P,
}

/// Type for timestamp
pub type Moment<T> = <<T as Config>::Now as Time>::Moment;

/// `SignatureInfo` with generics defined for ease of use
pub type SignatureInfoOf<T> = SignatureInfo<<T as Config>::Signature, <T as Config>::PublicKey>;

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
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The origin which can change the privileged allowlist account and set time range for mints
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Gets the current on-chain time
        type Now: Time;

        /// A Signature can be verified with a specific `PublicKey`.
        type Signature: Verify<Signer = Self::PublicKey> + Decode + Parameter;

        /// A PublicKey can be converted into an `AccountId`. This is required by the
        /// `Signature` type.
        type PublicKey: IdentifyAccount<AccountId = Self::AccountId> + Decode + Parameter;

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

        /// Max size in bytes of `mint_name` entered in `RegisteredMint`
        #[pallet::constant]
        type RegistryBound: Get<u32>;
    }

    /// Counter for SBT AssetId. Increments by one every time a new asset id is requested.
    ///
    /// Should only ever be modified by `next_sbt_id_and_increment()`
    #[pallet::storage]
    pub(super) type NextSbtId<T: Config> = StorageValue<_, StandardAssetId, OptionQuery>;

    /// Counter for MintId. Increments by one every time a new mint type is created (Bab, Galxe, etc.)
    ///
    /// Should only ever be modified by `next_mint_id_and_increment()`
    #[pallet::storage]
    pub(super) type NextMintId<T: Config> = StorageValue<_, MintId, OptionQuery>;

    /// Account that can add evm accounts to allowlist
    #[pallet::storage]
    pub(super) type AllowlistAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Account that can reserve `AssetId` for free
    #[pallet::storage]
    pub(super) type FreeReserveAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Account that can access to force calls
    #[pallet::storage]
    pub(super) type ForceAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Allowlist for Evm Accounts
    #[pallet::storage]
    pub(super) type EvmAccountAllowlist<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MintId,
        Blake2_128Concat,
        EvmAddress,
        MintStatus,
        OptionQuery,
    >;

    /// Registers a number for mint type
    #[pallet::storage]
    pub(super) type MintIdRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MintId,
        RegisteredMint<Moment<T>, T::RegistryBound>,
        OptionQuery,
    >;

    /// Allows mint type to be public
    #[pallet::storage]
    pub(super) type PublicMintList<T: Config> = StorageMap<_, Blake2_128Concat, MintId, ()>;

    /// SBT Metadata maps `StandardAsset` to the corresponding SBT metadata
    ///
    /// Metadata is raw bytes that correspond to an image
    #[pallet::storage]
    pub(super) type SbtMetadataV2<T: Config> =
        StorageMap<_, Blake2_128Concat, StandardAssetId, MetadataV2<T::SbtMetadataBound>>;

    /// Allowlist accounts to be able to mint SBTs with designated `StandardAssetId`
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

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: From<AccountId> + Into<AccountId>,
    {
        /// Mints a zkSBT
        ///
        /// `TransferPost` is posted to private ledger and SBT metadata is stored onchain.
        /// `signature` parameter can be used to relay a tx.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::to_private())]
        #[transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn to_private(
            origin: OriginFor<T>,
            mint_id: Option<MintId>,
            chain_id: Option<u64>,
            signature: Option<SignatureInfoOf<T>>,
            post: Box<TransferPost>,
            metadata: BoundedVec<u8, T::SbtMetadataBound>,
        ) -> DispatchResultWithPostInfo {
            let mut minting_account = ensure_signed(origin)?;
            let mint_id = mint_id.unwrap_or(MANTA_MINT_ID);
            let chain_id = chain_id.unwrap_or(Zero::zero());

            Self::check_mint_time(mint_id)?;
            Self::check_mint_is_public(mint_id)?;

            if let Some(sig) = signature {
                // check that signature is valid
                ensure!(
                    Self::verify_crypto_sig(&sig, &post.proof, chain_id),
                    Error::<T>::BadSignature
                );
                // set verified signature account as the minting_account
                minting_account = sig.pub_key.into_account();
            }

            let (start_id, end_id) =
                ReservedIds::<T>::get(&minting_account).ok_or(Error::<T>::NotReserved)?;

            // Checks that it is indeed a to_private post with a value of 1 and has correct asset_id
            Self::check_post_shape(&post, start_id)?;

            let sbt_metadata = MetadataV2::<T::SbtMetadataBound> {
                mint_id,
                collection_id: None,
                item_id: None,
                extra: Some(metadata),
            };

            Self::check_and_insert_metadata(start_id, sbt_metadata)?;
            let increment_start_id = start_id
                .checked_add(One::one())
                .ok_or(ArithmeticError::Overflow)?;

            // If `ReservedIds` are all used remove from storage, otherwise increment the next `AssetId` to be used next time for minting SBT
            if increment_start_id > end_id {
                ReservedIds::<T>::remove(&minting_account)
            } else {
                ReservedIds::<T>::insert(&minting_account, (increment_start_id, end_id))
            }

            Self::post_transaction(vec![minting_account], *post)?;
            Ok(().into())
        }

        /// Reserves AssetIds to be used subsequently in `to_private` above.
        ///
        /// Increments AssetManager's AssetId counter.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::reserve_sbt())]
        #[transactional]
        pub fn reserve_sbt(origin: OriginFor<T>, reservee: Option<T::AccountId>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Use reservee account, if None then use account whom signed transaction
            let reserve_account = reservee.unwrap_or(who.clone());
            // ensure account does not have any `AssetId` already reserved
            ensure!(
                !ReservedIds::<T>::contains_key(&reserve_account),
                Error::<T>::AssetIdsAlreadyReserved
            );

            let free_account = FreeReserveAccount::<T>::get();
            // check if account is allowlist account... if it is can do operation for free
            if free_account.as_ref() != Some(&who) {
                // Charges fee to tx caller to reserve AssetIds
                <T as pallet::Config>::Currency::transfer(
                    &who,
                    &Self::account_id(),
                    T::ReservePrice::get(),
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            // Reserves uniques AssetIds to be used later to mint SBTs
            let asset_id_range: Vec<StandardAssetId> = (0..T::MintsPerReserve::get())
                .map(|_| Self::next_sbt_id_and_increment())
                .collect::<Result<Vec<StandardAssetId>, _>>()?;

            // The range of `AssetIds` that are reserved as SBTs
            let start_id: StandardAssetId = *asset_id_range.first().ok_or(Error::<T>::ZeroMints)?;
            let stop_id: StandardAssetId = *asset_id_range.last().ok_or(Error::<T>::ZeroMints)?;

            ReservedIds::<T>::insert(&reserve_account, (start_id, stop_id));
            Self::deposit_event(Event::<T>::SBTReserved {
                who,
                reserve_account,
                start_id,
                stop_id,
            });
            Ok(())
        }

        /// Adds EvmAddress to allowlist and reserve an unique AssetId for this account. Requires caller to be the `AllowlistAccount`.
        ///
        /// Uses `mint_id` to specify which mint, this is so an `EvmAddress` can have multiple free mints for different `MintIds`.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::allowlist_evm_account())]
        #[transactional]
        pub fn allowlist_evm_account(
            origin: OriginFor<T>,
            mint_id: MintId,
            evm_address: EvmAddress,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::check_mint_time(mint_id)?;

            let allowlist_account =
                AllowlistAccount::<T>::get().ok_or(Error::<T>::NotAllowlistAccount)?;
            ensure!(who == allowlist_account, Error::<T>::NotAllowlistAccount);

            ensure!(
                !EvmAccountAllowlist::<T>::contains_key(mint_id, evm_address),
                Error::<T>::AlreadyInAllowlist
            );

            let asset_id = Self::next_sbt_id_and_increment()?;
            let mint_status = MintStatus::Available(asset_id);
            EvmAccountAllowlist::<T>::insert(mint_id, evm_address, mint_status);

            Self::deposit_event(Event::<T>::AllowlistEvmAddress {
                address: evm_address,
                mint_id,
                asset_id,
            });
            Ok(())
        }

        /// Mint zkSBT using Evm allowlist, signature must correspond to an `EvmAddress` which has been added to allowlist.
        ///
        /// Requires a valid `Eip712Signature` which is generated from signing the zkp with an eth private key
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::mint_sbt_eth())]
        #[transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn mint_sbt_eth(
            origin: OriginFor<T>,
            post: Box<TransferPost>,
            chain_id: u64,
            eth_signature: Eip712Signature,
            mint_id: MintId,
            collection_id: Option<u128>,
            item_id: Option<u128>,
            metadata: Option<BoundedVec<u8, T::SbtMetadataBound>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // check that mint type is within time window
            Self::check_mint_time(mint_id)?;

            let address = Self::verify_eip712_signature(&post.proof, &eth_signature, chain_id)
                .ok_or(Error::<T>::BadSignature)?;

            let mint_status = EvmAccountAllowlist::<T>::get(mint_id, address)
                .ok_or(Error::<T>::NotAllowlisted)?;
            let asset_id = match mint_status {
                MintStatus::Available(asset) => asset,
                MintStatus::AlreadyMinted => return Err(Error::<T>::AlreadyMinted.into()),
            };
            // Change status to minted
            EvmAccountAllowlist::<T>::insert(mint_id, address, MintStatus::AlreadyMinted);

            Self::check_post_shape(&post, asset_id)?;
            let sbt_metadata = MetadataV2::<T::SbtMetadataBound> {
                mint_id,
                collection_id,
                item_id,
                extra: metadata,
            };
            Self::check_and_insert_metadata(asset_id, sbt_metadata)?;

            Self::post_transaction(vec![who], *post)?;
            Self::deposit_event(Event::<T>::MintSbtEvm {
                asset_id,
                mint_id,
                address,
            });
            Ok(().into())
        }

        /// Sets the privileged allowlist account. Requires `AdminOrigin`
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::change_allowlist_account())]
        #[transactional]
        pub fn change_allowlist_account(
            origin: OriginFor<T>,
            account: Option<T::AccountId>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            AllowlistAccount::<T>::set(account.clone());
            Self::deposit_event(Event::<T>::ChangeAllowlistAccount { account });
            Ok(())
        }

        /// Updates the time range of which a `MintId` will be valid. Also can update `mint_name` Requires `AdminOrigin`
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_mint_info())]
        #[transactional]
        pub fn update_mint_info(
            origin: OriginFor<T>,
            mint_id: MintId,
            start_time: Moment<T>,
            end_time: Option<Moment<T>>,
            mint_name: BoundedVec<u8, T::RegistryBound>,
            public: bool,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            if let Some(end) = end_time {
                ensure!(end > start_time, Error::<T>::InvalidTimeRange);
            }

            let mint_id_info = RegisteredMint::<Moment<T>, T::RegistryBound> {
                start_time,
                end_time,
                mint_name: mint_name.clone(),
            };
            MintIdRegistry::<T>::mutate(mint_id, |mint_info| {
                match mint_info {
                    Some(_) => *mint_info = Some(mint_id_info),
                    // if value does not exist then return error, can only create mint id from `new_mint_info`
                    None => return Err(Error::<T>::InvalidMintId),
                }
                Ok(())
            })?;
            if public {
                PublicMintList::<T>::insert(mint_id, ());
            } else {
                PublicMintList::<T>::remove(mint_id);
            }

            Self::deposit_event(Event::<T>::UpdateMintInfo {
                mint_id,
                start_time,
                end_time,
                mint_name: mint_name.to_vec(),
            });
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::new_mint_info())]
        pub fn new_mint_info(
            origin: OriginFor<T>,
            start_time: Moment<T>,
            end_time: Option<Moment<T>>,
            mint_name: BoundedVec<u8, T::RegistryBound>,
            public: bool,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            if let Some(end) = end_time {
                ensure!(end > start_time, Error::<T>::InvalidTimeRange);
            }
            let mint_chain_info = RegisteredMint::<Moment<T>, T::RegistryBound> {
                start_time,
                end_time,
                mint_name: mint_name.clone(),
            };
            let mint_id = Self::next_mint_id_and_increment()?;

            MintIdRegistry::<T>::insert(mint_id, mint_chain_info);
            // add or remove from `PublicMintList`
            if public {
                PublicMintList::<T>::insert(mint_id, ());
            }

            Self::deposit_event(Event::<T>::NewMintInfo {
                start_time,
                end_time,
                mint_id,
                mint_name: mint_name.to_vec(),
            });
            Ok(())
        }

        /// Sets the privileged reserve account. Requires `AdminOrigin`
        #[pallet::call_index(7)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::change_free_reserve_account())]
        #[transactional]
        pub fn change_free_reserve_account(
            origin: OriginFor<T>,
            account: Option<T::AccountId>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            FreeReserveAccount::<T>::set(account.clone());
            Self::deposit_event(Event::<T>::ChangeFreeReserveAccount { account });
            Ok(())
        }

        /// Remove allowlist evm account. Requires `AdminOrigin`
        #[pallet::call_index(8)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_allowlist_evm_account())]
        #[transactional]
        pub fn remove_allowlist_evm_account(
            origin: OriginFor<T>,
            mint_id: MintId,
            evm_address: EvmAddress,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            EvmAccountAllowlist::<T>::remove(mint_id, evm_address);
            Self::deposit_event(Event::<T>::RemoveAllowlistEvmAddress {
                address: evm_address,
                mint_id,
            });
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_next_sbt_id())]
        #[transactional]
        pub fn set_next_sbt_id(
            origin: OriginFor<T>,
            asset_id: Option<StandardAssetId>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            NextSbtId::<T>::set(asset_id);
            Self::deposit_event(Event::<T>::SetNextSbtId { asset_id });
            Ok(())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_to_private())]
        #[transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn force_to_private(
            origin: OriginFor<T>,
            post: Box<TransferPost>,
            mint_id: MintId,
            metadata: BoundedVec<u8, T::SbtMetadataBound>,
            minting_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let force_account = ForceAccount::<T>::get().ok_or(Error::<T>::NotForceAccount)?;
            ensure!(who == force_account, Error::<T>::NotForceAccount);

            let asset_id = id_from_field(post.asset_id.ok_or(Error::<T>::InvalidAssetId)?)
                .ok_or(Error::<T>::InvalidAssetId)?;
            Self::check_post_shape(&post, asset_id)?;
            let sbt_metadata = MetadataV2::<T::SbtMetadataBound> {
                mint_id,
                collection_id: None,
                item_id: None,
                extra: Some(metadata),
            };

            // check that asset id is below `NextSbtId`
            if let Some(next_asset_it) = NextSbtId::<T>::get() {
                // check that asset id is below max allowed number
                ensure!(asset_id < next_asset_it, Error::<T>::TooHighAssetId);
                Ok(())
            } else {
                Err(Error::<T>::TooHighAssetId)
            }?;

            Self::check_and_insert_metadata(asset_id, sbt_metadata)?;
            Self::post_transaction(vec![minting_account.clone()], *post)?;

            Self::deposit_event(Event::<T>::ForceToPrivate {
                asset: asset_id,
                source: minting_account,
            });
            Ok(Pays::No.into())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_mint_sbt_eth())]
        #[transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn force_mint_sbt_eth(
            origin: OriginFor<T>,
            post: Box<TransferPost>,
            mint_id: MintId,
            address: EvmAddress,
            collection_id: Option<u128>,
            item_id: Option<u128>,
            metadata: BoundedVec<u8, T::SbtMetadataBound>,
            minting_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let force_account = ForceAccount::<T>::get().ok_or(Error::<T>::NotForceAccount)?;
            ensure!(who == force_account, Error::<T>::NotForceAccount);

            let asset_id = id_from_field(post.asset_id.ok_or(Error::<T>::InvalidAssetId)?)
                .ok_or(Error::<T>::InvalidAssetId)?;
            Self::check_post_shape(&post, asset_id)?;
            let sbt_metadata = MetadataV2::<T::SbtMetadataBound> {
                mint_id,
                collection_id,
                item_id,
                extra: Some(metadata),
            };

            // check that asset id is below `NextSbtId`
            if let Some(next_asset_it) = NextSbtId::<T>::get() {
                // check that asset id is below max allowed number
                ensure!(asset_id < next_asset_it, Error::<T>::TooHighAssetId);
                Ok(())
            } else {
                Err(Error::<T>::TooHighAssetId)
            }?;
            Self::check_and_insert_metadata(asset_id, sbt_metadata)?;

            // defensively check whether this key already exists
            ensure!(
                !EvmAccountAllowlist::<T>::contains_key(mint_id, address),
                Error::<T>::AlreadyInAllowlist
            );
            // manually insert address, note no signature check.
            EvmAccountAllowlist::<T>::insert(mint_id, address, MintStatus::AlreadyMinted);
            Self::post_transaction(vec![minting_account], *post)?;

            Self::deposit_event(Event::<T>::ForceMintSbtEvm {
                address,
                mint_id,
                asset_id,
            });
            Ok(Pays::No.into())
        }

        /// Sets the privileged reserve account. Requires `AdminOrigin`
        #[pallet::call_index(12)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::change_force_account())]
        #[transactional]
        pub fn change_force_account(
            origin: OriginFor<T>,
            account: Option<T::AccountId>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ForceAccount::<T>::set(account.clone());
            Self::deposit_event(Event::<T>::ChangeForceAccount { account });
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
            /// Account which recieves reserved AssetIds, can be the same as the above account
            reserve_account: T::AccountId,
            /// Start of `AssetIds` reserved for use on private ledger
            start_id: StandardAssetId,
            /// End of `AssetIds` reserved for use private ledger, does not include this value
            stop_id: StandardAssetId,
        },
        /// Evm Address is Allowlisted
        AllowlistEvmAddress {
            /// Eth Address that is now allowlisted to mint an SBT
            address: EvmAddress,
            /// An integer that corresponds to mint type
            mint_id: MintId,
            /// AssetId that is reserved for above Eth address
            asset_id: StandardAssetId,
        },
        /// Evm Address is removed from allowlist
        RemoveAllowlistEvmAddress {
            /// Eth Address that is now allowlisted to mint an SBT
            address: EvmAddress,
            /// An integer that corresponds to mint type
            mint_id: MintId,
        },
        /// Sbt is minted using Allowlisted Eth account
        MintSbtEvm {
            /// Eth Address that is used to mint sbt
            address: EvmAddress,
            /// An integer that corresponds to the mint type
            mint_id: MintId,
            /// AssetId of minted SBT
            asset_id: StandardAssetId,
        },
        /// Privileged `AllowlistAccount` is changed
        ChangeAllowlistAccount {
            /// Account that is now the new privileged allowlist account
            account: Option<T::AccountId>,
        },
        UpdateMintInfo {
            /// `MintId` to be updated
            mint_id: MintId,
            /// Start time at which minting is valid
            start_time: Moment<T>,
            /// End time at which minting will no longer be valid, None represents no end time.
            end_time: Option<Moment<T>>,
            /// Name of mint
            mint_name: Vec<u8>,
        },
        NewMintInfo {
            /// new `MintId` generated
            mint_id: MintId,
            /// Start time at which minting is valid
            start_time: Moment<T>,
            /// End time at which minting will no longer be valid, None represents no end time.
            end_time: Option<Moment<T>>,
            /// Name of mint
            mint_name: Vec<u8>,
        },
        ChangeFreeReserveAccount {
            account: Option<T::AccountId>,
        },
        SetNextSbtId {
            asset_id: Option<StandardAssetId>,
        },
        ForceToPrivate {
            /// AssetId on private leger
            asset: StandardAssetId,
            /// Source Account
            source: T::AccountId,
        },
        ForceMintSbtEvm {
            /// Eth Address that is used to mint sbt
            address: EvmAddress,
            /// An integer that corresponds to the mint type
            mint_id: MintId,
            /// AssetId of minted SBT
            asset_id: StandardAssetId,
        },
        ChangeForceAccount {
            account: Option<T::AccountId>,
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

        /// Transfer Ledger `VerifyingContext` cannot be decoded
        TransferLedgerVerifyingContextDecodeError,

        /// Transfer Ledger Field Element Encoding Error
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

        /// Crypto Signature was not valid
        BadSignature,

        /// Eth account is not allowlisted for free mint, can also be caused by an incorrect signature (recovers an invalid account)
        NotAllowlisted,

        /// Account is not the privileged account able to allowlist eth addresses
        NotAllowlistAccount,

        /// Minting SBT is outside defined time range or chain_id is not set
        MintNotAvailable,

        /// `EvmAddress` is already in the allowlist
        AlreadyInAllowlist,

        /// SBT has already been minted with this `EvmAddress`
        AlreadyMinted,

        /// Time range is invalid (start_time > end_time)
        InvalidTimeRange,

        /// MintId does not exist, cannot update a nonexistent MintId
        InvalidMintId,

        /// Already has unused AssetIds reserved
        AssetIdsAlreadyReserved,

        /// Mint type is not public, only permissioned accounts can use this mint
        MintNotPublic,

        /// Account is not privileged account able to do force mints
        NotForceAccount,

        /// Duplicate asset id
        DuplicateAssetId,

        /// Force call is trying to use asset id above the maximum
        TooHighAssetId,
    }
}

impl<T> Pallet<T>
where
    T: Config,
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
        asset_value_encode(receivers_total)
    }

    /// Returns the diff of ledger state since the given `checkpoint` and `max_receivers`.
    /// This `Ledger` implementation has no senders by definition, cannot transfer SBTs.
    #[inline]
    pub fn pull_ledger_diff(
        checkpoint: Checkpoint,
        max_receivers: u64,
        _max_senders: u64,
    ) -> PullResponse {
        let (more_receivers, receivers) =
            Self::pull_receivers(*checkpoint.receiver_index, max_receivers);
        PullResponse {
            should_continue: more_receivers,
            receivers,
            senders: vec![],
            senders_receivers_total: Self::pull_ledger_total_count(),
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

    /// Returns and increments the [`NextMintId`] by one.
    #[inline]
    fn next_mint_id_and_increment() -> Result<MintId, DispatchError> {
        NextMintId::<T>::try_mutate(|maybe_val| {
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
                // If storage is empty, starts at value of one (Native SBT has value of zero)
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
    fn verify_eip712_signature(proof: &Proof, sig: &[u8; 65], chain_id: u64) -> Option<EvmAddress> {
        let msg = Self::eip712_signable_message(proof, chain_id);
        let msg_hash = keccak_256(msg.as_slice());

        recover_signer(sig, &msg_hash)
    }

    /// Eip-712 message to be signed
    #[inline]
    fn eip712_signable_message(proof: &Proof, chain_id: u64) -> Vec<u8> {
        let domain_separator = Self::evm_account_domain_separator(chain_id);
        let payload_hash = Self::evm_account_payload_hash(proof);

        let mut msg = b"\x19\x01".to_vec();
        msg.extend_from_slice(&domain_separator);
        msg.extend_from_slice(&payload_hash);
        msg
    }

    /// Creates `keccak_256` hash of Proof payload
    #[inline]
    fn evm_account_payload_hash(proof: &Proof) -> [u8; 32] {
        let tx_type_hash = &sha3_256("Transaction(bytes proof)");
        let mut tx_msg = tx_type_hash.to_vec();
        tx_msg.extend_from_slice(&keccak_256(proof.as_slice()));
        keccak_256(tx_msg.as_slice())
    }

    /// Creates Eip712 domain separator for minting zkSBT. This ensures signature will not have collisions
    #[inline]
    fn evm_account_domain_separator(chain_id: u64) -> [u8; 32] {
        let domain_hash =
            &sha3_256("EIP712Domain(string name,string version,uint256 chainId,bytes32 salt)");
        let mut domain_seperator_msg = domain_hash.to_vec();
        domain_seperator_msg.extend_from_slice(&sha3_256("Claim Free SBT")); // name
        domain_seperator_msg.extend_from_slice(&sha3_256("1")); // version
        domain_seperator_msg.extend_from_slice(&to_bytes(chain_id)); // chain id
        domain_seperator_msg.extend_from_slice(
            frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()).as_ref(),
        ); // genesis block hash
        keccak_256(domain_seperator_msg.as_slice())
    }

    /// Checks that mint type is available to mint within time window defined in `MintRegistrar`
    #[inline]
    fn check_mint_time(mint_id: MintId) -> DispatchResult {
        // skip check if it is native mint with id of 0
        if mint_id != MANTA_MINT_ID {
            let mint_chain_info =
                MintIdRegistry::<T>::get(mint_id).ok_or(Error::<T>::MintNotAvailable)?;

            let (start_time, end_time) = (mint_chain_info.start_time, mint_chain_info.end_time);

            // checks that current time falls within bounds
            let current_time = T::Now::now();
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
        }
        Ok(())
    }

    #[inline]
    fn check_mint_is_public(mint_id: MintId) -> DispatchResult {
        // mint id of 0 is always public
        if mint_id != MANTA_MINT_ID {
            ensure!(
                PublicMintList::<T>::contains_key(mint_id),
                Error::<T>::MintNotPublic
            );
        }
        Ok(())
    }

    /// Signature Verification using substrate crypto library in `sp_core::crypto`
    #[inline]
    fn verify_crypto_sig(sig_info: &SignatureInfoOf<T>, proof: &Proof, chain_id: u64) -> bool {
        // Eip712 msg with chain_id of zero
        let msg = Self::eip712_signable_message(proof, chain_id);
        let msg_hash = keccak_256(msg.as_slice());

        let wrap_msg = Self::wrap_msg_with_bytes(msg_hash);

        sig_info
            .sig
            .verify(wrap_msg.as_ref(), &sig_info.pub_key.clone().into_account())
    }

    /// Wrap `<Bytes>` and `</Bytes>` to [u8; 32] array.
    #[inline]
    fn wrap_msg_with_bytes(msg: [u8; 32]) -> Vec<u8> {
        let mut wrap_msg: Vec<u8> = Vec::new();
        wrap_msg.extend("<Bytes>".as_bytes());
        wrap_msg.extend_from_slice(&msg);
        wrap_msg.extend("</Bytes>".as_bytes());
        wrap_msg
    }

    #[inline]
    fn check_and_insert_metadata(
        asset_id: StandardAssetId,
        sbt_metadata: MetadataV2<T::SbtMetadataBound>,
    ) -> DispatchResult {
        // defensive check to ensure asset id is unique
        ensure!(
            !SbtMetadataV2::<T>::contains_key(asset_id),
            Error::<T>::DuplicateAssetId
        );
        SbtMetadataV2::<T>::insert(asset_id, sbt_metadata);
        Ok(())
    }

    /// Returns an Ethereum public key derived from an Ethereum secret key.
    #[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
    pub fn eth_public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
        libsecp256k1::PublicKey::from_secret_key(secret)
    }

    /// Returns an Ethereum address derived from an Ethereum secret key.
    /// Only for tests
    #[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
    pub fn eth_address(secret: &libsecp256k1::SecretKey) -> EvmAddress {
        EvmAddress::from_slice(&keccak_256(&Self::eth_public(secret).serialize()[1..65])[12..])
    }

    /// Constructs a message and signs it.
    #[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
    pub fn eth_sign(
        secret: &libsecp256k1::SecretKey,
        proof: &Proof,
        chain_id: u64,
    ) -> Eip712Signature {
        let msg = keccak_256(&Self::eip712_signable_message(proof, chain_id));
        let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        r
    }
}

/// Recover `EvmAddress` from `Eip712Signature` using the message hash
#[inline]
fn recover_signer(sig: &[u8; 65], msg_hash: &[u8; 32]) -> Option<H160> {
    secp256k1_ecdsa_recover(sig, msg_hash)
        .map(|pubkey| H160::from(H256::from_slice(&keccak_256(&pubkey))))
        .ok()
}

/// Utility function to get SHA3-256 hash of &str
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
        // for which you don't need a membership proof, but you still need a root (in this case
        // zeroed).
        if accumulator_output == [0u8; 32] {
            return Ok(Wrap(output));
        }
        Err(SenderLedgerError::NoSenderLedger)
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
            if let Some(_next_root) = next_root {
                ShardTrees::<T>::insert(shard_index, tree);
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
