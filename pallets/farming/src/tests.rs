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

use frame_support::{assert_err, assert_noop, assert_ok};
use manta_primitives::types::CalamariAssetId;
use sp_runtime::traits::AccountIdConversion;

use crate::{mock::*, *};

fn init_gauge_900() -> (PoolId, BalanceOf<Runtime>) {
    let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
    let tokens = 1000;
    let basic_rewards = vec![(KSM, 1000)];
    let gauge_basic_rewards = vec![(KSM, 900)];

    assert_ok!(Farming::create_farming_pool(
        RuntimeOrigin::signed(ALICE),
        tokens_proportion,
        basic_rewards,
        Some((KSM, 1000, gauge_basic_rewards)),
        0, // min_deposit_to_start
        0, // after_block_to_start
        0, // withdraw_limit_time
        0, // claim_limit_time
        5  // withdraw_limit_count
    ));

    let pool_id = 0;
    let charge_rewards = vec![(KSM, 300000)];
    assert_ok!(Farming::charge(
        RuntimeOrigin::signed(BOB),
        pool_id,
        charge_rewards
    ));
    assert_ok!(Farming::deposit(
        RuntimeOrigin::signed(ALICE),
        pool_id,
        tokens,
        Some((100, 100))
    ));
    (pool_id, tokens)
}

fn init_gauge_1000() -> (PoolId, BalanceOf<Runtime>) {
    let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
    let tokens = 1000;
    let basic_rewards = vec![(KSM, 1000)];
    let gauge_basic_rewards = (KSM, 1000, vec![(KSM, 1000)]);

    assert_ok!(Farming::create_farming_pool(
        RuntimeOrigin::signed(ALICE),
        tokens_proportion,
        basic_rewards,
        Some(gauge_basic_rewards),
        0,  // min_deposit_to_start
        0,  // after_block_to_start
        10, // withdraw_limit_time
        0,  // claim_limit_time
        1   // withdraw_limit_count
    ));

    let pool_id = 0;
    let charge_rewards = vec![(KSM, 100000)];

    assert_ok!(Farming::charge(
        RuntimeOrigin::signed(BOB),
        pool_id,
        charge_rewards
    ));
    assert_ok!(Farming::deposit(
        RuntimeOrigin::signed(ALICE),
        pool_id,
        tokens,
        None
    ));

    let share_info = Farming::shares_and_withdrawn_rewards(pool_id, &ALICE).unwrap();
    assert_eq!(share_info.share, tokens);
    (pool_id, tokens)
}

fn init_no_gauge() -> (PoolId, BalanceOf<Runtime>) {
    let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
    let tokens = 1000;
    let basic_rewards = vec![(KSM, 1000)];

    assert_ok!(Farming::create_farming_pool(
        RuntimeOrigin::signed(ALICE),
        tokens_proportion,
        basic_rewards,
        None,
        0,  // min_deposit_to_start
        0,  // after_block_to_start
        10, // withdraw_limit_time
        0,  // claim_limit_time
        1   // withdraw_limit_count
    ));

    let pool_id = 0;
    let charge_rewards = vec![(KSM, 100000)];

    assert_ok!(Farming::charge(
        RuntimeOrigin::signed(BOB),
        pool_id,
        charge_rewards
    ));
    assert_ok!(Farming::deposit(
        RuntimeOrigin::signed(ALICE),
        pool_id,
        tokens,
        None
    ));

    let share_info = Farming::shares_and_withdrawn_rewards(pool_id, &ALICE).unwrap();
    assert_eq!(share_info.share, tokens);
    (pool_id, tokens)
}

#[test]
fn precondition_check_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pool_id0, tokens) = init_gauge_1000();
            let pool_id = 1;
            let charge_rewards = vec![(KSM, 300000)];

            assert_noop!(
                Farming::create_farming_pool(
                    RuntimeOrigin::signed(ALICE),
                    vec![],
                    vec![(KSM, 1000)],
                    None,
                    0,  // min_deposit_to_start
                    0,  // after_block_to_start
                    10, // withdraw_limit_time
                    0,  // claim_limit_time
                    1   // withdraw_limit_count
                ),
                Error::<Runtime>::InvalidPoolParameter
            );

            assert_noop!(
                Farming::charge(RuntimeOrigin::signed(BOB), pool_id, charge_rewards),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::deposit(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    tokens,
                    Some((100, 100))
                ),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::withdraw(RuntimeOrigin::signed(ALICE), pool_id, Some(200)),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::withdraw_claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::reset_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None
                ),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::kill_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::PoolDoesNotExist
            );
            assert_noop!(
                Farming::gauge_withdraw(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::GaugePoolNotExist
            );
            assert_noop!(
                Farming::edit_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    None,
                    None,
                    None,
                    None,
                    None
                ),
                Error::<Runtime>::PoolDoesNotExist
            );

            // Pool state is Charged
            let pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id0).unwrap();
            assert_eq!(pool1.state, PoolState::Charged);
            // Charge again
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id0,
                vec![(KSM, 1000)]
            ));
            let pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id0).unwrap();
            assert_eq!(pool1.state, PoolState::Charged);

            assert_noop!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id0),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id0),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id0),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::reset_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id0,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::kill_pool(RuntimeOrigin::signed(ALICE), pool_id0),
                Error::<Runtime>::InvalidPoolState
            );

            // Pool state is Ongoing
            Farming::on_initialize(System::block_number() + 10);
            let pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id0).unwrap();
            assert_eq!(pool1.state, PoolState::Ongoing);

            assert_noop!(
                Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id0),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::reset_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id0,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::kill_pool(RuntimeOrigin::signed(ALICE), pool_id0),
                Error::<Runtime>::InvalidPoolState
            );

            // Charge again, change back state from Ongoing to Charged.
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id0,
                vec![(KSM, 1000)]
            ));
            let pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id0).unwrap();
            assert_eq!(pool1.state, PoolState::Charged);
        })
}

#[test]
fn no_gauge_farming_pool_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            // Stake KSM, get KSM reward
            let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
            let deposit_amount = 1000;
            let reward_amount: Balance = 800;
            let basic_rewards = vec![(KSM, reward_amount)];
            let total_rewards = 300_000;
            let alice_init_balance = 3000;
            assert_eq!(Assets::balance(KSM, &ALICE), alice_init_balance);

            let withdraw_limit_time = 7;

            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion.clone(),
                basic_rewards.clone(),
                None,
                2, // min_deposit_to_start
                1, // after_block_to_start
                withdraw_limit_time,
                6, // claim_limit_time
                5  // withdraw_limit_count
            ));
            assert_eq!(PoolNextId::<Runtime>::get(), 1);
            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion,
                basic_rewards,
                None,
                2, // min_deposit_to_start
                1, // after_block_to_start
                withdraw_limit_time,
                6, // claim_limit_time
                5  // withdraw_limit_count
            ));
            assert_eq!(PoolNextId::<Runtime>::get(), 2);

            // Query pool initial state, kill first pool with pool id = 0
            assert_eq!(Farming::pool_infos(0).unwrap().state, PoolState::UnCharged);
            assert_ok!(Farming::kill_pool(RuntimeOrigin::signed(ALICE), 0));
            assert_eq!(Farming::pool_infos(0), None);

            // Charge to the pool reward issuer
            let pool_id = 1;
            let charge_rewards = vec![(KSM, total_rewards)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id,
                charge_rewards
            ));
            let mut pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 0);
            assert_eq!(pool1.min_deposit_to_start, 2);
            assert_eq!(pool1.state, PoolState::Charged);
            assert!(pool1.rewards.is_empty());

            // Deposit failed because block number is not large then pool_info.after_block_to_start
            assert!(System::block_number() < pool1.after_block_to_start);
            assert_noop!(
                Farming::deposit(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    deposit_amount,
                    Some((100, 100))
                ),
                Error::<Runtime>::CanNotDeposit
            );

            System::set_block_number(System::block_number() + 3);
            // Deposit failed because gauge pool is not exist
            assert_noop!(
                Farming::deposit(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    deposit_amount,
                    Some((100, 100))
                ),
                Error::<Runtime>::GaugePoolNotExist
            );
            // Deposit success without gauge info
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                deposit_amount,
                None
            ));

            // User staked token transfer to keeper account when deposit
            assert_eq!(
                Assets::balance(KSM, &ALICE),
                alice_init_balance - deposit_amount
            );
            assert_eq!(Assets::balance(KSM, &pool1.keeper), deposit_amount);

            // reward info
            let mut reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(reward.share, deposit_amount);
            assert!(reward.withdrawn_rewards.is_empty());
            assert!(reward.withdraw_list.is_empty());
            assert_eq!(reward.claim_last_block, 3);

            // The pool state is still on `Charged` until new block produced
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, deposit_amount);
            assert_eq!(pool1.state, PoolState::Charged);
            assert!(pool1.rewards.is_empty());

            // Can't Claim if pool state is `Charged`
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );

            // OnInitialize hook change the pool state and also pool rewards
            Farming::on_initialize(System::block_number() + 3);
            Farming::on_initialize(0);
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, deposit_amount);
            assert_eq!(pool1.state, PoolState::Ongoing);
            assert_eq!(pool1.claim_limit_time, 6);
            assert_eq!(pool1.rewards.get(&KSM).unwrap(), &(reward_amount, 0));

            // Produce new block didn't change the reward info
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(reward.share, deposit_amount);
            assert!(reward.withdrawn_rewards.is_empty());
            assert!(reward.withdraw_list.is_empty());
            assert_eq!(reward.claim_last_block, 3);

            // Claim failed because of user share info not exist
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(BOB), pool_id),
                Error::<Runtime>::ShareInfoNotExists
            );
            // Claim failed because of block not reached, at least 3+6=9
            assert!(System::block_number() < reward.claim_last_block + pool1.claim_limit_time);
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::CanNotClaim
            );

            // Claim success, user get reward.
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(
                Assets::balance(KSM, &ALICE),
                alice_init_balance - deposit_amount + reward_amount
            );
            assert_eq!(
                Assets::balance(KSM, &pool1.reward_issuer),
                total_rewards - reward_amount
            );

            // Claim operation update pool info's rewards and also share info's withdrawn_rewards
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(
                pool1.rewards.get(&KSM).unwrap(),
                &(reward_amount, reward_amount)
            );
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(reward.withdrawn_rewards.get(&KSM).unwrap(), &reward_amount);
            // The withdraw list of user share info is still empty.
            assert!(reward.withdraw_list.is_empty());

            // Claim without new block.
            for i in 0..5 {
                System::set_block_number(System::block_number() + 100);
                assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));

                reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
                assert_eq!(reward.withdrawn_rewards.get(&KSM).unwrap(), &reward_amount);
                assert_eq!(reward.claim_last_block, 109 + i * 100);
                assert_eq!(reward.share, deposit_amount);
                assert!(reward.withdraw_list.is_empty());

                pool1 = Farming::pool_infos(pool_id).unwrap();
                assert_eq!(
                    pool1.rewards.get(&KSM).unwrap(),
                    &(reward_amount, reward_amount)
                );
                assert_eq!(pool1.total_shares, deposit_amount);

                assert_eq!(
                    Assets::balance(KSM, &ALICE),
                    alice_init_balance - deposit_amount + reward_amount
                );
                assert_eq!(
                    Assets::balance(KSM, &pool1.reward_issuer),
                    total_rewards - reward_amount
                );
                assert_eq!(Assets::balance(KSM, &pool1.keeper), deposit_amount);
            }
            assert_eq!(
                Assets::balance(KSM, &pool1.reward_issuer),
                total_rewards - reward_amount
            );
            assert_eq!(Assets::balance(KSM, &pool1.keeper), deposit_amount);
            // Claim with new block
            for i in 1..5 {
                System::set_block_number(System::block_number() + 6);
                Farming::on_initialize(System::block_number() + 3);
                assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));

                reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
                assert_eq!(
                    reward.withdrawn_rewards.get(&KSM).unwrap(),
                    &(reward_amount * (i + 1))
                );
                assert_eq!(reward.claim_last_block as u128, 509 + i * 6);
                assert_eq!(reward.share, deposit_amount);
                assert!(reward.withdraw_list.is_empty());

                pool1 = Farming::pool_infos(pool_id).unwrap();
                assert_eq!(
                    pool1.rewards.get(&KSM).unwrap(),
                    &(reward_amount * (i + 1), reward_amount * (i + 1))
                );
                assert_eq!(pool1.total_shares, deposit_amount);

                assert_eq!(
                    Assets::balance(KSM, &ALICE),
                    alice_init_balance - deposit_amount + reward_amount * (i + 1)
                );
                assert_eq!(
                    Assets::balance(KSM, &pool1.reward_issuer),
                    total_rewards - reward_amount * (i + 1)
                );
                // Because withdraw_list of user share is empty, keeper not return token to user.
                assert_eq!(Assets::balance(KSM, &pool1.keeper), deposit_amount);
            }
            assert_eq!(
                Assets::balance(KSM, &ALICE),
                alice_init_balance - deposit_amount + reward_amount * 5
            );
            assert_eq!(
                Assets::balance(KSM, &pool1.reward_issuer),
                total_rewards - reward_amount * 5
            );
            assert_eq!(Assets::balance(KSM, &pool1.keeper), deposit_amount);
            assert_eq!(
                pool1.rewards.get(&KSM).unwrap(),
                &(reward_amount * 5, reward_amount * 5)
            );

            // Withdraw failed because of share info not exist.
            assert_err!(
                Farming::withdraw(RuntimeOrigin::signed(BOB), pool_id, Some(800)),
                Error::<Runtime>::ShareInfoNotExists
            );

            // Claim again without new blocks, no new rewards
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert!(reward.withdraw_list.is_empty());

            let share_reward = reward.withdrawn_rewards.get(&KSM).unwrap();
            assert_eq!(share_reward, &(reward_amount * 5));
            let (total_reward, total_withdrawn_reward) = pool1.rewards.get(&KSM).unwrap();
            assert_eq!(
                pool1.rewards.get(&KSM).unwrap(),
                &(reward_amount * 5, reward_amount * 5)
            );

            let reward_amount1 = Farming::get_reward_amount(
                &reward,
                total_reward,
                total_withdrawn_reward,
                pool1.total_shares,
                &KSM,
            )
            .unwrap();
            assert_eq!(reward_amount1, (reward_amount * 5, 0));
            let reward_inflation =
                Farming::get_reward_inflation(reward.share, total_reward, pool1.total_shares);
            assert_eq!(reward_inflation, reward_amount * 5);

            let reward_inflation = Farming::get_reward_inflation(800, share_reward, reward.share);
            assert_eq!(reward_inflation, share_reward * 8 / 10);
            let reward_inflation = Farming::get_reward_inflation(200, share_reward, reward.share);
            assert_eq!(reward_inflation, share_reward * 2 / 10);
            let reward_inflation = Farming::get_reward_inflation(100, share_reward, reward.share);
            assert_eq!(reward_inflation, share_reward / 10);

            // Withdraw partial tokens
            assert_eq!(System::block_number(), 533);
            assert_eq!(
                Assets::balance(KSM, &ALICE),
                alice_init_balance - deposit_amount + reward_amount * 5
            );
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                Some(800)
            ));

            // Although withdraw also has claim, but no new rewards due to no new block
            // So both user and reward issuer account balance not change.
            assert_eq!(
                Assets::balance(KSM, &ALICE),
                alice_init_balance - deposit_amount + reward_amount * 5
            );
            assert_eq!(
                Assets::balance(KSM, &pool1.reward_issuer),
                total_rewards - reward_amount * 5
            );
            assert_eq!(Assets::balance(KSM, &pool1.keeper), deposit_amount);

            // Withdraw operation has only one operation `remove_share`.
            // And `remove_share` will claim rewards and also update user share info.
            // We already know that due to no new block, claim rewards actually has no reward.
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            pool1 = Farming::pool_infos(pool_id).unwrap();

            assert_eq!(pool1.total_shares, 200);
            assert_eq!(reward.withdraw_list, vec![(533 + withdraw_limit_time, 800)]);
            assert_eq!(reward.share, 200);
            assert_eq!(reward.withdrawn_rewards.get(&KSM).unwrap(), &reward_amount);
            assert_eq!(
                pool1.rewards.get(&KSM).unwrap(),
                &(reward_amount, reward_amount)
            );

            System::set_block_number(System::block_number() + 6);
            Farming::on_initialize(System::block_number() + 3);

            // Withdraw rest all of share
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                Some(300)
            ));
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 0);
            assert!(pool1.rewards.is_empty());
            assert_eq!(reward.share, 0);
            assert_eq!(reward.withdraw_list, vec![(540, 800), (546, 200)]);
            assert_eq!(reward.withdrawn_rewards.get(&KSM).unwrap(), &0);
        })
}

#[test]
fn no_gauge_staking_cases_should_work() {
    staking_case(KSM, KMA);
    staking_case(KMA, KSM);
    staking_all_kma(KMA, KSM);
    staking_minimum_kma(KMA, KSM);
}

fn staking_minimum_kma(stake_token: CalamariAssetId, reward_token: CalamariAssetId) {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let tokens_proportion = vec![(stake_token, Perbill::from_percent(100))];
            let reward_amount: Balance = 1;
            let basic_rewards = vec![(reward_token, reward_amount)];
            let total_rewards = 300_000;
            let charlie_init_kma = 1;
            assert_eq!(Balances::free_balance(&CHARLIE), charlie_init_kma);

            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion,
                basic_rewards,
                None,
                1, // min_deposit_to_start
                1, // after_block_to_start
                7, // withdraw_limit_time,
                6, // claim_limit_time
                5  // withdraw_limit_count
            ));

            // Charge to the pool reward issuer
            let pool_id = 0;
            let charge_rewards = vec![(reward_token, total_rewards)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id,
                charge_rewards
            ));

            System::set_block_number(System::block_number() + 3);
            // Deposit success without gauge info
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(CHARLIE),
                pool_id,
                charlie_init_kma,
                None
            ));

            let pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(Balances::free_balance(&CHARLIE), 0);
            assert_eq!(Balances::free_balance(&pool1.keeper), charlie_init_kma);

            Farming::on_initialize(System::block_number() + 10);
            System::set_block_number(System::block_number() + 10);
            assert_eq!(Balances::free_balance(&CHARLIE), 0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(CHARLIE), pool_id));
            assert_eq!(Balances::free_balance(&CHARLIE), 0);

            System::set_block_number(System::block_number() + 10);
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(CHARLIE),
                pool_id,
                None
            ));
            assert_eq!(Balances::free_balance(&CHARLIE), 0);
            let reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &CHARLIE).unwrap();
            assert_eq!(reward.withdraw_list, vec![(30, charlie_init_kma)]);

            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(CHARLIE),
                pool_id
            ));
            assert_eq!(Balances::free_balance(&CHARLIE), 0);

            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(CHARLIE),
                pool_id
            ));
            assert_eq!(Balances::free_balance(&CHARLIE), charlie_init_kma);
            assert_eq!(Balances::free_balance(pool1.keeper), 0);
        });
}

fn staking_all_kma(stake_token: CalamariAssetId, reward_token: CalamariAssetId) {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let tokens_proportion = vec![(stake_token, Perbill::from_percent(100))];
            let reward_amount: Balance = 800;
            let basic_rewards = vec![(reward_token, reward_amount)];
            let total_rewards = 300_000;
            let alice_init_kma = 3000;
            assert_eq!(Balances::free_balance(&ALICE), alice_init_kma);

            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion,
                basic_rewards,
                None,
                2, // min_deposit_to_start
                1, // after_block_to_start
                7, // withdraw_limit_time,
                6, // claim_limit_time
                5  // withdraw_limit_count
            ));

            // Charge to the pool reward issuer
            let pool_id = 0;
            let charge_rewards = vec![(reward_token, total_rewards)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id,
                charge_rewards
            ));

            System::set_block_number(System::block_number() + 3);
            // Deposit success without gauge info
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                alice_init_kma,
                None
            ));

            let pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(Balances::free_balance(&ALICE), 0);
            assert_eq!(Balances::free_balance(&pool1.keeper), alice_init_kma);

            Farming::on_initialize(System::block_number() + 10);
            System::set_block_number(System::block_number() + 10);
            assert_eq!(Balances::free_balance(&ALICE), 0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Balances::free_balance(&ALICE), 0);

            System::set_block_number(System::block_number() + 10);
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                None
            ));
            assert_eq!(Balances::free_balance(&ALICE), 0);
            let reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(reward.withdraw_list, vec![(30, alice_init_kma)]);

            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            assert_eq!(Balances::free_balance(&ALICE), 0);

            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            assert_eq!(Balances::free_balance(&ALICE), alice_init_kma);
            assert_eq!(Balances::free_balance(pool1.keeper), 0);
        });
}

fn staking_case(stake_token: CalamariAssetId, reward_token: CalamariAssetId) {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let tokens_proportion = vec![(stake_token, Perbill::from_percent(100))];
            let deposit_amount = 1000;
            let reward_amount: Balance = 800;
            let basic_rewards = vec![(reward_token, reward_amount)];
            let total_rewards = 300_000;
            let alice_init_ksm = 3000;
            let alice_init_kma = 3000;
            assert_eq!(Balances::free_balance(&ALICE), alice_init_kma);

            let withdraw_limit_time = 7;

            assert_ok!(Farming::create_farming_pool(
                RuntimeOrigin::signed(ALICE),
                tokens_proportion,
                basic_rewards,
                None,
                2, // min_deposit_to_start
                1, // after_block_to_start
                withdraw_limit_time,
                6, // claim_limit_time
                5  // withdraw_limit_count
            ));

            // Charge to the pool reward issuer
            let pool_id = 0;
            let charge_rewards = vec![(reward_token, total_rewards)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id,
                charge_rewards
            ));

            let mut pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 0);
            assert_eq!(pool1.min_deposit_to_start, 2);
            assert_eq!(pool1.state, PoolState::Charged);
            assert!(pool1.rewards.is_empty());

            System::set_block_number(System::block_number() + 3);
            // Deposit success without gauge info
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                deposit_amount,
                None
            ));

            // User staked token transfer to keeper account when deposit
            if stake_token == KSM {
                // Stake KSM, reward KMA
                assert_eq!(
                    Assets::balance(stake_token, &ALICE),
                    alice_init_ksm - deposit_amount
                );
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), deposit_amount);
                assert_eq!(Balances::free_balance(&ALICE), alice_init_kma);
            } else {
                // Stake KMA, reward KSM
                assert_eq!(
                    Balances::free_balance(&ALICE),
                    alice_init_kma - deposit_amount
                );
                assert_eq!(Balances::free_balance(&pool1.keeper), deposit_amount);
            }
            if reward_token == KSM {
                // Stake KMA, reward KSM
                assert_eq!(
                    Assets::balance(reward_token, &pool1.reward_issuer),
                    total_rewards
                );
            } else {
                // Stake KSM, reward KMA
                assert_eq!(Balances::free_balance(&pool1.reward_issuer), total_rewards);
            }

            // reward info
            let mut reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(reward.share, deposit_amount);
            assert!(reward.withdrawn_rewards.is_empty());
            assert!(reward.withdraw_list.is_empty());
            assert_eq!(reward.claim_last_block, 3);

            // The pool state is still on `Charged` until new block produced
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, deposit_amount);
            assert_eq!(pool1.state, PoolState::Charged);
            assert!(pool1.rewards.is_empty());

            // OnInitialize hook change the pool state and also pool rewards
            Farming::on_initialize(System::block_number() + 3);
            Farming::on_initialize(0);
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, deposit_amount);
            assert_eq!(pool1.state, PoolState::Ongoing);
            assert_eq!(pool1.claim_limit_time, 6);
            assert_eq!(
                pool1.rewards.get(&reward_token).unwrap(),
                &(reward_amount, 0)
            );

            // Produce new block didn't change the reward info
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(reward.share, deposit_amount);
            assert!(reward.withdrawn_rewards.is_empty());
            assert!(reward.withdraw_list.is_empty());
            assert_eq!(reward.claim_last_block, 3);

            // Claim success, user get reward.
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            if reward_token == KSM {
                // Stake KMA, reward KSM
                assert_eq!(
                    Assets::balance(reward_token, &ALICE),
                    alice_init_ksm + reward_amount
                );
                assert_eq!(
                    Assets::balance(reward_token, &pool1.reward_issuer),
                    total_rewards - reward_amount
                );
            } else {
                // Stake KSM, reward KMA
                assert_eq!(
                    Balances::free_balance(&ALICE),
                    alice_init_kma + reward_amount
                );
                assert_eq!(
                    Balances::free_balance(&pool1.reward_issuer),
                    total_rewards - reward_amount
                );
            }

            // Claim operation update pool info's rewards and also share info's withdrawn_rewards
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(
                pool1.rewards.get(&reward_token).unwrap(),
                &(reward_amount, reward_amount)
            );
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            assert_eq!(
                reward.withdrawn_rewards.get(&reward_token).unwrap(),
                &reward_amount
            );
            // The withdraw list of user share info is still empty.
            assert!(reward.withdraw_list.is_empty());

            // Claim without new block.
            for i in 0..5 {
                System::set_block_number(System::block_number() + 100);
                assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));

                reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
                assert_eq!(
                    reward.withdrawn_rewards.get(&reward_token).unwrap(),
                    &reward_amount
                );
                assert_eq!(reward.claim_last_block, 109 + i * 100);
                assert_eq!(reward.share, deposit_amount);
                assert!(reward.withdraw_list.is_empty());

                pool1 = Farming::pool_infos(pool_id).unwrap();
                assert_eq!(
                    pool1.rewards.get(&reward_token).unwrap(),
                    &(reward_amount, reward_amount)
                );
                assert_eq!(pool1.total_shares, deposit_amount);
            }
            if reward_token == KSM {
                // Stake KMA, reward KSM
                assert_eq!(
                    Assets::balance(reward_token, &pool1.reward_issuer),
                    total_rewards - reward_amount
                );
            } else {
                // Stake KSM, reward KMA
                assert_eq!(
                    Balances::free_balance(&pool1.reward_issuer),
                    total_rewards - reward_amount
                );
            }
            if stake_token == KSM {
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), deposit_amount);
            } else {
                assert_eq!(Balances::free_balance(&pool1.keeper), deposit_amount);
            }

            // Claim with new block
            for i in 1..5 {
                System::set_block_number(System::block_number() + 6);
                Farming::on_initialize(System::block_number() + 3);
                assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));

                reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
                assert_eq!(
                    reward.withdrawn_rewards.get(&reward_token).unwrap(),
                    &(reward_amount * (i + 1))
                );
                assert_eq!(reward.claim_last_block as u128, 509 + i * 6);
                assert_eq!(reward.share, deposit_amount);
                assert!(reward.withdraw_list.is_empty());

                pool1 = Farming::pool_infos(pool_id).unwrap();
                assert_eq!(
                    pool1.rewards.get(&reward_token).unwrap(),
                    &(reward_amount * (i + 1), reward_amount * (i + 1))
                );
                assert_eq!(pool1.total_shares, deposit_amount);
            }
            if reward_token == KSM {
                // Stake KMA, reward another 4 times KSM
                assert_eq!(
                    Assets::balance(reward_token, &ALICE),
                    alice_init_ksm + reward_amount * 5
                );
                assert_eq!(
                    Assets::balance(reward_token, &pool1.reward_issuer),
                    total_rewards - reward_amount * 5
                );
            } else {
                // Stake KSM, reward another 4 times KMA
                assert_eq!(
                    Balances::free_balance(&ALICE),
                    alice_init_kma + reward_amount * 5
                );
                assert_eq!(
                    Balances::free_balance(&pool1.reward_issuer),
                    total_rewards - reward_amount * 5
                );
            }
            if stake_token == KSM {
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), deposit_amount);
            } else {
                assert_eq!(Balances::free_balance(&pool1.keeper), deposit_amount);
            }
            assert_eq!(
                pool1.rewards.get(&reward_token).unwrap(),
                &(reward_amount * 5, reward_amount * 5)
            );

            // Withdraw failed because of share info not exist.
            assert_err!(
                Farming::withdraw(RuntimeOrigin::signed(BOB), pool_id, Some(800)),
                Error::<Runtime>::ShareInfoNotExists
            );

            // Claim again without new blocks, no new rewards
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert!(reward.withdraw_list.is_empty());

            let share_reward = reward.withdrawn_rewards.get(&reward_token).unwrap();
            assert_eq!(share_reward, &(reward_amount * 5));
            let (total_reward, total_withdrawn_reward) = pool1.rewards.get(&reward_token).unwrap();
            assert_eq!(
                pool1.rewards.get(&reward_token).unwrap(),
                &(reward_amount * 5, reward_amount * 5)
            );

            let reward_amount1 = Farming::get_reward_amount(
                &reward,
                total_reward,
                total_withdrawn_reward,
                pool1.total_shares,
                &reward_token,
            )
            .unwrap();
            assert_eq!(reward_amount1, (reward_amount * 5, 0));
            let reward_inflation =
                Farming::get_reward_inflation(reward.share, total_reward, pool1.total_shares);
            assert_eq!(reward_inflation, reward_amount * 5);

            let reward_inflation = Farming::get_reward_inflation(800, share_reward, reward.share);
            assert_eq!(reward_inflation, share_reward * 8 / 10);
            let reward_inflation = Farming::get_reward_inflation(200, share_reward, reward.share);
            assert_eq!(reward_inflation, share_reward * 2 / 10);
            let reward_inflation = Farming::get_reward_inflation(100, share_reward, reward.share);
            assert_eq!(reward_inflation, share_reward / 10);

            // Withdraw partial tokens
            assert_eq!(System::block_number(), 533);
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                Some(800)
            ));
            // Although withdraw also has claim, but no new rewards due to no new block
            // So both user and reward issuer account balance not change.
            if reward_token == KSM {
                // Stake KMA, reward KSM
                assert_eq!(
                    Assets::balance(reward_token, &ALICE),
                    alice_init_ksm + reward_amount * 5
                );
                assert_eq!(
                    Assets::balance(reward_token, &pool1.reward_issuer),
                    total_rewards - reward_amount * 5
                );
            } else {
                // Stake KSM, reward KMA
                assert_eq!(
                    Balances::free_balance(&ALICE),
                    alice_init_kma + reward_amount * 5
                );
                assert_eq!(
                    Balances::free_balance(&pool1.reward_issuer),
                    total_rewards - reward_amount * 5
                );
            }
            if stake_token == KSM {
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), deposit_amount);
            } else {
                assert_eq!(Balances::free_balance(&pool1.keeper), deposit_amount);
            }

            // Withdraw operation has only one operation `remove_share`.
            // And `remove_share` will claim rewards and also update user share info.
            // We already know that due to no new block, claim rewards actually has no reward.
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            pool1 = Farming::pool_infos(pool_id).unwrap();

            assert_eq!(pool1.total_shares, 200);
            assert_eq!(reward.withdraw_list, vec![(533 + withdraw_limit_time, 800)]);
            assert_eq!(reward.share, 200);
            assert_eq!(
                reward.withdrawn_rewards.get(&reward_token).unwrap(),
                &reward_amount
            );
            assert_eq!(
                pool1.rewards.get(&reward_token).unwrap(),
                &(reward_amount, reward_amount)
            );

            System::set_block_number(System::block_number() + 6);
            Farming::on_initialize(System::block_number() + 3);

            // Withdraw rest all of share
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                Some(300)
            ));
            reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 0);
            assert!(pool1.rewards.is_empty());
            assert_eq!(reward.share, 0);
            assert_eq!(reward.withdraw_list, vec![(540, 800), (546, 200)]);
            assert_eq!(reward.withdrawn_rewards.get(&reward_token).unwrap(), &0);
            if reward_token == KSM {
                // Stake KMA, reward KSM
                assert_eq!(
                    Assets::balance(reward_token, &ALICE),
                    alice_init_ksm + reward_amount * 6
                );
                assert_eq!(
                    Assets::balance(reward_token, &pool1.reward_issuer),
                    total_rewards - reward_amount * 6
                );
            } else {
                // Stake KSM, reward KMA
                assert_eq!(
                    Balances::free_balance(&ALICE),
                    alice_init_kma + reward_amount * 6
                );
                assert_eq!(
                    Balances::free_balance(&pool1.reward_issuer),
                    total_rewards - reward_amount * 6
                );
            }
            if stake_token == KSM {
                // Stake KSM, reward KMA
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), deposit_amount);
            } else {
                // Stake KMA, reward KSM
                assert_eq!(Balances::free_balance(&pool1.keeper), deposit_amount);
            }

            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            if stake_token == KSM {
                // Stake KSM, reward KMA
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), 200);
            } else {
                // Stake KMA, reward KSM
                assert_eq!(Balances::free_balance(&pool1.keeper), 200);
            }

            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            if stake_token == KSM {
                // Stake KSM, reward KMA
                assert_eq!(Assets::balance(stake_token, &pool1.keeper), 0);
            } else {
                // Stake KMA, reward KSM
                assert_eq!(Balances::free_balance(&pool1.keeper), 0);
            }
        })
}

#[test]
fn gauge_farming_pool_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            // Stake KSM, get KSM reward
            let tokens_proportion = vec![(KSM, Perbill::from_percent(100))];
            let deposit_amount = 1000;
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
            assert_eq!(PoolNextId::<Runtime>::get(), 1);
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
            assert_eq!(PoolNextId::<Runtime>::get(), 2);

            // Query pool initial state, kill the pool
            assert_eq!(Farming::pool_infos(0).unwrap().state, PoolState::UnCharged);
            assert_ok!(Farming::kill_pool(RuntimeOrigin::signed(ALICE), 0));
            assert_eq!(Farming::pool_infos(0), None);

            // Charge to the pool
            let pool_id = 1;
            let charge_rewards = vec![(KSM, 300000)];
            assert_ok!(Farming::charge(
                RuntimeOrigin::signed(BOB),
                pool_id,
                charge_rewards
            ));
            let mut pool1: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 0);
            assert_eq!(pool1.min_deposit_to_start, 2);
            assert_eq!(pool1.state, PoolState::Charged);

            // Deposit to the pool
            assert_err!(
                Farming::deposit(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    deposit_amount,
                    Some((100, 100))
                ),
                Error::<Runtime>::CanNotDeposit
            );
            System::set_block_number(System::block_number() + 3);
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                deposit_amount,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 1000);
            assert_eq!(pool1.min_deposit_to_start, 2);
            assert_eq!(pool1.state, PoolState::Charged);

            // OnInitialize hook change the pool state
            Farming::on_initialize(System::block_number() + 3);
            Farming::on_initialize(0);
            pool1 = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool1.total_shares, 1000);
            assert_eq!(pool1.min_deposit_to_start, 2);
            assert_eq!(pool1.state, PoolState::Ongoing);

            // Claim to get rewards
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::CanNotClaim
            );
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 3008);

            System::set_block_number(System::block_number() + 100);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 4698);

            // Withdraw part tokens
            assert_ok!(Farming::withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                Some(800)
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 4698);

            // Claim again
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::CanNotClaim
            );
            assert_eq!(Assets::balance(KSM, &ALICE), 4698);
            System::set_block_number(System::block_number() + 6);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 5498);
        })
}

#[test]
fn deposit_no_gauge_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            deposit_should_work(false);
        })
}

#[test]
fn deposit_gauge_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            deposit_should_work(true);
        })
}

fn deposit_should_work(use_gauge: bool) {
    assert_eq!(Assets::balance(KSM, &ALICE), 3000);
    let (pool_id, tokens) = if use_gauge {
        init_gauge_1000()
    } else {
        init_no_gauge()
    };
    let keeper: AccountId = <Runtime as Config>::Keeper::get().into_sub_account_truncating(pool_id);
    let reward_issuer: AccountId =
        <Runtime as Config>::RewardIssuer::get().into_sub_account_truncating(pool_id);

    assert_eq!(Assets::balance(KSM, &ALICE), 2000);
    assert_eq!(Assets::balance(KSM, &keeper), 1000);

    System::set_block_number(System::block_number() + 1);
    if use_gauge {
        assert_ok!(Farming::deposit(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            tokens,
            Some((100, 100))
        ));
        assert_eq!(Assets::balance(KSM, &ALICE), 900);
        assert_eq!(Assets::balance(KSM, &keeper), 2100);

        System::set_block_number(System::block_number() + 1);
        assert_ok!(Farming::deposit(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            0,
            Some((100, 100))
        ));
        assert_eq!(Assets::balance(KSM, &ALICE), 800);
        assert_eq!(Assets::balance(KSM, &keeper), 2200);
    } else {
        assert_ok!(Farming::deposit(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            tokens,
            None
        ));
        assert_eq!(Assets::balance(KSM, &ALICE), 1000);
        assert_eq!(Assets::balance(KSM, &keeper), 2000);

        System::set_block_number(System::block_number() + 1);
        assert_ok!(Farming::deposit(
            RuntimeOrigin::signed(ALICE),
            pool_id,
            100,
            None
        ));
        assert_eq!(Assets::balance(KSM, &ALICE), 900);
        assert_eq!(Assets::balance(KSM, &keeper), 2100);
    }

    if use_gauge {
        let mut gauge_basic_rewards = BTreeMap::<CurrencyIdOf<Runtime>, BalanceOf<Runtime>>::new();
        gauge_basic_rewards.entry(KSM).or_insert(tokens);
        let gauge_pool_info = GaugePoolInfo {
            pool_id,
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
        assert_eq!(Farming::gauge_pool_infos(0), Some(gauge_pool_info));
    } else {
        assert_eq!(Farming::gauge_pool_infos(0), None);
    }

    Farming::on_initialize(0);
    assert_eq!(Farming::pool_infos(0).unwrap().state, PoolState::Ongoing);
    assert!(Farming::pool_infos(0).unwrap().rewards.is_empty());
    if use_gauge {
        assert_eq!(
            Farming::gauge_pool_infos(0).unwrap().rewards.get(&KSM),
            Some(&(1000, 0, 0))
        );
    }

    Farming::on_initialize(0);
    assert_eq!(
        Farming::pool_infos(0).unwrap().rewards.get(&KSM),
        Some(&(1000, 0))
    );
    if use_gauge {
        assert_eq!(
            Farming::gauge_pool_infos(0).unwrap().rewards.get(&KSM),
            Some(&(2000, 0, 0))
        );
    }

    System::set_block_number(System::block_number() + 1000);
    Farming::on_initialize(0);
    assert_eq!(
        Farming::pool_infos(0).unwrap().rewards.get(&KSM),
        Some(&(2000, 0))
    );
    if use_gauge {
        assert_eq!(
            Farming::gauge_pool_infos(0).unwrap().rewards.get(&KSM),
            Some(&(3000, 0, 0))
        );
    }
}

#[test]
fn withdraw_gauge_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            withdraw_should_work(true);
        })
}

#[test]
fn withdraw_no_gauge_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| withdraw_should_work(false))
}

fn withdraw_should_work(use_gauge: bool) {
    let (pool_id, tokens) = if use_gauge {
        init_gauge_1000()
    } else {
        init_no_gauge()
    };
    let pool = PoolInfos::<Runtime>::get(pool_id).unwrap();
    let reward_issuer = pool.reward_issuer;
    let token_keeper = pool.keeper;

    assert_eq!(Assets::balance(KSM, &ALICE), 2_000);
    assert_eq!(Assets::balance(KSM, &reward_issuer), 100_000);
    assert_eq!(Assets::balance(KSM, &token_keeper), 1_000);

    Farming::on_initialize(0);
    Farming::on_initialize(0);
    System::set_block_number(System::block_number() + 1);

    let reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
    assert!(reward.withdraw_list.is_empty());
    assert_eq!(pool.withdraw_limit_count, 1);

    // withdraw
    assert_ok!(Farming::withdraw(
        RuntimeOrigin::signed(ALICE),
        pool_id,
        Some(800)
    ));

    // withdraw contains claim reward operation
    // reward issuer transfer 1000 reward token to user
    assert_eq!(Assets::balance(KSM, &reward_issuer), 99_000);
    assert_eq!(Assets::balance(KSM, &ALICE), 3_000);

    let reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
    assert_eq!(reward.withdraw_list.len(), 1);
    // user share info withdraw list size is not less than pool info `withdraw_limit_count`
    assert_err!(
        Farming::withdraw(RuntimeOrigin::signed(ALICE), pool_id, Some(100)),
        Error::<Runtime>::WithdrawLimitCountExceeded
    );

    // Alice claim reward manually, but due to no new block, so no reward
    assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
    assert_eq!(Assets::balance(KSM, &ALICE), 3_000);
    assert_eq!(Assets::balance(KSM, &reward_issuer), 99_000);
    assert_eq!(Assets::balance(KSM, &token_keeper), 1_000);

    // Bob deposit
    System::set_block_number(System::block_number() + 100);
    if use_gauge {
        assert_ok!(Farming::deposit(
            RuntimeOrigin::signed(BOB),
            pool_id,
            tokens,
            Some((100, 100))
        ));
    } else {
        assert_ok!(Farming::deposit(
            RuntimeOrigin::signed(BOB),
            pool_id,
            tokens,
            None
        ));
    }

    // deposit operation transfer user staked token to pool keeper account
    if use_gauge {
        assert_eq!(Assets::balance(KSM, &token_keeper), 2_100);
    } else {
        assert_eq!(Assets::balance(KSM, &token_keeper), 2_000);
    }

    // Alice claim again, because new block produces, so has reward now
    Farming::on_initialize(0);
    assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
    assert_eq!(Assets::balance(KSM, &ALICE), 3_966);
    assert_eq!(Assets::balance(KSM, &reward_issuer), 98_834);

    // withdraw
    assert_ok!(Farming::withdraw(
        RuntimeOrigin::signed(ALICE),
        pool_id,
        Some(200)
    ));
    assert_eq!(Assets::balance(KSM, &ALICE), 3_966);
    assert_eq!(Assets::balance(KSM, &reward_issuer), 98_834);
    if use_gauge {
        assert_eq!(Assets::balance(KSM, &token_keeper), 1_300);
    } else {
        assert_eq!(Assets::balance(KSM, &token_keeper), 1_200);
    }

    // `withdraw_claim` operation will transfer back user stake token
    // User unStake 200 KSM, so keeper transfer back 200 KSM to user.
    System::set_block_number(System::block_number() + 100);
    assert_ok!(Farming::withdraw_claim(
        RuntimeOrigin::signed(ALICE),
        pool_id
    ));
    assert_eq!(Assets::balance(KSM, &ALICE), 4_166);
    assert_eq!(Assets::balance(KSM, &reward_issuer), 98_834);
    if use_gauge {
        assert_eq!(Assets::balance(KSM, &token_keeper), 1_100);
    } else {
        assert_eq!(Assets::balance(KSM, &token_keeper), 1_000);
    }

    // claim
    let reward = SharesAndWithdrawnRewards::<Runtime>::get(pool_id, &ALICE).unwrap();
    assert!(reward.withdraw_list.is_empty());
    assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
    assert_eq!(Assets::balance(KSM, &ALICE), 4_166);
    assert_eq!(Assets::balance(KSM, &reward_issuer), 98_834);
    if use_gauge {
        assert_eq!(Assets::balance(KSM, &token_keeper), 1_100);
    } else {
        assert_eq!(Assets::balance(KSM, &token_keeper), 1_000);
    }

    // `process_withdraw_list` remove the share info.
    // due to withdraw_list of share info is empty, so there's no token transfer.
    assert_eq!(Farming::shares_and_withdrawn_rewards(pool_id, &ALICE), None);
    assert_eq!(Assets::balance(KSM, &TREASURY_ACCOUNT), 0);
}

#[test]
fn claim() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pool_id, _tokens) = init_gauge_1000();
            let keeper: AccountId =
                <Runtime as Config>::Keeper::get().into_sub_account_truncating(pool_id);
            let reward_issuer: AccountId =
                <Runtime as Config>::RewardIssuer::get().into_sub_account_truncating(pool_id);

            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            assert_err!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );

            System::set_block_number(System::block_number() + 100);
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 2000);
            assert_eq!(Assets::balance(KSM, &keeper), 1000);
            assert_eq!(Assets::balance(KSM, &reward_issuer), 100_000);

            Farming::on_initialize(0);
            assert_ok!(Farming::withdraw_claim(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 2000);

            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            assert_eq!(Assets::balance(KSM, &keeper), 1000);
            assert_eq!(Assets::balance(KSM, &reward_issuer), 99_000);

            Farming::on_initialize(0);
            assert_ok!(Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id));

            assert_ok!(Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 5000); // 3000 + 1000 + 1000
            assert_eq!(Assets::balance(KSM, &keeper), 0);
            assert_eq!(Assets::balance(KSM, &reward_issuer), 98_000);
        });
}

#[test]
fn gauge() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pool_id, tokens) = init_gauge_900();
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            if let Some(gauge_pool_infos) = Farming::gauge_pool_infos(0) {
                assert!(gauge_pool_infos.rewards.is_empty());
            };
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 2918);

            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 10);
            assert_noop!(
                Farming::deposit(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    tokens,
                    Some((100, 2000))
                ),
                Error::<Runtime>::GaugeMaxBlockOverflow
            );
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                tokens,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 1818);

            System::set_block_number(System::block_number() + 20);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 3163);

            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(BOB),
                pool_id,
                10,
                None
            ));
            assert_eq!(Assets::balance(KSM, &BOB), 9699990);

            System::set_block_number(System::block_number() + 200);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 5383);
            assert_eq!(Assets::balance(KSM, &BOB), 9699990);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(BOB),
                pool_id,
                0,
                Some((100, 100))
            ));

            System::set_block_number(System::block_number() + 200);
            assert_noop!(
                Farming::force_gauge_claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::RetireLimitNotSet
            );
            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            assert_ok!(Farming::force_gauge_claim(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            assert_eq!(Assets::balance(KSM, &BOB), 9699991);

            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                tokens,
                Some((100, 1))
            ));
            System::set_block_number(System::block_number() + 200);
            assert_noop!(
                Farming::deposit(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    tokens,
                    Some((100, 1))
                ),
                Error::<Runtime>::LastGaugeNotClaim
            );
        })
}

#[test]
fn gauge_withdraw() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pool_id, _tokens) = init_gauge_900();
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            if let Some(gauge_pool_infos) = Farming::gauge_pool_infos(0) {
                assert_eq!(gauge_pool_infos.gauge_amount, 100)
            };
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            Farming::on_initialize(0);
            assert_ok!(Farming::gauge_withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 1918);
            System::set_block_number(System::block_number() + 1000);
            assert_ok!(Farming::gauge_withdraw(
                RuntimeOrigin::signed(ALICE),
                pool_id
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 3782);
            if let Some(gauge_pool_infos) = Farming::gauge_pool_infos(0) {
                assert_eq!(gauge_pool_infos.gauge_amount, 0)
            };
        })
}

#[test]
fn pool_admin_operation_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pool_id, tokens) = init_gauge_1000();
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                tokens,
                Some((100, 100))
            ));
            System::set_block_number(System::block_number() + 1);
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                0,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 800);

            // Not allow retire or reset, kill pool if pool is not Dead
            assert_noop!(
                Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::kill_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::reset_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None
                ),
                Error::<Runtime>::InvalidPoolState
            );

            // Close the pool
            let pool: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool.state, PoolState::Ongoing);
            assert_ok!(Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id));
            let pool: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool.state, PoolState::Dead);

            // Pool is dead, not allow to close again, deposit, reset, kill or edit.
            assert_noop!(
                Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::kill_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::reset_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None
                ),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::edit_pool(
                    RuntimeOrigin::signed(ALICE),
                    pool_id,
                    None,
                    None,
                    None,
                    None,
                    None
                ),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::deposit(RuntimeOrigin::signed(ALICE), pool_id, 0, Some((100, 100))),
                Error::<Runtime>::InvalidPoolState
            );

            // Retire pool
            assert_noop!(
                Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::RetireLimitNotSet
            );
            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            System::set_block_number(System::block_number() + 1000);

            assert_ok!(Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id));
            let pool: PoolInfoOf<Runtime> = Farming::pool_infos(pool_id).unwrap();
            assert_eq!(pool.state, PoolState::Retired);

            // claim all rewards automatically to user
            assert_eq!(Assets::balance(KSM, &ALICE), 3000);
            assert_eq!(Farming::shares_and_withdrawn_rewards(pool_id, &ALICE), None);

            // Pool is retired, not allow to retire again, deposit, withdraw, claim, close
            assert_noop!(
                Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::deposit(RuntimeOrigin::signed(ALICE), pool_id, 0, Some((100, 100))),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::withdraw(RuntimeOrigin::signed(ALICE), pool_id, None),
                Error::<Runtime>::InvalidPoolState
            );
            assert_noop!(
                Farming::claim(RuntimeOrigin::signed(ALICE), pool_id),
                Error::<Runtime>::InvalidPoolState
            );

            // Kill the pool
            assert_ok!(Farming::kill_pool(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Farming::pool_infos(pool_id), None);
        })
}

#[test]
fn reset() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            let (pool_id, _tokens) = init_gauge_900();
            assert_eq!(Assets::balance(KSM, &ALICE), 1900);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 1);
            Farming::on_initialize(0);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 2918);
            assert_ok!(Farming::close_pool(RuntimeOrigin::signed(ALICE), pool_id));
            assert_ok!(Farming::set_retire_limit(RuntimeOrigin::signed(ALICE), 10));
            assert_ok!(Farming::retire_pool(RuntimeOrigin::signed(ALICE), pool_id));
            let basic_rewards = vec![(KSM, 1000)];
            assert_ok!(Farming::reset_pool(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                None,
                None,
                None,
                None,
                None,
                None,
                Some((KSM, 1000, basic_rewards)),
            ));
            let keeper: AccountId =
                <Runtime as Config>::Keeper::get().into_sub_account_truncating(pool_id);
            let reward_issuer: AccountId =
                <Runtime as Config>::RewardIssuer::get().into_sub_account_truncating(pool_id);
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
                pool_id,
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
                pool_id,
                charge_rewards
            ));
            assert_ok!(Farming::deposit(
                RuntimeOrigin::signed(ALICE),
                pool_id,
                1,
                Some((100, 100))
            ));
            assert_eq!(Assets::balance(KSM, &ALICE), 3817);
            Farming::on_initialize(0);
            System::set_block_number(System::block_number() + 20);
            assert_ok!(Farming::claim(RuntimeOrigin::signed(ALICE), pool_id));
            assert_eq!(Assets::balance(KSM, &ALICE), 4017);
        })
}
