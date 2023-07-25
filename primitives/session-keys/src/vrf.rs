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

//! VRF Keys

use sp_application_crypto::KeyTypeId;
use sp_runtime::{BoundToRuntimeAppPublic, ConsensusEngineId};

/// Implementation of [`BoundToRuntimeAppPublic`] with the public key set to [`VrfId`]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VrfSessionKey;

impl BoundToRuntimeAppPublic for VrfSessionKey {
    type Public = VrfId;
}

/// Consensus Engine Identifier for the [`VrfId`] Key
pub const VRF_ENGINE_ID: ConsensusEngineId = *b"rand";

/// Key Type Identifier for the [`VrfId`] Key
pub const VRF_KEY_ID: KeyTypeId = KeyTypeId(VRF_ENGINE_ID);

/// The strongly-typed crypto wrappers to be used by VRF in the keystore.
mod vrf_crypto {
    use sp_application_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, crate::vrf::VRF_KEY_ID);
}

/// Public Key for VRF
pub type VrfId = vrf_crypto::Public;

/// Signature for the [`VrfId`] Key
pub type VrfSignature = vrf_crypto::Signature;

sp_application_crypto::with_pair! {
    /// Key Pair for the [`VrfId`] Key
    pub type VrfPair = vrf_crypto::Pair;
}
