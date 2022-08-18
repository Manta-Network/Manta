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
use sp_runtime::app_crypto::AppKey;
use sp_consensus::NeverCanAuthor;
use futures::TryFutureExt;
use sc_client_api::HeaderBackend;
use sc_consensus::{
    import_queue::{BasicQueue, Verifier as VerifierT},
    BlockImport, BlockImportParams,
};
use sc_consensus_aura::BuildVerifierParams;
use sc_telemetry::TelemetryHandle;
use session_key_primitives::aura::AuraId;
use nimbus_primitives::NIMBUS_ENGINE_ID;
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::{error::Error as ConsensusError, CacheKeyId};
use sp_consensus_aura::AuraApi;
use sc_consensus_slots::InherentDataProviderExt;
use sp_core::Pair;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::{
    traits::{Block as BlockT, Header as HeaderT},
};

const LOG_TARGET: &str = "aura-nimbus-consensus";

struct AuraOrNimbusVerifier<C, Block: BlockT, CIDP_AURA: CreateInherentDataProviders<Block,ExtraArgs>, CIDP_NIMBUS: CreateInherentDataProviders<Block,ExtraArgs>> {
    auraVerifier: sc_consensus_aura::AuraVerifier<C, <AuraId as AppKey>::Pair, NeverCanAuthor, CIDP_AURA>,
    nimbusVerifier: nimbus_consensus::Verifier<C, Block, CIDP_NIMBUS>,
}
impl<C, Block, CIDP_AURA, CIDP_NIMBUS> AuraOrNimbusVerifier<C, Block, CIDP_AURA, CIDP_NIMBUS>
where
    Block: BlockT,
    CIDP_AURA: CreateInherentDataProviders<Block, ()>, // TODO: Get rid of CIDP
	CIDP_AURA::InherentDataProviders: InherentDataProviderExt + Send,
    CIDP_NIMBUS: CreateInherentDataProviders<Block, ()>, // TODO: Get rid of CIDP
{
    pub fn new(
        client: Arc<C>,
        telemetry: Option<TelemetryHandle>,
    ) -> Self
    where
        C: ProvideRuntimeApi<Block> + Send + Sync + 'static,
        <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + sp_consensus_aura::AuraApi<Block, AuraId>,
        // Aura Traits
        C: sc_client_api::UsageProvider<Block> + sc_client_api::AuxStore
    {
        let create_aura_inherent : CIDP_AURA = move |_, _| async {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client).unwrap();
            let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
            Ok((timestamp, slot))
        };
        let create_nimbus_inherent : CIDP_NIMBUS = move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();
            Ok((time,))
        };
        Self{
            auraVerifier: sc_consensus_aura::build_verifier(BuildVerifierParams {
                client: client.clone(),
                create_inherent_data_providers: create_aura_inherent,
                // NOTE: We only support verification of historic aura blocks, not new block proposals using aura
                can_author_with: NeverCanAuthor {},
                check_for_equivocation: sc_consensus_aura::CheckForEquivocation::No,
                telemetry,
            }),
            nimbusVerifier: nimbus_consensus::Verifier {
                client: client.clone(),
                create_inherent_data_providers: create_nimbus_inherent,
                _marker: PhantomData::<Block> {},
            },
        }
    }
}

#[async_trait::async_trait]
impl<C, Block, CIDP_AURA, CIDP_NIMBUS> VerifierT<Block> for AuraOrNimbusVerifier<C, Block, CIDP_AURA, CIDP_NIMBUS>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + Send + Sync,
    <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
    // Aura Traits
    C: sc_client_api::AuxStore,
    C: sc_client_api::BlockOf,
    <C as sp_api::ProvideRuntimeApi<Block>>::Api: sp_consensus_aura::AuraApi<Block, AuraId>,
    sp_consensus_aura::sr25519::AuthorityPair: sp_application_crypto::Pair,
    CIDP_AURA: CreateInherentDataProviders<Block, ()>, // TODO: Get rid of CIDP
	CIDP_AURA::InherentDataProviders: InherentDataProviderExt + Send,
    CIDP_NIMBUS: CreateInherentDataProviders<Block, ()>, // TODO: Get rid of CIDP
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
        // We assume the outermost digest item is the block seal ( we have no two-step consensus )
        let seal = block_params
            .header
            .digest()
            .logs()
            .first()
            .expect("Block should have at least one digest/seal on it");

        // delegate to Aura or nimbus verifiers
        if seal.seal_try_to(&NIMBUS_ENGINE_ID).is_some() {
            self.nimbusVerifier
                .verify(block_params)
                .map_err(Into::into)
                .await
        } else if seal.seal_try_to(&AURA_ENGINE_ID).is_some() {
            self.auraVerifier
                .verify(block_params)
                .map_err(Into::into)
                .await
        } else {
            Err("NoSealFound".to_string())
        }
    }
}

pub fn import_queue<C, Block: BlockT, I>(
    client: Arc<C>,
    block_import: I,
    spawner: &impl sp_core::traits::SpawnEssentialNamed,
    registry: Option<&substrate_prometheus_endpoint::Registry>,
    telemetry: Option<TelemetryHandle>,
) -> ClientResult<BasicQueue<Block, I::Transaction>>
where
    I: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
    I::Transaction: Send,
    C::Api: BlockBuilderApi<Block>,
    C: ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C: HeaderBackend<Block> + sc_client_api::BlockOf + sc_client_api::AuxStore + sc_client_api::UsageProvider<Block>,
    // <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId> + ApiExt<Block>,
    <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId>,
    // <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, <AuraId as AppKey> as Pair>,
{
    let verifier = AuraOrNimbusVerifier::new(client.clone(), telemetry);
    Ok(BasicQueue::new(
        verifier,
        Box::new(block_import),
        None,
        spawner,
        registry,
    ))
}
