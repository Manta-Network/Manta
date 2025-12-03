// Copyright 2020-2024 Manta Network.
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

//! Types for Chameleon Staking

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{Perbill, RuntimeDebug};

/// Validator information
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct ValidatorInfo<AccountId, Balance> {
    /// Validator controller account
    pub controller: AccountId,
    /// Validator stash account
    pub stash: AccountId,
    /// Self-bonded amount
    pub self_stake: Balance,
    /// Total stake including delegations
    pub total_stake: Balance,
    /// Number of delegators
    pub delegator_count: u32,
    /// Commission rate (percentage of delegator rewards)
    pub commission: Perbill,
    /// Status
    pub status: ValidatorStatus,
    /// Performance metrics
    pub performance: ValidatorPerformance,
}

/// Delegation record
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct Delegation<AccountId, Balance> {
    pub delegator: AccountId,
    pub validator: AccountId,
    pub amount: Balance,
    pub delegated_at: u32, // Block number
}

/// Validator status
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum ValidatorStatus {
    Active,
    Waiting,
    Unbonding { unlock_at: u32 },
    Slashed,
}

/// Validator performance metrics
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct ValidatorPerformance {
    pub uptime_percent: Perbill,
    pub blocks_produced: u32,
    pub blocks_missed: u32,
    pub last_active_era: u32,
}

/// Unbonding request
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UnbondingRequest<Balance, BlockNumber> {
    pub amount: Balance,
    pub unlock_at: BlockNumber,
}

/// Slashing offense types
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum SlashingOffense {
    /// Extended downtime (>12 hours)
    ExtendedDowntime,
    /// Double signing
    DoubleSigning,
}

/// Era reward information
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct EraReward<Balance> {
    pub era: u32,
    pub total_reward: Balance,
    pub validator_reward: Balance,
    pub delegator_reward: Balance,
}

/// Validator reward calculation result
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct ValidatorReward<AccountId, Balance> {
    pub validator: AccountId,
    pub self_reward: Balance,
    pub commission: Balance,
    pub delegator_pool: Balance,
}

/// Delegator reward calculation result
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct DelegatorReward<AccountId, Balance> {
    pub delegator: AccountId,
    pub validator: AccountId,
    pub reward: Balance,
}

impl Default for ValidatorStatus {
    fn default() -> Self {
        ValidatorStatus::Active
    }
}

impl Default for ValidatorPerformance {
    fn default() -> Self {
        ValidatorPerformance {
            uptime_percent: Perbill::one(),
            blocks_produced: 0,
            blocks_missed: 0,
            last_active_era: 0,
        }
    }
}