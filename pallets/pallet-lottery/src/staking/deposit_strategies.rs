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
use frame_support::traits::Get;
#[cfg(not(feature = "runtime-benchmarks"))]
use frame_support::traits::Randomness;
use pallet_parachain_staking::BalanceOf;
use sp_runtime::{
    traits::{Saturating, Zero},
    Percent,
};
use sp_std::{vec, vec::Vec};

#[named]
pub(super) fn reactivate_bottom_collators<T: Config>(
    active_collators: &[T::AccountId],
    new_deposit: BalanceOf<T>,
) -> Vec<(T::AccountId, BalanceOf<T>)> {
    log::trace!(function_name!());

    let mut deposits: Vec<(T::AccountId, BalanceOf<T>)> = vec![];
    let mut remaining_deposit = new_deposit;

    // We only consider collators we're already staked to that are also currently active (and not being unstaked)
    for collator in StakedCollators::<T>::iter_keys().filter(|coll| active_collators.contains(coll))
    {
        let staked = StakedCollators::<T>::get(collator.clone());
        let info = pallet_parachain_staking::Pallet::<T>::candidate_info(collator.clone())
            .expect("is active collator, therefore it has collator info. qed");
        if staked < info.lowest_top_delegation_amount {
            // TODO: Small optimization: sort collators ascending by missing amount so we get the largest amount of collators active before running out of funds
            let this_deposit = core::cmp::min(
                remaining_deposit,
                info.lowest_top_delegation_amount - staked + 1u32.into(),
            );
            deposits.push((collator.clone(), this_deposit));
            remaining_deposit -= this_deposit;
            if remaining_deposit.is_zero() {
                break;
            }
        }
    }
    deposits
}

/// second concern: We want to maximize staking APY earned, so we want to balance the staking pools with our deposits while conserving gas
pub(super) fn split_to_underallocated_collators<T: Config>(
    active_collators: &[T::AccountId],
    new_deposit: BalanceOf<T>,
) -> Vec<(T::AccountId, BalanceOf<T>)> {
    let mut deposits: Vec<(T::AccountId, BalanceOf<T>)> = vec![];
    let mut remaining_deposit = new_deposit;

    if active_collators.len().is_zero() || new_deposit.is_zero() {
        return deposits;
    }
    // We only consider active collators for deposits
    // TODO: Small optimization: Also consider points / pointsAwarded to not stake to collators missing blocks
    let mut collators_and_counted_balances: Vec<_> = active_collators
        .iter()
        .cloned()
        .map(|collator| {
            (
                collator.clone(),
                pallet_parachain_staking::Pallet::<T>::candidate_info(collator)
                    .expect("is active collator, therefore it has collator info. qed")
                    .total_counted,
            )
        })
        .collect();
    // sort ascending by counted stake
    collators_and_counted_balances.sort_by(|a, b| a.1.cmp(&b.1));
    debug_assert!(
        collators_and_counted_balances.len() == 1
            || pallet_parachain_staking::Pallet::<T>::candidate_info(
                collators_and_counted_balances[0].0.clone()
            )
            .unwrap()
            .total_counted
                <= pallet_parachain_staking::Pallet::<T>::candidate_info(
                    collators_and_counted_balances[1].0.clone()
                )
                .unwrap()
                .total_counted
    );

    let median_collator_balance =
        collators_and_counted_balances[collators_and_counted_balances.len() / 2].1;

    // build collator => deviation from median map
    let mut underallocated_collators: Vec<_> =
        collators_and_counted_balances[..collators_and_counted_balances.len() / 2].to_vec();
    underallocated_collators = underallocated_collators
        .into_iter()
        .filter_map(|(collator, balance)| {
            let underallocation = median_collator_balance.saturating_sub(balance);
            if !underallocation.is_zero() {
                Some((collator, underallocation))
            } else {
                None
            }
        })
        .collect();
    // After this calculation, underallocated_collators is in descending order of underallocation

    // take up to 4 collators with the highest deficit ( stopping at median )
    let num_collators_to_take = core::cmp::min(4, underallocated_collators.len());
    underallocated_collators = underallocated_collators[..num_collators_to_take].to_vec();

    debug_assert!(
        underallocated_collators.is_empty()
            || pallet_parachain_staking::Pallet::<T>::candidate_info(
                underallocated_collators[0].0.clone()
            )
            .unwrap()
            .total_counted
                <= median_collator_balance
    );
    debug_assert!(
        underallocated_collators.len() < 2
            || pallet_parachain_staking::Pallet::<T>::candidate_info(
                underallocated_collators[0].0.clone()
            )
            .unwrap()
            .total_counted
                <= pallet_parachain_staking::Pallet::<T>::candidate_info(
                    underallocated_collators[1].0.clone()
                )
                .unwrap()
                .total_counted
    );
    debug_assert!(
        underallocated_collators.len() < 2
            || underallocated_collators[0].1 >= underallocated_collators[1].1
    );
    log::debug!(
        "Total Underallocated collators: {:?}",
        underallocated_collators.len()
    );
    if !underallocated_collators.is_empty() {
        let total_underallocation = underallocated_collators
                .iter()
                .cloned()
                .map(|a| a.1)
                .reduce(|acc, balance| acc + balance)
                .expect("reduce returns None on empty iterator. we checked that `underallocated_collators` is not empty. qed");
        log::debug!(
            "Underallocated tokens {:?} on selected collators: {:?}",
            total_underallocation,
            underallocated_collators
        );
        for (account, tokens_to_reach_median) in underallocated_collators {
            // If a proportional deposit is over the min deposit and can get us into the top balance, deposit it, if not just skip it
            let info = pallet_parachain_staking::Pallet::<T>::candidate_info(account.clone())
                .expect("is active collator, therefor it has collator info. qed");
            let collator_proportion =
                Percent::from_rational(tokens_to_reach_median, total_underallocation);
            let to_reach_mean = collator_proportion.mul_ceil(new_deposit);
            let to_deposit = to_reach_mean.min(remaining_deposit);
            let our_stake = StakedCollators::<T>::get(account.clone());
            if to_deposit > <T as pallet_parachain_staking::Config>::MinDelegation::get()
                && to_deposit + our_stake > info.lowest_top_delegation_amount
            {
                let this_deposit = core::cmp::min(to_deposit, remaining_deposit);
                deposits.push((account.clone(), this_deposit));
                remaining_deposit -= this_deposit;
                log::debug!(
                    "Selected collator {:?} for deposit of {:?} token",
                    account.clone(),
                    to_deposit
                );
            };
            if remaining_deposit.is_zero() {
                break;
            }
        }
    }
    // if we had to skip a collator above due to not getting into the top deposit, we just lump the rest into the collator with the lowest stake
    if !deposits.is_empty() && !remaining_deposit.is_zero() {
        let mut underallocated_collators = deposits
            .pop()
            .expect("we checked that deposits is not empty, therefore pop will return Some. qed");
        underallocated_collators.1 += remaining_deposit;
        remaining_deposit.set_zero();
        deposits.push(underallocated_collators);
    }
    deposits
}

/// fallback: just assign to a random active collator ( choose a different collator for each invocation )
pub(crate) fn stake_to_random_collator<T: Config>(
    active_collators: &[T::AccountId],
    new_deposit: BalanceOf<T>,
) -> Option<(T::AccountId, BalanceOf<T>)> {
    use sp_runtime::traits::SaturatedConversion;
    if active_collators.len().is_zero() || new_deposit.is_zero() {
        return None;
    }

    let block_number = <frame_system::Pallet<T>>::block_number().saturated_into::<u128>();
    let extrinsic_index = <frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default();
    let nonce: u128 = block_number ^ extrinsic_index as u128;
    let randomness_output: sp_core::U256;
    #[cfg(feature = "runtime-benchmarks")]
    {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(nonce as u64);
        randomness_output = rng.gen::<u128>().into();
    }
    #[cfg(not(feature = "runtime-benchmarks"))]
    {
        randomness_output = sp_core::U256::from_big_endian(
            T::RandomnessSource::random(&nonce.to_be_bytes()).0.as_ref(),
        );
    }
    // NOTE: The following line introduces modulo bias, but since this is just a fallback it is accepted
    let random_index: usize = randomness_output.low_u64() as usize % active_collators.len();
    if let Some(random_collator) = active_collators.get(random_index) {
        log::warn!(
            "Staking {:?} randomly to {:?}",
            new_deposit,
            random_collator
        );
        Some((random_collator.clone(), new_deposit))
    } else {
        log::error!("Could not stake {:?} randomly", new_deposit);
        None
    }
}
