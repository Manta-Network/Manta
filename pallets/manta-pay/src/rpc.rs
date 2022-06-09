// Copyright 2019-2022 Manta Network.
// This file is part of pallet-manta-pay.
//
// pallet-manta-pay is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pallet-manta-pay is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pallet-manta-pay.  If not, see <http://www.gnu.org/licenses/>.

//! MantaPay RPC Interfaces

use crate::{runtime::PullLedgerDiffApi, PullResponse};
use alloc::sync::Arc;
use core::marker::PhantomData;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use manta_pay::signer::Checkpoint;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block};

/// Pull API
#[rpc(server)]
pub trait PullApi {
	/// Returns the update required to be synchronized with the ledger starting from
	/// `checkpoint`.
	#[rpc(name = "mantaPay_pull_ledger_diff")]
	fn pull_ledger_diff(&self, checkpoint: Checkpoint) -> Result<PullResponse>;
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

impl<B, C> PullApi for Pull<B, C>
where
	B: Block,
	C: 'static + ProvideRuntimeApi<B> + HeaderBackend<B>,
	C::Api: PullLedgerDiffApi<B>,
{
	#[inline]
	fn pull_ledger_diff(&self, checkpoint: Checkpoint) -> Result<PullResponse> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		api.pull_ledger_diff(&at, checkpoint.into())
			.map_err(|err| Error {
				code: ErrorCode::ServerError(1),
				message: "Unable to compute state diff for pull".into(),
				data: Some(err.to_string().into()),
			})
	}
}
