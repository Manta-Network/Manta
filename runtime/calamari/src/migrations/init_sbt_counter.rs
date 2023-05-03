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

use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use sp_runtime::traits::PhantomData;

pub struct InitializeSbtCounter<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for InitializeSbtCounter<T>
where
    T: pallet_manta_sbt::Config,
    <T as frame_system::Config>::AccountId: From<[u8; 32]>,
    <T as frame_system::Config>::AccountId: Into<[u8; 32]>,
{
    fn on_runtime_upgrade() -> Weight {
        let storage_version = pallet_manta_sbt::Pallet::<T>::on_chain_storage_version();

        if storage_version == 0 {
            // set next mint id to three
            pallet_manta_sbt::Pallet::<T>::set_next_mint_id(3);
            // set storage version to 1, prevents double migration
            StorageVersion::new(1).put::<pallet_manta_sbt::Pallet<T>>();
        }
        T::BlockWeights::get().max_block // simply use the whole block
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        let storage_version = pallet_manta_sbt::Pallet::<T>::on_chain_storage_version();
        if storage_version == 0 {
            Ok(())
        } else {
            Err("Storage version has already been migrated")
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let storage_version = pallet_manta_sbt::Pallet::<T>::on_chain_storage_version();
        if storage_version == 1 {
            Ok(())
        } else {
            Err("Migration failed to execute")
        }
    }
}
