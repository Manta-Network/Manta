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

//! Service builder
//! 
//! STANDALONE DEVNET CONFIGURATION
//! - Relay chain interface building disabled
//! - Node runs in dev mode only (no parachain collator)
//! - For parachain deployment, restore relay chain crates from git history

#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

pub use manta_primitives::types::{AccountId, Balance, Block, Hash, Header, Nonce};
use sp_core::sr25519::Pair as CollatorPair;
use std::sync::Arc;

use sc_service::{Configuration, TaskManager};
use sc_telemetry::TelemetryWorkerHandle;

use cumulus_client_cli::CollatorOptions;
use cumulus_relay_chain_interface::{RelayChainInterface, RelayChainResult};

// Phase 2: Relay chain building disabled for standalone devnet
// These crates pull polkadot-service → polkadot-runtime-common → pallet-identity errors
// use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
// use cumulus_relay_chain_minimal_node::build_minimal_relay_chain_node_with_rpc;

/// Stub relay chain interface for standalone devnet
/// Returns error since relay chain is not available in standalone mode
pub async fn build_relay_chain_interface(
    _polkadot_config: Configuration,
    _parachain_config: &Configuration,
    _telemetry_worker_handle: Option<TelemetryWorkerHandle>,
    _task_manager: &mut TaskManager,
    _collator_options: CollatorOptions,
) -> RelayChainResult<(
    Arc<(dyn RelayChainInterface + 'static)>,
    Option<CollatorPair>,
)> {
    // Phase 2: Relay chain disabled for standalone devnet
    // Node should use dev mode (start_dev_node) instead of parachain mode
    Err(cumulus_relay_chain_interface::RelayChainError::GenericError(
        "Relay chain interface disabled for standalone devnet. Use --dev mode.".to_string()
    ))
}

