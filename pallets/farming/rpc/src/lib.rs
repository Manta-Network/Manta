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

use std::{marker::PhantomData, sync::Arc};

use codec::Codec;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
pub use manta_farming_rpc_runtime_api::{self as runtime_api, FarmingRuntimeApi};
use manta_primitives::types::Balance;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

#[rpc(client, server)]
pub trait FarmingRpcApi<BlockHash, AccountId, CurrencyId, PoolId>
where
    AccountId: Send + Sync + 'static,
    PoolId: Send + Sync + 'static,
{
    /// rpc method for getting farming rewards
    #[method(name = "farming_getFarmingRewards")]
    async fn get_farming_rewards(
        &self,
        who: AccountId,
        pid: PoolId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(CurrencyId, NumberOrHex)>>;

    /// rpc method for getting gauge rewards
    #[method(name = "farming_getGaugeRewards")]
    async fn get_gauge_rewards(
        &self,
        who: AccountId,
        pid: PoolId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(CurrencyId, NumberOrHex)>>;
}

#[derive(Clone, Debug)]
pub struct FarmingRpc<C, Block> {
    client: Arc<C>,
    _marker: PhantomData<Block>,
}

impl<C, Block> FarmingRpc<C, Block> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<C, Block, AccountId, CurrencyId, PoolId>
    FarmingRpcApiServer<<Block as BlockT>::Hash, AccountId, CurrencyId, PoolId>
    for FarmingRpc<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: FarmingRuntimeApi<Block, AccountId, CurrencyId, PoolId>,
    AccountId: Codec + Send + Sync + 'static,
    CurrencyId: Codec,
    PoolId: Codec + Send + Sync + 'static,
{
    async fn get_farming_rewards(
        &self,
        who: AccountId,
        pid: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<(CurrencyId, NumberOrHex)>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let rs: Result<Vec<(CurrencyId, Balance)>, _> = api.get_farming_rewards(&at, who, pid);

        match rs {
            Ok(rewards) => Ok(rewards
                .into_iter()
                .map(|(token, amount)| (token, NumberOrHex::Hex(amount.into())))
                .collect()),
            Err(e) => Err(CallError::Custom(ErrorObject::owned(
                ErrorCode::InternalError.code(),
                "Failed to get farming rewards.",
                Some(format!("{:?}", e)),
            ))),
        }
        .map_err(jsonrpsee::core::Error::Call)
    }

    async fn get_gauge_rewards(
        &self,
        who: AccountId,
        pid: PoolId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<(CurrencyId, NumberOrHex)>> {
        let api = self.client.runtime_api();
        let at = BlockId::<Block>::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let rs: Result<Vec<(CurrencyId, Balance)>, _> = api.get_gauge_rewards(&at, who, pid);

        match rs {
            Ok(rewards) => Ok(rewards
                .into_iter()
                .map(|(token, amount)| (token, NumberOrHex::Hex(amount.into())))
                .collect()),
            Err(e) => Err(CallError::Custom(ErrorObject::owned(
                ErrorCode::InternalError.code(),
                "Failed to get gauge rewards.",
                Some(format!("{:?}", e)),
            ))),
        }
        .map_err(jsonrpsee::core::Error::Call)
    }
}
