// Copyright 2020-2024 Manta Network.
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

//! Nimbus-based Parachain Node Service

use crate::{client::RuntimeApiCommon, rpc};
use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_parachain_inherent::{MockValidationDataInherentDataProvider, MockXcmConfig};
use cumulus_client_service::{
    build_network, prepare_node_config, start_relay_chain_tasks, CollatorSybilResistance,
    DARecoveryProfile, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{relay_chain::CollatorPair, ParaId};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
use futures::{Stream, StreamExt};
use hex_literal::hex;
pub use manta_primitives::types::{AccountId, Balance, Block, Hash, Header, Nonce};
use nimbus_consensus::NimbusManualSealConsensusDataProvider;
use polkadot_service::HeaderBackend;
use sc_consensus::{ImportQueue, LongestChain};
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::NetworkBlock;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_service::{Configuration, Error, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use session_key_primitives::{AuraId, NimbusId};
use sp_api::ConstructRuntimeApi;
use sp_consensus::SyncOracle;
use sp_core::H256;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::BlakeTwo256;
use std::{sync::Arc, time::Duration};
use substrate_prometheus_endpoint::Registry;

/// Relaychain raw storage key for timestamp
pub const TIMESTAMP_NOW: &[u8] =
    &hex!["f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"];

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = sp_io::SubstrateHostFunctions;

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions = (
    sp_io::SubstrateHostFunctions,
    frame_benchmarking::benchmarking::HostFunctions,
);

/// Native Manta Parachain executor instance.
pub struct MantaRuntimeExecutor;
impl sc_executor::NativeExecutionDispatch for MantaRuntimeExecutor {
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        manta_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        manta_runtime::native_version()
    }
}

/// Native Calamari Parachain executor instance.
pub struct CalamariRuntimeExecutor;
impl sc_executor::NativeExecutionDispatch for CalamariRuntimeExecutor {
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        calamari_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        calamari_runtime::native_version()
    }
}

/// We use wasm executor only now.
pub type DefaultExecutorType = WasmExecutor<HostFunctions>;

/// Backend Type
pub type FullBackend = TFullBackend<Block>;

/// Full Client Implementation Type
pub type FullClient<RuntimeApi> = TFullClient<Block, RuntimeApi, DefaultExecutorType>;

/// Default Import Queue Type
pub type DefaultImportQueue = sc_consensus::DefaultImportQueue<Block>;

/// Full Transaction Pool Type
pub type TransactionPool<RuntimeApi> = sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>;

/// Block Import type
pub type ParachainBlockImport<RuntimeApi> =
    TParachainBlockImport<Block, Arc<FullClient<RuntimeApi>>, FullBackend>;

/// Components Needed for Chain Ops Subcommands
pub type PartialComponents<RuntimeApi> = sc_service::PartialComponents<
    FullClient<RuntimeApi>,
    FullBackend,
    Option<LongestChain<FullBackend, Block>>,
    DefaultImportQueue,
    TransactionPool<RuntimeApi>,
    (
        ParachainBlockImport<RuntimeApi>,
        Option<Telemetry>,
        Option<TelemetryWorkerHandle>,
    ),
>;

/// State Backend Type
pub type StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this function if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi>(
    config: &Configuration,
    local_dev_service: bool,
) -> Result<PartialComponents<RuntimeApi>, Error>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon + sp_consensus_aura::AuraApi<Block, AuraId>,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let heap_pages = config
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let executor = WasmExecutor::<HostFunctions>::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);
    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());
    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });
    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let maybe_select_chain = if local_dev_service {
        Some(sc_consensus::LongestChain::new(backend.clone()))
    } else {
        None
    };

    let (import_queue, block_import) = if local_dev_service {
        let block_import = ParachainBlockImport::new(client.clone(), backend.clone());
        (
            crate::aura_or_nimbus_consensus::import_queue(
                client.clone(),
                client.clone(),
                backend.clone(),
                &task_manager.spawn_essential_handle(),
                config.prometheus_registry(),
                telemetry.as_ref().map(|telemetry| telemetry.handle()),
                !local_dev_service,
            )?,
            block_import,
        )
    } else {
        let parachain_block_import =
            ParachainBlockImport::new_with_delayed_best_block(client.clone(), backend.clone());
        (
            crate::aura_or_nimbus_consensus::import_queue(
                client.clone(),
                client.clone(),
                backend.clone(),
                &task_manager.spawn_essential_handle(),
                config.prometheus_registry(),
                telemetry.as_ref().map(|telemetry| telemetry.handle()),
                !local_dev_service,
            )?,
            parachain_block_import,
        )
    };

    Ok(PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: maybe_select_chain,
        other: (block_import, telemetry, telemetry_worker_handle),
    })
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
#[allow(clippy::too_many_arguments)]
pub async fn start_parachain_node<RuntimeApi, RB>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    rpc_ext_builder: RB,
    block_authoring_duration: Duration,
    async_backing: bool,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon + sp_consensus_aura::AuraApi<Block, AuraId>,
    RB: Fn(
            rpc::FullDeps<FullClient<RuntimeApi>, TransactionPool<RuntimeApi>>,
        ) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>
        + 'static,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi>(&parachain_config, false)?;
    let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

    let client = params.client.clone();
    let backend = params.backend.clone();

    let mut task_manager = params.task_manager;
    let (relay_chain_interface, collator_key) = crate::builder::build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
    )
    .await
    .map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

    let force_authoring = parachain_config.force_authoring;
    let collator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue_service = params.import_queue.service();
    let net_config = sc_network::config::FullNetworkConfiguration::new(&parachain_config.network);

    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
        build_network(cumulus_client_service::BuildNetworkParams {
            parachain_config: &parachain_config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: params.import_queue,
            relay_chain_interface: relay_chain_interface.clone(),
            para_id: id,
            sybil_resistance_level: CollatorSybilResistance::Resistant,
        })
        .await?;

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
                command_sink: None,
            };

            rpc_ext_builder(deps)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: params.keystore_container.keystore(),
        backend: backend.clone(),
        network: network.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
        sync_service: sync_service.clone(),
    })?;

    let announce_block = {
        let sync_service = sync_service.clone();
        Arc::new(move |hash, data| sync_service.announce_block(hash, data))
    };

    let overseer_handle = relay_chain_interface
        .clone()
        .overseer_handle()
        .map_err(|e| sc_service::Error::Application(Box::new(e)))?;

    let relay_chain_slot_duration = core::time::Duration::from_secs(6);

    start_relay_chain_tasks(StartRelayChainTasksParams {
        para_id: id,
        announce_block: announce_block.clone(),
        client: client.clone(),
        task_manager: &mut task_manager,
        da_recovery_profile: if collator {
            DARecoveryProfile::Collator
        } else {
            DARecoveryProfile::FullNode
        },
        relay_chain_interface: relay_chain_interface.clone(),
        import_queue: import_queue_service,
        relay_chain_slot_duration,
        recovery_handle: Box::new(overseer_handle.clone()),
        sync_service: sync_service.clone(),
    })?;

    if collator {
        start_consensus::<RuntimeApi, _>(
            async_backing,
            backend.clone(),
            client.clone(),
            block_import,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            params.keystore_container.keystore(),
            id,
            collator_key.expect("Command line arguments do not allow this. qed"),
            overseer_handle,
            announce_block,
            force_authoring,
            relay_chain_slot_duration,
            block_authoring_duration,
            sync_service.clone(),
        )?;
    }

    start_network.start_network();
    Ok((task_manager, client))
}

/// Start node with dev config
#[sc_tracing::logging::prefix_logs_with("Parachain")]
#[allow(clippy::too_many_arguments)]
pub async fn start_dev_node<RuntimeApi, RB>(
    parachain_config: Configuration,
    rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon + sp_consensus_aura::AuraApi<Block, AuraId>,
    RB: Fn(
            rpc::FullDeps<FullClient<RuntimeApi>, TransactionPool<RuntimeApi>>,
        ) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>
        + 'static,
{
    let params = new_partial::<RuntimeApi>(&parachain_config, true)?;
    let (block_import, mut telemetry, _telemetry_worker_handle) = params.other;

    let transaction_pool = params.transaction_pool.clone();
    let import_queue = params.import_queue;
    let client = params.client.clone();
    let backend = params.backend.clone();

    let keystore_container = params.keystore_container;
    let mut task_manager = params.task_manager;
    let net_config = sc_network::config::FullNetworkConfiguration::new(&parachain_config.network);

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: None,
            net_config,
            block_relay: None,
        })?;

    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let collator = parachain_config.role.is_authority();

    if collator {
        log::info!("Is running as Collator");
        let env = sc_basic_authorship::ProposerFactory::with_proof_recording(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let commands_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> =
            Box::new(
                // This bit cribbed from the implementation of instant seal.
                transaction_pool
                    .pool()
                    .validated_pool()
                    .import_notification_stream()
                    .map(|_| EngineCommand::SealNewBlock {
                        create_empty: false,
                        finalize: false,
                        parent_hash: None,
                        sender: None,
                    }),
            );

        let select_chain = params.select_chain.expect(
            "`new_partial` builds a `LongestChainRule` when building dev service.\
				We specified the dev service when calling `new_partial`.\
				Therefore, a `LongestChainRule` is present. qed.",
        );

        let client_set_aside_for_cidp = client.clone();
        let client_clone = client.clone();
        let keystore_clone = keystore_container.keystore().clone();

        let maybe_provide_vrf_digest =
            move |nimbus_id: NimbusId, parent: Hash| -> Option<sp_runtime::generic::DigestItem> {
                crate::client::vrf_pre_digest::<Block, FullClient<RuntimeApi>>(
                    &client_clone,
                    &keystore_clone,
                    nimbus_id,
                    parent,
                )
            };

        task_manager.spawn_essential_handle().spawn_blocking(
            "authorship_task",
            Some("block-authoring"),
            run_manual_seal(ManualSealParams {
                block_import,
                env,
                client: client.clone(),
                pool: transaction_pool.clone(),
                commands_stream,
                select_chain,
                consensus_data_provider: Some(Box::new(NimbusManualSealConsensusDataProvider {
                    keystore: keystore_container.keystore(),
                    client: client.clone(),
                    additional_digests_provider: maybe_provide_vrf_digest,
                    _phantom: Default::default(),
                })),
                create_inherent_data_providers: move |block: H256, ()| {
                    let maybe_current_para_block = client_set_aside_for_cidp.number(block);

                    let client_for_xcm = client_set_aside_for_cidp.clone();
                    async move {
                        let time = sp_timestamp::InherentDataProvider::from_system_time();

                        let current_para_block = maybe_current_para_block?
                            .ok_or(sp_blockchain::Error::UnknownBlock(block.to_string()))?;

                        let mocked_parachain = MockValidationDataInherentDataProvider {
                            current_para_block,
                            relay_offset: 1000,
                            relay_blocks_per_para_block: 2,
                            // TODO: Recheck
                            para_blocks_per_relay_epoch: 10,
                            relay_randomness_config: (),
                            xcm_config: MockXcmConfig::new(
                                &*client_for_xcm,
                                block,
                                Default::default(),
                                Default::default(),
                            ),
                            raw_downward_messages: Default::default(),
                            raw_horizontal_messages: Default::default(),
                            additional_key_values: None,
                        };

                        let randomness = session_key_primitives::inherent::InherentDataProvider;

                        Ok((time, mocked_parachain, randomness))
                    }
                },
            }),
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
                command_sink: None,
            };

            rpc_ext_builder(deps)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: keystore_container.keystore(),
        backend: backend.clone(),
        network: network.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
        sync_service: sync_service.clone(),
    })?;

    log::info!("⚠️  DEV STANDALONE MODE.");

    network_starter.start_network();
    Ok((task_manager, client))
}

#[allow(clippy::too_many_arguments)]
fn start_consensus<RuntimeApi, SO>(
    async_backing: bool,
    backend: Arc<FullBackend>,
    client: Arc<FullClient<RuntimeApi>>,
    block_import: ParachainBlockImport<RuntimeApi>,
    prometheus_registry: Option<&Registry>,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
    relay_chain_interface: Arc<dyn RelayChainInterface>,
    transaction_pool: Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>>,
    keystore: KeystorePtr,
    para_id: ParaId,
    collator_key: CollatorPair,
    overseer_handle: OverseerHandle,
    announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
    force_authoring: bool,
    relay_chain_slot_duration: Duration,
    block_authoring_duration: Duration,
    sync_oracle: SO,
) -> Result<(), sc_service::Error>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon + sp_consensus_aura::AuraApi<Block, AuraId>,
    sc_client_api::StateBackendFor<FullBackend, Block>: sc_client_api::StateBackend<BlakeTwo256>,
    SO: SyncOracle + Send + Sync + Clone + 'static,
{
    let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool,
        prometheus_registry,
        telemetry.clone(),
    );

    let proposer = Proposer::new(proposer_factory);

    let collator_service = CollatorService::new(
        client.clone(),
        Arc::new(task_manager.spawn_handle()),
        announce_block,
        client.clone(),
    );

    let create_inherent_data_providers = |_, _| async move {
        let time = sp_timestamp::InherentDataProvider::from_system_time();

        let nimbus = nimbus_primitives::InherentDataProvider;

        let session = session_key_primitives::inherent::InherentDataProvider;

        Ok((time, nimbus, session))
    };

    let client_clone = client.clone();
    let keystore_clone = keystore.clone();
    let maybe_provide_vrf_digest =
        move |nimbus_id: NimbusId, parent: Hash| -> Option<sp_runtime::generic::DigestItem> {
            crate::client::vrf_pre_digest::<Block, FullClient<RuntimeApi>>(
                &client_clone,
                &keystore_clone,
                nimbus_id,
                parent,
            )
        };

    if async_backing {
        log::info!("Collator started with asynchronous backing.");
        let client_clone = client.clone();
        let code_hash_provider = move |block_hash| {
            client_clone
                .code_at(block_hash)
                .ok()
                .map(polkadot_primitives::ValidationCode)
                .map(|c| c.hash())
        };
        task_manager.spawn_essential_handle().spawn(
            "nimbus",
            None,
            nimbus_consensus::collators::lookahead::run::<
                Block,
                _,
                _,
                _,
                FullBackend,
                _,
                _,
                _,
                _,
                _,
                _,
            >(nimbus_consensus::collators::lookahead::Params {
                additional_digests_provider: maybe_provide_vrf_digest,
                authoring_duration: block_authoring_duration,
                block_import,
                code_hash_provider,
                collator_key,
                collator_service,
                create_inherent_data_providers,
                force_authoring,
                keystore,
                overseer_handle,
                para_backend: backend,
                para_client: client,
                para_id,
                proposer,
                relay_chain_slot_duration,
                relay_client: relay_chain_interface,
                slot_duration: None,
                sync_oracle,
            }),
        );
    } else {
        log::info!("Collator started without asynchronous backing.");
        task_manager.spawn_essential_handle().spawn(
            "nimbus",
            None,
            nimbus_consensus::collators::basic::run::<
                Block,
                _,
                _,
                FullBackend,
                FullClient<RuntimeApi>,
                _,
                _,
                _,
                _,
            >(nimbus_consensus::collators::basic::Params {
                additional_digests_provider: maybe_provide_vrf_digest,
                //authoring_duration: Duration::from_millis(500),
                block_import,
                collator_key,
                collator_service,
                create_inherent_data_providers,
                force_authoring,
                keystore,
                overseer_handle,
                para_id,
                para_client: client,
                proposer,
                relay_client: relay_chain_interface,
            }),
        );
    };

    Ok(())
}
