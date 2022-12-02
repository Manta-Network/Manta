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

#![cfg(test)]

pub mod integrations_mock;
pub mod xcm_mock;

use sp_std::vec;
use xcm::{
    latest::{
        prelude::{
            All, Any, BuyExecution, ClearOrigin, Concrete, DepositAsset, DescendOrigin,
            InitiateReserveWithdraw, Limited, MultiAssets, ReserveAssetDeposited, Transact,
            TransferReserveAsset, Wild, WithdrawAsset,
        },
        OriginKind, Xcm,
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

pub(crate) const TO_PRIVATE: &[u8] = &[
    0, 1, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 4, 16, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 146, 139, 116, 182, 254, 69, 125, 31, 126, 240, 134, 211, 42, 169, 181,
    138, 238, 3, 47, 60, 187, 139, 235, 110, 144, 120, 176, 162, 167, 36, 115, 2, 208, 47, 207,
    230, 200, 47, 145, 154, 213, 10, 202, 216, 204, 26, 154, 229, 140, 178, 140, 161, 56, 169, 23,
    72, 167, 236, 39, 245, 41, 207, 105, 196, 13, 67, 109, 184, 57, 227, 57, 185, 231, 118, 57,
    199, 199, 68, 245, 175, 88, 222, 84, 119, 199, 233, 150, 182, 253, 246, 103, 236, 14, 28, 214,
    12, 8, 157, 7, 203, 202, 166, 62, 131, 162, 88, 70, 7, 2, 245, 139, 103, 115, 248, 219, 12, 4,
    48, 198, 100, 240, 78, 166, 94, 81, 201, 249, 66, 6, 90, 36, 238, 95, 238, 247, 51, 152, 35,
    40, 132, 139, 245, 134, 172, 114, 168, 14, 238, 245, 154, 238, 238, 105, 36, 111, 209, 28, 150,
    96, 82, 45, 138, 97, 190, 112, 52, 11, 183, 119, 58, 83, 59, 163, 123, 80, 203, 203, 99, 33,
    224, 131, 116, 159, 162, 179, 70, 164, 71, 62, 177, 168, 156, 40, 47, 207, 230, 200, 47, 145,
    154, 213, 10, 202, 216, 204, 26, 154, 229, 140, 178, 140, 161, 56, 169, 23, 72, 167, 236, 39,
    245, 41, 207, 105, 196, 13, 13, 240, 192, 184, 5, 225, 178, 133, 150, 199, 178, 242, 30, 238,
    98, 237, 0, 84, 175, 255, 122, 251, 6, 1, 32, 175, 171, 234, 72, 91, 221, 139, 12, 10, 163, 83,
    208, 49, 215, 221, 92, 106, 30, 154, 76, 175, 118, 188, 96, 117, 162, 191, 82, 218, 165, 68,
    20, 158, 219, 197, 25, 85, 58, 24, 212, 198, 60, 143, 23, 239, 239, 221, 6, 35, 227, 255, 146,
    95, 126, 89, 142, 31, 241, 139, 172, 177, 31, 159, 36, 210, 89, 169, 226, 107, 95, 1, 0, 2, 75,
    149, 120, 250, 35, 160, 163, 88, 88, 99, 252, 81, 244, 134, 31, 178, 151, 46, 92, 186, 69, 174,
    60, 174, 55, 47, 52, 157, 235, 173, 147, 57, 46, 35, 167, 37, 95, 160, 148, 228, 190, 200, 87,
    117, 45, 40, 98, 216, 104, 117, 132, 230, 135, 136, 67, 143, 91, 76, 42, 97, 162, 241, 13, 51,
    254, 43, 167, 160, 225, 26, 254, 136, 217, 235, 59, 166, 3, 25, 200, 229, 61, 107, 0, 6, 252,
    120, 44, 31, 70, 54, 62, 32, 186, 249, 167, 55, 16, 77, 24, 147, 244, 165, 23, 134, 154, 71,
    21, 5, 161, 160, 91, 125, 190, 110, 26, 230, 30, 251, 93, 132, 171, 81, 181, 73, 85, 88, 14,
];

pub use calamari_runtime::{Call, Runtime};
use codec::Decode;
use codec::Encode;
use frame_support::dispatch::GetDispatchInfo;
use pallet_manta_pay::types::TransferPost;
// Composition of self_reserve message composed by xTokens on the sender side
pub fn transact_message<T>() -> Xcm<T> {
    let remark = Call::System(frame_system::Call::<Runtime>::remark_with_event {
        remark: vec![1, 2, 3],
    });

    let call = Call::MantaPay(pallet_manta_pay::Call::<Runtime>::to_private {
        post: TransferPost::decode(&mut &*TO_PRIVATE).unwrap(),
    });
    let weight = call.get_dispatch_info().weight;
    println!("{}", weight);

    let weight = remark.get_dispatch_info().weight;
    println!("{}", weight);

    Xcm(vec![
        DescendOrigin(Here),
        WithdrawAsset(MultiAssets::from(vec![MultiAsset {
            id: Concrete(MultiLocation {
                parents: 1,
                interior: X1(Parachain(1)),
            }),
            fun: Fungible(10000000000000),
        }])),
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
        Transact {
            origin_type: OriginKind::SovereignAccount,
            require_weight_at_most: 100 as u64,
            call: remark.encode().into(),
        },
    ])
}
