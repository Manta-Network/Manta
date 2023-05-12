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

//! Implements a consensus that can verify blocks with both Nimbus and Aura
//! by adding a custom import_queue and Verifier that delegate based on
//! what type of seal a block has
//! NOTE: Does not change block *proposing*, set to Nimbus in service.rs
//! NOTE: Assumes running as a Parachain. Sovereign chain mode NOT SUPPORTED

use futures::TryFutureExt;
use log::debug;
use sc_client_api::HeaderBackend;
use sc_consensus::{
    import_queue::{BasicQueue, Verifier as VerifierT},
    BlockImport, BlockImportParams,
};
use sc_consensus_aura::{BuildVerifierParams, CompatibilityMode};
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use session_key_primitives::aura::AuraId;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::{error::Error as ConsensusError, CacheKeyId};
use sp_consensus_aura::AuraApi;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::{
    app_crypto::AppKey,
    traits::{Block as BlockT, Header as HeaderT},
};
use std::sync::Arc;

use nimbus_primitives::CompatibleDigestItem as NimbusDigestItem;
use sc_consensus_aura::CompatibleDigestItem as AuraDigestItem;

const LOG_TARGET: &str = "aura-nimbus-consensus";

struct AuraOrNimbusVerifier<Client, Block: BlockT, AuraCIDP, NimbusCIDP> {
    aura_verifier: sc_consensus_aura::AuraVerifier<
        Client,
        <AuraId as AppKey>::Pair,
        AuraCIDP,
        <<Block as BlockT>::Header as HeaderT>::Number,
    >,
    nimbus_verifier: nimbus_consensus::Verifier<Client, Block, NimbusCIDP>,
}
impl<Client, Block, AuraCIDP, NimbusCIDP> AuraOrNimbusVerifier<Client, Block, AuraCIDP, NimbusCIDP>
where
    Block: BlockT,
{
    pub fn new(
        client: Arc<Client>,
        create_inherent_data_providers_aura: AuraCIDP,
        create_inherent_data_providers_nimbus: NimbusCIDP,
        telemetry: Option<TelemetryHandle>,
    ) -> Self
    where
        Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
        <Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
        AuraCIDP: CreateInherentDataProviders<Block, ()> + 'static,
        NimbusCIDP: CreateInherentDataProviders<Block, ()> + 'static,
    {
        Self {
            aura_verifier: sc_consensus_aura::build_verifier(BuildVerifierParams {
                client: client.clone(),
                create_inherent_data_providers: create_inherent_data_providers_aura,
                check_for_equivocation: sc_consensus_aura::CheckForEquivocation::Yes,
                telemetry,
                compatibility_mode: CompatibilityMode::None,
            }),
            nimbus_verifier: nimbus_consensus::Verifier::new(
                client,
                create_inherent_data_providers_nimbus,
            ),
        }
    }
}

#[async_trait::async_trait]
impl<Client, Block, AuraCIDP, NimbusCIDP> VerifierT<Block>
    for AuraOrNimbusVerifier<Client, Block, AuraCIDP, NimbusCIDP>
where
    Block: BlockT,
    Client: ProvideRuntimeApi<Block> + Send + Sync,
    <Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId>,
    AuraCIDP: CreateInherentDataProviders<Block, ()> + 'static,
    <AuraCIDP as CreateInherentDataProviders<Block, ()>>::InherentDataProviders:
        InherentDataProviderExt,
    NimbusCIDP: CreateInherentDataProviders<Block, ()>,
    Client: sc_client_api::AuxStore + sc_client_api::BlockOf,
{
    async fn verify(
        &mut self,
        block_params: BlockImportParams<Block, ()>,
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
            .last()
            .ok_or("Block should have at least one digest/seal on it")?;

        // delegate verification to Aura or Nimbus verifiers
        if NimbusDigestItem::as_nimbus_seal(seal).is_some() {
            debug!(target: LOG_TARGET, "Verifying block with Nimbus");
            self.nimbus_verifier
                .verify(block_params)
                .map_err(Into::into)
                .await
        } else if AuraDigestItem::<<AuraId as AppKey>::Signature>::as_aura_seal(seal).is_some() {
            debug!(target: LOG_TARGET, "Verifying block with Aura");
            self.aura_verifier
                .verify(block_params)
                .map_err(Into::into)
                .await
        } else {
            Err("No Aura or Nimbus seal on block".to_string())
        }
    }
}

pub fn import_queue<Client, Block: BlockT, InnerBI>(
    client: Arc<Client>,
    block_import: InnerBI,
    backend: Arc<sc_client_db::Backend<Block>>,
    spawner: &impl sp_core::traits::SpawnEssentialNamed,
    registry: Option<&substrate_prometheus_endpoint::Registry>,
    telemetry: Option<TelemetryHandle>,
) -> ClientResult<BasicQueue<Block, InnerBI::Transaction>>
where
    InnerBI: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
    InnerBI::Transaction: Send,
    Client::Api: BlockBuilderApi<Block>,
    Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
    Client: sc_client_api::AuxStore + sc_client_api::UsageProvider<Block>,
    Client: HeaderBackend<Block> + sc_client_api::BlockOf,
    <Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId>,
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
                Ok((slot, timestamp))
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
        // NOTE: As of Aug2022, nimbus and aura simply delegate block import
        //       to cumulus. We're skipping wrapping both by using this directly.
        //       If in the future either of them diverge from this,
        //       we'll have to adapt to the change here and in
        //       node/src/service.rs:L467 aka. BuildNimbusConsensusParams
        Box::new(cumulus_client_consensus_common::ParachainBlockImport::new(
            block_import,
            backend,
        )),
        None,
        spawner,
        registry,
    ))
}
