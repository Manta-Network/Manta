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

use super::*;
use crate::Pallet as StorageBench;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{EventRecord, RawOrigin};

const SHARD_SIZE: usize = 256;
const MOCK_UTXO: Utxo = [42u8; UTXO_LENGTH];
const MOCK_ENCRYPTED_NOTE: EncryptedNote = EncryptedNote {
    ephemeral_public_key: [42u8; EPHEMERAL_PUBLIC_KEY_LENGTH],
    ciphertext: [42u8; CIPHER_TEXT_LENGTH],
};

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

fn set_up_shards_identity_identity_indices<T: Config>() {
    for shard_idx in 0..SHARD_SIZE {
        ShardsIdentityIdentityIndices::<T>::insert(shard_idx as u8, 0u64);
    }
}

fn set_up_shards_identity_identity<T: Config>(amount_per_shard: u64, lex_encoding: bool) {
    for shard_idx in 0..SHARD_SIZE {
        for _ in 0..amount_per_shard {
            let _ = StorageBench::<T>::do_insert_shard_element_identity_identity(
                shard_idx as u8,
                MOCK_UTXO.clone(),
                MOCK_ENCRYPTED_NOTE.clone(),
                lex_encoding,
            );
        }
    }
}

fn set_up_shards_twox_identity_indices<T: Config>() {
    for shard_idx in 0..SHARD_SIZE {
        ShardsTwoxIdentityIndices::<T>::insert(shard_idx as u8, 0u64);
    }
}

fn set_up_shards_twox_identity<T: Config>(amount_per_shard: u64, lex_encoding: bool) {
    for shard_idx in 0..SHARD_SIZE {
        for _ in 0..amount_per_shard {
            let _ = StorageBench::<T>::do_insert_shard_element_twox_identity(
                shard_idx as u8,
                MOCK_UTXO.clone(),
                MOCK_ENCRYPTED_NOTE.clone(),
                lex_encoding,
            );
        }
    }
}

fn set_up_shards_twox_twox_indices<T: Config>() {
    for shard_idx in 0..SHARD_SIZE {
        ShardsTwoxTwoxIndices::<T>::insert(shard_idx as u8, 0u64);
    }
}

fn set_up_shards_twox_twox<T: Config>(amount_per_shard: u64) {
    for shard_idx in 0..SHARD_SIZE {
        for _ in 0..amount_per_shard {
            let _ = StorageBench::<T>::do_insert_shard_element_twox_twox(
                shard_idx as u8,
                MOCK_UTXO.clone(),
                MOCK_ENCRYPTED_NOTE.clone(),
            );
        }
    }
}

benchmarks! {
    insert_shard_element_identity_identity {
        set_up_shards_identity_identity_indices::<T>();
        set_up_shards_identity_identity::<T>(20_000, false);
    }: insert_shard_element_identity_identity(RawOrigin::Root, 35, MOCK_UTXO.clone(), MOCK_ENCRYPTED_NOTE.clone(), false)
    verify {
        assert_last_event::<T>(
            Event::ShardElementWritten(MOCK_UTXO, MOCK_ENCRYPTED_NOTE).into()
        )
    }

    insert_shard_element_twox_identity {
        set_up_shards_twox_identity_indices::<T>();
        set_up_shards_twox_identity::<T>(20_000, true);
    }: insert_shard_element_twox_identity(RawOrigin::Root, 35, MOCK_UTXO.clone(), MOCK_ENCRYPTED_NOTE.clone(), true)
    verify {
        assert_last_event::<T>(
            Event::ShardElementWritten(MOCK_UTXO, MOCK_ENCRYPTED_NOTE).into()
        )
    }

    insert_shard_element_twox_twox {
        set_up_shards_twox_twox_indices::<T>();
        set_up_shards_twox_twox::<T>(20_000);
    }: insert_shard_element_twox_twox(RawOrigin::Root, 35, MOCK_UTXO.clone(), MOCK_ENCRYPTED_NOTE.clone())
    verify {
        assert_last_event::<T>(
            Event::ShardElementWritten(MOCK_UTXO, MOCK_ENCRYPTED_NOTE).into()
        )
    }

    point_read_shard_element_identity_identity {
        set_up_shards_identity_identity_indices::<T>();
        set_up_shards_identity_identity::<T>(20_000, false);
    }: point_read_shard_element_identity_identity(RawOrigin::Root, 35u8, 17_533u64, false)
    verify {}

    point_read_shard_element_twox_identity {
        set_up_shards_twox_identity_indices::<T>();
        set_up_shards_twox_identity::<T>(20_000, false);
    }: point_read_shard_element_twox_identity(RawOrigin::Root, 35u8, 17_533u64, false)
    verify {}

    point_read_shard_element_twox_twox {
        set_up_shards_twox_twox_indices::<T>();
        set_up_shards_twox_twox::<T>(20_000);
    }: point_read_shard_element_twox_twox(RawOrigin::Root, 35u8, 17_533u64)
    verify {}
}

impl_benchmark_test_suite!(StorageBench, crate::mock::new_test_ext(), crate::mock::Test,);
