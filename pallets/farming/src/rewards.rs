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

use crate::*;
use codec::HasCompact;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Saturating, Zero},
    RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ShareInfo<BalanceOf: HasCompact, CurrencyIdOf: Ord, BlockNumberFor, AccountIdOf> {
    pub who: AccountIdOf,
    pub share: BalanceOf,
    pub withdrawn_rewards: BTreeMap<CurrencyIdOf, BalanceOf>,
    pub claim_last_block: BlockNumberFor,
    pub withdraw_list: Vec<(BlockNumberFor, BalanceOf)>,
}

impl<BalanceOf, CurrencyIdOf, BlockNumberFor, AccountIdOf>
    ShareInfo<BalanceOf, CurrencyIdOf, BlockNumberFor, AccountIdOf>
where
    BalanceOf: Default + HasCompact,
    CurrencyIdOf: Ord,
{
    fn new(who: AccountIdOf, claim_last_block: BlockNumberFor) -> Self {
        Self {
            who,
            share: Default::default(),
            withdrawn_rewards: BTreeMap::new(),
            claim_last_block,
            withdraw_list: Default::default(),
        }
    }
}

/// The Reward Pool Info.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PoolInfo<BalanceOf: HasCompact, CurrencyIdOf: Ord, AccountIdOf, BlockNumberFor> {
    pub tokens_proportion: BTreeMap<CurrencyIdOf, Perbill>,
    pub basic_token: (CurrencyIdOf, Perbill),
    /// Total shares amount
    pub total_shares: BalanceOf,
    pub basic_rewards: BTreeMap<CurrencyIdOf, BalanceOf>,
    /// Reward infos <reward_currency, (total_reward, total_withdrawn_reward)>
    pub rewards: BTreeMap<CurrencyIdOf, (BalanceOf, BalanceOf)>,
    pub state: PoolState,
    pub keeper: AccountIdOf,
    pub reward_issuer: AccountIdOf,
    /// Gauge pool id
    pub gauge: Option<PoolId>,
    pub block_startup: Option<BlockNumberFor>,
    /// The minimum share to starting farming
    pub min_deposit_to_start: BalanceOf,
    /// The minimum block number to starting farming
    pub after_block_to_start: BlockNumberFor,
    /// The limit block number to withdraw
    pub withdraw_limit_time: BlockNumberFor,
    /// The limit block number to claim
    pub claim_limit_time: BlockNumberFor,
    /// The withdraw limit length
    pub withdraw_limit_count: u8,
}

impl<BalanceOf, CurrencyIdOf, AccountIdOf, BlockNumberFor>
    PoolInfo<BalanceOf, CurrencyIdOf, AccountIdOf, BlockNumberFor>
where
    BalanceOf: Default + HasCompact,
    CurrencyIdOf: Ord,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        keeper: AccountIdOf,
        reward_issuer: AccountIdOf,
        tokens_proportion: BTreeMap<CurrencyIdOf, Perbill>,
        basic_token: (CurrencyIdOf, Perbill),
        basic_rewards: BTreeMap<CurrencyIdOf, BalanceOf>,
        gauge: Option<PoolId>,
        min_deposit_to_start: BalanceOf,
        after_block_to_start: BlockNumberFor,
        withdraw_limit_time: BlockNumberFor,
        claim_limit_time: BlockNumberFor,
        withdraw_limit_count: u8,
    ) -> Self {
        Self {
            tokens_proportion,
            basic_token,
            total_shares: Default::default(),
            basic_rewards,
            rewards: BTreeMap::new(),
            state: PoolState::UnCharged,
            keeper,
            reward_issuer,
            gauge,
            block_startup: None,
            min_deposit_to_start,
            after_block_to_start,
            withdraw_limit_time,
            claim_limit_time,
            withdraw_limit_count,
        }
    }
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum PoolState {
    UnCharged,
    Charged,
    Ongoing,
    Dead,
    Retired,
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Action {
    Deposit,
    Withdraw,
    Claim,
    RetirePool,
    ClosePool,
    ResetPool,
    KillPool,
    EditPool,
}

impl PoolState {
    pub fn state_valid(action: Action, state: PoolState) -> bool {
        match action {
            Action::Deposit => state == PoolState::Ongoing || state == PoolState::Charged,
            Action::Withdraw => {
                state == PoolState::Ongoing
                    || state == PoolState::Charged
                    || state == PoolState::Dead
            }
            Action::Claim => state == PoolState::Ongoing || state == PoolState::Dead,
            Action::ClosePool => state == PoolState::Ongoing,
            Action::RetirePool => state == PoolState::Dead,
            Action::ResetPool => state == PoolState::Retired,
            Action::KillPool => state == PoolState::Retired || state == PoolState::UnCharged,
            Action::EditPool => {
                state == PoolState::Retired
                    || state == PoolState::Ongoing
                    || state == PoolState::Charged
                    || state == PoolState::UnCharged
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn add_share(
        who: &T::AccountId,
        pool_id: PoolId,
        pool_info: &mut PoolInfo<BalanceOf<T>, CurrencyIdOf<T>, AccountIdOf<T>, BlockNumberFor<T>>,
        add_amount: BalanceOf<T>,
    ) {
        if add_amount.is_zero() {
            return;
        }

        // update pool total share
        let initial_total_shares = pool_info.total_shares;
        pool_info.total_shares = pool_info.total_shares.saturating_add(add_amount);

        // update user share
        let n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
        let mut share_info = SharesAndWithdrawnRewards::<T>::get(pool_id, who)
            .unwrap_or_else(|| ShareInfo::new(who.clone(), n));
        share_info.share = share_info.share.saturating_add(add_amount);

        let mut withdrawn_inflation = Vec::<(CurrencyIdOf<T>, BalanceOf<T>)>::new();

        // update pool rewards
        pool_info.rewards.iter_mut().for_each(
            |(reward_currency, (total_reward, total_withdrawn_reward))| {
                let reward_inflation = if initial_total_shares.is_zero() {
                    Zero::zero()
                } else {
                    Self::get_reward_inflation(add_amount, total_reward, initial_total_shares)
                };
                *total_reward = total_reward.saturating_add(reward_inflation);
                *total_withdrawn_reward = total_withdrawn_reward.saturating_add(reward_inflation);

                withdrawn_inflation.push((*reward_currency, reward_inflation));
            },
        );

        // update withdrawn inflation for each reward currency of user share
        withdrawn_inflation
            .into_iter()
            .for_each(|(reward_currency, reward_inflation)| {
                share_info
                    .withdrawn_rewards
                    .entry(reward_currency)
                    .and_modify(|withdrawn_reward| {
                        *withdrawn_reward = withdrawn_reward.saturating_add(reward_inflation);
                    })
                    .or_insert(reward_inflation);
            });

        SharesAndWithdrawnRewards::<T>::insert(pool_id, who, share_info);
        PoolInfos::<T>::insert(pool_id, pool_info);
    }

    pub fn remove_share(
        who: &T::AccountId,
        pool: PoolId,
        remove_amount_input: Option<BalanceOf<T>>,
        withdraw_limit_time: BlockNumberFor<T>,
    ) -> DispatchResult {
        if let Some(remove_amount_input) = remove_amount_input {
            if remove_amount_input.is_zero() {
                return Ok(());
            }
        }

        // claim rewards firstly
        Self::claim_rewards(who, pool)?;

        SharesAndWithdrawnRewards::<T>::mutate(pool, who, |share_info_old| -> DispatchResult {
            let n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
            if let Some(mut share_info) = share_info_old.take() {
                let remove_amount;
                if let Some(remove_amount_input) = remove_amount_input {
                    remove_amount = remove_amount_input.min(share_info.share);
                } else {
                    remove_amount = share_info.share;
                }
                if remove_amount.is_zero() {
                    return Ok(());
                }

                PoolInfos::<T>::mutate(pool, |maybe_pool_info| -> DispatchResult {
                    let pool_info = maybe_pool_info
                        .as_mut()
                        .ok_or(Error::<T>::PoolDoesNotExist)?;
                    pool_info.total_shares = pool_info.total_shares.saturating_sub(remove_amount);

                    // update withdrawn rewards for each reward currency
                    share_info.withdrawn_rewards.iter_mut().try_for_each(
                        |(reward_currency, withdrawn_reward)| -> DispatchResult {
                            let withdrawn_amount = Self::get_reward_inflation(
                                remove_amount,
                                withdrawn_reward,
                                share_info.share,
                            );
                            if withdrawn_amount.is_zero() {
                                return Ok(());
                            }

                            if let Some((total_reward, total_withdrawn_reward)) =
                                pool_info.rewards.get_mut(reward_currency)
                            {
                                *total_reward = total_reward.saturating_sub(withdrawn_amount);
                                *total_withdrawn_reward =
                                    total_withdrawn_reward.saturating_sub(withdrawn_amount);

                                // remove if all reward is withdrawn
                                if total_reward.is_zero() {
                                    pool_info.rewards.remove(reward_currency);
                                }
                            }
                            *withdrawn_reward = withdrawn_reward.saturating_sub(withdrawn_amount);
                            Ok(())
                        },
                    )?;
                    Ok(())
                })?;

                share_info
                    .withdraw_list
                    .push((n + withdraw_limit_time, remove_amount));
                share_info.share = share_info.share.saturating_sub(remove_amount);
                *share_info_old = Some(share_info);
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn claim_rewards(who: &T::AccountId, pool: PoolId) -> DispatchResult {
        SharesAndWithdrawnRewards::<T>::mutate_exists(
            pool,
            who,
            |maybe_share_withdrawn| -> DispatchResult {
                let n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
                if let Some(share_info) = maybe_share_withdrawn {
                    if share_info.share.is_zero() {
                        return Ok(());
                    }

                    PoolInfos::<T>::mutate(pool, |maybe_pool_info| -> DispatchResult {
                        let pool_info = maybe_pool_info
                            .as_mut()
                            .ok_or(Error::<T>::PoolDoesNotExist)?;

                        let total_shares = pool_info.total_shares;
                        pool_info.rewards.iter_mut().try_for_each(
							|(reward_currency, (total_reward, total_withdrawn_reward))|  -> DispatchResult {
                                let (withdrawn_reward, reward_to_withdraw) = Self::get_reward_amount(
                                    share_info, total_reward, total_withdrawn_reward, total_shares, reward_currency)?;

								if reward_to_withdraw.is_zero() {
									return Ok(());
								}

								*total_withdrawn_reward =
									total_withdrawn_reward.saturating_add(reward_to_withdraw);
								share_info.withdrawn_rewards.insert(
									*reward_currency,
									withdrawn_reward.saturating_add(reward_to_withdraw),
								);

                                Self::farming_token_transfer(reward_currency, reward_to_withdraw, who, &pool_info.reward_issuer)
							},
						)?;
                        Ok(())
                    })?;
                    share_info.claim_last_block = n;
                };
                Ok(())
            },
        )?;
        Ok(())
    }

    pub fn process_withdraw_list(
        who: &T::AccountId,
        pool: PoolId,
        pool_info: &PoolInfo<BalanceOf<T>, CurrencyIdOf<T>, AccountIdOf<T>, BlockNumberFor<T>>,
        if_remove: bool,
    ) -> DispatchResult {
        SharesAndWithdrawnRewards::<T>::mutate_exists(
            pool,
            who,
            |share_info_old| -> DispatchResult {
                if let Some(mut share_info) = share_info_old.take() {
                    let n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
                    let mut tmp: Vec<(BlockNumberFor<T>, BalanceOf<T>)> = Default::default();
                    share_info.withdraw_list.iter().try_for_each(
                        |(dest_block, remove_amount)| -> DispatchResult {
                            if *dest_block <= n {
                                let native_amount = pool_info
                                    .basic_token
                                    .1
                                    .saturating_reciprocal_mul(*remove_amount);
                                pool_info.tokens_proportion.iter().try_for_each(
                                    |(token, &proportion)| -> DispatchResult {
                                        let withdraw_amount = proportion * native_amount;

                                        Self::farming_token_transfer(
                                            token,
                                            withdraw_amount,
                                            who,
                                            &pool_info.keeper,
                                        )
                                    },
                                )?;
                            } else {
                                tmp.push((*dest_block, *remove_amount));
                            };
                            Ok(())
                        },
                    )?;
                    share_info.withdraw_list = tmp;

                    // if withdraw_list and share both are empty, and if_remove is true, remove it.
                    if !share_info.withdraw_list.is_empty()
                        || !share_info.share.is_zero()
                        || !if_remove
                    {
                        *share_info_old = Some(share_info);
                    };
                };
                Ok(())
            },
        )
    }
}
