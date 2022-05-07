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

//! Common RPC Extensions

use crate::rpc::{Builder, RpcExtension};
use frame_rpc_system::{AccountNonceApi, FullSystem, SystemApi};
use manta_primitives::types::{AccountId, Balance, Block, Index as Nonce};
use pallet_transaction_payment_rpc::{
	TransactionPayment, TransactionPaymentApi, TransactionPaymentRuntimeApi,
};
use sc_client_api::HeaderBackend;
use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_service::{Error, RpcExtensionBuilder};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;

/// Common RPC Extension Marker
pub struct Common;

impl<C, P> RpcExtensionBuilder for Builder<C, P, Common>
where
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: BlockBuilder<Block>
		+ AccountNonceApi<Block, AccountId, Nonce>
		+ TransactionPaymentRuntimeApi<Block, Balance>,
	P: 'static + TransactionPool,
{
	type Output = RpcExtension;

	#[inline]
	fn build(&self, deny: DenyUnsafe, _: SubscriptionTaskExecutor) -> Result<Self::Output, Error> {
		let mut io = RpcExtension::default();
		io.extend_with(
			FullSystem::new(self.client.clone(), self.transaction_pool.clone(), deny).to_delegate(),
		);
		io.extend_with(TransactionPayment::new(self.client.clone()).to_delegate());
		Ok(io)
	}
}
