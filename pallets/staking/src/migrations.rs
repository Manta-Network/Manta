// Copyright 2020-2021 Manta Network.
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

//! Storage migrations for the Staking pallet.

use super::*;

pub mod v7 {
	use super::*;

	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		assert!(CounterForValidators::<T>::get().is_zero(), "CounterForValidators already set.");
		assert!(CounterForNominators::<T>::get().is_zero(), "CounterForNominators already set.");
		assert!(StorageVersion::<T>::get() == Releases::V6_0_0);
		Ok(())
	}

	pub fn migrate<T: Config>() -> Weight {
		log!(info, "Migrating staking to Releases::V7_0_0");
		let validator_count = Validators::<T>::iter().count() as u32;
		let nominator_count = Nominators::<T>::iter().count() as u32;

		CounterForValidators::<T>::put(validator_count);
		CounterForNominators::<T>::put(nominator_count);

		StorageVersion::<T>::put(Releases::V7_0_0);
		log!(info, "Completed staking migration to Releases::V7_0_0");

		T::DbWeight::get().reads_writes(validator_count.saturating_add(nominator_count).into(), 2)
	}
}

pub mod v6 {
	use super::*;
	use frame_support::{generate_storage_alias, traits::Get, weights::Weight};

	// NOTE: value type doesn't matter, we just set it to () here.
	generate_storage_alias!(Staking, SnapshotValidators => Value<()>);
	generate_storage_alias!(Staking, SnapshotNominators => Value<()>);
	generate_storage_alias!(Staking, QueuedElected => Value<()>);
	generate_storage_alias!(Staking, QueuedScore => Value<()>);
	generate_storage_alias!(Staking, EraElectionStatus => Value<()>);
	generate_storage_alias!(Staking, IsCurrentSessionFinal => Value<()>);

	/// check to execute prior to migration.
	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		// these may or may not exist.
		log!(info, "SnapshotValidators.exits()? {:?}", SnapshotValidators::exists());
		log!(info, "SnapshotNominators.exits()? {:?}", SnapshotNominators::exists());
		log!(info, "QueuedElected.exits()? {:?}", QueuedElected::exists());
		log!(info, "QueuedScore.exits()? {:?}", QueuedScore::exists());
		// these must exist.
		assert!(IsCurrentSessionFinal::exists(), "IsCurrentSessionFinal storage item not found!");
		assert!(EraElectionStatus::exists(), "EraElectionStatus storage item not found!");
		Ok(())
	}

	/// Migrate storage to v6.
	pub fn migrate<T: Config>() -> Weight {
		log!(info, "Migrating staking to Releases::V6_0_0");

		SnapshotValidators::kill();
		SnapshotNominators::kill();
		QueuedElected::kill();
		QueuedScore::kill();
		EraElectionStatus::kill();
		IsCurrentSessionFinal::kill();

		StorageVersion::<T>::put(Releases::V6_0_0);
		log!(info, "Done.");
		T::DbWeight::get().writes(6 + 1)
	}
}
