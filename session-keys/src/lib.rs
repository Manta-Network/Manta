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

//! Primitives for session keys
#![cfg_attr(not(feature = "std"), no_std)]

pub mod aura;
pub mod nimbus;
mod nimbus_session_adapter;
pub mod vrf;
pub use vrf::*;

#[cfg(feature = "std")]
pub mod helpers;

pub mod v1 {
    use crate::aura::AuraId;
    sp_runtime::impl_opaque_keys! {
        pub struct SessionKeys<Runtime> {
            pub aura: pallet_aura::Pallet<Runtime>,
        }
    }
    impl<Runtime> SessionKeys<Runtime> {
        pub fn new<T, U>(tuple: (AuraId, T, U)) -> Self {
            let (aura, _, _) = tuple;
            Self { aura }
        }
    }
}
pub mod v2 {
    use crate::{
        aura::AuraId,
        nimbus_session_adapter::{AuthorInherentWithNoOpSession, VrfWithNoOpSession},
        vrf::VrfId,
    };
    use nimbus_primitives::NimbusId;

    sp_runtime::impl_opaque_keys! {
        pub struct SessionKeys<Runtime: frame_system::Config> {
            pub aura: pallet_aura::Pallet<Runtime>,
            pub nimbus: AuthorInherentWithNoOpSession<Runtime>,
            pub vrf: VrfWithNoOpSession,
        }
    }

    impl<Runtime: frame_system::Config> SessionKeys<Runtime> {
        pub fn new(tuple: (AuraId, NimbusId, VrfId)) -> Self {
            let (aura, nimbus, vrf) = tuple;
            Self { aura, nimbus, vrf }
        }
    }
}
pub use latest::SessionKeys;
use v2 as latest;

pub mod migrations {
    type OldSessionKeys<R> = crate::v1::SessionKeys<R>;
    type SessionKeys<R> = crate::v2::SessionKeys<R>;
    /// This function is fed into `upgrade_keys` to update Session pallet storage on RT upgrade
    pub fn transform_session_keys_v1_v2<R: frame_system::Config>(
        _v: manta_primitives::types::AccountId,
        old: OldSessionKeys<R>,
    ) -> SessionKeys<R> {
        let unique_dummy_nimbus_id = crate::nimbus::from_aura_key(old.aura.clone());
        SessionKeys {
            aura: old.aura,
            nimbus: unique_dummy_nimbus_id.clone(),
            vrf: unique_dummy_nimbus_id.into(),
        }
    }
}

/// A Trait to lookup keys from AuthorIds
pub trait KeysLookup<AuthorId, Keys> {
    fn lookup_keys(author: &AuthorId) -> Option<Keys>;
}

// A dummy impl used in simple tests
impl<AuthorId, Keys> KeysLookup<AuthorId, Keys> for () {
    fn lookup_keys(_: &AuthorId) -> Option<Keys> {
        None
    }
}
