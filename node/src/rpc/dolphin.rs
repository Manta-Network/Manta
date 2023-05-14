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

//! Dolphin RPC Extensions

use super::*;
use manta_farming_rpc_api::{FarmingRpc, FarmingRpcApiServer};
use manta_farming_rpc_runtime_api::FarmingRuntimeApi;
use manta_primitives::types::{DolphinAssetId, PoolId};
use pallet_manta_pay::{
    rpc::{Pull, PullApiServer},
    runtime::PullLedgerDiffApi,
};
use zenlink_protocol::AssetId;
use zenlink_protocol_rpc::{ZenlinkProtocol, ZenlinkProtocolApiServer};
use zenlink_protocol_runtime_api::ZenlinkProtocolApi as ZenlinkProtocolRuntimeApi;
use zenlink_stable_amm_rpc::{StableAmm, StableAmmApiServer};

/// Instantiate all RPC extensions for dolphin.
pub fn create_dolphin_full<C, P>(deps: FullDeps<C, P>) -> Result<RpcExtension, sc_service::Error>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        + Send
        + Sync
        + 'static,
    C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    C::Api: PullLedgerDiffApi<Block>,
    C::Api: FarmingRuntimeApi<Block, AccountId, DolphinAssetId, PoolId>,
    C::Api: ZenlinkProtocolRuntimeApi<Block, AccountId, AssetId>,
    C::Api: zenlink_stable_amm_runtime_api::StableAmmApi<
        Block,
        DolphinAssetId,
        Balance,
        AccountId,
        PoolId,
    >,
    P: TransactionPool + Sync + Send + 'static,
{
    use frame_rpc_system::{System, SystemApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};

    let mut module = RpcExtension::new(());
    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    module
        .merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;
    module
        .merge(TransactionPayment::new(client.clone()).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;

    let manta_pay_rpc: jsonrpsee::RpcModule<Pull<Block, C>> = Pull::new(client.clone()).into_rpc();
    module
        .merge(manta_pay_rpc)
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;

    module
        .merge(FarmingRpc::new(client.clone()).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;
    module
        .merge(ZenlinkProtocol::new(client.clone()).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;
    module
        .merge(StableAmm::new(client).into_rpc())
        .map_err(|e| sc_service::Error::Other(e.to_string()))?;

    Ok(module)
}
