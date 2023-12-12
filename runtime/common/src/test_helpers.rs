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

use crate::Weight;
use sp_std::vec;
use xcm::{
    latest::{
        prelude::{
            All, AllCounted, BuyExecution, ClearOrigin, Concrete, Definite, DepositAsset,
            DepositReserveAsset, InitiateReserveWithdraw, Limited, MultiAssets,
            ReserveAssetDeposited, TransferReserveAsset, Wild, WithdrawAsset, X1,
        },
        Xcm,
    },
    v3::{
        Fungibility::*,
        Junction::{AccountId32, Parachain},
        Junctions::*,
        MultiAsset, MultiLocation, Parent,
    },
};

// The weight input needs to be more than or equal to what the message actually weighs
// in that case the barrier will be substituted with the lower weight, otherwise it won't pass
pub const ADVERTISED_DEST_WEIGHT: Weight =
    Weight::from_parts(100_000_000_000u64, 100_000_000_000u64);

// Composition of self_reserve message composed by xTokens on the sender side
pub fn self_reserve_xcm_message_receiver_side<T>() -> Xcm<T> {
    let assets = MultiAssets::from(vec![MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X1(Parachain(1)),
        }),
        fun: Fungible(10000000000000),
    }]);
    Xcm(vec![
        ReserveAssetDeposited(assets.clone()),
        ClearOrigin,
        BuyExecution {
            fees: MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: X1(Parachain(1)),
                }),
                fun: Fungible(10000000000000),
            },
            weight_limit: Limited(Weight::from_ref_time(100000000000u64)),
        },
        DepositAsset {
            assets: Definite(assets),
            beneficiary: MultiLocation {
                parents: 0,
                interior: X1(AccountId32 {
                    network: None,
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
    let assets = MultiAssets::from(vec![MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X1(Parachain(1)),
        }),
        fun: Fungible(10000000000000),
    }]);
    Xcm(vec![
        WithdrawAsset(assets.clone()),
        ClearOrigin,
        BuyExecution {
            fees: MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: X1(Parachain(1)),
                }),
                fun: Fungible(10000000000000),
            },
            weight_limit: Limited(Weight::from_ref_time(100000000000u64)),
        },
        DepositAsset {
            assets: Definite(assets),
            beneficiary: MultiLocation {
                parents: 0,
                interior: X1(AccountId32 {
                    network: None,
                    id: [0; 32],
                }),
            },
        },
    ])
}

// Composition of to_reserve message composed by xTokens on the sender side
pub fn to_reserve_xcm_message_sender_side<T>() -> Xcm<T> {
    let assets = MultiAssets::from(vec![MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X1(Parachain(1)),
        }),
        fun: Fungible(10000000000000),
    }]);
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
            assets: Definite(assets),
            reserve: dummy_multi_location.clone(),
            xcm: Xcm(vec![
                BuyExecution {
                    fees: MultiAsset {
                        id: Concrete(dummy_multi_location),
                        fun: Fungible(10000000000000),
                    },
                    weight_limit: Limited(Weight::from_parts(30, 30)),
                },
                DepositAsset {
                    assets: Wild(All),
                    beneficiary: MultiLocation {
                        parents: 0,
                        interior: X1(AccountId32 {
                            network: None,
                            id: [0; 32],
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
                weight_limit: Limited(3999999999.into()),
            },
            DepositAsset {
                assets: Wild(All),
                beneficiary: MultiLocation {
                    parents: 0,
                    interior: X1(AccountId32 {
                        network: None,
                        id: [0; 32],
                    }),
                },
            },
        ]),
    }])
}

pub fn to_non_reserve_xcm_message_receiver_side<T>() -> Xcm<T> {
    let assets = MultiAssets::from(vec![MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X1(Parachain(1)),
        }),
        fun: Fungible(10000000000000),
    }]);
    Xcm(vec![
        WithdrawAsset(assets.clone()),
        ClearOrigin,
        BuyExecution {
            fees: MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: X1(Parachain(1)),
                }),
                fun: Fungible(10000000000000),
            },
            weight_limit: Limited(Weight::from_ref_time(100000000000u64)),
        },
        DepositReserveAsset {
            assets: AllCounted(1u32).into(),
            dest: MultiLocation {
                parents: 1,
                interior: X1(Parachain(1)),
            },
            xcm: Xcm(vec![
                BuyExecution {
                    fees: MultiAsset {
                        id: Concrete(MultiLocation {
                            parents: 1,
                            interior: X1(Parachain(1)),
                        }),
                        fun: Fungible(10000000000000),
                    },
                    weight_limit: Limited(Weight::from_ref_time(100000000000u64)),
                },
                DepositAsset {
                    assets: Definite(assets),
                    beneficiary: MultiLocation {
                        parents: 0,
                        interior: X1(AccountId32 {
                            network: None,
                            id: [0; 32],
                        }),
                    },
                },
            ]),
        },
    ])
}
