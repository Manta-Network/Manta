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

use crate::{
    client::{RuntimeApiCommon, RuntimeApiNimbus},
    rpc,
};
use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_common::{
    ParachainBlockImport as TParachainBlockImport, ParachainConsensus,
};
use cumulus_client_service::{
    build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
    CollatorSybilResistance, DARecoveryProfile, StartCollatorParams, StartFullNodeParams,
    StartRelayChainTasksParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_relay_chain_interface::RelayChainInterface;
pub use manta_primitives::types::{AccountId, Balance, Block, Hash, Header, Nonce};
use sc_consensus::ImportQueue;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::{config::SyncMode, NetworkBlock, NetworkService};
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_service::{
    Configuration, Error, SpawnTaskHandle, TFullBackend, TFullClient, TaskManager, WarpSyncParams,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use session_key_primitives::AuraId;
use sp_api::ConstructRuntimeApi;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry;

//const LOG_TARGET_SYNC: &str = "sync::cumulus";

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

/// Full Client Implementation Type
pub type FullClient<RuntimeApi> = TFullClient<Block, RuntimeApi, DefaultExecutorType>;

/// Default Import Queue Type
pub type DefaultImportQueue = sc_consensus::DefaultImportQueue<Block>;

/// Full Transaction Pool Type
pub type TransactionPool<RuntimeApi> = sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>;

/// Block Import type
pub type ParachainBlockImport<RuntimeApi> =
    TParachainBlockImport<Block, Arc<FullClient<RuntimeApi>>, TFullBackend<Block>>;

/// Components Needed for Chain Ops Subcommands
pub type PartialComponents<RuntimeApi> = sc_service::PartialComponents<
    FullClient<RuntimeApi>,
    TFullBackend<Block>,
    (),
    DefaultImportQueue,
    TransactionPool<RuntimeApi>,
    (
        ParachainBlockImport<RuntimeApi>,
        Option<Telemetry>,
        Option<TelemetryWorkerHandle>,
    ),
>;

/// State Backend Type
pub type StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi>(
    config: &Configuration,
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

    let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

    let import_queue = crate::aura_or_nimbus_consensus::import_queue(
        // single step block import pipeline, after nimbus/aura seal, import block into client
        client.clone(),
        block_import.clone(),
        backend.clone(),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        telemetry.as_ref().map(|telemetry| telemetry.handle()),
    )?;

    Ok(PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (block_import, telemetry, telemetry_worker_handle),
    })
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
pub async fn start_parachain_node<RuntimeApi, RB>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCommon + RuntimeApiNimbus + sp_consensus_aura::AuraApi<Block, AuraId>,
    RB: Fn(
            rpc::FullDeps<FullClient<RuntimeApi>, TransactionPool<RuntimeApi>>,
        ) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>
        + 'static,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi>(&parachain_config)?;
    let (_block_import, mut telemetry, telemetry_worker_handle) = params.other;

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

    let client = params.client.clone();
    let backend = params.backend.clone();

    let force_authoring = parachain_config.force_authoring;
    let collator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue = params.import_queue.service();
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
    if collator {
        /*let parachain_consensus = build_consensus(
            id,
            client.clone(),
            backend,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            network,
            params.keystore_container.keystore(),
            force_authoring,
        )?;*/
        let spawner = task_manager.spawn_handle();
        start_relay_chain_tasks(StartRelayChainTasksParams {
            para_id: id,
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            da_recovery_profile: DARecoveryProfile::Collator,
            relay_chain_interface,
            import_queue,
            relay_chain_slot_duration,
            recovery_handle: Box::new(overseer_handle),
            sync_service,
        })?;
    } else {
        start_relay_chain_tasks(StartRelayChainTasksParams {
            para_id: id,
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            da_recovery_profile: DARecoveryProfile::FullNode,
            relay_chain_interface,
            import_queue,
            relay_chain_slot_duration,
            recovery_handle: Box::new(overseer_handle),
            sync_service,
        })?;
    }

    start_network.start_network();
    Ok((task_manager, client))
}

/*/// Start a dev node using nimbus instant-sealing consensus without relaychain attached.
pub async fn start_dev_nimbus_node(
    config: Configuration,
) -> sc_service::error::Result<TaskManager>
{
    use sc_consensus::LongestChain;

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain: _maybe_select_chain,
        transaction_pool,
        other: (_, _),
    } = new_partial(&config)?;

    let net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        build_network(cumulus_client_service::BuildNetworkParams {
            parachain_config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            net_config,
            sybil_resistance_level: CollatorSybilResistance::Resistant,
        }).await?;

    let role = config.role.clone();
    let select_chain = LongestChain::new(backend.clone());

    if role.is_authority() {
        let dev_consensus = crate::builder::build_dev_nimbus_consensus(
            client.clone(),
            transaction_pool.clone(),
            &keystore_container,
            select_chain,
            &task_manager,
        )?;

        task_manager.spawn_essential_handle().spawn_blocking(
            "authorship_task",
            Some("block-authoring"),
            dev_consensus,
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
                command_sink: None,
            };

            rpc::create_calamari_full::<FullClient, TransactionPool>(deps)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client,
        transaction_pool,
        task_manager: &mut task_manager,
        config,
        keystore: keystore_container.keystore(),
        backend,
        network,
        system_rpc_tx,
        tx_handler_controller,
        telemetry: None,
        sync_service,
    })?;

    network_starter.start_network();

    Ok(task_manager)
}*/
