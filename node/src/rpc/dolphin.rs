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

//! Dolphin RPC Extensions

use super::*;
use crate::service::{Client, StateBackend};
use codec::Encode;
use frame_support::{storage::storage_prefix, StorageHasher, Twox64Concat};
use manta_primitives::types::Block;
use pallet_manta_pay::{
    types::{ReceiverChunk, SenderChunk, VoidNumber},
    Checkpoint, PullResponse,
};
use polkadot_service::NativeExecutionDispatch;
use sc_client_api::{HeaderBackend, StorageProvider};
use sp_api::{ApiExt, ConstructRuntimeApi};
use sp_core::storage::StorageKey;
use sp_offchain::OffchainWorkerApi;
use sp_runtime::{generic::BlockId, traits::BlakeTwo256};
use sp_session::SessionKeys;
use sp_transaction_pool::runtime_api::TaggedTransactionQueue;
use std::sync::Arc;

use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};

/// The storage name: MantaPay
const MANTA_PAY_KEY_PREFIX: [u8; 8] = *b"MantaPay";
/// The storage name: Shards
const MANTA_PAY_STORAGE_SHARDS_NAME: [u8; 6] = *b"Shards";
/// The storage name: VoidNumberSetInsertionOrder
const MANTA_PAY_STORAGE_VOID_NAME: [u8; 27] = *b"VoidNumberSetInsertionOrder";

const PULL_MAX_SENDER_UPDATE_SIZE: u64 = 32768;
const PULL_MAX_RECEIVER_UPDATE_SIZE: u64 = 32768;

/// Instantiate all RPC extensions for dolphin.
pub fn create_dolphin_full<RuntimeApi, Executor>(
    deps: FullDeps<
        Client<RuntimeApi, Executor>,
        crate::service::TransactionPool<RuntimeApi, Executor>,
    >,
) -> Result<RpcExtension, sc_service::Error>
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: TaggedTransactionQueue<Block>
        + sp_api::Metadata<Block>
        + SessionKeys<Block>
        + ApiExt<Block, StateBackend = StateBackend>
        + OffchainWorkerApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + cumulus_primitives_core::CollectCollationInfo<Block>
        + pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
        + frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
        + pallet_manta_pay::runtime::PullLedgerDiffApi<Block>,
    StateBackend: sp_api::StateBackend<BlakeTwo256>,
    Executor: NativeExecutionDispatch + 'static,
{
    use frame_rpc_system::{SystemApiServer, SystemRpc};
    use pallet_transaction_payment_rpc::{TransactionPaymentApiServer, TransactionPaymentRpc};

    let mut module = RpcExtension::new(());
    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    module
        .merge(SystemRpc::new(client.clone(), pool, deny_unsafe).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;
    module
        .merge(TransactionPaymentRpc::new(client.clone()).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;

    let runtime_manta_pay_rpc = {
        use pallet_manta_pay::rpc::PullApiServer;
        pallet_manta_pay::rpc::Pull::new(client.clone()).into_rpc()
    };
    module
        .merge(runtime_manta_pay_rpc)
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;

    let prototype_manta_pay_rpc = Pull::new(client).into_rpc();
    module
        .merge(prototype_manta_pay_rpc)
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;

    Ok(module)
}

/// Pull API
#[rpc(server)]
pub trait PullApi {
    /// Returns the update required to be synchronized with the ledger starting from
    /// `checkpoint`.
    #[method(name = "mantaPay_pullLedgerDiff", blocking)]
    fn pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<PullResponse>;
}

pub struct Pull<R, E: NativeExecutionDispatch> {
    pub client: Arc<Client<R, E>>,
}

impl<R, E: NativeExecutionDispatch> Pull<R, E> {
    /// Builds a new [`Pull`] RPC API implementation.
    pub fn new(client: Arc<Client<R, E>>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<R, E> PullApiServer for Pull<R, E>
where
    R: ConstructRuntimeApi<Block, Client<R, E>> + Send + Sync + 'static,
    R::RuntimeApi: ApiExt<Block, StateBackend = StateBackend>,
    StateBackend: sp_api::StateBackend<BlakeTwo256>,
    E: NativeExecutionDispatch + 'static,
{
    fn pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<PullResponse> {
        let client = &self.client;
        let best_hash = client.info().best_hash;
        let block_id = BlockId::<Block>::hash(best_hash);

        let (more_receivers, receivers) = pull_receivers(
            client.clone(),
            &block_id,
            *checkpoint.receiver_index,
            max_receivers,
        );
        let (more_senders, senders) = pull_senders(
            client.clone(),
            &block_id,
            checkpoint.sender_index,
            max_senders,
        );
        let pr = PullResponse {
            should_continue: more_receivers || more_senders,
            receivers,
            senders,
        };

        Ok(pr)
    }
}

fn create_full_map_key(storage_name: &[u8], key: impl Encode) -> StorageKey {
    let prefix = storage_prefix(&MANTA_PAY_KEY_PREFIX, storage_name);
    let key = Twox64Concat::hash(&key.encode());

    let full_key = prefix.into_iter().chain(key).collect();
    StorageKey(full_key)
}

fn create_full_doublemap_key(
    storage_name: &[u8],
    key1: impl Encode,
    key2: impl Encode,
) -> StorageKey {
    let prefix = storage_prefix(&MANTA_PAY_KEY_PREFIX, storage_name);
    let key1 = Twox64Concat::hash(&key1.encode());
    let key2 = Twox64Concat::hash(&key2.encode());
    let key = key1.into_iter().chain(key2.into_iter());

    let full_key = prefix.into_iter().chain(key).collect();
    StorageKey(full_key)
}

fn pull_senders<RuntimeApi, Executor>(
    client: Arc<Client<RuntimeApi, Executor>>,
    block_id: &BlockId<Block>,
    sender_index: usize,
    max_update_request: u64,
) -> (bool, SenderChunk)
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: ApiExt<Block, StateBackend = StateBackend>,
    StateBackend: sp_api::StateBackend<BlakeTwo256>,
    Executor: NativeExecutionDispatch + 'static,
{
    let mut senders = Vec::new();
    let max_sender_index = if max_update_request > PULL_MAX_SENDER_UPDATE_SIZE {
        (sender_index as u64) + PULL_MAX_SENDER_UPDATE_SIZE
    } else {
        (sender_index as u64) + max_update_request
    };

    for idx in (sender_index as u64)..max_sender_index {
        let key = create_full_map_key(&MANTA_PAY_STORAGE_VOID_NAME, idx);
        match client.storage(block_id, &key) {
            Ok(Some(next)) => {
                let mut _next: VoidNumber = [0u8; 32];
                _next.copy_from_slice(&next.0[..32]);
                senders.push(_next);
            }
            _ => return (false, senders),
        }
    }
    let key = create_full_map_key(&MANTA_PAY_STORAGE_VOID_NAME, max_sender_index as u64);
    let should_continue = matches!(client.storage(block_id, &key), Ok(Some(_)));

    (should_continue, senders)
}

fn pull_receivers<RuntimeApi, Executor>(
    client: Arc<Client<RuntimeApi, Executor>>,
    block_id: &BlockId<Block>,
    receiver_indices: [usize; 256],
    max_update_request: u64,
) -> (bool, ReceiverChunk)
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: ApiExt<Block, StateBackend = StateBackend>,
    StateBackend: sp_api::StateBackend<BlakeTwo256>,
    Executor: NativeExecutionDispatch + 'static,
{
    let mut more_receivers = false;
    let mut receivers = Vec::new();
    let mut receivers_pulled: u64 = 0;
    let max_update = if max_update_request > PULL_MAX_RECEIVER_UPDATE_SIZE {
        PULL_MAX_RECEIVER_UPDATE_SIZE
    } else {
        max_update_request
    };

    for (shard_index, utxo_index) in receiver_indices.into_iter().enumerate() {
        more_receivers |= pull_receivers_for_shard(
            client.clone(),
            block_id,
            shard_index as u8,
            utxo_index,
            max_update,
            &mut receivers,
            &mut receivers_pulled,
        );
        // if max capacity is reached and there is more to pull, then we return
        if receivers_pulled == max_update && more_receivers {
            break;
        }
    }
    (more_receivers, receivers)
}

fn pull_receivers_for_shard<RuntimeApi, Executor>(
    client: Arc<Client<RuntimeApi, Executor>>,
    block_id: &BlockId<Block>,
    shard_index: u8,
    receiver_index: usize,
    max_update: u64,
    receivers: &mut ReceiverChunk,
    receivers_pulled: &mut u64,
) -> bool
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: ApiExt<Block, StateBackend = StateBackend>,
    StateBackend: sp_api::StateBackend<BlakeTwo256>,
    Executor: NativeExecutionDispatch + 'static,
{
    let max_receiver_index = (receiver_index as u64) + max_update;
    for idx in (receiver_index as u64)..max_receiver_index {
        let key = create_full_doublemap_key(&MANTA_PAY_STORAGE_SHARDS_NAME, shard_index, idx);
        if *receivers_pulled == max_update {
            let should_continue = matches!(client.storage(block_id, &key), Ok(Some(_)));
            return should_continue;
        }
        match client.storage(block_id, &key) {
            Ok(Some(next)) => {
                // println!("pull_receivers_for_shard next: {:?}, key: {:?}", next, hex::encode(&key));
                let next = serde_json::from_slice(&next.0).unwrap_or(Default::default());
                *receivers_pulled += 1;
                receivers.push(next);
            }
            _ => return false,
        }
    }
    let key = create_full_doublemap_key(
        &MANTA_PAY_STORAGE_SHARDS_NAME,
        shard_index,
        max_receiver_index,
    );
    matches!(client.storage(block_id, &key), Ok(Some(_)))
}
