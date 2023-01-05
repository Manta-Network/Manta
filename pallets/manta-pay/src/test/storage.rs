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

use frame_support::{
    pallet_prelude::ValueQuery, storage::types::StorageDoubleMap, traits::StorageInstance, Identity,
};
use sp_io::TestExternalities;

///
struct Prefix;

impl StorageInstance for Prefix {
    const STORAGE_PREFIX: &'static str = "foo";

    #[inline]
    fn pallet_prefix() -> &'static str {
        "test"
    }
}

///
type DoubleMap = StorageDoubleMap<Prefix, Identity, u16, Identity, u32, u64, ValueQuery>;

///
#[test]
fn double_map_iterator_test() {
    TestExternalities::default().execute_with(|| {
        for i in 0..4 {
            DoubleMap::insert(0_u16, i as u32, i as u64);
        }
        for i in 0..5 {
            DoubleMap::insert(1_u16, i as u32, i as u64)
        }
        assert_eq!(
            DoubleMap::iter().collect::<Vec<_>>(),
            vec![
                (0, 0, 0),
                (0, 1, 1),
                (0, 2, 2),
                (0, 3, 3),
                (1, 0, 0),
                (1, 1, 1),
                (1, 2, 2),
                (1, 3, 3),
                (1, 4, 4)
            ]
        );
        assert_eq!(
            DoubleMap::iter_prefix(0_u16).collect::<Vec<_>>(),
            vec![(0, 0), (1, 1), (2, 2), (3, 3)]
        );
        assert_eq!(
            DoubleMap::iter_prefix_values(1_u16).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(
            DoubleMap::iter_prefix_from(1, DoubleMap::hashed_key_for(1, 2)).collect::<Vec<_>>(),
            vec![(3, 3), (4, 4)]
        );
    })
}
