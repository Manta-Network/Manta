use core::marker::PhantomData;
use frame_support::dispatch::DispatchResultWithPostInfo;
use crate::{Config, Pallet, pallet::*, types};
use crate::types::{ReceiverChunk, Checkpoint, PullResponse, TransferPost, SenderChunk, asset_value_encode, AssetType};
use manta_pay::{
    config::{self, utxo::MerkleTreeConfiguration},
};
use manta_pay::parameters::load_transfer_parameters;
use manta_util::{Array, into_array_unchecked};
use sp_runtime::traits::{AccountIdConversion, Get};
use manta_primitives::assets::{AssetConfig, AssetRegistry};
use manta_primitives::types::StandardAssetId;
use crate::errors::FungibleLedgerError;
use alloc::{vec::Vec};
use manta_accounting::transfer;
use crate::ledger::ProxyLedger;
use crate::ledgers::ftledger::FTTransferLedger;
use crate::ledgers::nftledger::NFTTransferLedger;

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
            NullifierSetInsertionOrder::<T>::contains_key(max_sender_index),
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
            senders_receivers_total: asset_value_encode(senders_receivers_total),
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
    pub fn post_transaction(
        origin: Option<T::AccountId>,
        sources: Vec<T::AccountId>,
        sinks: Vec<T::AccountId>,
        post: TransferPost,
        asset_type: AssetType
    ) -> DispatchResultWithPostInfo {
        let transfer_post = config::TransferPost::try_from(post)
            .map_err(|_| Error::<T>::InvalidSerializedForm)?;
        let transfer_asset_type = transfer::AssetType::try_from(asset_type.clone())
            .map_err(|_| Error::<T>::InvalidSerializedForm)?;

        let post = match asset_type {
            AssetType::FT | AssetType::PFT => {
                transfer_post.post(
                    transfer_asset_type,
                    &load_transfer_parameters(),
                    &mut FTTransferLedger {
                        ledger: ProxyLedger(PhantomData)
                    },
                    &(),
                    sources,
                    sinks,
                )
            },
            AssetType::NFT | AssetType::SBT => {
                transfer_post.post(
                    transfer_asset_type,
                    &load_transfer_parameters(),
                    &mut NFTTransferLedger {
                        ledger: ProxyLedger(PhantomData)
                    },
                    &(),
                    sources,
                    sinks,
                )
            }
        };

        let event = post.map_err(Error::<T>::from)?.convert(origin);
        Self::deposit_event(event);
        Ok(().into())
    }

    pub fn get_metadata(asset_id: config::AssetId) -> Result<
        <<T::AssetConfig as AssetConfig<T>>::AssetRegistry as AssetRegistry<T>>::Metadata,
        FungibleLedgerError
    > {
        let id = Self::id_from_field(types::fp_encode(asset_id).map_err(|_| FungibleLedgerError::EncodeError)?)
            .ok_or(FungibleLedgerError::UnknownAsset)?;
        let metadata = <T::AssetConfig as AssetConfig<T>>::AssetRegistry::get_metadata(&id)
            .ok_or(FungibleLedgerError::UnknownAsset)?;
        Ok(metadata)
    }

    ///
    #[inline]
    pub fn id_from_field(id: [u8; 32]) -> Option<StandardAssetId> {
        if 0u128.to_le_bytes() == id[16..32] {
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
