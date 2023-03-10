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

#![cfg(test)]

pub mod integrations_mock;
pub mod xcm_mock;

use sp_std::vec;
use xcm::{
    latest::{
        prelude::{
            All, Any, BuyExecution, ClearOrigin, Concrete, DepositAsset, InitiateReserveWithdraw,
            Limited, MultiAssets, ReserveAssetDeposited, TransferReserveAsset, Wild, WithdrawAsset,
        },
        Xcm,
    },
    v1::{
        Fungibility::*,
        Junction::{AccountId32, Parachain},
        Junctions::*,
        MultiAsset, MultiLocation,
    },
};

// 4_000_000_000 is a typical configuration value provided to dApp developers for `dest_weight`
// argument when sending xcm message to Calamari. ie moonbeam, sub-wallet, phala, etc
pub const ADVERTISED_DEST_WEIGHT: u64 = 4_000_000_000;

// Composition of self_reserve message composed by xTokens on the sender side
pub fn self_reserve_xcm_message_receiver_side<T>() -> Xcm<T> {
    Xcm(vec![
        ReserveAssetDeposited(MultiAssets::from(vec![MultiAsset {
            id: Concrete(MultiLocation {
                parents: 1,
                interior: X1(Parachain(1)),
            }),
            fun: Fungible(10000000000000),
        }])),
        ClearOrigin,
        BuyExecution {
            fees: MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: X1(Parachain(1)),
                }),
                fun: Fungible(10000000000000),
            },
            weight_limit: Limited(3999999999),
        },
        DepositAsset {
            assets: Wild(All),
            max_assets: 1,
            beneficiary: MultiLocation {
                parents: 0,
                interior: X1(AccountId32 {
                    network: Any,
                    id: [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0,
                    ],
                }),
            },
        },
    ])
}

// Composition of to_reserve message composed by xTokens on the receiver side
pub fn to_reserve_xcm_message_receiver_side<T>() -> Xcm<T> {
    Xcm(vec![
        WithdrawAsset(MultiAssets::from(vec![MultiAsset {
            id: Concrete(MultiLocation {
                parents: 1,
                interior: X1(Parachain(1)),
            }),
            fun: Fungible(10000000000000),
        }])),
        ClearOrigin,
        BuyExecution {
            fees: MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: X1(Parachain(1)),
                }),
                fun: Fungible(10000000000000),
            },
            weight_limit: Limited(3999999999),
        },
        DepositAsset {
            assets: Wild(All),
            max_assets: 1,
            beneficiary: MultiLocation {
                parents: 0,
                interior: X1(AccountId32 {
                    network: Any,
                    id: [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0,
                    ],
                }),
            },
        },
    ])
}

// Composition of to_reserve message composed by xTokens on the sender side
pub fn to_reserve_xcm_message_sender_side<T>() -> Xcm<T> {
    let dummy_multi_location = MultiLocation {
        parents: 1,
        interior: X1(Parachain(1)),
    };
    let dummy_assets = MultiAssets::from(vec![MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X1(Parachain(1)),
        }),
        fun: Fungible(10000000000000),
    }]);
    Xcm(vec![
        WithdrawAsset(dummy_assets),
        InitiateReserveWithdraw {
            assets: Wild(All),
            reserve: dummy_multi_location.clone(),
            xcm: Xcm(vec![
                BuyExecution {
                    fees: MultiAsset {
                        id: Concrete(dummy_multi_location),
                        fun: Fungible(10000000000000),
                    },
                    weight_limit: Limited(3999999999),
                },
                DepositAsset {
                    assets: Wild(All),
                    max_assets: 1,
                    beneficiary: MultiLocation {
                        parents: 0,
                        interior: X1(AccountId32 {
                            network: Any,
                            id: [
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            ],
                        }),
                    },
                },
            ]),
        },
    ])
}

// Composition of self_reserve message composed by xTokens on the sender side
pub fn self_reserve_xcm_message_sender_side<T>() -> Xcm<T> {
    let dummy_multi_location = MultiLocation {
        parents: 1,
        interior: X1(Parachain(1)),
    };
    let dummy_assets = MultiAssets::from(vec![MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X1(Parachain(1)),
        }),
        fun: Fungible(10000000000000),
    }]);
    Xcm(vec![TransferReserveAsset {
        assets: dummy_assets,
        dest: dummy_multi_location.clone(),
        xcm: Xcm(vec![
            BuyExecution {
                fees: MultiAsset {
                    id: Concrete(dummy_multi_location),
                    fun: Fungible(10000000000000),
                },
                weight_limit: Limited(3999999999),
            },
            DepositAsset {
                assets: Wild(All),
                max_assets: 1,
                beneficiary: MultiLocation {
                    parents: 0,
                    interior: X1(AccountId32 {
                        network: Any,
                        id: [
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0,
                        ],
                    }),
                },
            },
        ]),
    }])
}
