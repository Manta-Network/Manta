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

//! Helpers for handling nimbus keys
#![cfg_attr(not(feature = "std"), no_std)]

use crate::aura::AuraId;
use nimbus_primitives::NimbusId;
use sp_application_crypto::{sr25519, UncheckedFrom};

/// Clones an aura pubkey and represents it as a nimbus pubkey
/// Note: Not implementing From/Into because neither Nimbus nor Aura are our crates
pub fn from_aura_key(aura_id: AuraId) -> NimbusId {
    let aura_as_sr25519: sr25519::Public = aura_id.into();
    let sr25519_as_bytes: [u8; 32] = aura_as_sr25519.into();
    sr25519::Public::unchecked_from(sr25519_as_bytes).into()
}
