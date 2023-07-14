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

mod deposit_strategies;
mod withdraw_strategies;

use super::*;
use frame_support::{dispatch::RawOrigin, ensure, traits::EstimateCallFee};
use pallet_parachain_staking::BalanceOf;
use sp_runtime::{
    traits::{Saturating, Zero},
    DispatchResult, Percent,
};
use sp_std::{vec, vec::Vec};

impl<T: Config> Pallet<T> {
    #[named]
    /// distributes a given amount of tokens to zero or more collators for staking
    /// if it can't distribute all tokens for some reason, it returns an empty vec
    pub(crate) fn calculate_deposit_distribution(
        new_deposit: BalanceOf<T>,
    ) -> Vec<(T::AccountId, BalanceOf<T>)> {
        log::trace!(function_name!());
        log::debug!(
            "Calculating distribution for deposit of {:?} tokens",
            new_deposit
        );
        if new_deposit < Self::min_deposit() {
            log::debug!(
                "Requested deposit of {:?} is below limit for staking of {:?}. Skipping assignment",
                new_deposit,
                Self::min_deposit(),
            );
            return vec![];
        }
        let mut deposits: Vec<(T::AccountId, BalanceOf<T>)> = vec![];
        let mut remaining_deposit = new_deposit;

        // Only deposit to active collators (according to ParachainStaking) that we are not currently undelegating from (re-delegating would fail) and that received some rewards in the last round
        let top_collator_accounts = pallet_parachain_staking::Pallet::<T>::selected_candidates();
        if top_collator_accounts.is_empty() {
            log::error!("FATAL: ParachainStaking returned no active collators"); // NOTE: guaranteed by ParachainStaking to not happen
            return vec![];
        }
        let collators_we_are_unstaking_from = UnstakingCollators::<T>::get()
            .iter()
            .cloned()
            .map(|uc| uc.account)
            .collect::<Vec<_>>();

        let round_info = pallet_parachain_staking::Pallet::<T>::round();
        // NOTE: This is O(n^2) but both vecs are << 100 elements
        let deposit_eligible_collators = top_collator_accounts
            .iter()
            .filter(|account| {
                !collators_we_are_unstaking_from.contains(account)
                    && (round_info.current <= 1
                        || !pallet_parachain_staking::Pallet::<T>::awarded_pts(
                            round_info.current - 1,
                            account,
                        )
                        .is_zero())
            })
            .cloned()
            .collect::<Vec<_>>();

        // first concern: If we fell out of the active set on one or more collators, we need to get back into it
        deposits.append(&mut deposit_strategies::reactivate_bottom_collators::<T>(
            deposit_eligible_collators.as_slice(),
            new_deposit,
        ));
        // `reactivate_bottom_collators` has only distributed the funds needed for reactivation, we can have some left over
        remaining_deposit -= deposits
            .iter()
            .map(|deposit| deposit.1)
            .reduce(|sum, elem| sum + elem)
            .unwrap_or_else(|| 0u32.into());

        // If we have re-activated any collators and have leftover funds, we just distribute all surplus tokens to them evenly and call it a day
        if !deposits.is_empty() {
            if !remaining_deposit.is_zero() {
                let deposit_per_collator =
                    Percent::from_rational(1, deposits.len()).mul_ceil(remaining_deposit); // this overshoots the amount if there's a remainder
                for deposit in &mut deposits {
                    let add = remaining_deposit.saturating_sub(deposit_per_collator); // we correct the overshoot here
                    deposit.1 += add;
                    remaining_deposit -= add;
                }
            }
            return deposits;
        }

        // second concern: We want to maximize staking APY earned, so we want to balance the staking pools with our deposits while conserving gas
        deposits.append(
            &mut deposit_strategies::split_to_underallocated_collators::<T>(
                deposit_eligible_collators.as_slice(),
                remaining_deposit,
            ),
        );
        remaining_deposit -= deposits
            .iter()
            .map(|deposit| deposit.1)
            .reduce(|sum, elem| sum + elem)
            .unwrap_or_else(|| 0u32.into());
        // fallback: just assign to a random active collator ( choose a different collator for each invocation )
        if !remaining_deposit.is_zero() {
            log::warn!(
                "Failed to distribute {:?} tokens by strategy",
                remaining_deposit
            );
            if let Some(deposit) = deposit_strategies::stake_to_random_collator::<T>(
                deposit_eligible_collators.as_slice(),
                remaining_deposit,
            ) {
                deposits.push(deposit);
                remaining_deposit = new_deposit
                    - deposits
                        .iter()
                        .map(|deposit| deposit.1)
                        .reduce(|sum, elem| sum + elem)
                        .unwrap_or_else(|| 0u32.into());
            }
        }
        if deposits.is_empty() {
            log::error!("FATAL: Could not find any collator to stake to");
        }
        log::debug!("Deposits: {:?}", deposits);
        if !remaining_deposit.is_zero() {
            log::error!(
                "FATAL: We have {:?} unstaked balance left over after depositing, returning empty vec",
                remaining_deposit
            );
            deposits.clear();
        }
        deposits
    }

    #[named]
    pub(crate) fn calculate_withdrawal_distribution(
        withdrawal_amount: BalanceOf<T>,
    ) -> Vec<T::AccountId> {
        log::trace!(function_name!());
        if withdrawal_amount.is_zero() {
            return vec![];
        }
        let mut withdrawals = vec![];
        let mut remaining_balance = withdrawal_amount;

        // Only unstake collators we're staked to and not already unstaking from
        let staked_collators: Vec<_> = StakedCollators::<T>::iter_keys().collect();
        let collators_we_are_unstaking_from: Vec<_> = UnstakingCollators::<T>::get()
            .iter()
            .cloned()
            .map(|uc| uc.account)
            .collect();
        // NOTE: This is O(n^2) but both vecs are << 100 elements
        let withdrawal_eligible_collators: Vec<_> = staked_collators
            .iter()
            .filter(|account| !collators_we_are_unstaking_from.contains(account))
            .cloned()
            .collect();
        if withdrawal_eligible_collators.is_empty() {
            return vec![];
        }
        // first concern: If there are inactive collators we are staked with, prefer these
        let (mut collators, balance_unstaked) = withdraw_strategies::unstake_inactive_collators::<T>(
            &withdrawal_eligible_collators,
            remaining_balance,
        );
        withdrawals.append(&mut collators);
        remaining_balance = remaining_balance.saturating_sub(balance_unstaked);
        if remaining_balance.is_zero() && !withdrawals.is_empty() {
            return withdrawals;
        }
        // If we have balance to withdraw left over, we have to unstake some healthy collator.
        // Unstake starting from the highest overallocated collator ( since that yields the lowest APY ) going down until request is satisfied
        let (mut collators, balance_unstaked) = withdraw_strategies::unstake_least_apy_collators::<T>(
            &withdrawal_eligible_collators
                .into_iter()
                .filter(|collator| !withdrawals.contains(collator))
                .collect(),
            remaining_balance,
        );
        withdrawals.append(&mut collators);
        remaining_balance = remaining_balance.saturating_sub(balance_unstaked);

        if !remaining_balance.is_zero() {
            log::error!(
                "FATAL: We have {:?} left that COULD NOT BE UNSTAKED",
                remaining_balance
            );
            return vec![]; // NOTE: Ensure we do not continue with the partial withdrawal
        }
        if withdrawals.is_empty() {
            log::error!("COULD NOT SELECT ANY COLLATOR TO WITHDRAW FROM");
        } else {
            log::debug!("Withdrawals: {:?}", withdrawals.len());
        }
        withdrawals
    }

    #[named]
    pub(crate) fn do_stake_one_collator(
        collator: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        log::trace!(function_name!());
        // preconditions
        if Self::surplus_funds().is_zero() {
            return Err(Error::<T>::PotBalanceTooLow.into());
        }
        let candidate_delegation_count;
        if let Some(info) = pallet_parachain_staking::Pallet::<T>::candidate_info(&collator) {
            candidate_delegation_count = info.delegation_count;
        } else {
            return Err(Error::<T>::NoCollatorForDeposit.into());
        };
        let delegation_count = StakedCollators::<T>::iter_keys().count() as u32;

        // If we're already delegated to this collator, we must call `delegate_more`
        if StakedCollators::<T>::get(&collator).is_zero() {
            // Ensure the pallet has enough gas to pay for this
            let fee_estimate: BalanceOf<T> = T::EstimateCallFee::estimate_call_fee(
                &pallet_parachain_staking::Call::delegate {
                    candidate: collator.clone(),
                    amount,
                    candidate_delegation_count: candidate_delegation_count + 1,
                    delegation_count: delegation_count + 1,
                },
                None::<u64>.into(),
            );
            ensure!(
                Self::surplus_funds() > fee_estimate,
                Error::<T>::PotBalanceTooLowToPayTxFee
            );
            pallet_parachain_staking::Pallet::<T>::delegate(
                RawOrigin::Signed(Self::account_id()).into(),
                collator.clone(),
                amount,
                candidate_delegation_count + 1,
                delegation_count + 1,
            )
            .map_err(|e| {
                log::error!(
                    "Could not delegate {:?} to collator {:?} with error {:?}",
                    amount.clone(),
                    collator.clone(),
                    e
                );
                e.error
            })?;
        } else {
            // Ensure the pallet has enough gas to pay for this
            let fee_estimate: BalanceOf<T> = T::EstimateCallFee::estimate_call_fee(
                &pallet_parachain_staking::Call::delegator_bond_more {
                    candidate: collator.clone(),
                    more: amount,
                },
                None::<u64>.into(),
            );
            ensure!(
                Self::surplus_funds() > fee_estimate,
                Error::<T>::PotBalanceTooLowToPayTxFee
            );
            pallet_parachain_staking::Pallet::<T>::delegator_bond_more(
                RawOrigin::Signed(Self::account_id()).into(),
                collator.clone(),
                amount,
            )
            .map_err(|e| {
                log::error!(
                    "Could not bond more {:?} to collator {:?} with error {:?}",
                    amount.clone(),
                    collator.clone(),
                    e
                );
                e.error
            })?;
        }
        StakedCollators::<T>::mutate(&collator, |balance| *balance += amount);

        log::debug!("Delegated {:?} tokens to {:?}", amount, collator);
        Ok(())
    }

    #[named]
    pub(crate) fn do_unstake_collator(
        now: T::BlockNumber,
        some_collator: T::AccountId,
    ) -> DispatchResult {
        log::trace!(function_name!());
        let delegated_amount_to_be_unstaked = StakedCollators::<T>::take(some_collator.clone());
        if delegated_amount_to_be_unstaked.is_zero() {
            log::error!("requested to unstake a collator that isn't staked");
            return Err(Error::<T>::NoCollatorForWithdrawal.into());
        };
        log::debug!(
            "Unstaking collator {:?} with balance {:?}",
            some_collator,
            delegated_amount_to_be_unstaked.clone()
        );
        // Ensure the pallet has enough gas to pay for this
        let fee_estimate: BalanceOf<T> = T::EstimateCallFee::estimate_call_fee(
            &pallet_parachain_staking::Call::schedule_revoke_delegation {
                collator: some_collator.clone(),
            },
            None::<u64>.into(),
        );
        ensure!(
            Self::surplus_funds() > fee_estimate,
            Error::<T>::PotBalanceTooLowToPayTxFee
        );
        // unstake from parachain staking
        // NOTE: All funds that were delegated here will no longer produce staking rewards
        pallet_parachain_staking::Pallet::<T>::schedule_revoke_delegation(
            RawOrigin::Signed(Self::account_id()).into(),
            some_collator.clone(),
        )
        .map_err(|e| e.error)?;

        // Update bookkeeping
        SurplusUnstakingBalance::<T>::mutate(|bal| {
            *bal = (*bal).saturating_add(delegated_amount_to_be_unstaked);
        });
        UnstakingCollators::<T>::mutate(|collators| {
            collators.push(UnstakingCollator {
                account: some_collator.clone(),
                since: now,
            })
        });
        Ok(())
    }
}
