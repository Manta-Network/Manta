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

//! Migration creates some helper function to make storage migration more convenient.

#![cfg_attr(not(feature = "std"), no_std)]

use manta_primitives::constants::RocksDbWeight;

use frame_support::{
    dispatch::Weight,
    migrations::migrate_from_pallet_version_to_storage_version,
    traits::{GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess},
};
#[cfg(feature = "try-runtime")]
use frame_support::{ensure, traits::StorageVersion};

use sp_std::marker::PhantomData;

/// MigratePalletPv2Sv means a wrapped handler to automatically upgrade our pallet
/// from PalletVersion(Pv) to StorageVersion(Sv).
///
/// It's actually a simple rewriting about storage flag: delete [pallet_name] + '__STORAGE_VERSION__' key
/// and reset [pallet_name] + '__PALLET_VERSION__' key.
/// So It's a one-time job, and should be removed soon to minimize runtime size.
pub struct MigratePalletPv2Sv<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for MigratePalletPv2Sv<T>
where
    T: GetStorageVersion + PalletInfoAccess,
{
    fn on_runtime_upgrade() -> Weight {
        let db_weight = RocksDbWeight::get();
        migrate_from_pallet_version_to_storage_version::<T>(&db_weight)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        let storage_version = StorageVersion::get::<T>();
        frame_support::debug(&"----PreUpgrade----");
        frame_support::debug(&T::module_name());
        frame_support::debug(&T::name());
        frame_support::debug(&storage_version);
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let storage_version = StorageVersion::get::<T>();
        frame_support::debug(&"----PostUpgrade----");
        frame_support::debug(&T::module_name());
        frame_support::debug(&T::name());
        frame_support::debug(&storage_version);
        ensure!(storage_version == StorageVersion::new(1), T::module_name());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use frame_support::traits::{CrateVersion, StorageInstance, StorageVersion};
    use sp_io::TestExternalities;

    pub struct DummyPrefix;

    impl StorageInstance for DummyPrefix {
        fn pallet_prefix() -> &'static str {
            "test_pv2sv"
        }

        const STORAGE_PREFIX: &'static str = "foo";
    }

    // just used for below migration test.
    // avoiding declare a huge Runtime part.
    struct MockForMigrationTesting {}

    impl GetStorageVersion for MockForMigrationTesting {
        fn current_storage_version() -> StorageVersion {
            StorageVersion::new(10)
        }

        fn on_chain_storage_version() -> StorageVersion {
            StorageVersion::get::<Self>()
        }
    }

    impl PalletInfoAccess for MockForMigrationTesting {
        fn index() -> usize {
            0
        }

        fn name() -> &'static str {
            "test_pv_2_sv"
        }

        fn module_name() -> &'static str {
            "test_module_name"
        }

        fn crate_version() -> CrateVersion {
            CrateVersion {
                major: 4,
                minor: 0,
                patch: 0,
            }
        }
    }

    #[test]
    fn test_pv_2_sv_works() {
        // 1. write old pallet version into storage.
        // 2. call utility
        // 3. test whether it works.
        const PALLET_VERSION_STORAGE_KEY_POSTFIX: &[u8] = b":__PALLET_VERSION__:";
        fn pallet_version_key(name: &str) -> [u8; 32] {
            frame_support::storage::storage_prefix(
                name.as_bytes(),
                PALLET_VERSION_STORAGE_KEY_POSTFIX,
            )
        }

        let mut db = TestExternalities::default();
        db.execute_with(|| {
            sp_io::storage::set(
                &pallet_version_key(MockForMigrationTesting::name()),
                &[1, 0, 0],
            );
            assert_eq!(
                MockForMigrationTesting::on_chain_storage_version(),
                StorageVersion::new(0)
            );
            let weight = MigratePalletPv2Sv::<MockForMigrationTesting>::on_runtime_upgrade();
            assert_eq!(100_000 * 1000 * 2, weight);
            assert!(
                sp_io::storage::get(&pallet_version_key(MockForMigrationTesting::name())).is_none()
            );
            assert_eq!(
                MockForMigrationTesting::on_chain_storage_version(),
                StorageVersion::new(10)
            );
        })
    }
}
