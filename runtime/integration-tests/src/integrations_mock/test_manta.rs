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

#![cfg(feature = "manta")]

use super::*;
use frame_support::{
    traits::{PalletInfo, StorageInfo, StorageInfoTrait},
    StorageHasher, Twox128,
};

#[test]
fn verify_pallet_prefixes() {
    fn is_pallet_prefix<P: 'static>(name: &str) {
        // Compares the unhashed pallet prefix in the `StorageInstance` implementation by every
        // storage item in the pallet P. This pallet prefix is used in conjunction with the
        // item name to get the unique storage key: hash(PalletPrefix) + hash(StorageName)
        // https://github.com/paritytech/substrate/blob/master/frame/support/procedural/src/pallet/
        // expand/storage.rs#L389-L401
        assert_eq!(
            <Runtime as frame_system::Config>::PalletInfo::name::<P>(),
            Some(name)
        );
    }

    is_pallet_prefix::<manta_runtime::System>("System");
    is_pallet_prefix::<manta_runtime::ParachainSystem>("ParachainSystem");
    is_pallet_prefix::<manta_runtime::Timestamp>("Timestamp");
    is_pallet_prefix::<manta_runtime::ParachainInfo>("ParachainInfo");

    is_pallet_prefix::<manta_runtime::Balances>("Balances");
    is_pallet_prefix::<manta_runtime::TransactionPayment>("TransactionPayment");

    is_pallet_prefix::<manta_runtime::ParachainStaking>("ParachainStaking");

    is_pallet_prefix::<manta_runtime::AuthorInherent>("AuthorInherent");
    is_pallet_prefix::<manta_runtime::AuraAuthorFilter>("AuraAuthorFilter");

    is_pallet_prefix::<manta_runtime::Authorship>("Authorship");
    is_pallet_prefix::<manta_runtime::CollatorSelection>("CollatorSelection");
    is_pallet_prefix::<manta_runtime::Session>("Session");
    is_pallet_prefix::<manta_runtime::Aura>("Aura");

    is_pallet_prefix::<manta_runtime::Treasury>("Treasury");

    is_pallet_prefix::<manta_runtime::Preimage>("Preimage");

    is_pallet_prefix::<manta_runtime::XcmpQueue>("XcmpQueue");
    is_pallet_prefix::<manta_runtime::PolkadotXcm>("PolkadotXcm");
    is_pallet_prefix::<manta_runtime::CumulusXcm>("CumulusXcm");
    is_pallet_prefix::<manta_runtime::DmpQueue>("DmpQueue");
    is_pallet_prefix::<manta_runtime::XTokens>("XTokens");

    is_pallet_prefix::<manta_runtime::Utility>("Utility");
    is_pallet_prefix::<manta_runtime::Multisig>("Multisig");

    is_pallet_prefix::<manta_runtime::Sudo>("Sudo");

    is_pallet_prefix::<manta_runtime::Assets>("Assets");
    is_pallet_prefix::<manta_runtime::AssetManager>("AssetManager");

    let prefix = |pallet_name, storage_name| {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
        res[16..32].copy_from_slice(&Twox128::hash(storage_name));
        res.to_vec()
    };
    assert_eq!(
        <Timestamp as StorageInfoTrait>::storage_info(),
        vec![
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"Now".to_vec(),
                prefix: prefix(b"Timestamp", b"Now"),
                max_values: Some(1),
                max_size: Some(8),
            },
            StorageInfo {
                pallet_name: b"Timestamp".to_vec(),
                storage_name: b"DidUpdate".to_vec(),
                prefix: prefix(b"Timestamp", b"DidUpdate"),
                max_values: Some(1),
                max_size: Some(1),
            }
        ]
    );
    assert_eq!(
        <Balances as StorageInfoTrait>::storage_info(),
        vec![
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"TotalIssuance".to_vec(),
                prefix: prefix(b"Balances", b"TotalIssuance"),
                max_values: Some(1),
                max_size: Some(16),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"InactiveIssuance".to_vec(),
                prefix: prefix(b"Balances", b"InactiveIssuance"),
                max_values: Some(1),
                max_size: Some(16),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Account".to_vec(),
                prefix: prefix(b"Balances", b"Account"),
                max_values: None,
                max_size: Some(112),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Locks".to_vec(),
                prefix: prefix(b"Balances", b"Locks"),
                max_values: None,
                max_size: Some(1299),
            },
            StorageInfo {
                pallet_name: b"Balances".to_vec(),
                storage_name: b"Reserves".to_vec(),
                prefix: prefix(b"Balances", b"Reserves"),
                max_values: None,
                max_size: Some(1249),
            },
        ]
    );
}

#[test]
fn verify_manta_pallet_indices() {
    fn is_pallet_index<P: 'static>(index: usize) {
        assert_eq!(
            <manta_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
            Some(index)
        );
    }

    is_pallet_index::<manta_runtime::System>(0);
    is_pallet_index::<manta_runtime::ParachainSystem>(1);
    is_pallet_index::<manta_runtime::Timestamp>(2);
    is_pallet_index::<manta_runtime::ParachainInfo>(3);

    is_pallet_index::<manta_runtime::Balances>(10);
    is_pallet_index::<manta_runtime::TransactionPayment>(11);

    is_pallet_index::<manta_runtime::ParachainStaking>(48);

    is_pallet_index::<manta_runtime::AuthorInherent>(60);
    is_pallet_index::<manta_runtime::AuraAuthorFilter>(63);

    is_pallet_index::<manta_runtime::Authorship>(20);
    is_pallet_index::<manta_runtime::CollatorSelection>(21);
    is_pallet_index::<manta_runtime::Session>(22);
    is_pallet_index::<manta_runtime::Aura>(23);

    is_pallet_index::<manta_runtime::Treasury>(26);

    is_pallet_index::<manta_runtime::Preimage>(28);

    is_pallet_index::<manta_runtime::XcmpQueue>(30);
    is_pallet_index::<manta_runtime::PolkadotXcm>(31);
    is_pallet_index::<manta_runtime::CumulusXcm>(32);
    is_pallet_index::<manta_runtime::DmpQueue>(33);
    is_pallet_index::<manta_runtime::XTokens>(34);

    is_pallet_index::<manta_runtime::Utility>(40);
    is_pallet_index::<manta_runtime::Multisig>(41);

    is_pallet_index::<manta_runtime::Sudo>(42);

    is_pallet_index::<manta_runtime::Assets>(45);
    is_pallet_index::<manta_runtime::AssetManager>(46);
}
