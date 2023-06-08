// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.
use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

#[test]
fn set_babe_randomness_results_is_mandatory() {
    use frame_support::dispatch::{DispatchClass, GetDispatchInfo};

    let info = crate::Call::<Test>::set_babe_randomness_results {}.get_dispatch_info();
    assert_eq!(info.class, DispatchClass::Mandatory);
}
