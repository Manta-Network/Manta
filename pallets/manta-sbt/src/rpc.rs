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

use crate::{runtime::SBTPullLedgerDiffApi, Checkpoint, PullResponse};
use alloc::sync::Arc;
use core::marker::PhantomData;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use manta_support::manta_pay::DensePullResponse;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block};

/// Pull Ledger Diff Error Code
pub const PULL_LEDGER_DIFF_ERROR: i32 = 1;

/// Pull API
#[rpc(server)]
pub trait SBTPullApi {
    /// Returns the update required to be synchronized with the ledger starting from
    /// `checkpoint`.
    #[method(name = "mantaSBT_pull_ledger_diff", blocking)]
    fn sbt_pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<PullResponse>;

    #[method(name = "mantaSBT_dense_pull_ledger_diff", blocking)]
    fn sbt_dense_pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<DensePullResponse>;
}

/// Pull RPC API Implementation
pub struct SBTPull<B, C> {
    /// Client
    client: Arc<C>,

    /// Type Parameter Marker
    __: PhantomData<B>,
}

impl<B, C> SBTPull<B, C> {
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
impl<B, C> SBTPullApiServer for SBTPull<B, C>
where
    B: Block,
    C: 'static + ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: SBTPullLedgerDiffApi<B>,
{
    #[inline]
    fn sbt_pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<PullResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.sbt_pull_ledger_diff(&at, checkpoint.into(), max_receivers, max_senders)
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
    fn sbt_dense_pull_ledger_diff(
        &self,
        checkpoint: Checkpoint,
        max_receivers: u64,
        max_senders: u64,
    ) -> RpcResult<DensePullResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().finalized_hash);
        api.sbt_pull_ledger_diff(&at, checkpoint.into(), max_receivers, max_senders)
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
}
