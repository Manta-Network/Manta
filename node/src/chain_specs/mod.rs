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

//! Chain Specification Definitions

// NOTE: Tolerate clippy warning originating in ChainSpecGroup, which is a dependency.
#![allow(clippy::derive_partial_eq_without_eq)]
// NOTE: Missing documentation on all `ChainSpecGroup` implementations.
#![allow(missing_docs)]

use manta_primitives::{
    constants,
    types::{AccountId, Balance},
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::sr25519;

pub mod calamari;
pub mod manta;

pub use self::{calamari::*, manta::*};
pub use calamari_runtime::currency::KMA;
pub use manta_runtime::currency::MANTA;

/// Calamari Endowment: 10 endowment so that total supply is 10B
pub const CALAMARI_ENDOWMENT: Balance = 1_000_000_000 * KMA;

/// Manta Endowment: 10 endowment so that total supply is 1B
pub const MANTA_ENDOWMENT: Balance = 100_000_000 * MANTA;

/// Staging Telemetry URL
pub const STAGING_TELEMETRY_URL: &str = "wss://api.telemetry.manta.systems/submit/";

/// Manta Network Chain Spec
pub type ChainSpec = sc_service::GenericChainSpec<manta_runtime::GenesisConfig, Extensions>;

/// The extensions for the [`ChainSpec`].
#[derive(
    ChainSpecExtension, ChainSpecGroup, Clone, Debug, Deserialize, Eq, PartialEq, Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}
