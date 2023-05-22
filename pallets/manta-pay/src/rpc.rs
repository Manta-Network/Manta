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

//! MantaPay RPC Interfaces

use crate::runtime::PullLedgerDiffApi;
use alloc::sync::Arc;
use core::marker::PhantomData;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use manta_support::manta_pay::{
    Checkpoint, DenseInitialSyncResponse, DensePullResponse, InitialSyncResponse, PullResponse,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block};

/// Pull Ledger Diff Error Code
pub const PULL_LEDGER_DIFF_ERROR: i32 = 1;

/// Pull API
#[rpc(server)]
pub trait PullApi {
    /// Returns the update required to be synchronized with the ledger starting from
    /// `checkpoint`.
    #[method(name = "mantaPay_pull_ledger_diff", blocking)]
    fn pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<PullResponse>;

    #[method(name = "mantaPay_dense_pull_ledger_diff", blocking)]
    fn dense_pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<DensePullResponse>;

    /// Returns the update required for the initial synchronization with the ledger.
    #[method(name = "mantaPay_initial_pull", blocking)]
    fn initial_pull(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
    ) -> RpcResult<InitialSyncResponse>;

    #[method(name = "mantaPay_dense_initial_pull", blocking)]
    fn dense_initial_pull(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
    ) -> RpcResult<DenseInitialSyncResponse>;

    #[method(name = "mantaPay_pull_ledger_total_count", blocking)]
    fn pull_ledger_total_count(&self) -> RpcResult<[u8; 16]>;
}

/// Pull RPC API Implementation
pub struct Pull<B, C> {
    /// Client
    client: Arc<C>,

    /// Type Parameter Marker
    __: PhantomData<B>,
}

impl<B, C> Pull<B, C> {
    /// Builds a new [`Pull`] RPC API implementation.
    #[inline]
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            __: PhantomData,
        }
    }
}

#[async_trait]
impl<B, C> PullApiServer for Pull<B, C>
where
    B: Block,
    C: 'static + ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: PullLedgerDiffApi<B>,
{
    #[inline]
    fn pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<PullResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.pull_ledger_diff(&at, checkpoint.into(), max_receivers, max_senders)
            .map_err(|err| {
                CallError::Custom(ErrorObject::owned(
                    PULL_LEDGER_DIFF_ERROR,
                    "Unable to compute state diff for pull",
                    Some(format!("{err:?}")),
                ))
                .into()
            })
    }

    #[inline]
    fn dense_pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<DensePullResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.pull_ledger_diff(&at, checkpoint.into(), max_receivers, max_senders)
            .map(Into::into)
            .map_err(|err| {
                CallError::Custom(ErrorObject::owned(
                    PULL_LEDGER_DIFF_ERROR,
                    "Unable to compute dense state diff for pull",
                    Some(format!("{err:?}")),
                ))
                .into()
            })
    }

    #[inline]
    fn initial_pull(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
    ) -> RpcResult<InitialSyncResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.initial_pull(&at, checkpoint.into(), max_receivers)
            .map_err(|err| {
                CallError::Custom(ErrorObject::owned(
                    PULL_LEDGER_DIFF_ERROR,
                    "Unable to compute state diff for initial pull",
                    Some(format!("{err:?}")),
                ))
                .into()
            })
    }

    #[inline]
    fn dense_initial_pull(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
    ) -> RpcResult<DenseInitialSyncResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.initial_pull(&at, checkpoint.into(), max_receivers)
            .map(Into::into)
            .map_err(|err| {
                CallError::Custom(ErrorObject::owned(
                    PULL_LEDGER_DIFF_ERROR,
                    "Unable to compute state diff for initial pull",
                    Some(format!("{err:?}")),
                ))
                .into()
            })
    }

    #[inline]
    fn pull_ledger_total_count(&self) -> RpcResult<[u8; 16]> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.pull_ledger_total_count(&at).map_err(|err| {
            CallError::Custom(ErrorObject::owned(
                PULL_LEDGER_DIFF_ERROR,
                "Unable to compute total count for pull",
                Some(format!("{err:?}")),
            ))
            .into()
        })
    }
}
