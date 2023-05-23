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

#![cfg(test)]

use frame_support::{assert_err, assert_ok};
use sp_runtime::traits::AccountIdConversion;

use crate::{mock::*, *};

fn init_gauge() -> (PoolId, BalanceOf<Runtime>) {
    let mut tokens_proportion_map = BTreeMap::<CurrencyIdOf<Runtime>, Perbill>::new();
    tokens_proportion_map
        .entry(KSM)
        .or_insert(Perbill::from_percent(100));
    let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
    let tokens = 1000;
    let basic_rewards = vec![(KSM, 1000)];
    let gauge_basic_rewards = vec![(KSM, 900)];

    assert_ok!(Farming::create_farming_pool(
        RuntimeOrigin::signed(ALICE),
        tokens_proportion,
        basic_rewards,
        Some((KSM, 1000, gauge_basic_rewards)),
        0,
        0,
        0,
        0,
        5
    ));

    let pid = 0;
    let charge_rewards = vec![(KSM, 300000)];
    assert_ok!(Farming::charge(
        RuntimeOrigin::signed(BOB),
        pid,
        charge_rewards
    ));
    assert_ok!(Farming::deposit(
        RuntimeOrigin::signed(ALICE),
        pid,
        tokens,
        Some((100, 100))
    ));
    (pid, tokens)
}

fn init_no_gauge() -> (PoolId, BalanceOf<Runtime>) {
    let mut tokens_proportion_map = BTreeMap::<CurrencyIdOf<Runtime>, Perbill>::new();
    tokens_proportion_map
        .entry(KSM)
        .or_insert(Perbill::from_percent(100));
    let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
    let tokens = 1000;
    let basic_rewards = vec![(KSM, 1000)];
    let gauge_basic_rewards = vec![(KSM, 1000)];

    assert_ok!(Farming::create_farming_pool(
        RuntimeOrigin::signed(ALICE),
        tokens_proportion,
        basic_rewards,
        Some((KSM, 1000, gauge_basic_rewards)),
        0,
        0,
        10,
        0,
        1
    ));

    let pid = 0;
    let charge_rewards = vec![(KSM, 100000)];

    assert_ok!(Farming::charge(
        RuntimeOrigin::signed(BOB),
        pid,
        charge_rewards
    ));
    assert_ok!(Farming::deposit(
        RuntimeOrigin::signed(ALICE),
        pid,
        tokens,
        None
    ));
    (pid, tokens)
}

#[test]
fn claim() {
    let keeper_account = FarmingKeeperPalletId::get().into_account_truncating();

    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, _tokens) = init_no_gauge();
            // assert_eq!(Farming::shares_and_withdrawn_rewards(pid, &ALICE), ShareInfo::default());
            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pid),
                Error::<Runtime>::InvalidPoolState
            );
            System::set_block_number(System::block_number() + 100);
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 2000);
            Farming::on_initialize(0);
            assert_ok!(Farming::withdraw_claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 2000);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            Farming::on_initialize(0);
            assert_ok!(Farming::close_pool(RuntimeOrigin::signed(ALICE), pid));

            // Fund token to keeper_account
            assert_ok!(Assets::mint(
                RuntimeOrigin::signed(ALICE),
                KSM,
                keeper_account,
                100
            ));

            assert_ok!(Farming::force_retire_pool(
                RuntimeOrigin::signed(ALICE),
                pid
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 5000); // 3000 + 1000 + 1000
            Farming::on_initialize(0);
            assert_err!(
                Farming::force_retire_pool(RuntimeOrigin::signed(ALICE), pid),
                Error::<Runtime>::InvalidPoolState
            );
        });
}

#[test]
fn deposit() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, tokens) = init_no_gauge();
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                tokens,
                Some((100, 100))
            ));
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                0,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 800);
            let keeper: AccountId =
                <Runtime as Config>::Keeper::get().into_sub_account_truncating(pid);
            let reward_issuer: AccountId =
                <Runtime as Config>::RewardIssuer::get().into_sub_account_truncating(pid);
            let mut gauge_basic_rewards =
                BTreeMap::<CurrencyIdOf<Runtime>, BalanceOf<Runtime>>::new();
            gauge_basic_rewards.entry(KSM).or_insert(1000);
            let gauge_pool_info2 = GaugePoolInfo {
                pid,
                token: KSM,
                keeper,
                reward_issuer,
                rewards: BTreeMap::<
                    CurrencyIdOf<Runtime>,
                    (BalanceOf<Runtime>, BalanceOf<Runtime>, BalanceOf<Runtime>),
                >::new(),
                gauge_basic_rewards,
                max_block: 1000,
                gauge_amount: 200,
                total_time_factor: 39900,
                gauge_last_block: System::block_number(),
                gauge_state: GaugeState::Bonded,
            };
            assert_eq!(Farming::gauge_pool_infos(0), Some(gauge_pool_info2));
            Farming::on_initialize(0);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1000);
        })
}

#[test]
fn withdraw() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, tokens) = init_no_gauge();
            assert_eq!(Assets::balance(KSM, &ALICE), 2000);
            Farming::on_initialize(0);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pid,
                Some(800)
            ));
            assert_err!(
                Farming::withdraw(RuntimeOrigin::signed(ALICE), pid, Some(100)),
                Error::<Runtime>::WithdrawLimitCountExceeded
            );
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            System::set_block_number(System::block_number() + 100);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(BOB),
                pid,
                tokens,
                None
            ));
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 3966);
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pid,
                Some(200)
            ));
            System::set_block_number(System::block_number() + 100);
            assert_ok!(Farming::withdraw_claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 4166);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Farming::shares_and_withdrawn_rewards(pid, &ALICE), None);
            assert_eq!(Assets::balance(KSM, &ALICE), 4166);
            // let ed = <Runtime as Config>::MultiCurrency::minimum_balance(KSM);
            // assert_eq!(Assets::balance(KSM, &TREASURY_ACCOUNT), ed);
        })
}

#[test]
fn gauge() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, tokens) = init_gauge();
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            if let Some(gauge_pool_infos) = Farming::gauge_pool_infos(0) {
                assert_eq!(
                    gauge_pool_infos.rewards,
                    BTreeMap::<
                        CurrencyIdOf<Runtime>,
                        (BalanceOf<Runtime>, BalanceOf<Runtime>, BalanceOf<Runtime>),
                    >::new()
                )
            };
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 2918);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 10);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                tokens,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 1818);
            System::set_block_number(System::block_number() + 20);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 3163);
            assert_ok!(Farming::deposit(RuntimeOrigin::signed(BOB), pid, 10, None));
            assert_eq!(Assets::balance(KSM, &BOB), 9699990);
            System::set_block_number(System::block_number() + 200);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 5383);
            assert_eq!(Assets::balance(KSM, &BOB), 9699990);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(BOB),
                pid,
                0,
                Some((100, 100))
            ));
            System::set_block_number(System::block_number() + 200);
            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            assert_ok!(Farming::force_gauge_claim(
                RuntimeOrigin::signed(ALICE),
                pid
            ));
            assert_eq!(Assets::balance(KSM, &BOB), 9699991);
        })
}

#[test]
fn gauge_withdraw() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, _tokens) = init_gauge();
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            if let Some(gauge_pool_infos) = Farming::gauge_pool_infos(0) {
                assert_eq!(gauge_pool_infos.gauge_amount, 100)
            };
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            Farming::on_initialize(0);
            assert_ok!(Farming::gauge_withdraw(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 1918);
            System::set_block_number(System::block_number() + 1000);
            assert_ok!(Farming::gauge_withdraw(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 3782);
            if let Some(gauge_pool_infos) = Farming::gauge_pool_infos(0) {
                assert_eq!(gauge_pool_infos.gauge_amount, 0)
            };
        })
}

#[test]
fn retire() {
    let keeper_account = FarmingKeeperPalletId::get().into_account_truncating();

    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, tokens) = init_no_gauge();
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                tokens,
                Some((100, 100))
            ));
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                0,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 800);
            assert_ok!(Farming::close_pool(RuntimeOrigin::signed(ALICE), pid));

            // Fund token to keeper_account
            assert_ok!(Assets::mint(
                RuntimeOrigin::signed(ALICE),
                KSM,
                keeper_account,
                100
            ));

            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            System::set_block_number(System::block_number() + 1000);
            assert_ok!(Farming::force_retire_pool(
                RuntimeOrigin::signed(ALICE),
                pid
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            assert_eq!(Farming::shares_and_withdrawn_rewards(pid, &ALICE), None);
        })
}

#[test]
fn reset() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pid, _tokens) = init_gauge();
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 2918);
            assert_ok!(Farming::close_pool(RuntimeOrigin::signed(ALICE), pid));
            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            assert_ok!(Farming::force_retire_pool(
                RuntimeOrigin::signed(ALICE),
                pid
            ));
            let basic_rewards = vec![(KSM, 1000)];
            assert_ok!(Farming::reset_pool(
                RuntimeOrigin::signed(ALICE),
                pid,
                None,
                None,
                None,
                None,
                None,
                None,
                Some((KSM, 1000, basic_rewards)),
            ));
            let keeper: AccountId =
                <Runtime as Config>::Keeper::get().into_sub_account_truncating(pid);
            let reward_issuer: AccountId =
                <Runtime as Config>::RewardIssuer::get().into_sub_account_truncating(pid);
            let mut basic_rewards_map =
                BTreeMap::<CurrencyIdOf<Runtime>, BalanceOf<Runtime>>::new();
            basic_rewards_map.entry(KSM).or_insert(1000);
            let mut tokens_proportion_map = BTreeMap::<CurrencyIdOf<Runtime>, Perbill>::new();
            tokens_proportion_map
                .entry(KSM)
                .or_insert(Perbill::from_percent(100));
            let pool_infos = PoolInfo {
                tokens_proportion: tokens_proportion_map,
                total_shares: Default::default(),
                basic_token: (KSM, Perbill::from_percent(100)),
                basic_rewards: basic_rewards_map.clone(),
                rewards: BTreeMap::new(),
                state: PoolState::UnCharged,
                keeper: keeper.clone(),
                reward_issuer: reward_issuer.clone(),
                gauge: Some(1),
                block_startup: None,
                min_deposit_to_start: Default::default(),
                after_block_to_start: Default::default(),
                withdraw_limit_time: Default::default(),
                claim_limit_time: Default::default(),
                withdraw_limit_count: 5,
            };
            assert_eq!(Farming::pool_infos(0), Some(pool_infos));
            let gauge_pool_info = GaugePoolInfo {
                pid,
                token: KSM,
                keeper,
                reward_issuer,
                rewards: BTreeMap::<
                    CurrencyIdOf<Runtime>,
                    (BalanceOf<Runtime>, BalanceOf<Runtime>, BalanceOf<Runtime>),
                >::new(),
                gauge_basic_rewards: basic_rewards_map,
                max_block: 1000,
                gauge_amount: 0,
                total_time_factor: 0,
                gauge_last_block: System::block_number(),
                gauge_state: GaugeState::Bonded,
            };
            assert_eq!(Farming::gauge_pool_infos(1), Some(gauge_pool_info));
            assert_eq!(Assets::balance(KSM, &ALICE), 3918);
            let charge_rewards = vec![(KSM, 300000)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pid,
                charge_rewards
            ));
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                1,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 3817);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 20);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 4017);
        })
}

#[test]
fn create_farming_pool() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let mut tokens_proportion_map = BTreeMap::<CurrencyIdOf<Runtime>, Perbill>::new();
            tokens_proportion_map
                .entry(KSM)
                .or_insert(Perbill::from_percent(100));
            let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
            let tokens = 1000;
            let basic_rewards = vec![(KSM, 1000)];
            let gauge_basic_rewards = vec![(KSM, 900)];

            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion.clone(),
                basic_rewards.clone(),
                Some((KSM, 1000, gauge_basic_rewards.clone())),
                2,
                1,
                7,
                6,
                5
            ));
            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion,
                basic_rewards,
                Some((KSM, 1000, gauge_basic_rewards)),
                2,
                1,
                7,
                6,
                5
            ));
            if let Some(pool_infos) = Farming::pool_infos(0) {
                assert_eq!(pool_infos.state, PoolState::UnCharged)
            };
            assert_ok!(Farming::kill_pool(RuntimeOrigin::signed(ALICE), 0));

            let pid = 1;
            let charge_rewards = vec![(KSM, 300000)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pid,
                charge_rewards
            ));
            if let Some(pool_infos) = Farming::pool_infos(0) {
                assert_eq!(pool_infos.total_shares, 0);
                assert_eq!(pool_infos.min_deposit_to_start, 2);
                assert_eq!(pool_infos.state, PoolState::Charged)
            };
            assert_err!(
                Farming::deposit(RuntimeOrigin::signed(ALICE), pid, tokens, Some((100, 100))),
                Error::<Runtime>::CanNotDeposit
            );
            System::set_block_number(System::block_number() + 3);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pid,
                tokens,
                Some((100, 100))
            ));
            Farming::on_initialize(System::block_number() + 3);
            Farming::on_initialize(0);
            if let Some(pool_infos) = Farming::pool_infos(0) {
                assert_eq!(pool_infos.total_shares, 1000);
                assert_eq!(pool_infos.min_deposit_to_start, 2);
                assert_eq!(pool_infos.state, PoolState::Ongoing)
            };
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pid),
                Error::<Runtime>::CanNotClaim
            );
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 3008);
            System::set_block_number(System::block_number() + 100);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 4698);
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pid,
                Some(800)
            ));
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pid),
                Error::<Runtime>::CanNotClaim
            );
            assert_eq!(Assets::balance(KSM, &ALICE), 4698);
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pid));
            assert_eq!(Assets::balance(KSM, &ALICE), 5498);
        })
}