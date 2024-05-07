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

//! Fake Runtime API to use in place of native runtime

use frame_support::weights::Weight;
use manta_primitives::types::{AccountId, Balance, Block, CalamariAssetId, Nonce, PoolId};
use manta_support::manta_pay::{InitialSyncResponse, PullResponse, RawCheckpoint};
use session_key_primitives::NimbusId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    traits::Block as BlockT,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use zenlink_protocol::{AssetBalance, AssetId as ZenlinkAssetId, PairInfo};

pub struct Runtime;

sp_api::impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> sp_version::RuntimeVersion {
            unimplemented!()
        }

        fn execute_block(_: Block) {
            unimplemented!()
        }

        fn initialize_block(_: &<Block as BlockT>::Header) {
            unimplemented!()
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            unimplemented!()
        }

        fn metadata_at_version(_: u32) -> Option<OpaqueMetadata> {
            unimplemented!()
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            unimplemented!()
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            unimplemented!()
        }

        fn authorities() -> Vec<AuraId> {
            unimplemented!()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(_: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            unimplemented!()
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            unimplemented!()
        }

        fn inherent_extrinsics(_: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            unimplemented!()
        }

        fn check_inherents(_: Block, _: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
            unimplemented!()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            _: TransactionSource,
            _: <Block as BlockT>::Extrinsic,
            _: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            unimplemented!()
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(_: &<Block as BlockT>::Header) {
            unimplemented!()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(_: Option<Vec<u8>>) -> Vec<u8> {
            unimplemented!()
        }

        fn decode_session_keys(
            _: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            unimplemented!()
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            _: <Block as BlockT>::Extrinsic,
            _: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            unimplemented!()
        }
        fn query_fee_details(
            _: <Block as BlockT>::Extrinsic,
            _: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            unimplemented!()
        }
        fn query_weight_to_fee(_: Weight) -> Balance {
            unimplemented!()
        }
        fn query_length_to_fee(_: u32) -> Balance {
            unimplemented!()
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(_: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            unimplemented!()
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade(_: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            unimplemented!()
        }

        fn execute_block(
            _: Block,
            _: bool,
            _: bool,
            _: frame_try_runtime::TryStateSelect,
        ) -> Weight {
            unimplemented!()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(_: AccountId) -> Nonce {
            unimplemented!()
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(_: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            unimplemented!()
        }

        fn dispatch_benchmark(
            _: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            unimplemented!()
        }
    }

    impl pallet_lottery::runtime::LotteryApi<Block> for Runtime {
        fn not_in_drawing_freezeout(
        ) -> bool {
            unimplemented!();
        }
        fn current_prize_pool() -> u128 {
            unimplemented!()
        }
        fn next_drawing_at() -> Option<u128> {
            unimplemented!()
        }
    }

    impl pallet_manta_pay::runtime::PullLedgerDiffApi<Block> for Runtime {
        fn pull_ledger_diff(
            _checkpoint: RawCheckpoint,
            _max_receiver: u64,
            _max_sender: u64
        ) -> PullResponse {
            unimplemented!()
        }
        fn pull_ledger_total_count() -> [u8; 16] {
            unimplemented!()
        }
        fn initial_pull(_checkpoint: RawCheckpoint, _max_receiver: u64) -> InitialSyncResponse {
            unimplemented!()
        }
    }

    impl pallet_manta_sbt::runtime::SBTPullLedgerDiffApi<Block> for Runtime {
        fn sbt_pull_ledger_diff(
            _checkpoint: RawCheckpoint,
            _max_receiver: u64,
            _max_sender: u64
        ) -> PullResponse {
            unimplemented!()
        }
        fn sbt_pull_ledger_total_count() -> [u8; 16] {
            unimplemented!()
        }
    }

    impl nimbus_primitives::NimbusApi<Block> for Runtime {
        fn can_author(_author: NimbusId, _relay_parent: u32, _parent_header: &<Block as BlockT>::Header) -> bool {
            unimplemented!()
        }
    }

    // zenlink runtime outer apis
    impl zenlink_protocol_runtime_api::ZenlinkProtocolApi<Block, AccountId, ZenlinkAssetId> for Runtime {

        fn get_balance(
            _asset_id: ZenlinkAssetId,
            _owner: AccountId
        ) -> AssetBalance {
            unimplemented!()
        }

        fn get_pair_by_asset_id(
            _asset_0: ZenlinkAssetId,
            _asset_1: ZenlinkAssetId
        ) -> Option<PairInfo<AccountId, AssetBalance, ZenlinkAssetId>> {
            unimplemented!()
        }

        fn get_amount_in_price(
            _supply: AssetBalance,
            _path: Vec<ZenlinkAssetId>
        ) -> AssetBalance {
            unimplemented!()
        }

        fn get_amount_out_price(
            _supply: AssetBalance,
            _path: Vec<ZenlinkAssetId>
        ) -> AssetBalance {
            unimplemented!()
        }

        fn get_estimate_lptoken(
            _token_0: ZenlinkAssetId,
            _token_1: ZenlinkAssetId,
            _amount_0_desired: AssetBalance,
            _amount_1_desired: AssetBalance,
            _amount_0_min: AssetBalance,
            _amount_1_min: AssetBalance,
        ) -> AssetBalance{
            unimplemented!()
        }

        fn calculate_remove_liquidity(
            _asset_0: ZenlinkAssetId,
            _asset_1: ZenlinkAssetId,
            _amount: AssetBalance,
        ) -> Option<(AssetBalance, AssetBalance)> {
            unimplemented!()
        }
    }

    impl pallet_farming_rpc_runtime_api::FarmingRuntimeApi<Block, AccountId, CalamariAssetId, PoolId> for Runtime {
        fn get_farming_rewards(_who: AccountId, _pid: PoolId) -> Vec<(CalamariAssetId, Balance)> {
            unimplemented!()
        }

        fn get_gauge_rewards(_who: AccountId, _pid: PoolId) -> Vec<(CalamariAssetId, Balance)> {
            unimplemented!()
        }
    }
}
