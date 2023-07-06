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

//! Lottery RPC Interfaces

use crate::runtime::LotteryApi;
use core::marker::PhantomData;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block};
use sp_std::sync::Arc;

pub const LOTTERY_ERROR: i32 = 777;

#[rpc(server)]
pub trait LotteryRpc {
    #[method(name = "lottery_not_in_drawing_freezeout", blocking)]
    fn not_in_drawing_freezeout(&self) -> RpcResult<bool>;

    #[method(name = "lottery_current_prize_pool", blocking)]
    fn current_prize_pool(&self) -> RpcResult<u128>;

    #[method(name = "lottery_next_drawing_at", blocking)]
    fn next_drawing_at(&self) -> RpcResult<Option<u128>>;
}

/// Lottery RPC API Implementation
pub struct Lottery<B, C> {
    /// Client
    client: Arc<C>,

    /// Type Parameter Marker
    __: PhantomData<B>,
}

impl<B, C> Lottery<B, C> {
    /// Builds a new [`Pull`] RPC API implementation.
    #[inline]
    pub fn new(client: codec::alloc::sync::Arc<C>) -> Self {
        Self {
            client,
            __: PhantomData,
        }
    }
}

#[async_trait]
impl<B, C> LotteryRpcServer for Lottery<B, C>
where
    B: Block,
    C: 'static + ProvideRuntimeApi<B> + HeaderBackend<B>,
    C::Api: LotteryApi<B>,
{
    #[inline]
    fn not_in_drawing_freezeout(&self) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at: BlockId<_> = BlockId::hash(self.client.info().best_hash);
        api.not_in_drawing_freezeout(&at).map_err(|err| {
            CallError::Custom(ErrorObject::owned(
                LOTTERY_ERROR,
                "Unable to compute drawing freezeout",
                Some(format!("{err:?}")),
            ))
            .into()
        })
    }

    #[inline]
    fn current_prize_pool(&self) -> RpcResult<u128> {
        let api = self.client.runtime_api();
        let at: BlockId<_> = BlockId::hash(self.client.info().best_hash);
        api.current_prize_pool(&at).map_err(|err| {
            CallError::Custom(ErrorObject::owned(
                LOTTERY_ERROR,
                "Unable to compute current prize pool",
                Some(format!("{err:?}")),
            ))
            .into()
        })
    }

    #[inline]
    fn next_drawing_at(&self) -> RpcResult<Option<u128>> {
        let api = self.client.runtime_api();
        let at: BlockId<_> = BlockId::hash(self.client.info().best_hash);
        api.next_drawing_at(&at).map_err(|err| {
            CallError::Custom(ErrorObject::owned(
                LOTTERY_ERROR,
                "Unable to compute next drawing",
                Some(format!("{err:?}")),
            ))
            .into()
        })
    }
}
