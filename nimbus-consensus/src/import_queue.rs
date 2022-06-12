// Copyright 2019-2021 PureStake Inc.
// This file is part of Nimbus.

// Nimbus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Nimbus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Nimbus.  If not, see <http://www.gnu.org/licenses/>.

use std::{marker::PhantomData, sync::Arc};

use log::debug;
use nimbus_primitives::{digests::CompatibleDigestItem, NimbusId, NimbusPair, NIMBUS_ENGINE_ID};
use sc_consensus::{
	import_queue::{BasicQueue, Verifier as VerifierT},
	BlockImport, BlockImportParams,
};
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::{ByteArray, Pair as _};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::{error::Error as ConsensusError, CacheKeyId};
use sp_inherents::{CreateInherentDataProviders, InherentDataProvider};
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
	DigestItem,
};

/// The Nimbus verifier strips the seal digest, and checks that it is a valid signature by
/// the same key that was injected into the runtime and noted in the Seal digest.
/// From Nimbu's perspective any block that faithfully reports its authorship to the runtime
/// is valid. The intention is that the runtime itself may then put further restrictions on
/// the identity of the author.
struct Verifier<Client, Block, CIDP> {
	client: Arc<Client>,
	create_inherent_data_providers: CIDP,
	_marker: PhantomData<Block>,
}

#[async_trait::async_trait]
impl<Client, Block, CIDP> VerifierT<Block> for Verifier<Client, Block, CIDP>
where
	Block: BlockT,
	Client: ProvideRuntimeApi<Block> + Send + Sync,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
	CIDP: CreateInherentDataProviders<Block, ()>,
{
	async fn verify(
		&mut self,
		mut block_params: BlockImportParams<Block, ()>,
	) -> Result<
		(
			BlockImportParams<Block, ()>,
			Option<Vec<(CacheKeyId, Vec<u8>)>>,
		),
		String,
	> {
		debug!(
			target: crate::LOG_TARGET,
			"🪲 Header hash before popping digest {:?}",
			block_params.header.hash()
		);
		// Grab the seal digest. Assume it is last (since it is a seal after-all).
		let seal = block_params
			.header
			.digest_mut()
			.pop()
			.expect("Block should have at least one digest on it");

		let signature = seal
			.as_nimbus_seal()
			.ok_or_else(|| String::from("HeaderUnsealed"))?;

		debug!(
			target: crate::LOG_TARGET,
			"🪲 Header hash after popping digest {:?}",
			block_params.header.hash()
		);

		debug!(
			target: crate::LOG_TARGET,
			"🪲 Signature according to verifier is {:?}", signature
		);

		// Grab the author information from either the preruntime digest or the consensus digest
		//TODO use the trait
		let claimed_author = block_params
			.header
			.digest()
			.logs
			.iter()
			.find_map(|digest| match *digest {
				DigestItem::Consensus(id, ref author_id) if id == NIMBUS_ENGINE_ID => {
					Some(author_id.clone())
				}
				DigestItem::PreRuntime(id, ref author_id) if id == NIMBUS_ENGINE_ID => {
					Some(author_id.clone())
				}
				_ => None,
			})
			.expect("Expected one consensus or pre-runtime digest that contains author id bytes");

		debug!(
			target: crate::LOG_TARGET,
			"🪲 Claimed Author according to verifier is {:?}", claimed_author
		);

		// Verify the signature
		let valid_signature = NimbusPair::verify(
			&signature,
			block_params.header.hash(),
			&NimbusId::from_slice(&claimed_author)
				.map_err(|_| "Invalid Nimbus ID (wrong length)")?,
		);

		debug!(
			target: crate::LOG_TARGET,
			"🪲 Valid signature? {:?}", valid_signature
		);

		if !valid_signature {
			return Err("Block signature invalid".into());
		}

		// This part copied from RelayChainConsensus. I guess this is the inherent checking.
		if let Some(inner_body) = block_params.body.take() {
			let inherent_data_providers = self
				.create_inherent_data_providers
				.create_inherent_data_providers(*block_params.header.parent_hash(), ())
				.await
				.map_err(|e| e.to_string())?;

			let inherent_data = inherent_data_providers
				.create_inherent_data()
				.map_err(|e| format!("{:?}", e))?;

			let block = Block::new(block_params.header.clone(), inner_body);

			let inherent_res = self
				.client
				.runtime_api()
				.check_inherents(
					&BlockId::Hash(*block_params.header.parent_hash()),
					block.clone(),
					inherent_data,
				)
				.map_err(|e| format!("{:?}", e))?;

			if !inherent_res.ok() {
				for (i, e) in inherent_res.into_errors() {
					match inherent_data_providers.try_handle_error(&i, &e).await {
						Some(r) => r.map_err(|e| format!("{:?}", e))?,
						None => Err(format!(
							"Unhandled inherent error from `{}`.",
							String::from_utf8_lossy(&i)
						))?,
					}
				}
			}

			let (_, inner_body) = block.deconstruct();
			block_params.body = Some(inner_body);
		}

		block_params.post_digests.push(seal);

		// The standard is to use the longest chain rule. This is overridden by the `NimbusBlockImport` in the parachain context.
		block_params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::LongestChain);

		debug!(
			target: crate::LOG_TARGET,
			"🪲 Just finished verifier. posthash from params is {:?}",
			&block_params.post_hash()
		);

		Ok((block_params, None))
	}
}

/// Start an import queue for a Cumulus collator that does not uses any special authoring logic.
pub fn import_queue<Client, Block: BlockT, I, CIDP>(
	client: Arc<Client>,
	block_import: I,
	create_inherent_data_providers: CIDP,
	spawner: &impl sp_core::traits::SpawnEssentialNamed,
	registry: Option<&substrate_prometheus_endpoint::Registry>,
	parachain: bool,
) -> ClientResult<BasicQueue<Block, I::Transaction>>
where
	I: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
	I::Transaction: Send,
	Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
{
	let verifier = Verifier {
		client,
		create_inherent_data_providers,
		_marker: PhantomData,
	};

	Ok(BasicQueue::new(
		verifier,
		Box::new(NimbusBlockImport::new(block_import, parachain)),
		None,
		spawner,
		registry,
	))
}

/// Nimbus specific block import.
///
/// Nimbus supports both parachain and non-parachain contexts. In the parachain
/// context, new blocks should not be imported as best. Cumulus's ParachainBlockImport
/// handles this correctly, but does not work in non-parachain contexts.
/// This block import has a field indicating whether we should apply parachain rules or not.
///
/// There may be additional nimbus-specific logic here in the future, but for now it is
/// only the conditional parachain logic
pub struct NimbusBlockImport<I> {
	inner: I,
	parachain_context: bool,
}

impl<I> NimbusBlockImport<I> {
	/// Create a new instance.
	pub fn new(inner: I, parachain_context: bool) -> Self {
		Self {
			inner,
			parachain_context,
		}
	}
}

#[async_trait::async_trait]
impl<Block, I> BlockImport<Block> for NimbusBlockImport<I>
where
	Block: BlockT,
	I: BlockImport<Block> + Send,
{
	type Error = I::Error;
	type Transaction = I::Transaction;

	async fn check_block(
		&mut self,
		block: sc_consensus::BlockCheckParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		self.inner.check_block(block).await
	}

	async fn import_block(
		&mut self,
		mut block_import_params: sc_consensus::BlockImportParams<Block, Self::Transaction>,
		cache: std::collections::HashMap<sp_consensus::CacheKeyId, Vec<u8>>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		// If we are in the parachain context, best block is determined by the relay chain
		// except during initial sync
		if self.parachain_context {
			block_import_params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::Custom(
				block_import_params.origin == sp_consensus::BlockOrigin::NetworkInitialSync,
			));
		}

		// Now continue on to the rest of the import pipeline.
		self.inner.import_block(block_import_params, cache).await
	}
}
