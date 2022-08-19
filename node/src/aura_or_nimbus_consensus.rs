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

//! Implements a consensus that can  verify with both Nimbus and Aura
//! NOTE: Nimbus is used for proposing exclusively

use futures::TryFutureExt;
use nimbus_primitives::NIMBUS_ENGINE_ID;
use sc_client_api::HeaderBackend;
use sc_consensus::{
    import_queue::{BasicQueue, Verifier as VerifierT},
    BlockImport, BlockImportParams,
};
use sc_consensus_aura::BuildVerifierParams;
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use session_key_primitives::aura::AuraId;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::NeverCanAuthor;
use sp_consensus::{error::Error as ConsensusError, CacheKeyId};
use sp_consensus_aura::AuraApi;
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_core::Pair;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::app_crypto::AppKey;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::{marker::PhantomData, sync::Arc};

use nimbus_primitives::CompatibleDigestItem as NimbusDigestItem;
use sc_consensus_aura::CompatibleDigestItem as AuraDigestItem;

const LOG_TARGET: &str = "aura-nimbus-consensus";

struct AuraOrNimbusVerifier<C, Block: BlockT, CIDP_AURA, CIDP_NIMBUS> {
    auraVerifier:
        sc_consensus_aura::AuraVerifier<C, <AuraId as AppKey>::Pair, NeverCanAuthor, CIDP_AURA>,
    nimbusVerifier: nimbus_consensus::Verifier<C, Block, CIDP_NIMBUS>,
}
impl<C, Block, CIDP_AURA, CIDP_NIMBUS> AuraOrNimbusVerifier<C, Block, CIDP_AURA, CIDP_NIMBUS>
where
    Block: BlockT,
{
    pub fn new(
        client: Arc<C>,
        create_inherent_data_providers_aura: CIDP_AURA,
        create_inherent_data_providers_nimbus: CIDP_NIMBUS,
        telemetry: Option<TelemetryHandle>,
    ) -> Self
    where
        C: ProvideRuntimeApi<Block> + Send + Sync + 'static,
        <C as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
        CIDP_AURA: CreateInherentDataProviders<Block, ()> + 'static,
        CIDP_NIMBUS: CreateInherentDataProviders<Block, ()> + 'static,
    {
        Self {
            auraVerifier: sc_consensus_aura::build_verifier(BuildVerifierParams {
                client: client.clone(),
                create_inherent_data_providers: create_inherent_data_providers_aura,
                // NOTE: We only support verification of historic aura blocks, not new block proposals using aura
                can_author_with: NeverCanAuthor {},
                check_for_equivocation: sc_consensus_aura::CheckForEquivocation::No,
                telemetry,
            }),
            nimbusVerifier: nimbus_consensus::build_verifier(
                nimbus_consensus::BuildVerifierParams {
                    client: client.clone(),
                    create_inherent_data_providers: create_inherent_data_providers_nimbus,
                    _marker: PhantomData::<Block> {},
                },
            ),
        }
    }
}

#[async_trait::async_trait]
impl<C, Block, CIDP_AURA, CIDP_NIMBUS> VerifierT<Block>
    for AuraOrNimbusVerifier<C, Block, CIDP_AURA, CIDP_NIMBUS>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + Send + Sync,
    <C as ProvideRuntimeApi<Block>>::Api:
        BlockBuilderApi<Block> + sp_consensus_aura::AuraApi<Block, AuraId>,
    CIDP_AURA: CreateInherentDataProviders<Block, ()> + 'static,
    <CIDP_AURA as CreateInherentDataProviders<Block, ()>>::InherentDataProviders:
        InherentDataProviderExt,
    CIDP_NIMBUS: CreateInherentDataProviders<Block, ()>,
    C: sc_client_api::AuxStore + sc_client_api::BlockOf,
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
        if NimbusDigestItem::as_nimbus_seal(seal).is_some() {
            self.nimbusVerifier
                .verify(block_params)
                .map_err(Into::into)
                .await
        } else if AuraDigestItem::<<AuraId as AppKey>::Signature>::as_aura_seal(seal).is_some() {
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
    C: sc_client_api::AuxStore + sc_client_api::UsageProvider<Block>,
    C: HeaderBackend<Block> + sc_client_api::BlockOf,
    <C as ProvideRuntimeApi<Block>>::Api:
        BlockBuilderApi<Block> + sp_consensus_aura::AuraApi<Block, AuraId>,
{
    let verifier = AuraOrNimbusVerifier::new(
        client.clone(),
        move |_, _| {
            let client2 = client.clone();
            async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                let slot_duration =
                    cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();
                let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                            *timestamp,
                            slot_duration,
                        );
                Ok((timestamp, slot))
            }
        },
        move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();
            Ok((time,))
        },
        telemetry,
    );
    Ok(BasicQueue::new(
        verifier,
        Box::new(cumulus_client_consensus_common::ParachainBlockImport::new(
            block_import,
        )), // TODO: Check if this sets non-longest fork choice rule
        None,
        spawner,
        registry,
    ))
}
