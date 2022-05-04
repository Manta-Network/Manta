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

use crate::{
	types::{decode, EncryptedNote, Utxo, VoidNumber},
	Config, Pallet, Shards, VoidNumberSet,
};
use alloc::sync::Arc;
use core::marker::PhantomData;
use jsonrpc_core::{Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
use manta_accounting::wallet::ledger;
use manta_pay::{
	config,
	signer::{Checkpoint, RawCheckpoint},
};
use manta_util::serde::{Deserialize, Serialize};
use scale_codec::{Decode, Encode, Error as CodecError};
use scale_info::TypeInfo;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block};

/// Pull API
#[rpc(server)]
pub trait PullApi {
	/// Returns the update required to be synchronized with the ledger starting from
	/// `checkpoint`.
	#[rpc(name = "manta_pay_pull")]
	fn pull(&self, checkpoint: Checkpoint) -> jsonrpc_core::Result<PullResponse>;
}

/// Receiver Chunk Data Type
pub type ReceiverChunk = Vec<(config::Utxo, config::EncryptedNote)>;

/// Sender Chunk Data Type
pub type SenderChunk = Vec<config::VoidNumber>;

/// Ledger Source Pull Response
#[derive(
	Clone, Debug, Decode, Default, Deserialize, Encode, Eq, Hash, PartialEq, Serialize, TypeInfo,
)]
#[serde(crate = "manta_util::serde", deny_unknown_fields)]
pub struct PullResponse {
	/// Pull Continuation Flag
	///
	/// The `should_continue` flag is set to `true` if the client should request more data from the
	/// ledger to finish the pull.
	pub should_continue: bool,

	/// Ledger Checkpoint
	///
	/// If the `should_continue` flag is set to `true` then `checkpoint` is the next [`Checkpoint`]
	/// to request data from the ledger. Otherwise, it represents the current ledger state.
	pub checkpoint: Checkpoint,

	/// Ledger Receiver Chunk
	pub receivers: ReceiverChunk,

	/// Ledger Sender Chunk
	pub senders: SenderChunk,
}

impl<T> Pallet<T>
where
	T: Config,
{
	/// Maximum Number of Updates per Shard
	const PULL_MAX_PER_SHARD_UPDATE_SIZE: usize = 128;

	/// Maximum Size of Sender Data Update
	const PULL_MAX_SENDER_UPDATE_SIZE: usize = 1024;

	/// Pulls receiver data from the ledger starting at the `receiver_index`.
	#[inline]
	fn pull_receivers(
		receiver_index: &mut [usize; 256],
	) -> Result<(bool, ReceiverChunk), CodecError> {
		let mut more_receivers = false;
		let mut receivers = Vec::new();
		for (i, index) in receiver_index.iter_mut().enumerate() {
			more_receivers |= Self::pull_receivers_for_shard(i as u8, index, &mut receivers)?;
		}
		Ok((more_receivers, receivers))
	}

	/// Pulls receiver data from the shard at `shard_index` starting at the `receiver_index`,
	/// pushing the results back to `receivers`.
	#[inline]
	fn pull_receivers_for_shard(
		shard_index: u8,
		receiver_index: &mut usize,
		receivers: &mut ReceiverChunk,
	) -> Result<bool, CodecError> {
		let mut iter = if *receiver_index == 0 {
			Shards::<T>::iter_prefix(shard_index)
		} else {
			let raw_key = Shards::<T>::hashed_key_for(shard_index, *receiver_index as u64 - 1);
			Shards::<T>::iter_prefix_from(shard_index, raw_key)
		};
		for _ in 0..Self::PULL_MAX_PER_SHARD_UPDATE_SIZE {
			match iter.next() {
				Some((_, (utxo, encrypted_note))) => {
					*receiver_index += 1;
					receivers.push((decode(utxo)?, encrypted_note));
				}
				_ => return Ok(false),
			}
		}
		Ok(iter.next().is_some())
	}

	/// Pulls sender data from the ledger starting at the `sender_index`.
	#[inline]
	fn pull_senders(sender_index: &mut usize) -> Result<(bool, SenderChunk), CodecError> {
		let mut senders = Vec::new();
		let mut iter = VoidNumberSet::<T>::iter().skip(*sender_index);
		for _ in 0..Self::PULL_MAX_SENDER_UPDATE_SIZE {
			match iter.next() {
				Some((sender, _)) => {
					*sender_index += 1;
					senders.push(decode(sender)?);
				}
				_ => return Ok((false, senders)),
			}
		}
		Ok((iter.next().is_some(), senders))
	}

	/// Returns the update required to be synchronized with the ledger starting from
	/// `checkpoint`.
	#[inline]
	pub fn pull(mut checkpoint: Checkpoint) -> Result<PullResponse, CodecError> {
		let (more_receivers, receivers) = Self::pull_receivers(&mut checkpoint.receiver_index)?;
		let (more_senders, senders) = Self::pull_senders(&mut checkpoint.sender_index)?;
		Ok(PullResponse {
			should_continue: more_receivers || more_senders,
			checkpoint,
			receivers,
			senders,
		})
	}
}

sp_api::decl_runtime_apis! {
	pub trait MantaPayPullRuntimeApi {
		fn pull(checkpoint: RawCheckpoint) -> Result<PullResponse, CodecError>;
	}
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
	C::Api: MantaPayPullRuntimeApi<B>,
{
	#[inline]
	fn pull(&self, checkpoint: Checkpoint) -> jsonrpc_core::Result<PullResponse> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		match api.pull(&at, checkpoint.into()) {
			Ok(Ok(response)) => Ok(response),
			Ok(Err(err)) => Err(RpcError {
				code: ErrorCode::ServerError(1),
				message: "Unable to compute state diff for pull".into(),
				data: Some(err.to_string().into()),
			}),
			Err(err) => Err(RpcError {
				code: ErrorCode::ServerError(2),
				message: "Error during runtime API".into(),
				data: Some(err.to_string().into()),
			}),
		}
	}
}
