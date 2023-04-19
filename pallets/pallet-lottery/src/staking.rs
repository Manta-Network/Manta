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

use super::*;
use codec::alloc::collections::BTreeSet;
use frame_support::ensure;
use frame_support::traits::Get;
use pallet_parachain_staking::BalanceOf;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::Zero;
use sp_runtime::PerThing;
use sp_runtime::Percent;
use sp_std::{vec, vec::Vec};
use frame_support::traits::Randomness;

impl<T: Config> Pallet<T> {
    fn calculate_deposit_distribution(
        new_deposit: BalanceOf<T>,
    ) -> Vec<(T::AccountId, BalanceOf<T>)> {
        if new_deposit < <T as pallet_parachain_staking::Config>::MinDelegation::get() {
            return vec![];
        }
        let mut deposits: Vec<(T::AccountId, BalanceOf<T>)> = vec![];

        // first concern: If we fell out of the active set on one or more collators, we need to get back into it
        let active_collators = pallet_parachain_staking::Pallet::<T>::selected_candidates()
            .into_iter()
            .collect();
        let collators_we_are_staked_with: BTreeSet<_> = StakedCollators::<T>::iter_keys().collect();
        let active_collators_we_are_staked_with =
            collators_we_are_staked_with.intersection(&active_collators);

        let mut remaining_deposit = new_deposit;
        for collator in active_collators_we_are_staked_with {
            let staked = StakedCollators::<T>::get(collator.clone());
            let info = pallet_parachain_staking::Pallet::<T>::candidate_info(collator.clone())
                .expect("is active collator, therefor it has collator info. qed");
            if staked < info.lowest_top_delegation_amount {
                let deposit =
                    remaining_deposit.saturating_sub(info.lowest_top_delegation_amount - staked);
                deposits.push((collator.clone(), deposit)); // TODO: Sort collators ascending by missing amount so we get the largest amount of collators active before running out of funds
                remaining_deposit = remaining_deposit.saturating_sub(deposit);
                if remaining_deposit.is_zero() {
                    break;
                }
            }
        }
        // If we have any collators to re-activate, we distribute all tokens to those and call it a day
        if !deposits.is_empty() {
            if !remaining_deposit.is_zero() {
                // distribute remaining tokens evenly
                let deposit_per_collator =
                    Percent::from_rational(1, deposits.len().into()).mul_ceil(remaining_deposit); // this overshoots the amount if there's a remainder
                for deposit in &mut deposits {
                    let add = remaining_deposit.saturating_sub(deposit_per_collator);
                    deposit.1 += add;
                    remaining_deposit -= add;
                }
            }
            return deposits;
        }

        // second concern: We want to maximize staking APY earned, so we want to balance the staking pools with our deposits while conserving gas
        let mut collator_balances: Vec<(T::AccountId, BalanceOf<T>)> = vec![];
        // We only consider active collators for deposits
        let total_staked = pallet_parachain_staking::Pallet::<T>::staked(
            pallet_parachain_staking::Pallet::<T>::round().current,
        ); // TODO: Also consider points / pointsAwarded to not stake to collators missing blocks
        let mean_stake =
            Percent::from_rational(1, pallet_parachain_staking::Pallet::<T>::total_selected())
                .mul_ceil(total_staked)
                .into(); // this overshoots the amount if there's a remainder

        // build collator => deviation from mean map
        let mut underallocated_collators = vec![];
        for collator in active_collators.iter() {
            let our_stake = StakedCollators::<T>::get(collator.clone()).clone();
            let info = pallet_parachain_staking::Pallet::<T>::candidate_info(collator.clone())
                .expect("is active collator, therefor it has collator info. qed");
            let stake_on_collator = info.total_counted.saturating_sub(our_stake);
            if stake_on_collator < mean_stake {
                underallocated_collators.push((collator.clone(), mean_stake - stake_on_collator));
            }
        }
        underallocated_collators.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        let (_rest, mut last) =
            underallocated_collators.split_at(underallocated_collators.len().saturating_sub(4)); // TODO: 4 is hardcoded make configurable
        let total_underallocation = last
            .into_iter()
            .map(|a| a.1)
            .reduce(|acc, balance| acc + balance)
            .unwrap();
        let deposit_to_distribute = remaining_deposit;
        for (account, balance) in last {
            // If a proportional deposit is over the min deposit and can get us into the top balance, deposit it, if not just skip it
            let info = pallet_parachain_staking::Pallet::<T>::candidate_info(account.clone())
                .expect("is active collator, therefor it has collator info. qed");
            let collator_proportion =
                Percent::from_rational(balance.clone(), total_underallocation);
            let tokens = collator_proportion.mul_ceil(deposit_to_distribute);
            let deposit = remaining_deposit.saturating_sub(tokens);
            let our_stake = StakedCollators::<T>::get(account.clone());
            if deposit > <T as pallet_parachain_staking::Config>::MinDelegation::get()
                && our_stake + deposit > info.lowest_top_delegation_amount
            {
                deposits.push((account.clone(), tokens.min(deposit)));
            };
            if remaining_deposit.is_zero() {
                break;
            }
        }
        // if we had to skip a collator above due to not getting into the top deposit, we just lump the rest into the collator with the lowest stake
        if !deposits.is_empty() && !remaining_deposit.is_zero() {
            let mut last = deposits.pop().unwrap();
            last.1 += remaining_deposit;
            remaining_deposit -= last.1;
            deposits.push(last);
        }

        // fallback : just assign to a random active collator
        if !remaining_deposit.is_zero() {
            let active_collators = pallet_parachain_staking::Pallet::<T>::selected_candidates();
            // TODO: Better randomness
            use sp_runtime::traits::SaturatedConversion;
            let nonce : u128 = Self::total_pot().saturated_into();
            let random = sp_core::U256::from_big_endian(T::RandomnessSource::random(&nonce.to_be_bytes()).0.as_ref());
            let random_index : usize = random.low_u64() as usize % active_collators.len();
            if let Some(random_collator) = active_collators.get(random_index)
            {
                deposits.push((random_collator.clone(), remaining_deposit));
                log::warn!(
                    "Failed to select staking outputs. Staking randomly to {:?}",
                    random_collator
                );
            }
        }

        if !remaining_deposit.is_zero() {
            log::error!(
                "We have {:?} unstaked balance left over after depositing",
                remaining_deposit
            );
        }
        if deposits.is_empty() {
            log::error!("COULD NOT FIND ANY COLLATOR TO STAKE TO");
        }
        deposits
    }

    fn calculate_withdrawal_distribution(withdrawal_amount: BalanceOf<T>) -> Vec<T::AccountId> {
        if withdrawal_amount.is_zero() {
            return vec![];
        }
        let mut withdrawals = vec![];
        let mut remaining_balance = withdrawal_amount;
        // first concern: If there are inactive collators we are staked with, prefer these
        let active_collators: BTreeSet<_> =
            pallet_parachain_staking::Pallet::<T>::selected_candidates()
                .into_iter()
                .collect();
        let collators_we_are_staked_with: BTreeSet<_> = StakedCollators::<T>::iter_keys().collect();
        let inactive_collators_we_are_staked_with: BTreeSet<_> = collators_we_are_staked_with
            .difference(&active_collators)
            .cloned()
            .collect();
        for collator in inactive_collators_we_are_staked_with {
            // unstake them
            let balance = StakedCollators::<T>::get(&collator);
            remaining_balance = remaining_balance.saturating_sub(balance);
            withdrawals.push(collator.clone());
            if remaining_balance.is_zero() {
                return withdrawals;
            }
        }

        // If we have balance to withdraw left over, we have to unstake some healthy collator. Choose the one with the smallest amount to cover the withdrawal amount
        // TODO
        vec![]
    }
}
