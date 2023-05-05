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

//! Manta Public Testnet Genesis Configuration

use super::*;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use sp_core::crypto::UncheckedInto;

pub fn genesis_spec() -> MantaChainSpec {
    let genesis_collators: Vec<Collator> = vec![
        Collator::new(
            hex!("0c9429df04f4c051d022a262d5c786d4cde9688bd230c139814f935f14709975").into(),
            Some("/dns/c1.calamari.seabird.systems/tcp/30433/p2p/12D3KooWGdvxcAc9KK4ihY4GhC6Mh9QEFiqfVMvHBKHs7HYACPMm".parse().unwrap()),
            SessionKeys::new((
                hex!("0c9429df04f4c051d022a262d5c786d4cde9688bd230c139814f935f14709975").unchecked_into(),
                hex!("0c9429df04f4c051d022a262d5c786d4cde9688bd230c139814f935f14709975").unchecked_into()
            ))
        ),
        Collator::new(
            hex!("969bf93aac86684a129d71ab97335a8f462b46516d916c9b62bbf3c1ac2d860e").into(),
            Some("/dns/c2.calamari.seabird.systems/tcp/30433/p2p/12D3KooWRC2JTv5UdGqesaCwP61CeohNmNQk6jBQUK2AB3PmSsNZ".parse().unwrap()),
            SessionKeys::new((
                hex!("969bf93aac86684a129d71ab97335a8f462b46516d916c9b62bbf3c1ac2d860e").unchecked_into(),
                hex!("969bf93aac86684a129d71ab97335a8f462b46516d916c9b62bbf3c1ac2d860e").unchecked_into()
            ))
        ),
        Collator::new(
            hex!("2234305637d7b6c529caf0169870b2319ece97d59bff1c2c1258dfbeffee9620").into(),
            Some("/dns/c3.calamari.seabird.systems/tcp/30433/p2p/12D3KooWBWE83sP71QNfhPJr6umSWRyV2rF3tG1ZtauMyEyLprC3".parse().unwrap()),
            SessionKeys::new((
                hex!("2234305637d7b6c529caf0169870b2319ece97d59bff1c2c1258dfbeffee9620").unchecked_into(),
                hex!("2234305637d7b6c529caf0169870b2319ece97d59bff1c2c1258dfbeffee9620").unchecked_into()
            ))
        ),
        Collator::new(
            hex!("543074b204c62b78c366486839dd0cd7d07f16f3d0226bbcc8a3b41d4a88887a").into(),
            Some("/dns/c4.calamari.seabird.systems/tcp/30433/p2p/12D3KooWCvAKZDGGd3B81QRhJh8a3TsNbojEtdH8WF2p8mFsVyv1".parse().unwrap()),
            SessionKeys::new((
                hex!("543074b204c62b78c366486839dd0cd7d07f16f3d0226bbcc8a3b41d4a88887a").unchecked_into(),
                hex!("543074b204c62b78c366486839dd0cd7d07f16f3d0226bbcc8a3b41d4a88887a").unchecked_into()
            ))
        ),
        Collator::new(
            hex!("acc4608e1c31f3f98ea1a0f3a8a74ade8aca396f269c26c1dd9019e812b11503").into(),
            Some("/dns/c5.calamari.seabird.systems/tcp/30433/p2p/12D3KooWQghBNgKanHtS4pCwBGeexw2QPvfgV15eMjr2YckVHCAD".parse().unwrap()),
            SessionKeys::new((
                hex!("acc4608e1c31f3f98ea1a0f3a8a74ade8aca396f269c26c1dd9019e812b11503").unchecked_into(),
                hex!("acc4608e1c31f3f98ea1a0f3a8a74ade8aca396f269c26c1dd9019e812b11503").unchecked_into()
            ))
        ),
    ];
    let genesis_collators_clone = genesis_collators.clone(); // so we can move it into the constructor closure

    MantaChainSpec::from_genesis(
        "Manta Parachain Staging",
        "manta",
        ChainType::Live,
        move || manta_devnet_genesis(genesis_collators_clone.clone()),
        genesis_collators
            .into_iter()
            .filter_map(|collator| collator.nodeid)
            .collect(),
        Some(
            TelemetryEndpoints::new(vec![(
                "/dns/api.telemetry.manta.systems/tcp/443/x-parity-wss/%2Fsubmit%2F".to_string(),
                0,
            )])
            .unwrap(),
        ),
        Some(MANTA_PROTOCOL_ID),
        None,
        Some(manta_properties()),
        Extensions {
            relay_chain: POLKADOT_STAGING_NET.into(),
            para_id: MANTA_PARACHAIN_ID,
        },
    )
}
