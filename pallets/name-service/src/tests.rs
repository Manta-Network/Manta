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
//
//! Unit tests for the Name Service.

#![cfg(test)]

use super::*;
use crate::mock::{new_test_ext, NameService, RuntimeOrigin as MockOrigin};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);

#[test]
fn register_should_work() {
    NameService::register(MockOrigin::signed(ALICE), "test".as_bytes().to_vec(), ALICE);
}
