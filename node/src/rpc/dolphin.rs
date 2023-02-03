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
use pallet_manta_pay::{
    rpc::{Pull, PullApiServer},
    runtime::PullLedgerDiffApi,
};

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

    Ok(module)
}
