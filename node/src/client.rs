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

//! RuntimeApi for client

use async_backing_primitives::UnincludedSegmentApi;
use manta_primitives::types::{AccountId, Balance, Block, Nonce};
use nimbus_primitives::{DigestsProvider, NimbusApi, NimbusId};
use session_keys_primitives::VrfApi;
use sp_core::H256;
use sp_keystore::{Keystore, KeystorePtr};
use std::sync::Arc;

/// RuntimeApiCommon + RuntimeApiNimbus: nimbus
///
/// Common RuntimeApi trait bound
pub trait RuntimeApiCommon:
    sp_api::Metadata<Block>
    + sp_api::ApiExt<Block>
    + sp_block_builder::BlockBuilder<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
    + frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
    + cumulus_primitives_core::CollectCollationInfo<Block>
    + VrfApi<Block>
    + NimbusApi<Block>
    + UnincludedSegmentApi<Block>
{
}

impl<Api> RuntimeApiCommon for Api where
    Api: sp_api::Metadata<Block>
        + sp_api::ApiExt<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
        + frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
        + cumulus_primitives_core::CollectCollationInfo<Block>
        + VrfApi<Block>
        + NimbusApi<Block>
        + UnincludedSegmentApi<Block>
{
}

/// Uses the runtime API to get the VRF inputs and sign them with the VRF key that
/// corresponds to the authoring NimbusId.
pub fn vrf_pre_digest<B, C>(
    _client: &C,
    _keystore: &KeystorePtr,
    _nimbus_id: NimbusId,
    _parent: H256,
) -> Option<sp_runtime::generic::DigestItem>
where
    B: sp_runtime::traits::Block<Hash = sp_core::H256>,
    C: sp_api::ProvideRuntimeApi<B>,
    C::Api: VrfApi<B>,
{
    None
}

/// Implementation for Vrf Digest call, our runtimes will just return None
pub struct VrfDigestsProvider<B, C> {
    /// client
    pub client: Arc<C>,
    /// keystore
    pub keystore: Arc<dyn Keystore>,
    _marker: std::marker::PhantomData<B>,
}

impl<B, C> VrfDigestsProvider<B, C> {
    /// New instance of `VrfDigestsProvider`
    pub fn new(client: Arc<C>, keystore: Arc<dyn Keystore>) -> Self {
        Self {
            client,
            keystore,
            _marker: Default::default(),
        }
    }
}

impl<B, C> DigestsProvider<NimbusId, H256> for VrfDigestsProvider<B, C>
where
    B: sp_runtime::traits::Block<Hash = sp_core::H256>,
    C: sp_api::ProvideRuntimeApi<B>,
    C::Api: VrfApi<B>,
{
    type Digests = Option<sp_runtime::generic::DigestItem>;

    // vrf is not used
    fn provide_digests(&self, _nimbus_id: NimbusId, _parent: H256) -> Self::Digests {
        None
    }
}
