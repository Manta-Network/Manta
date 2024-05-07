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
#![allow(clippy::too_many_arguments)]

pub use manta_primitives::types::{AccountId, Balance, Block, Hash, Header, Nonce};
use polkadot_service::CollatorPair;
use std::sync::Arc;

use sc_service::{Configuration, TaskManager};
use sc_telemetry::TelemetryWorkerHandle;

use cumulus_client_cli::{CollatorOptions, RelayChainMode};
use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
use cumulus_relay_chain_interface::{RelayChainInterface, RelayChainResult};
use cumulus_relay_chain_minimal_node::build_minimal_relay_chain_node_with_rpc;

/// build relaychain interface for parachain mode
pub async fn build_relay_chain_interface(
    polkadot_config: Configuration,
    parachain_config: &Configuration,
    telemetry_worker_handle: Option<TelemetryWorkerHandle>,
    task_manager: &mut TaskManager,
    collator_options: CollatorOptions,
) -> RelayChainResult<(
    Arc<(dyn RelayChainInterface + 'static)>,
    Option<CollatorPair>,
)> {
    if let RelayChainMode::ExternalRpc(rpc) = collator_options.relay_chain_mode {
        build_minimal_relay_chain_node_with_rpc(polkadot_config, task_manager, rpc).await
    } else {
        build_inprocess_relay_chain(
            polkadot_config,
            parachain_config,
            telemetry_worker_handle,
            task_manager,
            None,
        )
    }
}
