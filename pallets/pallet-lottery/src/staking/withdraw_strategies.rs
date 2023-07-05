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
use pallet_parachain_staking::BalanceOf;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{vec, vec::Vec};

pub(super) fn unstake_inactive_collators<T: Config>(
    eligible_collators: &Vec<T::AccountId>,
    withdrawal_amount: BalanceOf<T>,
) -> (Vec<T::AccountId>, BalanceOf<T>) {
    let mut withdrawals = vec![];
    let mut unstaked = 0u32.into();

    if eligible_collators.len().is_zero() || withdrawal_amount.is_zero() {
        return (withdrawals, 0u32.into());
    }
    // first concern: If there are inactive collators we are staked with, prefer these
    let round_info = pallet_parachain_staking::Pallet::<T>::round();
    let selected = pallet_parachain_staking::Pallet::<T>::selected_candidates();
    let inactive_eligible_collators = eligible_collators.iter().filter(
            |collator|{
                // no longer selected for block rewards
                !selected.contains(collator) ||
                // did not receive any points last round unless this is the first round
                (round_info.current > 1 && pallet_parachain_staking::Pallet::<T>::awarded_pts(round_info.current-1,collator).is_zero())
            });
    // since these collators are inactive, we just unstake in any order until we have satisfied the withdrawal request
    for collator in inactive_eligible_collators {
        let our_stake = StakedCollators::<T>::get(collator);
        log::debug!("Unstaking {:?} from inactive {:?}", our_stake, collator);
        unstaked += our_stake;
        withdrawals.push(collator.clone());
        if unstaked >= withdrawal_amount {
            return (withdrawals, unstaked);
        }
    }
    log::debug!(
        "Remaining after inactive: {:?}",
        withdrawal_amount.saturating_sub(unstaked)
    );
    (withdrawals, unstaked)
}

pub(super) fn unstake_least_apy_collators<T: Config>(
    eligible_collators: &Vec<T::AccountId>,
    withdrawal_amount: BalanceOf<T>,
) -> (Vec<T::AccountId>, BalanceOf<T>) {
    // If we have balance to withdraw left over, we have to unstake some healthy collator.
    // Unstake starting from the highest overallocated collator ( since that yields the lowest APY ) going down until request is satisfied
    let mut withdrawals = vec![];
    let mut unstaked = 0u32.into();

    if eligible_collators.len().is_zero() || withdrawal_amount.is_zero() {
        return (withdrawals, unstaked);
    }

    let selected = pallet_parachain_staking::Pallet::<T>::selected_candidates();
    let mut apy_ordered_active_collators_we_are_staked_with: Vec<_> = eligible_collators
        .iter()
        .filter(|collator| selected.contains(collator))
        .cloned()
        .collect();
    apy_ordered_active_collators_we_are_staked_with.sort_by(|a, b| {
        let ainfo = pallet_parachain_staking::Pallet::<T>::candidate_info(a.clone())
            .expect("is a selected collator, therefore it has collator info. qed");
        let binfo = pallet_parachain_staking::Pallet::<T>::candidate_info(b.clone())
            .expect("is a selected collator, therefore it has collator info. qed");
        binfo.total_counted.cmp(&ainfo.total_counted)
    });
    log::debug!(
        "Active collators: {:?}",
        apy_ordered_active_collators_we_are_staked_with.len()
    );
    for c in apy_ordered_active_collators_we_are_staked_with {
        let our_stake = StakedCollators::<T>::get(c.clone());
        log::debug!("Unstaking {:?} from active {:?}", our_stake, c);
        withdrawals.push(c);
        unstaked += our_stake;
        if unstaked >= withdrawal_amount {
            break;
        }
    }
    (withdrawals, unstaked)
}
