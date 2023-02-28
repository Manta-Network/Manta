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

use subxt::config::extrinsic_params::BaseExtrinsicParams;
use subxt::config::polkadot::PlainTip;
use subxt::config::{SubstrateConfig, WithExtrinsicParams};
use subxt::subxt;

pub mod utils;

pub use sp_core::*;
pub use sp_runtime::*;
pub use subxt::*;

/// Manta runtime APIs
#[subxt(runtime_metadata_path = "metadata/manta.scale")]
pub mod manta_runtime {}

// Calamari runtime APIs
#[subxt(runtime_metadata_path = "metadata/calamari.scale")]
pub mod calamari_runtime {}

// Dolphin runtime APIs
#[subxt(runtime_metadata_path = "metadata/dolphin.scale")]
pub mod dolphin_runtime {}

pub type MantaExtrinsicParams<T> = BaseExtrinsicParams<T, PlainTip>;
pub type MantaConfig = WithExtrinsicParams<SubstrateConfig, MantaExtrinsicParams<SubstrateConfig>>;
