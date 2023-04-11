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

//! Manta Local Dev Network Configurations

use super::*;
use session_key_primitives::util::unchecked_account_id;

pub fn genesis_spec_dev() -> MantaChainSpec {
    let genesis_collators: Vec<Collator> = vec![Collator::new(
        unchecked_account_id::<sr25519::Public>("Alice"),
        None,
        SessionKeys::from_seed_unchecked("Alice"),
    )];
    let genesis_collators_clone = genesis_collators.clone(); // so we can move it into the constructor closure

    MantaChainSpec::from_genesis(
        "Manta Parachain Dev",
        "manta",
        ChainType::Local,
        move || manta_devnet_genesis(genesis_collators_clone.clone()),
        genesis_collators
            .into_iter()
            .filter_map(|collator| collator.nodeid)
            .collect(),
        None,
        Some(MANTA_PROTOCOL_ID),
        None,
        Some(manta_properties()),
        Extensions {
            relay_chain: POLKADOT_RELAYCHAIN_LOCAL_NET.into(),
            para_id: MANTA_PARACHAIN_ID,
        },
    )
}

pub fn genesis_spec_local() -> MantaChainSpec {
    let genesis_collators: Vec<Collator> = vec![
        Collator::new(
            unchecked_account_id::<sr25519::Public>("Alice"),
            None,
            SessionKeys::from_seed_unchecked("Alice"),
        ),
        Collator::new(
            unchecked_account_id::<sr25519::Public>("Bob"),
            None,
            SessionKeys::from_seed_unchecked("Bob"),
        ),
        Collator::new(
            unchecked_account_id::<sr25519::Public>("Charlie"),
            None,
            SessionKeys::from_seed_unchecked("Charlie"),
        ),
        Collator::new(
            unchecked_account_id::<sr25519::Public>("Dave"),
            None,
            SessionKeys::from_seed_unchecked("Dave"),
        ),
        Collator::new(
            unchecked_account_id::<sr25519::Public>("Eve"),
            None,
            SessionKeys::from_seed_unchecked("Eve"),
        ),
    ];
    let genesis_collators_clone = genesis_collators.clone(); // so we can move it into the constructor closure

    MantaChainSpec::from_genesis(
        "Manta Parachain Local",
        "manta",
        ChainType::Local,
        move || manta_devnet_genesis(genesis_collators_clone.clone()),
        genesis_collators
            .into_iter()
            .filter_map(|collator| collator.nodeid)
            .collect(),
        None,
        Some(MANTA_PROTOCOL_ID),
        None,
        Some(manta_properties()),
        Extensions {
            relay_chain: POLKADOT_RELAYCHAIN_LOCAL_NET.into(),
            para_id: MANTA_PARACHAIN_ID,
        },
    )
}
