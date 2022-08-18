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

//! Implements a consensus that can propose blocks with Nimbus and verify with both Nimbus and Aura

use std::{marker::PhantomData, sync::Arc};

use nimbus_primitives::{digests::CompatibleDigestItem};
use sc_consensus::{
    import_queue::{BasicQueue, Verifier as VerifierT},
    BlockImport, BlockImportParams,
};
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::{Pair as _};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::{error::Error as ConsensusError, CacheKeyId};
use sp_inherents::{CreateInherentDataProviders};
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, Header as HeaderT},
};
use sc_client_api::HeaderBackend;
use sc_consensus_aura::BuildVerifierParams;
use sp_runtime::traits::Hash;
use nimbus_consensus::NimbusBlockImport;
use sc_telemetry::TelemetryHandle;
use sp_core::Pair;
use sp_api::ApiExt;
use session_key_primitives::aura::AuraId;
use sp_consensus_aura::AuraApi;
use sc_consensus::ImportResult;
use codec::alloc::collections::HashMap;
use nimbus_primitives::NimbusApi;
use sc_consensus::BlockCheckParams;
use cumulus_client_consensus_common::ParachainBlockImport;
use futures::TryFutureExt;

const LOG_TARGET: &str = "aura-nimbus-consensus";

/// A block-import handler that selects aura or nimbus import dynamically
pub struct AuraOrNimbusBlockImport<Block: BlockT, C, I: BlockImport<Block>> {
    inner_aura: I,
    inner_nimbus: I,
    client: Arc<C>,
    _phantom: PhantomData<(Block, AuraId)>,
}

impl<Block: BlockT, C, I: Clone + BlockImport<Block>> Clone for AuraOrNimbusBlockImport<Block, C, I> {
    fn clone(&self) -> Self {
        AuraOrNimbusBlockImport {
            inner_aura: self.inner_aura.clone(),
            inner_nimbus: self.inner_nimbus.clone(),
            client: self.client.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<Block: BlockT, C, I: BlockImport<Block>> AuraOrNimbusBlockImport<Block, C, I> {
    /// New aura block import.
    pub fn new(
        inner_aura: I,
        inner_nimbus: I,
        client: Arc<C>,
    ) -> Self {
        Self {
            inner_aura,
            inner_nimbus,
            client,
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<Block: BlockT, C, I> BlockImport<Block> for AuraOrNimbusBlockImport<Block, C, I> where
    I: BlockImport<Block> + Send,
    I::Transaction: Send + 'static,
    I::Error: Into<ConsensusError>,
    C: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
{
    type Error = ConsensusError;
    type Transaction = I::Transaction;
    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        let at = &BlockId::hash(block.hash);
        if self.client.runtime_api().has_api::<dyn AuraApi<Block, AuraId>>(at).unwrap_or(false) {
            self.inner_aura.check_block(block).await
        } else if self.client.runtime_api().has_api::<dyn NimbusApi<Block>>(at).unwrap_or(false) {
            self.inner_nimbus.check_block(block).await
        } else {
            Err(sp_consensus::Error::ClientImport("No aura or nimbus support in runtime".to_string()))
        }
    }

   async fn import_block(
        &mut self,
        block: BlockImportParams<Block, Self::Transaction>,
        new_cache: HashMap<CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        let at = &BlockId::hash(block.header.hash());
        if self.client.runtime_api().has_api::<dyn AuraApi<Block, AuraId>>(at).unwrap_or(false) {
            self.inner_aura.import_block(block, new_cache).map_err(Into::into).await
        } else if self.client.runtime_api().has_api::<dyn NimbusApi<Block>>(at).unwrap_or(false) {
            self.inner_nimbus.import_block(block, new_cache).map_err(Into::into).await
        } else {
            Err(sp_consensus::Error::ClientImport("No aura or nimbus seal in block".to_string()))
        }
    }
}

struct AuraOrNimbusVerifier<C, Block, CIDP> {
    auraVerifier: sc_consensus_aura::AuraVerifier<C, AuraId, sp_consensus::NeverCanAuthor, CIDP>,
    nimbusVerifier: nimbus_consensus::Verifier<C, Block, CIDP>,
}
impl<C, Block, CIDP> AuraOrNimbusVerifier<C, Block, CIDP>
where
    Block: BlockT,
    CIDP: CreateInherentDataProviders<Block, ()>,
{
    pub fn new( client : C, create_inherent_data_providers: CIDP, telemetry: Option<TelemetryHandle> ) -> Self
    where
        C: ProvideRuntimeApi<Block> + Clone + Send + Sync + 'static,
        <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>
     {
        // TODO: Check if this can be expressed as CIDP
        let aura_inherent = Box::new(|_, _| async {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client).unwrap();
            let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
            Ok((timestamp, slot))
        });
        Self{
            auraVerifier: sc_consensus_aura::build_verifier(
                BuildVerifierParams {
                    client: client.clone(),
                    aura_inherent,
                    // NOTE: We only support verification of historic aura blocks, not new block proposals using aura
                    can_author_with: sp_consensus::NeverCanAuthor{},
                    check_for_equivocation: sc_consensus_aura::CheckForEquivocation::No,
                    telemetry
                }
            ),
            nimbusVerifier: nimbus_consensus::Verifier {
                client:client.clone(),
                create_inherent_data_providers,
                _marker: PhantomData::<Block>{},
            }
        }
    }
}

#[async_trait::async_trait]
impl<C, Block, CIDP> VerifierT<Block> for AuraOrNimbusVerifier<C, Block, CIDP>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + Send + Sync,
    <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
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
    >{
        // We assume the outermost digest item is the block seal ( we have no two-step consensus )
        let seal = block_params
            .header
            .digest()
            .logs()
            .first()
            .expect("Block should have at least one digest on it");

        // let isNimbus = seal.into::<nimbus_primitives::CompatibleDigestItem>().as_nimbus_seal().is_ok();
        // let isAura = seal.into::<sp_consensus_aura::digests::CompatibleDigestItem>().as_aura_seal().is_ok();
        let isNimbus = seal.into().as_nimbus_seal().is_ok();
        let isAura = seal.into().as_aura_seal().is_ok();

        if !(isAura || isNimbus) {
            return Err("NoSealFound".to_string());
        }

        // delegate to Aura or nimbus verifiers
        if isNimbus {
            self.nimbusVerifier.verify(block_params).map_err(Into::into).await
        } else {
            self.auraVerifier.verify(block_params).map_err(Into::into).await
        }
    }
}

pub fn import_queue<C, Block: BlockT, I, CIDP>(
	client: Arc<C>,
	block_import: I,
	create_inherent_data_providers: CIDP,
	spawner: &impl sp_core::traits::SpawnEssentialNamed,
	registry: Option<&substrate_prometheus_endpoint::Registry>,
    telemetry: Option<TelemetryHandle>,
) -> ClientResult<BasicQueue<Block, I::Transaction>>
where
	I: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
	I::Transaction: Send,
	C::Api: BlockBuilderApi<Block>,
	C: ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C: HeaderBackend<Block> ,
	// <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId> + ApiExt<Block>,
	<C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId>,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static
{
    let verifier = AuraOrNimbusVerifier::new(
        client.clone(),
        create_inherent_data_providers,
        telemetry
    );

    let auraBlockImport = Arc::new(futures::lock::Mutex::new(ParachainBlockImport::new(block_import))); // see cumulus/client/consensus/aura/src/import_queue.rs:90
    let nimbusBlockImport = NimbusBlockImport::new(block_import, true); // true = always parachain mode
    Ok(BasicQueue::new(
        verifier,
        Box::new(AuraOrNimbusBlockImport::new(auraBlockImport,nimbusBlockImport,client) as I::Transaction),
        None,
        spawner,
        registry,
    ))
}
