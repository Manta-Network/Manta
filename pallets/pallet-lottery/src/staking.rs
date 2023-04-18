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

use pallet_parachain_staking::BalanceOf;

use super::*;
use array_tool::vec::Intersect;
use frame_support::ensure;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::Zero;

impl<T: Config> Pallet<T> {
    fn calculate_deposit_distribution(
        new_deposit: BalanceOf<T>,
    ) -> Vec<(T::AccountId, BalanceOf<T>)> {
        if new_deposit.is_zero() {
            return vec![];
        }
        let mut deposits: Vec<(T::AccountId, BalanceOf<T>)> = vec![];

        // first concern: If we fell out of the active set on one or more collators, we need to get back into it
        let active_collators = pallet_parachain_staking::Pallet::<T>::selected_candidates();
        let collators_we_are_staked_with: Vec<_> = StakedCollators::<T>::iter_keys().collect();
        let active_collators_we_are_staked_with =
            collators_we_are_staked_with.intersect(active_collators);

        let mut remaining_deposit = new_deposit;
        for collator in active_collators_we_are_staked_with {
            let staked = StakedCollators::<T>::get(collator.clone());
            let info = pallet_parachain_staking::Pallet::<T>::candidate_info(collator.clone())
                .expect("is active collator, therefor it has collator info. qed");
            if staked < info.lowest_top_delegation_amount {
                let deposit =
                    remaining_deposit.saturating_sub(info.lowest_top_delegation_amount - staked);
                deposits.push((collator, deposit));
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
                    sp_arithmetic::Percent::from_rational(1, deposits.len().into()).mul_ceil(remaining_deposit); // this overshoots the amount if there's a remainder
                for deposit in &mut deposits {
                    let add = remaining_deposit.saturating_sub(deposit_per_collator);
                    deposit.1 += add;
                    remaining_deposit -= add;
                }
            }
            return deposits;
        }

        // // second concern: We want to maximize staking APY earned, so we want to balance the staking pools with our deposits while conserving gas
        // let mut collator_balances: Vec<(T::AccountId,BalanceOf<T>)> = vec![];
        // // We only consider active collators for deposits
        // for collator in pallet_parachain_staking::Pallet::<T>::selected_candidates(){
        //     let info = pallet_parachain_staking::Pallet::<T>::candidate_info(collator).expect("is active collator, therefor it has collator info. qed");
        //     if ()
        //     collator_balances.push((collator,info.total_counted));
        // }
        // let average_deposit = collator_balances.map(|x|x.1).mean();
        // collator_balances.sort_unstable_by(|a,b| a.1.cmp(b.1));
        // let eligible_for_deposit = collator_balances[..5]; // TODO: hardcoded ( to bound the gas cost )

        deposits
    }
}
