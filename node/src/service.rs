use cumulus_client_consensus_aura::{
	build_aura_consensus, BuildAuraConsensusParams, SlotProportion,
};
use cumulus_client_consensus_common::{
	ParachainBlockImport, ParachainCandidate, ParachainConsensus,
};
use cumulus_client_consensus_relay_chain::Verifier as RelayChainVerifier;
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::{
	relay_chain::v1::{Hash as PHash, PersistedValidationData},
	ParaId,
};
use manta_primitives::Header;
use polkadot_primitives::v0::CollatorPair;

use futures::lock::Mutex;
use sc_client_api::ExecutorProvider;
use sc_executor::native_executor_instance;
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::{ApiExt, ConstructRuntimeApi};
use sp_consensus::{
	import_queue::{BasicQueue, CacheKeyId, Verifier as VerifierT},
	BlockImportParams, BlockOrigin, SlotData,
};
use sp_consensus_aura::AuraApi;
use sp_runtime::{
	generic::{self, BlockId},
	traits::{BlakeTwo256, Header as HeaderT},
	OpaqueExtrinsic,
};
use std::sync::Arc;

pub use sc_executor::NativeExecutor;

pub type Block = generic::Block<Header, OpaqueExtrinsic>;

// Native Manta Parachain executor instance.
native_executor_instance!(
	pub MantaPCRuntimeExecutor,
	manta_pc_runtime::api::dispatch,
	manta_pc_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

enum BuildOnAccess<R> {
	Uninitialized(Option<Box<dyn FnOnce() -> R + Send + Sync>>),
	Initialized(R),
}

impl<R> BuildOnAccess<R> {
	fn get_mut(&mut self) -> &mut R {
		loop {
			match self {
				Self::Uninitialized(f) => {
					*self = Self::Initialized((f.take().unwrap())());
				}
				Self::Initialized(ref mut r) => return r,
			}
		}
	}
}

/// Special [`ParachainConsensus`] implementation that waits for the upgrade from
/// shell to a parachain runtime that implements Aura.
struct WaitForAuraConsensus<Client> {
	client: Arc<Client>,
	aura_consensus: Arc<Mutex<BuildOnAccess<Box<dyn ParachainConsensus<Block>>>>>,
}

impl<Client> Clone for WaitForAuraConsensus<Client> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			aura_consensus: self.aura_consensus.clone(),
		}
	}
}

#[async_trait::async_trait]
impl<Client> ParachainConsensus<Block> for WaitForAuraConsensus<Client>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
	async fn produce_candidate(
		&mut self,
		parent: &Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<Block>> {
		let block_id = BlockId::hash(parent.hash());
		if self
			.client
			.runtime_api()
			.has_api::<dyn AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_consensus
				.lock()
				.await
				.get_mut()
				.produce_candidate(parent, relay_parent, validation_data)
				.await
		} else {
			log::debug!("Waiting for runtime with AuRa api");
			None
		}
	}
}

struct Verifier<Client> {
	client: Arc<Client>,
	aura_verifier: BuildOnAccess<Box<dyn VerifierT<Block>>>,
	relay_chain_verifier: Box<dyn VerifierT<Block>>,
}

#[async_trait::async_trait]
impl<Client> VerifierT<Block> for Verifier<Client>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
	async fn verify(
		&mut self,
		origin: BlockOrigin,
		header: Header,
		justifications: Option<sp_runtime::Justifications>,
		body: Option<Vec<<Block as sp_runtime::traits::Block>::Extrinsic>>,
	) -> Result<
		(
			BlockImportParams<Block, ()>,
			Option<Vec<(CacheKeyId, Vec<u8>)>>,
		),
		String,
	> {
		let block_id = BlockId::hash(*header.parent_hash());

		if self
			.client
			.runtime_api()
			.has_api::<dyn AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_verifier
				.get_mut()
				.verify(origin, header, justifications, body)
				.await
		} else {
			self.relay_chain_verifier
				.verify(origin, header, justifications, body)
				.await
		}
	}
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor>(
	config: &Configuration,
) -> Result<
	PartialComponents<
		TFullClient<Block, RuntimeApi, Executor>,
		TFullBackend<Block>,
		(),
		sp_consensus::DefaultImportQueue<Block, TFullClient<Block, RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, TFullClient<Block, RuntimeApi, Executor>>,
		(Option<Telemetry>, Option<TelemetryWorkerHandle>),
	>,
	sc_service::Error,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, Executor>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<
			Block,
			StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
		> + sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
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

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let client2 = client.clone();
	let telemetry_handle = telemetry.as_ref().map(|telemetry| telemetry.handle());

	let aura_verifier = move || {
		let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

		Box::new(cumulus_client_consensus_aura::build_verifier::<
			sp_consensus_aura::sr25519::AuthorityPair,
			_,
			_,
			_,
		>(cumulus_client_consensus_aura::BuildVerifierParams {
			client: client2.clone(),
			create_inherent_data_providers: move |_, _| async move {
				let time = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
						*time,
						slot_duration.slot_duration(),
					);

				Ok((time, slot))
			},
			can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
				client2.executor().clone(),
			),
			telemetry: telemetry_handle,
		})) as Box<_>
	};

	let relay_chain_verifier = Box::new(RelayChainVerifier::new(client.clone(), |_, _| async {
		Ok(())
	})) as Box<_>;

	let verifier = Verifier {
		client: client.clone(),
		relay_chain_verifier,
		aura_verifier: BuildOnAccess::Uninitialized(Some(Box::new(aura_verifier))),
	};

	let registry = config.prometheus_registry();
	let spawner = task_manager.spawn_essential_handle();

	let import_queue = BasicQueue::new(
		verifier,
		Box::new(ParachainBlockImport::new(client.clone())),
		None,
		&spawner,
		registry,
	);

	let params = PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (telemetry, telemetry_worker_handle),
	};

	Ok(params)
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
pub async fn start_node<RuntimeApi, Executor, RB>(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: ParaId,
	rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, Executor>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<
			Block,
			StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
		> + sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
	RB: Fn(
			Arc<TFullClient<Block, RuntimeApi, Executor>>,
		) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
		+ Send
		+ 'static,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial::<RuntimeApi, Executor>(&parachain_config)?;
	let (mut telemetry, telemetry_worker_handle) = params.other;

	let relay_chain_full_node = cumulus_client_service::build_polkadot_full_node(
		polkadot_config,
		collator_key.clone(),
		telemetry_worker_handle,
	)
	.map_err(|e| match e {
		polkadot_service::Error::Sub(x) => x,
		s => format!("{}", s).into(),
	})?;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = build_block_announce_validator(
		relay_chain_full_node.client.clone(),
		id,
		Box::new(relay_chain_full_node.network.clone()),
		relay_chain_full_node.backend.clone(),
	);

	let force_authoring = parachain_config.force_authoring;
	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let mut task_manager = params.task_manager;
	let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);
	let (network, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: import_queue.clone(),
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
		})?;

	let rpc_client = client.clone();
	let rpc_extensions_builder = Box::new(move |_, _| rpc_ext_builder(rpc_client.clone()));

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		on_demand: None,
		remote_blockchain: None,
		rpc_extensions_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, data))
	};

	if validator {
		let client2 = client.clone();
		let relay_chain_backend = relay_chain_full_node.backend.clone();
		let relay_chain_client = relay_chain_full_node.client.clone();
		let keystore = params.keystore_container.sync_keystore();
		let spawn_handle = task_manager.spawn_handle();

		let parachain_consensus = Box::new(WaitForAuraConsensus {
			client: client2.clone(),
			aura_consensus: Arc::new(Mutex::new(BuildOnAccess::Uninitialized(Some(Box::new(
				move || {
					let slot_duration =
						cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

					let proposer_factory =
						sc_basic_authorship::ProposerFactory::with_proof_recording(
							spawn_handle,
							client2.clone(),
							transaction_pool,
							prometheus_registry.as_ref(),
							telemetry.as_ref().map(|t| t.handle()),
						);

					let relay_chain_backend2 = relay_chain_backend.clone();
					let relay_chain_client2 = relay_chain_client.clone();

					build_aura_consensus::<
						sp_consensus_aura::sr25519::AuthorityPair,
						_,
						_,
						_,
						_,
						_,
						_,
						_,
						_,
						_,
					>(BuildAuraConsensusParams {
						proposer_factory,
						create_inherent_data_providers:
							move |_, (relay_parent, validation_data)| {
								let parachain_inherent =
								cumulus_primitives_parachain_inherent::ParachainInherentData::create_at_with_client(
									relay_parent,
									&relay_chain_client,
									&*relay_chain_backend,
									&validation_data,
									id,
								);
								async move {
									let time =
										sp_timestamp::InherentDataProvider::from_system_time();

									let slot =
									sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
										*time,
										slot_duration.slot_duration(),
									);

									let parachain_inherent =
										parachain_inherent.ok_or_else(|| {
											Box::<dyn std::error::Error + Send + Sync>::from(
												"Failed to create parachain inherent",
											)
										})?;
									Ok((time, slot, parachain_inherent))
								}
							},
						block_import: client2.clone(),
						relay_chain_client: relay_chain_client2,
						relay_chain_backend: relay_chain_backend2,
						para_client: client2.clone(),
						backoff_authoring_blocks: Option::<()>::None,
						sync_oracle: network.clone(),
						keystore,
						force_authoring,
						slot_duration,
						// We got around 500ms for proposing
						block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
						telemetry: telemetry.map(|t| t.handle()),
					})
				},
			))))),
		});

		let spawner = task_manager.spawn_handle();

		let params = StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			collator_key,
			relay_chain_full_node,
			spawner,
			parachain_consensus,
			import_queue,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			para_id: id,
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			relay_chain_full_node,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}
