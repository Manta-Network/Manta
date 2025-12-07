//! Runtime API implementations for the Chameleon solochain runtime.

use crate::{
    AccountId, Aura, Balance, Block, Executive, Grandpa, Nonce, Runtime, SessionKeys,
    System, TransactionPayment, VERSION,
};
use frame_support::weights::Weight;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::OpaqueMetadata;
use sp_runtime::{
    traits::Block as BlockT,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_version::RuntimeVersion;

/// Wasm binary unwrap helper.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    crate::WASM_BINARY.expect(
        "Development wasm binary is not available.",
    )
}

pub const RUNTIME_API_VERSIONS: sp_version::ApisVec = sp_version::create_apis_vec!([
    (sp_api::runtime_decl_for_core::ID, sp_api::runtime_decl_for_core::VERSION),
    (sp_api::runtime_decl_for_metadata::ID, sp_api::runtime_decl_for_metadata::VERSION),
    (sp_block_builder::runtime_decl_for_block_builder::ID, sp_block_builder::runtime_decl_for_block_builder::VERSION),
    (sp_transaction_pool::runtime_api::runtime_decl_for_tagged_transaction_queue::ID, sp_transaction_pool::runtime_api::runtime_decl_for_tagged_transaction_queue::VERSION),
    (sp_offchain::runtime_decl_for_offchain_worker_api::ID, sp_offchain::runtime_decl_for_offchain_worker_api::VERSION),
    (sp_session::runtime_decl_for_session_keys::ID, sp_session::runtime_decl_for_session_keys::VERSION),
    (sp_consensus_aura::runtime_decl_for_aura_api::ID, sp_consensus_aura::runtime_decl_for_aura_api::VERSION),
    (sp_consensus_grandpa::runtime_decl_for_grandpa_api::ID, sp_consensus_grandpa::runtime_decl_for_grandpa_api::VERSION),
    (frame_system_rpc_runtime_api::runtime_decl_for_account_nonce_api::ID, frame_system_rpc_runtime_api::runtime_decl_for_account_nonce_api::VERSION),
    (pallet_transaction_payment_rpc_runtime_api::runtime_decl_for_transaction_payment_api::ID, pallet_transaction_payment_rpc_runtime_api::runtime_decl_for_transaction_payment_api::VERSION),
    (sp_genesis_builder::runtime_decl_for_genesis_builder::ID, sp_genesis_builder::runtime_decl_for_genesis_builder::VERSION),
]);

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> alloc::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> alloc::vec::Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<alloc::vec::Vec<u8>>) -> alloc::vec::Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: alloc::vec::Vec<u8>,
        ) -> Option<alloc::vec::Vec<(alloc::vec::Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> alloc::vec::Vec<AuraId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: sp_consensus_grandpa::AuthorityId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }

        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }

        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }

        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: alloc::vec::Vec<u8>) -> sp_genesis_builder::Result {
            frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<alloc::vec::Vec<u8>> {
            frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, crate::genesis_config_presets::get_preset)
        }

        fn preset_names() -> alloc::vec::Vec<sp_genesis_builder::PresetId> {
            crate::genesis_config_presets::preset_names()
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl frame_benchmarking::Benchmark<Block> for Runtime {
    fn benchmark_metadata(_extra: bool) -> (
        alloc::vec::Vec<frame_benchmarking::BenchmarkList>,
        alloc::vec::Vec<frame_support::traits::StorageInfo>,
    ) {
        (alloc::vec::Vec::new(), alloc::vec::Vec::new())
    }

    fn dispatch_benchmark(
        _config: frame_benchmarking::BenchmarkConfig,
    ) -> Result<alloc::vec::Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
        Ok(alloc::vec::Vec::new())
    }
}

#[cfg(feature = "try-runtime")]
impl frame_try_runtime::TryRuntime<Block> for Runtime {
    fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
        let weight = Executive::try_runtime_upgrade(checks).unwrap();
        (weight, crate::configs::RuntimeBlockWeights::get().max_block)
    }

    fn execute_block(
        block: Block,
        state_root_check: bool,
        signature_check: bool,
        select: frame_try_runtime::TryStateSelect,
    ) -> Weight {
        Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
    }
}
