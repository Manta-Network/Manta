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

use crate::{BalanceOf, Config, Error, Pallet, RandomnessResults, RelayEpoch};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
use sp_std::vec::Vec;

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Shared request info, a subset of `RequestInfo`
pub enum RequestType {
	/// Babe one epoch ago
	BabeEpoch(u64),
}

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Type of request
/// Represents a request for the most recent randomness at or after the inner first field
/// Expiration is second inner field
pub enum RequestInfo {
	/// Babe one epoch ago
	BabeEpoch(u64, u64),
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Raw randomness snapshot, the unique value for a `RequestType` in `RandomnessResults` map
pub struct RandomnessResult<Hash> {
	/// Randomness once available
	pub randomness: Option<Hash>,
	/// Number of randomness requests for the type
	pub request_count: u64,
}

impl<Hash: Clone> RandomnessResult<Hash> {
	pub fn new() -> RandomnessResult<Hash> {
		RandomnessResult {
			randomness: None,
			request_count: 1u64,
		}
	}
}
