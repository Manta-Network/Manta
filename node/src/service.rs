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

//! Nimbus-based Parachain Node Service

use crate::{
    client::{RuntimeApiCommon, RuntimeApiNimbus},
    rpc,
};
use codec::Decode;
use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_common::ParachainConsensus;
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_relay_chain_interface::RelayChainInterface;
use futures::{channel::oneshot, FutureExt, StreamExt};
use jsonrpsee::RpcModule;
pub use manta_primitives::types::{AccountId, Balance, Block, Hash, Header, Index as Nonce};
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

const LOG_TARGET_SYNC: &str = "sync::cumulus";

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
pub type Client<RuntimeApi> = TFullClient<Block, RuntimeApi, DefaultExecutorType>;

/// Default Import Queue Type
pub type DefaultImportQueue<RuntimeApi> =
    sc_consensus::DefaultImportQueue<Block, Client<RuntimeApi>>;

/// Full Transaction Pool Type
pub type TransactionPool<RuntimeApi> = sc_transaction_pool::FullPool<Block, Client<RuntimeApi>>;

/// Components Needed for Chain Ops Subcommands
pub type PartialComponents<RuntimeApi> = sc_service::PartialComponents<
    Client<RuntimeApi>,
    TFullBackend<Block>,
    (),
    DefaultImportQueue<RuntimeApi>,
    TransactionPool<RuntimeApi>,
    (Option<Telemetry>, Option<TelemetryWorkerHandle>),
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
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCommon<StateBackend = StateBackend> + sp_consensus_aura::AuraApi<Block, AuraId>,
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

    let import_queue = crate::aura_or_nimbus_consensus::import_queue(
        // single step block import pipeline, after nimbus/aura seal, import block into client
        client.clone(),
        client.clone(),
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
        other: (telemetry, telemetry_worker_handle),
    })
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, BIC, FullRpc>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    full_rpc: FullRpc,
    build_consensus: BIC,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<Client<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon<StateBackend = StateBackend>
        + RuntimeApiNimbus
        + sp_consensus_aura::AuraApi<Block, AuraId>,
    FullRpc: Fn(
            rpc::FullDeps<Client<RuntimeApi>, TransactionPool<RuntimeApi>>,
        ) -> Result<RpcModule<()>, Error>
        + 'static,
    BIC: FnOnce(
        ParaId,
        Arc<Client<RuntimeApi>>,
        Arc<sc_client_db::Backend<Block>>,
        Option<&Registry>,
        Option<TelemetryHandle>,
        &TaskManager,
        Arc<dyn RelayChainInterface>,
        Arc<TransactionPool<RuntimeApi>>,
        Arc<NetworkService<Block, Hash>>,
        KeystorePtr,
        bool,
    ) -> Result<Box<dyn ParachainConsensus<Block>>, Error>,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi>(&parachain_config)?;
    let (mut telemetry, telemetry_worker_handle) = params.other;

    let mut task_manager = params.task_manager;
    let (relay_chain_interface, collator_key) = crate::builder::build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
        hwbench.clone(),
    )
    .await
    .map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

    let client = params.client.clone();
    let backend = params.backend.clone();
    let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);

    let force_authoring = parachain_config.force_authoring;
    let collator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue = params.import_queue.service();
    let net_config = sc_network::config::FullNetworkConfiguration::new(&parachain_config.network);

    let warp_sync_params = match parachain_config.network.sync_mode {
        SyncMode::Warp => {
            let target_block = warp_sync_get::<Block, _>(
                id,
                relay_chain_interface.clone(),
                task_manager.spawn_handle().clone(),
            );
            Some(WarpSyncParams::WaitForTarget(target_block))
        }
        _ => None,
    };

    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: params.import_queue,
            block_announce_validator_builder: Some(Box::new(|_| {
                Box::new(block_announce_validator)
            })),
            warp_sync_params,
            net_config,
        })?;

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
            };

            full_rpc(deps)
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
        .overseer_handle()
        .map_err(|e| sc_service::Error::Application(Box::new(e)))?;

    let relay_chain_slot_duration = core::time::Duration::from_secs(6);
    if collator {
        let parachain_consensus = build_consensus(
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
        )?;
        let spawner = task_manager.spawn_handle();
        start_collator(StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            relay_chain_interface,
            spawner,
            parachain_consensus,
            import_queue,
            collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
            relay_chain_slot_duration,
            recovery_handle: Box::new(overseer_handle),
            sync_service,
        })
        .await?;
    } else {
        start_full_node(StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            relay_chain_interface,
            relay_chain_slot_duration,
            import_queue,
            recovery_handle: Box::new(overseer_handle),
            sync_service,
        })?;
    }

    start_network.start_network();
    Ok((task_manager, client))
}

/// Creates a new background task to wait for the relay chain to sync up and retrieve the parachain header
fn warp_sync_get<B, RCInterface>(
    para_id: ParaId,
    relay_chain_interface: RCInterface,
    spawner: SpawnTaskHandle,
) -> oneshot::Receiver<<B as BlockT>::Header>
where
    B: BlockT + 'static,
    RCInterface: RelayChainInterface + 'static,
{
    let (sender, receiver) = oneshot::channel::<B::Header>();
    spawner.spawn(
        "cumulus-parachain-wait-for-target-block",
        None,
        async move {
            log::debug!(
                target: "calamari-network",
                "waiting for announce block in a background task...",
            );

            let _ = wait_for_target_block::<B, _>(sender, para_id, relay_chain_interface)
                .await
                .map_err(|e| {
                    log::error!(
                        target: LOG_TARGET_SYNC,
                        "Unable to determine parachain target block {:?}",
                        e
                    )
                });
        }
        .boxed(),
    );

    receiver
}

/// Waits for the relay chain to have finished syncing and then gets the parachain header that corresponds to the last finalized relay chain block.
async fn wait_for_target_block<B, RCInterface>(
    sender: oneshot::Sender<<B as BlockT>::Header>,
    para_id: ParaId,
    relay_chain_interface: RCInterface,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT + 'static,
    RCInterface: RelayChainInterface + Send + 'static,
{
    let mut imported_blocks = relay_chain_interface
        .import_notification_stream()
        .await?
        .fuse();
    while imported_blocks.next().await.is_some() {
        let is_syncing = relay_chain_interface
            .is_major_syncing()
            .await
            .map_err(|e| {
                Box::<dyn std::error::Error + Send + Sync>::from(format!(
                    "Unable to determine sync status. {e}"
                ))
            })?;

        if !is_syncing {
            let relay_chain_best_hash = relay_chain_interface
                .finalized_block_hash()
                .await
                .map_err(|e| Box::new(e) as Box<_>)?;

            let validation_data = relay_chain_interface
                .persisted_validation_data(
                    relay_chain_best_hash,
                    para_id,
                    polkadot_primitives::OccupiedCoreAssumption::TimedOut,
                )
                .await
                .map_err(|e| format!("{e:?}"))?
                .ok_or("Could not find parachain head in relay chain")?;

            let target_block = B::Header::decode(&mut &validation_data.parent_head.0[..])
                .map_err(|e| format!("Failed to decode parachain head: {e}"))?;

            log::debug!(target: LOG_TARGET_SYNC, "Target block reached {:?}", target_block);
            let _ = sender.send(target_block);
            return Ok(());
        }
    }

    Err("Stopping following imported blocks. Could not determine parachain target block".into())
}

/// Start a calamari parachain node.
pub async fn start_parachain_node<RuntimeApi, FullRpc>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
    full_rpc: FullRpc,
) -> sc_service::error::Result<(TaskManager, Arc<Client<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon<StateBackend = StateBackend>
        + RuntimeApiNimbus
        + sp_consensus_aura::AuraApi<Block, AuraId>,
    FullRpc: Fn(
            rpc::FullDeps<Client<RuntimeApi>, TransactionPool<RuntimeApi>>,
        ) -> Result<RpcModule<()>, Error>
        + 'static,
{
    start_node_impl::<RuntimeApi, _, _>(
        parachain_config,
        polkadot_config,
        collator_options,
        id,
        full_rpc,
        crate::builder::build_nimbus_consensus,
        hwbench,
    )
    .await
}

/// Start a dev node using nimbus instant-sealing consensus without relaychain attached.
pub async fn start_dev_nimbus_node<RuntimeApi, FullRpc>(
    config: Configuration,
    full_rpc: FullRpc,
) -> sc_service::error::Result<TaskManager>
where
    RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCommon<StateBackend = StateBackend>
        + RuntimeApiNimbus
        + sp_consensus_aura::AuraApi<Block, AuraId>,
    FullRpc: Fn(
            rpc::FullDeps<Client<RuntimeApi>, TransactionPool<RuntimeApi>>,
        ) -> Result<RpcModule<()>, Error>
        + 'static,
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
    } = new_partial::<RuntimeApi>(&config)?;

    let net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: None,
            net_config,
        })?;

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
            };

            full_rpc(deps)
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
        sync_service: sync_service.clone(),
    })?;

    network_starter.start_network();

    Ok(task_manager)
}
