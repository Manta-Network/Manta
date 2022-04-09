// Copyright 2020-2022 Manta Network.
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

#![allow(unused_imports)]
#![allow(dead_code)]
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use manta_primitives::{
	constants,
	types::{AccountId, AuraId, Balance, Signature},
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub mod calamari;
pub use self::calamari::*;
pub use calamari_runtime::currency::KMA;
pub mod manta;
pub use self::manta::*;
pub use manta_runtime::currency::MANTA;
pub mod dolphin;
pub use self::dolphin::*;
pub use dolphin_runtime::currency::DOL;

const CALAMARI_ENDOWMENT: Balance = 1_000_000_000 * KMA; // 10 endowment so that total supply is 10B

const DOLPHIN_ENDOWMENT: Balance = 1_000_000_000 * DOL; // 10 endowment so that total supply is 10B

const MANTA_ENDOWMENT: Balance = 100_000_000 * MANTA; // 10 endowment so that total supply is 1B

const STAGING_TELEMETRY_URL: &str = "wss://api.telemetry.manta.systems/submit/";

// A generic chain spec
pub type ChainSpec = sc_service::GenericChainSpec<manta_runtime::GenesisConfig, Extensions>;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
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
