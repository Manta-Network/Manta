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

#![allow(clippy::unnecessary_cast)]

use core::marker::PhantomData;
#[allow(deprecated)]
use frame_support::migration::remove_storage_prefix;
use frame_support::{
    dispatch::GetStorageVersion,
    migration::{have_storage_value, put_storage_value, storage_key_iter},
    pallet_prelude::{StorageVersion, Weight},
    traits::{Get, OnRuntimeUpgrade},
    Blake2_128Concat, StorageHasher,
};
use sp_runtime::DispatchError;
use sp_runtime::{traits::ConstU32, WeakBoundedVec};
use sp_std::vec::Vec;
use xcm::{v3::*, VersionedMultiLocation};

use codec::{Decode, Encode, MaxEncodedLen};
use manta_primitives::types::CalamariAssetId;
use scale_info::TypeInfo;
use xcm_v1::*;

pub mod xcm_v1 {
    use super::*;

    pub mod v0 {
        use super::*;

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, scale_info::TypeInfo,
        )]
        pub enum MultiLocation {
            /// The interpreting consensus system.
            Null,
            /// A relative path comprising 1 junction.
            X1(Junction),
            /// A relative path comprising 2 junctions.
            X2(Junction, Junction),
            /// A relative path comprising 3 junctions.
            X3(Junction, Junction, Junction),
            /// A relative path comprising 4 junctions.
            X4(Junction, Junction, Junction, Junction),
            /// A relative path comprising 5 junctions.
            X5(Junction, Junction, Junction, Junction, Junction),
            /// A relative path comprising 6 junctions.
            X6(Junction, Junction, Junction, Junction, Junction, Junction),
            /// A relative path comprising 7 junctions.
            X7(
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
            ),
            /// A relative path comprising 8 junctions.
            X8(
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
            ),
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub enum Junction {
            /// The consensus system of which the context is a member and state-wise super-set.
            ///
            /// NOTE: This item is *not* a sub-consensus item: a consensus system may not identify itself trustlessly as
            /// a location that includes this junction.
            Parent,
            /// An indexed parachain belonging to and operated by the context.
            ///
            /// Generally used when the context is a Polkadot Relay-chain.
            Parachain(#[codec(compact)] u32),
            /// A 32-byte identifier for an account of a specific network that is respected as a sovereign endpoint within
            /// the context.
            ///
            /// Generally used when the context is a Substrate-based chain.
            AccountId32 { network: NetworkId, id: [u8; 32] },
            /// An 8-byte index for an account of a specific network that is respected as a sovereign endpoint within
            /// the context.
            ///
            /// May be used when the context is a Frame-based chain and includes e.g. an indices pallet.
            AccountIndex64 {
                network: NetworkId,
                #[codec(compact)]
                index: u64,
            },
            /// A 20-byte identifier for an account of a specific network that is respected as a sovereign endpoint within
            /// the context.
            ///
            /// May be used when the context is an Ethereum or Bitcoin chain or smart-contract.
            AccountKey20 { network: NetworkId, key: [u8; 20] },
            /// An instanced, indexed pallet that forms a constituent part of the context.
            ///
            /// Generally used when the context is a Frame-based chain.
            PalletInstance(u8),
            /// A non-descript index within the context location.
            ///
            /// Usage will vary widely owing to its generality.
            ///
            /// NOTE: Try to avoid using this and instead use a more specific item.
            GeneralIndex(#[codec(compact)] u128),
            /// A nondescript datum acting as a key within the context location.
            ///
            /// Usage will vary widely owing to its generality.
            ///
            /// NOTE: Try to avoid using this and instead use a more specific item.
            GeneralKey(WeakBoundedVec<u8, ConstU32<32>>),
            /// The unambiguous child.
            ///
            /// Not currently used except as a fallback when deriving ancestry.
            OnlyChild,
            /// A pluralistic body existing within consensus.
            ///
            /// Typical to be used to represent a governance origin of a chain, but could in principle be used to represent
            /// things such as multisigs also.
            Plurality { id: BodyId, part: BodyPart },
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub enum NetworkId {
            /// Unidentified/any.
            Any,
            /// Some named network.
            Named(WeakBoundedVec<u8, ConstU32<32>>),
            /// The Polkadot Relay chain
            Polkadot,
            /// Kusama.
            Kusama,
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub enum BodyId {
            /// The only body in its context.
            Unit,
            /// A named body.
            Named(WeakBoundedVec<u8, ConstU32<32>>),
            /// An indexed body.
            Index(#[codec(compact)] u32),
            /// The unambiguous executive body (for Polkadot, this would be the Polkadot council).
            Executive,
            /// The unambiguous technical body (for Polkadot, this would be the Technical Committee).
            Technical,
            /// The unambiguous legislative body (for Polkadot, this could be considered the opinion of a majority of
            /// lock-voters).
            Legislative,
            /// The unambiguous judicial body (this doesn't exist on Polkadot, but if it were to get a "grand oracle", it
            /// may be considered as that).
            Judicial,
            /// The unambiguous defense body (for Polkadot, an opinion on the topic given via a public referendum
            /// on the `staking_admin` track).
            Defense,
            /// The unambiguous administration body (for Polkadot, an opinion on the topic given via a public referendum
            /// on the `general_admin` track).
            Administration,
            /// The unambiguous treasury body (for Polkadot, an opinion on the topic given via a public referendum
            /// on the `treasurer` track).
            Treasury,
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub enum BodyPart {
            /// The body's declaration, under whatever means it decides.
            Voice,
            /// A given number of members of the body.
            Members {
                #[codec(compact)]
                count: u32,
            },
            /// A given number of members of the body, out of some larger caucus.
            Fraction {
                #[codec(compact)]
                nom: u32,
                #[codec(compact)]
                denom: u32,
            },
            /// No less than the given proportion of members of the body.
            AtLeastProportion {
                #[codec(compact)]
                nom: u32,
                #[codec(compact)]
                denom: u32,
            },
            /// More than than the given proportion of members of the body.
            MoreThanProportion {
                #[codec(compact)]
                nom: u32,
                #[codec(compact)]
                denom: u32,
            },
        }
    }

    pub mod v1 {
        use super::*;

        #[derive(
            Clone, Decode, Encode, Eq, PartialEq, Ord, PartialOrd, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub struct MultiLocation {
            /// The number of parent junctions at the beginning of this `MultiLocation`.
            pub parents: u8,
            /// The interior (i.e. non-parent) junctions that this `MultiLocation` contains.
            pub interior: Junctions,
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub enum Junctions {
            /// The interpreting consensus system.
            Here,
            /// A relative path comprising 1 junction.
            X1(Junction),
            /// A relative path comprising 2 junctions.
            X2(Junction, Junction),
            /// A relative path comprising 3 junctions.
            X3(Junction, Junction, Junction),
            /// A relative path comprising 4 junctions.
            X4(Junction, Junction, Junction, Junction),
            /// A relative path comprising 5 junctions.
            X5(Junction, Junction, Junction, Junction, Junction),
            /// A relative path comprising 6 junctions.
            X6(Junction, Junction, Junction, Junction, Junction, Junction),
            /// A relative path comprising 7 junctions.
            X7(
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
            ),
            /// A relative path comprising 8 junctions.
            X8(
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
                Junction,
            ),
        }

        #[derive(
            Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo, MaxEncodedLen,
        )]
        pub enum Junction {
            /// An indexed parachain belonging to and operated by the context.
            ///
            /// Generally used when the context is a Polkadot Relay-chain.
            Parachain(#[codec(compact)] u32),
            /// A 32-byte identifier for an account of a specific network that is respected as a sovereign endpoint within
            /// the context.
            ///
            /// Generally used when the context is a Substrate-based chain.
            AccountId32 {
                network: super::v0::NetworkId,
                id: [u8; 32],
            },
            /// An 8-byte index for an account of a specific network that is respected as a sovereign endpoint within
            /// the context.
            ///
            /// May be used when the context is a Frame-based chain and includes e.g. an indices pallet.
            AccountIndex64 {
                network: super::v0::NetworkId,
                #[codec(compact)]
                index: u64,
            },
            /// A 20-byte identifier for an account of a specific network that is respected as a sovereign endpoint within
            /// the context.
            ///
            /// May be used when the context is an Ethereum or Bitcoin chain or smart-contract.
            AccountKey20 {
                network: super::v0::NetworkId,
                key: [u8; 20],
            },
            /// An instanced, indexed pallet that forms a constituent part of the context.
            ///
            /// Generally used when the context is a Frame-based chain.
            PalletInstance(u8),
            /// A non-descript index within the context location.
            ///
            /// Usage will vary widely owing to its generality.
            ///
            /// NOTE: Try to avoid using this and instead use a more specific item.
            GeneralIndex(#[codec(compact)] u128),
            /// A nondescript datum acting as a key within the context location.
            ///
            /// Usage will vary widely owing to its generality.
            ///
            /// NOTE: Try to avoid using this and instead use a more specific item.
            GeneralKey(WeakBoundedVec<u8, ConstU32<32>>),
            /// The unambiguous child.
            ///
            /// Not currently used except as a fallback when deriving ancestry.
            OnlyChild,
            /// A pluralistic body existing within consensus.
            ///
            /// Typical to be used to represent a governance origin of a chain, but could in principle be used to represent
            /// things such as multisigs also.
            // v1 reuses v0's BodyId and BodyPart
            Plurality {
                id: super::v0::BodyId,
                part: super::v0::BodyPart,
            },
        }
    }

    #[derive(Encode, Decode, TypeInfo)]
    #[codec(encode_bound())]
    #[codec(decode_bound())]
    pub enum VersionedMultiLocation {
        V0(v0::MultiLocation),
        V1(v1::MultiLocation),
    }

    /// Asset Location
    #[derive(Decode, Encode, TypeInfo)]
    pub struct AssetLocation(pub VersionedMultiLocation);
}

pub struct XcmV1ToV3<T>(PhantomData<T>);
impl<T: frame_system::Config + pallet_asset_manager::Config> OnRuntimeUpgrade for XcmV1ToV3<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut writes = 0;
        let mut reads = 0;

        // AssetIdLocation
        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let asset_id_location: Vec<_> = storage_key_iter::<
            CalamariAssetId,
            AssetLocation,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();

        reads += asset_id_location.len();
        writes += asset_id_location.len();

        for (asset_id, v1_location) in asset_id_location {
            if let xcm_v1::AssetLocation(xcm_v1::VersionedMultiLocation::V1(v1::MultiLocation {
                parents,
                interior,
            })) = v1_location
            {
                match interior {
                    // we have two X1::Junction::Parachain: 2004 and 2084
                    // { Parachain }: 2
                    v1::Junctions::X1(v1::Junction::Parachain(para_id)) => {
                        let v3_junctions = Junctions::X1(Junction::Parachain(para_id));
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&asset_id.encode()),
                            v3_location,
                        );
                    }
                    // { Parachain, GeneralKey }: 16
                    v1::Junctions::X2(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::GeneralKey(general_key),
                    ) => {
                        let v3_general_key = {
                            let mut data = [0u8; 32];
                            data.copy_from_slice(&general_key[..]);
                            Junction::GeneralKey {
                                length: general_key.len() as u8,
                                data,
                            }
                        };
                        let v3_junctions =
                            Junctions::X2(Junction::Parachain(para_id), v3_general_key);
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&asset_id.encode()),
                            v3_location,
                        );
                    }
                    // { Parachain, PalletInstance }: 1
                    v1::Junctions::X2(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::PalletInstance(pallet_index),
                    ) => {
                        let v3_junctions = Junctions::X2(
                            Junction::Parachain(para_id),
                            Junction::PalletInstance(pallet_index),
                        );
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&asset_id.encode()),
                            v3_location,
                        );
                    }
                    // { Parachain, PalletInstance, GeneralIndex }: 1
                    v1::Junctions::X3(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::PalletInstance(pallet_index),
                        v1::Junction::GeneralIndex(general_index),
                    ) => {
                        let v3_junctions = Junctions::X3(
                            Junction::Parachain(para_id),
                            Junction::PalletInstance(pallet_index),
                            Junction::GeneralIndex(general_index),
                        );
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&asset_id.encode()),
                            v3_location,
                        );
                    }
                    _ => (),
                }
            }
        }
        log::info!(target: "OnRuntimeUpgrade", "✅ AssetManager's AssetIdLocation has been updated to xcm v3.");

        // LocationAssetId
        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let location_asset_id: Vec<_> = storage_key_iter::<
            AssetLocation,
            CalamariAssetId,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();

        reads += location_asset_id.len();
        writes += location_asset_id.len();

        for (v1_location, asset_id) in location_asset_id {
            if let xcm_v1::AssetLocation(xcm_v1::VersionedMultiLocation::V1(v1::MultiLocation {
                parents,
                interior,
            })) = v1_location
            {
                match interior {
                    // we have two X1::Junction::Parachain: 2004 and 2084
                    // { Parachain }: 2
                    v1::Junctions::X1(v1::Junction::Parachain(para_id)) => {
                        let v3_junctions = Junctions::X1(Junction::Parachain(para_id));
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            asset_id,
                        );
                    }
                    // { Parachain, GeneralKey }: 16
                    v1::Junctions::X2(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::GeneralKey(general_key),
                    ) => {
                        let v3_general_key = {
                            let mut data = [0u8; 32];
                            data.copy_from_slice(&general_key[..]);
                            Junction::GeneralKey {
                                length: general_key.len() as u8,
                                data,
                            }
                        };
                        let v3_junctions =
                            Junctions::X2(Junction::Parachain(para_id), v3_general_key);
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            asset_id,
                        );
                    }
                    // { Parachain, PalletInstance }: 1
                    v1::Junctions::X2(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::PalletInstance(pallet_index),
                    ) => {
                        let v3_junctions = Junctions::X2(
                            Junction::Parachain(para_id),
                            Junction::PalletInstance(pallet_index),
                        );
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            asset_id,
                        );
                    }
                    // { Parachain, PalletInstance, GeneralIndex }: 1
                    v1::Junctions::X3(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::PalletInstance(pallet_index),
                        v1::Junction::GeneralIndex(general_index),
                    ) => {
                        let v3_junctions = Junctions::X3(
                            Junction::Parachain(para_id),
                            Junction::PalletInstance(pallet_index),
                            Junction::GeneralIndex(general_index),
                        );
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            asset_id,
                        );
                    }
                    _ => (),
                }
            }
        }
        log::info!(target: "OnRuntimeUpgrade", "✅ AssetManager's LocationAssetId has been updated to xcm v3.");

        // MinXcmFee
        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"MinXcmFee";
        let min_xcm_fee: Vec<_> = storage_key_iter::<
            AssetLocation,
            CalamariAssetId,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();

        reads += min_xcm_fee.len();
        writes += min_xcm_fee.len();

        for (v1_location, fees) in min_xcm_fee {
            if let xcm_v1::AssetLocation(xcm_v1::VersionedMultiLocation::V1(v1::MultiLocation {
                parents,
                interior,
            })) = v1_location
            {
                match interior {
                    // we have two X1::Junction::Parachain: 2004 and 2084
                    // { Parachain }: 2
                    v1::Junctions::X1(v1::Junction::Parachain(para_id)) => {
                        let v3_junctions = Junctions::X1(Junction::Parachain(para_id));
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            fees,
                        );
                    }
                    // { Parachain, GeneralKey }: 16
                    v1::Junctions::X2(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::GeneralKey(general_key),
                    ) => {
                        let v3_general_key = {
                            let mut data = [0u8; 32];
                            data.copy_from_slice(&general_key[..]);
                            Junction::GeneralKey {
                                length: general_key.len() as u8,
                                data,
                            }
                        };
                        let v3_junctions =
                            Junctions::X2(Junction::Parachain(para_id), v3_general_key);
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            fees,
                        );
                    }
                    // { Parachain, PalletInstance }: 1
                    v1::Junctions::X2(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::PalletInstance(pallet_index),
                    ) => {
                        let v3_junctions = Junctions::X2(
                            Junction::Parachain(para_id),
                            Junction::PalletInstance(pallet_index),
                        );
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            fees,
                        );
                    }
                    // { Parachain, PalletInstance, GeneralIndex }: 1
                    v1::Junctions::X3(
                        v1::Junction::Parachain(para_id),
                        v1::Junction::PalletInstance(pallet_index),
                        v1::Junction::GeneralIndex(general_index),
                    ) => {
                        let v3_junctions = Junctions::X3(
                            Junction::Parachain(para_id),
                            Junction::PalletInstance(pallet_index),
                            Junction::GeneralIndex(general_index),
                        );
                        let v3_location =
                            VersionedMultiLocation::V3(MultiLocation::new(parents, v3_junctions));
                        put_storage_value(
                            pallet_prefix,
                            storage_item_prefix,
                            &Blake2_128Concat::hash(&v3_location.encode()),
                            fees,
                        );
                    }
                    _ => (),
                }
            }
        }
        log::info!(target: "OnRuntimeUpgrade", "✅ AssetManager's MinXcmFee has been updated to xcm v3.");

        // pallet version
        if crate::AssetManager::on_chain_storage_version() < 3 {
            StorageVersion::new(3).put::<pallet_asset_manager::Pallet<T>>();
            writes += 1;
            log::info!(target: "OnRuntimeUpgrade", "✅ AssetManager's version has been updated to 3.");
        }
        reads += 1;

        T::DbWeight::get()
            .reads(reads as u64)
            .saturating_add(T::DbWeight::get().writes(writes as u64))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        // AssetIdLocation
        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let asset_id_location: Vec<_> = storage_key_iter::<
            CalamariAssetId,
            AssetLocation,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        log::info!(target: "AssetIdLocation", "storage items count: {}.", asset_id_location.len());
        for (asset_id, v1_location) in asset_id_location {
            log::info!(target: "AssetIdLocation", "asset id: {asset_id}, location: {v1_location}.");
        }

        // LocationAssetId
        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let location_asset_id: Vec<_> = storage_key_iter::<
            AssetLocation,
            CalamariAssetId,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        log::info!(target: "LocationAssetId", "storage items count: {}.", location_asset_id.len());
        for (v1_location, asset_id) in location_asset_id {
            log::info!(target: "LocationAssetId", "location: {v1_location}, asset id: {asset_id}.");
        }

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"MinXcmFee";
        let min_xcm_fee: Vec<_> = storage_key_iter::<
            AssetLocation,
            CalamariAssetId,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        log::info!(target: "MinXcmFee", "storage items count: {}.", min_xcm_fee.len());
        for (v1_location, fees) in min_xcm_fee {
            log::info!(target: "MinXcmFee", "location: {v1_location}, min xcm fees: {fees}.");
        }

        let version = crate::AssetManager::on_chain_storage_version();
        log::info!(target: "StorageVersion", "version: {version}.");

        Ok(Vec::new())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), DispatchError> {
        use manta_primitives::assets::AssetLocation;

        if crate::AssetManager::on_chain_storage_version() != 3 {
            return Err(DispatchError::Other(
                "AssetManager storage version is not 3, the migration wasn't executed.",
            ));
        }

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"AssetIdLocation";
        let asset_id_location: Vec<_> = storage_key_iter::<
            CalamariAssetId,
            AssetLocation,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        log::info!(target: "AssetIdLocation", "storage items count: {}.", asset_id_location.len());
        for (index, (asset_id, v3_location)) in asset_id_location.iter().enumerate() {
            if let AssetLocation(VersionedMultiLocation::V3(MultiLocation { parents, interior })) =
                v3_location
            {
                log::info!(target: "AssetIdLocation", "storage item: {index} has been updated to xcm v3.");
            }
        }

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"LocationAssetId";
        let location_asset_id: Vec<_> = storage_key_iter::<
            AssetLocation,
            CalamariAssetId,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        log::info!(target: "LocationAssetId", "storage items count: {}.", location_asset_id.len());
        for (index, (v3_location, asset_id)) in location_asset_id.iter().enumerate() {
            if let AssetLocation(VersionedMultiLocation::V3(MultiLocation { parents, interior })) =
                v3_location
            {
                log::info!(target: "LocationAssetId", "storage item: {index} has been updated to xcm v3.");
            }
        }

        let pallet_prefix: &[u8] = b"AssetManager";
        let storage_item_prefix: &[u8] = b"MinXcmFee";
        let min_xcm_fee: Vec<_> = storage_key_iter::<
            AssetLocation,
            CalamariAssetId,
            Blake2_128Concat,
        >(pallet_prefix, storage_item_prefix)
        .drain()
        .collect();
        log::info!(target: "MinXcmFee", "storage items count: {}.", min_xcm_fee.len());
        for (index, (v3_location, asset_id)) in asset_id_location.iter().enumerate() {
            if let AssetLocation(VersionedMultiLocation::V3(MultiLocation { parents, interior })) =
                v3_location
            {
                log::info!(target: "MinXcmFee", "storage item: {index} has been updated to xcm v3.");
            }
        }

        Ok(())
    }
}
