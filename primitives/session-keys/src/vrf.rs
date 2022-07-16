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

//! VRF Key type, which is sr25519
use crate::AuraId;
use sp_application_crypto::{sr25519, KeyTypeId, UncheckedFrom};
use sp_runtime::{BoundToRuntimeAppPublic, ConsensusEngineId};

/// Struct to implement `BoundToRuntimeAppPublic` by assigning Public = VrfId
pub struct VrfSessionKey;

impl BoundToRuntimeAppPublic for VrfSessionKey {
    type Public = VrfId;
}

/// Reinterprets Aura public key as a VRFId.
/// NO CORRESPONDING PRIVATE KEY TO THAT KEY WILL EXIST
pub fn dummy_key_from(aura_id: AuraId) -> VrfId {
    let aura_as_sr25519: sr25519::Public = aura_id.into();
    let sr25519_as_bytes: [u8; 32] = aura_as_sr25519.into();
    sr25519::Public::unchecked_from(sr25519_as_bytes).into()
}

/// The ConsensusEngineId for VRF keys
pub const VRF_ENGINE_ID: ConsensusEngineId = *b"rand";

/// The KeyTypeId used for VRF keys
pub const VRF_KEY_ID: KeyTypeId = KeyTypeId(VRF_ENGINE_ID);

// The strongly-typed crypto wrappers to be used by VRF in the keystore
mod vrf_crypto {
    use sp_application_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, crate::vrf::VRF_KEY_ID);
}

/// A vrf public key.
pub type VrfId = vrf_crypto::Public;

/// A vrf signature.
pub type VrfSignature = vrf_crypto::Signature;

sp_application_crypto::with_pair! {
    /// A vrf key pair
    pub type VrfPair = vrf_crypto::Pair;
}

#[test]
fn creating_dummy_vrf_id_from_aura_id_is_sane() {
    for x in 0u8..10u8 {
        let aura_id: AuraId = sr25519::Public::unchecked_from([x; 32]).into();
        let expected_vrf_id: VrfId = sr25519::Public::unchecked_from([x; 32]).into();
        let aura_to_vrf_id: VrfId = dummy_key_from(aura_id);
        assert_eq!(expected_vrf_id, aura_to_vrf_id);
    }
}
