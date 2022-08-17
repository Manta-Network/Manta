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
use sc_network::NetworkService;
use sc_client_api::HeaderBackend;
use sc_client_api::AuxStore;
use sc_client_api::BlockOf;
use sc_consensus_aura::BuildVerifierParams;
use sp_runtime::traits::Hash;
use sp_keystore::SyncCryptoStorePtr;
use cumulus_client_consensus_common::ParachainConsensus;
use sp_consensus::Error;
use nimbus_consensus::BuildNimbusConsensusParams;
use nimbus_consensus::NimbusConsensus;
use nimbus_consensus::NimbusBlockImport;
use cumulus_primitives_core::ParaId;
use cumulus_relay_chain_interface::RelayChainInterface;
use sc_service::TaskManager;
use sc_telemetry::TelemetryHandle;
use substrate_prometheus_endpoint::Registry;
use crate::service_aura::Client;
use sc_consensus::DefaultImportQueue;
use codec::Encode;
use codec::Decode;
use sp_core::Pair;
use sp_api::ApiExt;
use session_key_primitives::aura::AuraId;
// use sc_consensus_aura::AuthorityId;
use sp_consensus_aura::AuraApi;
use sc_consensus::ImportResult;
use codec::alloc::collections::HashMap;
use nimbus_primitives::NimbusApi;
use core::fmt::Debug;
use sc_consensus::BlockCheckParams;

type AuthorityId<P> = AuraId;

const LOG_TARGET: &str = "aura-nimbus-consensus";

/// A block-import handler that selects aura or nimbus import dynamically.
pub struct AuraOrNimbusBlockImport<Block: BlockT, C, I: BlockImport<Block>, P> {
    inner_aura: I,
    inner_nimbus: I,
    client: Arc<C>,
    _phantom: PhantomData<(Block, P)>,
}

impl<Block: BlockT, C, I: Clone + BlockImport<Block>, P> Clone for AuraOrNimbusBlockImport<Block, C, I, P> {
    fn clone(&self) -> Self {
        AuraOrNimbusBlockImport {
            inner_aura: self.inner_aura.clone(),
            inner_nimbus: self.inner_nimbus.clone(),
            client: self.client.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<Block: BlockT, C, I: BlockImport<Block>, P> AuraOrNimbusBlockImport<Block, C, I, P> {
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

impl<Block: BlockT, C, I, P> BlockImport<Block> for AuraOrNimbusBlockImport<Block, C, I, P> where
    I: BlockImport<Block, Transaction = sp_api::TransactionFor<C, Block>> + Send + Sync,
    I::Error: Into<ConsensusError>,
    C: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    P: Pair + Send + Sync + 'static,
    P::Public: Clone + Eq + Send + Sync + Hash + Debug + Encode + Decode,
    P::Signature: Encode + Decode,
{
    type Error = ConsensusError;
    type Transaction = sp_api::TransactionFor<C, Block>;

    fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {

        if self.client.runtime_api().has_api::<dyn AuraApi<Block, AuraId>>(block).unwrap_or(false) {
            self.inner_aura.check_block(block).map_err(Into::into)
        } else if self.client.runtime_api().has_api::<dyn NimbusApi<Block>>(block).unwrap_or(false) {
            self.inner_nimbus.check_block(block).map_err(Into::into)
        } else {
            Err("No supported consensus mechanism found in block {}",block)
        }
    }

    fn import_block(
        &mut self,
        block: BlockImportParams<Block, Self::Transaction>,
        new_cache: HashMap<CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        if self.client.runtime_api().has_api::<dyn AuraApi<Block, AuraId>>(&block).unwrap_or(false) {
            self.inner_aura.import_block(block, new_cache).map_err(Into::into)
        } else if self.client.runtime_api().has_api::<dyn NimbusApi<Block>>(&block).unwrap_or(false) {
            self.inner_nimbus.import_block(block, new_cache).map_err(Into::into)
        } else {
            Err("No supported consensus mechanism found in block {}",block)
        }
    }
}

struct AuraOrNimbusVerifier<Client, Block, CIDP> {
    auraVerifier: sc_consensus_aura::AuraVerifier<Client, P, CAW, CIDP>,
    nimbusVerifier: nimbus_consensus::Verifier<Client, Block, CIDP>,
}
impl<CIDP, Block> AuraOrNimbusVerifier<Client<RuntimeApi>, Block, CIDP> where
    Block: BlockT,
    CIDP: CreateInherentDataProviders<Block, ()>,
{
    pub fn new ( client : Client, create_inherent_data_providers: CIDP, telemetry: TelemetryHandle ){
        AuraOrNimbusVerifier{
            auraVerifier: sc_consensus_aura::build_verifier(
                BuildVerifierParams {
                    client: client.clone(),
                    create_inherent_data_providers,
                    // NOTE: We only support verification of historic aura blocks, not new block proposals using aura
                    can_author_with: sp_consensus::NeverCanAuthor{},
                    check_for_equivocation: sc_consensus_aura::CheckForEquivocation::No,
                    telemetry
                }
            ),
            nimbusVerifier: nimbus_consensus::import_queue::Verifier {
                client:client.clone(),
                create_inherent_data_providers: client.clone(),
                _marker: PhantomData::<Block>{},
            }
        }
    }
}

#[async_trait::async_trait]
impl<Client, Block, CIDP> VerifierT<Block> for AuraOrNimbusVerifier<Client, Block, CIDP>
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
            target: LOG_TARGET,
            "🪲 Header hash before popping digest {:?}",
            block_params.header.hash()
        );
        // We assume the outermost digest item is the block seal ( we have no two-step consensus )
        let seal = block_params
            .header
            .logs()
            .first()
            .expect("Block should have at least one digest on it");

        let isNimbus = seal.into::<nimbus_primitives::CompatibleDigestItem>().as_nimbus_seal().is_ok();
        let isAura = seal.into::<sp_consensus_aura::digests::CompatibleDigestItem>().as_aura_seal().is_ok();

        if !(isAura || isNimbus) {
            Err("NoSealFound")
        }

        // delegate to Aura or nimbus verifiers
        if isNimbus {
            self.nimbusVerifier.verify(block_params)
        }
        else {
            self.auraVerifier.verify(block_params)
        }
    }
}

pub fn import_queue<B, I, C, P, S>(
    block_import: I,
    client: Arc<C>,
    inherent_data_providers: sp_inherents::InherentDataProvider,
    spawner: &S,
    registry: Option<&Registry>,
    telemetry: Option<TelemetryHandle>,
) -> Result<DefaultImportQueue<B, C>, sp_consensus::Error> where
B: BlockT,
C::Api: BlockBuilderApi<B> + AuraApi<B, AuthorityId<P>>,
// C::Api: BlockBuilderApi<B> + AuraApi<B, AuthorityId<P>> + ApiExt<B, Error = sp_blockchain::Error>,
C: 'static + ProvideRuntimeApi<B> + BlockOf + Send + Sync + AuxStore + HeaderBackend<B>,
I: BlockImport<B, Error=ConsensusError, Transaction = sp_api::TransactionFor<C, B>> + Send + Sync + 'static,
DigestItemFor<B>: CompatibleDigestItem<P>,
P: Pair + Send + Sync + 'static,
P::Public: Clone + Eq + Send + Sync + Hash + Debug + Encode + Decode,
P::Signature: Encode + Decode,
S: sp_core::traits::SpawnNamed,
{
    let verifier = AuraOrNimbusVerifier::new(
        client,
        inherent_data_providers,
        telemetry
    );

    let auraBlockImport = Arc::new(futures::lock::Mutex::new(ParachainBlockImport::new(block_import))); // see cumulus/client/consensus/aura/src/import_queue.rs:90
    let nimbusBlockImport = NimbusBlockImport::new(block_import, true);// true = always parachain mode
    Ok(BasicQueue::new(
        verifier,
        Box::new(AuraOrNimbusBlockImport::new(auraBlockImport,nimbusBlockImport)),
        None,
        spawner,
        registry,
    ))
}
