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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{
    AwardedPts, BalanceOf, Call, CandidateBondLessRequest, Config, DelegationAction, Pallet,
    Points, Range, Round, ScheduledRequest,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec};
use frame_support::traits::{tokens::fungible::Inspect, Currency, Get, OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use pallet_parachain_staking::benchmarks::*;
use pallet_parachain_staking::Pallet as Staking;
use sp_runtime::{Perbill, Percent};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Minimum delegator stake
fn max_collator_count<T: Config>() -> u32 {
    63
}

// Simulate staking on finalize by manually setting points
// fn parachain_staking_on_finalize<T: Config>(author: T::AccountId) {
//     let now = <Round<T>>::get().current;
//     let score_plus_20 = <AwardedPts<T>>::get(now, &author).saturating_add(20);
//     <AwardedPts<T>>::insert(now, author, score_plus_20);
//     <Points<T>>::mutate(now, |x| *x = x.saturating_add(20));
// }

/// Run to end block and author
// fn roll_to_and_author<T: Config>(round_delay: u32, author: T::AccountId) {
//     let total_rounds = round_delay + 1u32;
//     let round_length: T::BlockNumber = Pallet::<T>::round().length.into();
//     let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
//     let end = Pallet::<T>::round().first + (round_length * total_rounds.into());
//     while now < end {
//         parachain_staking_on_finalize::<T>(author.clone());
//         <frame_system::Pallet<T>>::on_finalize(<frame_system::Pallet<T>>::block_number());
//         <frame_system::Pallet<T>>::set_block_number(
//             <frame_system::Pallet<T>>::block_number() + 1u32.into(),
//         );
//         <frame_system::Pallet<T>>::on_initialize(<frame_system::Pallet<T>>::block_number());
//         Pallet::<T>::on_initialize(<frame_system::Pallet<T>>::block_number());
//         now += 1u32.into();
//     }
// }

const USER_SEED: u32 = 999666;
benchmarks! {
    // USER DISPATCHABLES

    deposit {
        // TODO: Parametric over large amount of users already deposited?
        const u32 DEPOSIT = 10_000_000_000_000_000_000
        // fill collators
        for _ in 0..max_collator_count()
        {
            let mut collator_seed: u32 = 444;
            create_funded_collator(
                "collator",
                collator_seed+=1,
                0.into(),
                true,
                max_collator_count()
            );
        }
        let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
    }: _(RawOrigin::Signed(caller.clone()), DEPOSIT)
    verify {
        assert_eq!(Pallet::<T>::total_pot(), DEPOSIT);
        assert_eq!(Pallet::<T>::active_balance_per_user(caller), DEPOSIT);
    }

    // request_withdraw{
    //     let inflation_range: Range<Perbill> = Range {
    //         min: Perbill::from_perthousand(1),
    //         ideal: Perbill::from_perthousand(2),
    //         max: Perbill::from_perthousand(3),
    //     };

    // }: _(RawOrigin::Root, inflation_range)
    // verify {
    //     // assert_eq!(Pallet::<T>::inflation_config().annual, inflation_range);
    // }

    // claim_my_winnings {
    //     let parachain_bond_account: T::AccountId = account("TEST", 0u32, USER_SEED);
    // }: _(RawOrigin::Root, parachain_bond_account.clone())
    // verify {
    //     // assert_eq!(Pallet::<T>::parachain_bond_info().account, parachain_bond_account);
    // }

    // // ROOT DISPATCHABLES

    // rebalance_stake {
    // }: _(RawOrigin::Root, Percent::from_percent(33))
    // verify {
    //     assert_eq!(Pallet::<T>::parachain_bond_info().percent, Percent::from_percent(33));
    // }

    // start_lottery {
    //     Pallet::<T>::set_blocks_per_round(RawOrigin::Root.into(), 100u32)?;
    // }: _(RawOrigin::Root, 100u32)
    // verify {
    //     assert_eq!(Pallet::<T>::total_selected(), 100u32);
    // }

    // stop_lottery {}: _(RawOrigin::Root, Perbill::from_percent(33))
    // verify {
    //     assert_eq!(Pallet::<T>::collator_commission(), Perbill::from_percent(33));
    // }

    // draw_lottery {}: _(RawOrigin::Root, 1200u32)
    // verify {
    //     // assert_eq!(Pallet::<T>::round().length, 1200u32);
    // }

    // process_matured_withdrawals {
    //     let x in 3..1_000;
    //     // Worst Case Complexity is insertion into an ordered list so \exists full list before call
    //     let mut candidate_count = 1u32;
    //     for i in 2..x {
    //         let seed = USER_SEED - i;
    //         let collator = create_funded_collator::<T>(
    //             "collator",
    //             seed,
    //             0u32.into(),
    //             true,
    //             candidate_count
    //         )?;
    //         candidate_count += 1u32;
    //     }
    //     let (caller, min_candidate_stk) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
    // }: _(RawOrigin::Signed(caller.clone()), min_candidate_stk, Pallet::<T>::total_selected() + candidate_count)
    // verify {
    //     // assert!(Pallet::<T>::is_candidate(&caller));
    // }

    // liquidate_lottery {
    //     let x in 3..1_000;
    //     // Worst Case Complexity is removal from an ordered list so \exists full list before call
    //     let mut candidate_count = 1u32;
    //     for i in 2..x {
    //         let seed = USER_SEED - i;
    //         let collator = create_funded_collator::<T>(
    //             "collator",
    //             seed,
    //             0u32.into(),
    //             true,
    //             candidate_count
    //         )?;
    //         candidate_count += 1u32;
    //     }
    //     let caller: T::AccountId = create_funded_collator::<T>(
    //         "caller",
    //         USER_SEED,
    //         0u32.into(),
    //         true,
    //         candidate_count,
    //     )?;
    //     candidate_count += 1u32;
    // }: _(RawOrigin::Signed(caller.clone()), Pallet::<T>::total_selected() + candidate_count)
    // verify {
    //     // assert!(Pallet::<T>::candidate_info(&caller).unwrap().is_leaving());
    // }
}

#[cfg(test)]
mod tests {
    use crate::{benchmarks::*, mock::Test};
    use frame_support::assert_ok;
    use sp_io::TestExternalities;

    pub fn new_test_ext() -> TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        TestExternalities::new(t)
    }
    #[test]
    fn bench_deposit() {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::test_benchmark_deposit());
        });
    }
    // #[test]
    // fn bench_request_withdraw() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_request_withdraw());
    //     });
    // }
    // #[test]
    // fn bench_claim_my_winnings() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_claim_my_winnings());
    //     });
    // }
    // #[test]
    // fn bench_rebalance_stake() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_rebalance_stake());
    //     });
    // }
    // #[test]
    // fn bench_start_lottery() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_start_lottery());
    //     });
    // }
    // #[test]
    // fn bench_stop_lottery() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_stop_lottery());
    //     });
    // }
    // #[test]
    // fn bench_draw_lottery() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_draw_lottery());
    //     });
    // }
    // #[test]
    // fn bench_process_matured_withdrawals() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_process_matured_withdrawals());
    //     });
    // }
    // #[test]
    // fn bench_liquidate_lottery() {
    //     new_test_ext().execute_with(|| {
    //         assert_ok!(Pallet::<Test>::test_benchmark_liquidate_lottery());
    //     });
    // }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::benchmarks::tests::new_test_ext(),
    crate::mock::Test
);
