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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;
use manta_primitives::types::PoolId;
use orml_traits::{arithmetic::CheckedAdd, MultiCurrency};
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, AtLeast32BitUnsigned, One, Saturating, Zero},
    ArithmeticError, Perbill, SaturatedConversion,
};
use sp_std::{borrow::ToOwned, collections::btree_map::BTreeMap, vec::Vec};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod gauge;
#[cfg(test)]
mod mock;
pub mod rewards;
#[cfg(test)]
mod tests;
pub mod weights;
pub use gauge::*;
pub use pallet::*;
pub use rewards::*;
pub use weights::WeightInfo;

#[allow(type_alias_bounds)]
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[allow(type_alias_bounds)]
pub type CurrencyIdOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
    <T as frame_system::Config>::AccountId,
>>::CurrencyId;

#[allow(type_alias_bounds)]
type BalanceOf<T: Config> =
    <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

#[allow(type_alias_bounds)]
type GaugeInitType<T: Config> = (
    CurrencyIdOf<T>,
    BlockNumberFor<T>,
    Vec<(CurrencyIdOf<T>, BalanceOf<T>)>,
);

#[allow(type_alias_bounds)]
type PoolInfoOf<T: Config> =
    PoolInfo<BalanceOf<T>, CurrencyIdOf<T>, AccountIdOf<T>, BlockNumberFor<T>>;

#[allow(type_alias_bounds)]
type GaugePoolInfoOf<T: Config> =
    GaugePoolInfo<BalanceOf<T>, CurrencyIdOf<T>, AccountIdOf<T>, BlockNumberFor<T>>;

#[allow(type_alias_bounds)]
type GaugeInfoOf<T> = GaugeInfo<BalanceOf<T>, BlockNumberFor<T>, AccountIdOf<T>>;

#[allow(type_alias_bounds)]
type ShareInfoOf<T> = ShareInfo<BalanceOf<T>, CurrencyIdOf<T>, BlockNumberFor<T>, AccountIdOf<T>>;

#[allow(type_alias_bounds)]
type RewardOf<T> = Vec<(CurrencyIdOf<T>, BalanceOf<T>)>;

#[frame_support::pallet]
#[allow(clippy::too_many_arguments)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency ID type
        type CurrencyId: Parameter
            + AtLeast32BitUnsigned
            + Default
            + Member
            + Copy
            + MaybeSerializeDeserialize
            + Ord
            + TypeInfo
            + MaxEncodedLen;

        /// ORML MultiCurrency
        type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = Self::CurrencyId>;

        /// ROOT Origin
        type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Set default weight.
        type WeightInfo: WeightInfo;

        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// ModuleID for creating sub account
        #[pallet::constant]
        type Keeper: Get<PalletId>;

        #[pallet::constant]
        type RewardIssuer: Get<PalletId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FarmingPoolCreated {
            pid: PoolId,
        },
        FarmingPoolReset {
            pid: PoolId,
        },
        FarmingPoolClosed {
            pid: PoolId,
        },
        FarmingPoolKilled {
            pid: PoolId,
        },
        FarmingPoolEdited {
            pid: PoolId,
        },
        Charged {
            who: AccountIdOf<T>,
            pid: PoolId,
            rewards: Vec<(CurrencyIdOf<T>, BalanceOf<T>)>,
        },
        Deposited {
            who: AccountIdOf<T>,
            pid: PoolId,
            add_value: BalanceOf<T>,
            gauge_info: Option<(BalanceOf<T>, BlockNumberFor<T>)>,
        },
        Withdrawn {
            who: AccountIdOf<T>,
            pid: PoolId,
            remove_value: Option<BalanceOf<T>>,
        },
        Claimed {
            who: AccountIdOf<T>,
            pid: PoolId,
        },
        WithdrawClaimed {
            who: AccountIdOf<T>,
            pid: PoolId,
        },
        GaugeWithdrawn {
            who: AccountIdOf<T>,
            gid: PoolId,
        },
        AllForceGaugeClaimed {
            gid: PoolId,
        },
        PartiallyForceGaugeClaimed {
            gid: PoolId,
        },
        AllRetired {
            pid: PoolId,
        },
        PartiallyRetired {
            pid: PoolId,
        },
        RetireLimitSet {
            limit: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pool not exist
        PoolDoesNotExist,
        /// Gauge pool not exist
        GaugePoolNotExist,
        /// Gauge info not exist
        GaugeInfoNotExist,
        /// Share info is not exist
        ShareInfoNotExists,
        /// Pool state is invalid
        InvalidPoolState,
        /// Can not claim gauge
        LastGaugeNotClaim,
        /// claim_limit_time exceeded
        CanNotClaim,
        /// gauge pool max_block exceeded
        GaugeMaxBlockOverflow,
        /// withdraw_limit_time exceeded
        WithdrawLimitCountExceeded,
        /// Can not deposit to pool yet
        CanNotDeposit,
    }

    /// The next farming pool id.
    #[pallet::storage]
    #[pallet::getter(fn pool_next_id)]
    pub type PoolNextId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

    /// The next gauge farming pool id.
    #[pallet::storage]
    #[pallet::getter(fn gauge_pool_next_id)]
    pub type GaugePoolNextId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

    /// The retire limit of one operation when retire farming pool.
    #[pallet::storage]
    #[pallet::getter(fn retire_limit)]
    pub type RetireLimit<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Record reward pool info.
    ///
    /// map PoolId => PoolInfo
    #[pallet::storage]
    #[pallet::getter(fn pool_infos)]
    pub type PoolInfos<T: Config> = StorageMap<_, Twox64Concat, PoolId, PoolInfoOf<T>>;

    /// Record gauge farming pool info.
    ///
    /// map PoolId => GaugePoolInfo
    #[pallet::storage]
    #[pallet::getter(fn gauge_pool_infos)]
    pub type GaugePoolInfos<T: Config> = StorageMap<_, Twox64Concat, PoolId, GaugePoolInfoOf<T>>;

    ///
    #[pallet::storage]
    #[pallet::getter(fn gauge_infos)]
    pub type GaugeInfos<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PoolId, Twox64Concat, T::AccountId, GaugeInfoOf<T>>;

    /// Record share amount, reward currency and withdrawn reward amount for
    /// specific `AccountId` under `PoolId`.
    ///
    /// double_map (PoolId, AccountId) => ShareInfo
    #[pallet::storage]
    #[pallet::getter(fn shares_and_withdrawn_rewards)]
    pub type SharesAndWithdrawnRewards<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PoolId, Twox64Concat, T::AccountId, ShareInfoOf<T>>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            PoolInfos::<T>::iter().for_each(|(pid, mut pool_info)| match pool_info.state {
                PoolState::Ongoing => {
                    pool_info.basic_rewards.clone().iter().for_each(
                        |(reward_currency_id, reward_amount)| {
                            pool_info
                                .rewards
                                .entry(*reward_currency_id)
                                .and_modify(|(total_reward, _)| {
                                    *total_reward = total_reward.saturating_add(*reward_amount);
                                })
                                .or_insert((*reward_amount, Zero::zero()));
                        },
                    );
                    PoolInfos::<T>::insert(pid, &pool_info);
                }
                PoolState::Charged => {
                    if n >= pool_info.after_block_to_start
                        && pool_info.total_shares >= pool_info.min_deposit_to_start
                    {
                        pool_info.block_startup = Some(n);
                        pool_info.state = PoolState::Ongoing;
                        PoolInfos::<T>::insert(pid, &pool_info);
                    }
                }
                _ => (),
            });

            GaugePoolInfos::<T>::iter().for_each(|(gid, mut gauge_pool_info)| {
                if gauge_pool_info.gauge_state == GaugeState::Bonded {
                    gauge_pool_info.gauge_basic_rewards.clone().iter().for_each(
                        |(reward_currency_id, reward_amount)| {
                            gauge_pool_info
                                .rewards
                                .entry(*reward_currency_id)
                                .and_modify(|(total_reward, _, _)| {
                                    *total_reward = total_reward.saturating_add(*reward_amount);
                                })
                                .or_insert((*reward_amount, Zero::zero(), Zero::zero()));
                        },
                    );
                    GaugePoolInfos::<T>::insert(gid, &gauge_pool_info);
                }
            });

            T::WeightInfo::on_initialize()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        BlockNumberFor<T>: AtLeast32BitUnsigned + Copy,
        BalanceOf<T>: AtLeast32BitUnsigned + Copy,
    {
        /// `ControlOrigin` create the farming pool.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_farming_pool())]
        pub fn create_farming_pool(
            origin: OriginFor<T>,
            tokens_proportion: Vec<(CurrencyIdOf<T>, Perbill)>,
            basic_rewards: Vec<(CurrencyIdOf<T>, BalanceOf<T>)>,
            gauge_init: Option<GaugeInitType<T>>,
            #[pallet::compact] min_deposit_to_start: BalanceOf<T>,
            #[pallet::compact] after_block_to_start: BlockNumberFor<T>,
            #[pallet::compact] withdraw_limit_time: BlockNumberFor<T>,
            #[pallet::compact] claim_limit_time: BlockNumberFor<T>,
            #[pallet::compact] withdraw_limit_count: u8,
        ) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let pid = Self::pool_next_id();
            let keeper = T::Keeper::get().into_sub_account_truncating(pid);
            let reward_issuer = T::RewardIssuer::get().into_sub_account_truncating(pid);
            let basic_token = tokens_proportion[0];
            let tokens_proportion_map: BTreeMap<CurrencyIdOf<T>, Perbill> =
                tokens_proportion.into_iter().map(|(k, v)| (k, v)).collect();
            let basic_rewards_map: BTreeMap<CurrencyIdOf<T>, BalanceOf<T>> =
                basic_rewards.into_iter().map(|(k, v)| (k, v)).collect();

            let mut pool_info = PoolInfo::new(
                keeper,
                reward_issuer,
                tokens_proportion_map,
                basic_token,
                basic_rewards_map,
                None,
                min_deposit_to_start,
                after_block_to_start,
                withdraw_limit_time,
                claim_limit_time,
                withdraw_limit_count,
            );

            if let Some((gauge_token, max_block, gauge_basic_rewards)) = gauge_init {
                let gauge_basic_rewards_map: BTreeMap<CurrencyIdOf<T>, BalanceOf<T>> =
                    gauge_basic_rewards
                        .into_iter()
                        .map(|(k, v)| (k, v))
                        .collect();

                Self::create_gauge_pool(
                    pid,
                    &mut pool_info,
                    gauge_token,
                    gauge_basic_rewards_map,
                    max_block,
                )?;
            };

            PoolInfos::<T>::insert(pid, &pool_info);
            PoolNextId::<T>::mutate(|id| -> DispatchResult {
                *id = id
                    .checked_add(One::one())
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::FarmingPoolCreated { pid });
            Ok(())
        }

        /// After create farming pool, need to deposit reward token into the pool.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::charge())]
        pub fn charge(
            origin: OriginFor<T>,
            pid: PoolId,
            rewards: Vec<(CurrencyIdOf<T>, BalanceOf<T>)>,
        ) -> DispatchResult {
            let exchanger = ensure_signed(origin)?;

            let mut pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            rewards
                .iter()
                .try_for_each(|(reward_currency, reward)| -> DispatchResult {
                    T::MultiCurrency::transfer(
                        *reward_currency,
                        &exchanger,
                        &pool_info.reward_issuer,
                        *reward,
                    )
                })?;
            pool_info.state = PoolState::Charged;
            PoolInfos::<T>::insert(pid, pool_info);

            Self::deposit_event(Event::Charged {
                who: exchanger,
                pid,
                rewards,
            });
            Ok(())
        }

        /// User can deposit token to farming pool, and get share of pool.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::deposit())]
        pub fn deposit(
            origin: OriginFor<T>,
            pid: PoolId,
            #[pallet::compact] add_value: BalanceOf<T>,
            gauge_info: Option<(BalanceOf<T>, BlockNumberFor<T>)>,
        ) -> DispatchResult {
            let exchanger = ensure_signed(origin)?;

            let mut pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::Deposit, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            if let PoolState::Charged = pool_info.state {
                let n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
                ensure!(
                    n >= pool_info.after_block_to_start,
                    Error::<T>::CanNotDeposit
                );
            }

            // basic token proportion * add_value * token_proportion
            // if basic token proportion and token_proportion both equals to 100%, then the final amount to transfer is equal to add_value
            let native_amount = pool_info.basic_token.1.saturating_reciprocal_mul(add_value);
            pool_info.tokens_proportion.iter().try_for_each(
                |(token, proportion)| -> DispatchResult {
                    T::MultiCurrency::transfer(
                        *token,
                        &exchanger,
                        &pool_info.keeper,
                        *proportion * native_amount,
                    )
                },
            )?;
            Self::add_share(&exchanger, pid, &mut pool_info, add_value);

            if let Some((gauge_value, gauge_block)) = gauge_info {
                Self::gauge_add(
                    &exchanger,
                    pool_info.gauge.ok_or(Error::<T>::GaugePoolNotExist)?,
                    gauge_value,
                    gauge_block,
                )?;
            }

            Self::deposit_event(Event::Deposited {
                who: exchanger,
                pid,
                add_value,
                gauge_info,
            });
            Ok(())
        }

        /// User can withdraw token from farming pool, and remove share of pool, also this operation
        /// will claim rewards.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::withdraw())]
        pub fn withdraw(
            origin: OriginFor<T>,
            pid: PoolId,
            remove_value: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let exchanger = ensure_signed(origin)?;

            let pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::Withdraw, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            let share_info = Self::shares_and_withdrawn_rewards(pid, &exchanger)
                .ok_or(Error::<T>::ShareInfoNotExists)?;
            ensure!(
                share_info.withdraw_list.len() < pool_info.withdraw_limit_count.into(),
                Error::<T>::WithdrawLimitCountExceeded
            );

            Self::remove_share(&exchanger, pid, remove_value, pool_info.withdraw_limit_time)?;

            Self::deposit_event(Event::Withdrawn {
                who: exchanger,
                pid,
                remove_value,
            });
            Ok(())
        }

        /// User claim rewards when deposit token into the farming pool.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(origin: OriginFor<T>, pid: PoolId) -> DispatchResult {
            let exchanger = ensure_signed(origin)?;

            let pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::Claim, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            let current_block_number: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();
            let share_info = Self::shares_and_withdrawn_rewards(pid, &exchanger)
                .ok_or(Error::<T>::ShareInfoNotExists)?;
            ensure!(
                share_info.claim_last_block + pool_info.claim_limit_time <= current_block_number,
                Error::<T>::CanNotClaim
            );

            Self::claim_rewards(&exchanger, pid)?;
            if let Some(ref gid) = pool_info.gauge {
                Self::gauge_claim_inner(&exchanger, *gid)?;
            }
            Self::process_withdraw_list(&exchanger, pid, &pool_info, true)?;

            Self::deposit_event(Event::Claimed {
                who: exchanger,
                pid,
            });
            Ok(())
        }

        /// User can withdraw but not claim rewards from farming pool.
        /// This operation will transfer back user staked token from keeper account.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn withdraw_claim(origin: OriginFor<T>, pid: PoolId) -> DispatchResult {
            let exchanger = ensure_signed(origin)?;

            let pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            Self::process_withdraw_list(&exchanger, pid, &pool_info, false)?;

            Self::deposit_event(Event::WithdrawClaimed {
                who: exchanger,
                pid,
            });
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(0)]
        pub fn close_pool(origin: OriginFor<T>, pid: PoolId) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let mut pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::ClosePool, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            pool_info.state = PoolState::Dead;
            PoolInfos::<T>::insert(pid, pool_info);

            Self::deposit_event(Event::FarmingPoolClosed { pid });
            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight(0)]
        pub fn set_retire_limit(origin: OriginFor<T>, limit: u32) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            RetireLimit::<T>::mutate(|old_limit| {
                *old_limit = limit;
            });

            Self::deposit_event(Event::RetireLimitSet { limit });
            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight(0)]
        pub fn retire_pool(origin: OriginFor<T>, pid: PoolId) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let mut pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::RetirePool, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            let withdraw_limit_time = BlockNumberFor::<T>::default();
            let retire_limit = RetireLimit::<T>::get();
            let mut all_retired = true;
            let share_infos = SharesAndWithdrawnRewards::<T>::iter_prefix_values(pid);
            for (retire_count, share_info) in share_infos.enumerate() {
                if retire_count.saturated_into::<u32>() >= retire_limit {
                    all_retired = false;
                    break;
                }
                let who = share_info.who;
                Self::remove_share(&who, pid, None, withdraw_limit_time)?;
                Self::claim_rewards(&who, pid)?;
                if let Some(ref gid) = pool_info.gauge {
                    Self::gauge_claim_inner(&who, *gid)?;
                }
                Self::process_withdraw_list(&who, pid, &pool_info, true)?;
            }

            if all_retired {
                if let Some(ref gid) = pool_info.gauge {
                    let mut gauge_pool_info =
                        Self::gauge_pool_infos(gid).ok_or(Error::<T>::GaugePoolNotExist)?;
                    gauge_pool_info.gauge_state = GaugeState::Unbond;
                    GaugePoolInfos::<T>::insert(gid, gauge_pool_info);
                }
                pool_info.state = PoolState::Retired;
                pool_info.gauge = None;
                PoolInfos::<T>::insert(pid, pool_info);
                Self::deposit_event(Event::AllRetired { pid });
            } else {
                Self::deposit_event(Event::PartiallyRetired { pid });
            }
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(0)]
        pub fn reset_pool(
            origin: OriginFor<T>,
            pid: PoolId,
            basic_rewards: Option<Vec<(CurrencyIdOf<T>, BalanceOf<T>)>>,
            min_deposit_to_start: Option<BalanceOf<T>>,
            after_block_to_start: Option<BlockNumberFor<T>>,
            withdraw_limit_time: Option<BlockNumberFor<T>>,
            claim_limit_time: Option<BlockNumberFor<T>>,
            withdraw_limit_count: Option<u8>,
            gauge_init: Option<GaugeInitType<T>>,
        ) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let mut pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::ResetPool, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            if let Some(basic_rewards) = basic_rewards {
                let basic_rewards_map: BTreeMap<CurrencyIdOf<T>, BalanceOf<T>> =
                    basic_rewards.into_iter().map(|(k, v)| (k, v)).collect();
                pool_info.basic_rewards = basic_rewards_map;
            };
            if let Some(min_deposit_to_start) = min_deposit_to_start {
                pool_info.min_deposit_to_start = min_deposit_to_start;
            };
            if let Some(after_block_to_start) = after_block_to_start {
                pool_info.after_block_to_start = after_block_to_start;
            };
            if let Some(withdraw_limit_time) = withdraw_limit_time {
                pool_info.withdraw_limit_time = withdraw_limit_time;
            };
            if let Some(claim_limit_time) = claim_limit_time {
                pool_info.claim_limit_time = claim_limit_time;
            };
            if let Some(withdraw_limit_count) = withdraw_limit_count {
                pool_info.withdraw_limit_count = withdraw_limit_count;
            };
            if let Some((gauge_token, max_block, gauge_basic_rewards)) = gauge_init {
                let gauge_basic_rewards_map: BTreeMap<CurrencyIdOf<T>, BalanceOf<T>> =
                    gauge_basic_rewards
                        .into_iter()
                        .map(|(k, v)| (k, v))
                        .collect();

                Self::create_gauge_pool(
                    pid,
                    &mut pool_info,
                    gauge_token,
                    gauge_basic_rewards_map,
                    max_block,
                )?;
            };
            pool_info.total_shares = Default::default();
            pool_info.rewards = BTreeMap::new();
            pool_info.state = PoolState::UnCharged;
            pool_info.block_startup = None;
            PoolInfos::<T>::insert(pid, &pool_info);

            Self::deposit_event(Event::FarmingPoolReset { pid });
            Ok(())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(0)]
        pub fn kill_pool(origin: OriginFor<T>, pid: PoolId) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::KillPool, pool_info.state),
                Error::<T>::InvalidPoolState
            );
            #[allow(deprecated)]
            SharesAndWithdrawnRewards::<T>::remove_prefix(pid, None);
            PoolInfos::<T>::remove(pid);

            Self::deposit_event(Event::FarmingPoolKilled { pid });
            Ok(())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(0)]
        pub fn edit_pool(
            origin: OriginFor<T>,
            pid: PoolId,
            basic_rewards: Option<Vec<(CurrencyIdOf<T>, BalanceOf<T>)>>,
            withdraw_limit_time: Option<BlockNumberFor<T>>,
            claim_limit_time: Option<BlockNumberFor<T>>,
            gauge_basic_rewards: Option<Vec<(CurrencyIdOf<T>, BalanceOf<T>)>>,
            withdraw_limit_count: Option<u8>,
        ) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let mut pool_info = Self::pool_infos(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
            ensure!(
                PoolState::state_valid(Action::EditPool, pool_info.state),
                Error::<T>::InvalidPoolState
            );

            if let Some(basic_rewards) = basic_rewards {
                let basic_rewards_map: BTreeMap<CurrencyIdOf<T>, BalanceOf<T>> =
                    basic_rewards.into_iter().map(|(k, v)| (k, v)).collect();
                pool_info.basic_rewards = basic_rewards_map;
            };
            if let Some(withdraw_limit_time) = withdraw_limit_time {
                pool_info.withdraw_limit_time = withdraw_limit_time;
            };
            if let Some(claim_limit_time) = claim_limit_time {
                pool_info.claim_limit_time = claim_limit_time;
            };
            if let Some(withdraw_limit_count) = withdraw_limit_count {
                pool_info.withdraw_limit_count = withdraw_limit_count;
            };
            if let Some(gauge_basic_rewards) = gauge_basic_rewards {
                let gauge_basic_rewards_map: BTreeMap<CurrencyIdOf<T>, BalanceOf<T>> =
                    gauge_basic_rewards
                        .into_iter()
                        .map(|(k, v)| (k, v))
                        .collect();
                GaugePoolInfos::<T>::mutate(
                    pool_info.gauge.ok_or(Error::<T>::GaugePoolNotExist)?,
                    |gauge_pool_info_old| {
                        if let Some(mut gauge_pool_info) = gauge_pool_info_old.take() {
                            gauge_pool_info.gauge_basic_rewards = gauge_basic_rewards_map;
                            *gauge_pool_info_old = Some(gauge_pool_info);
                        }
                    },
                );
            };
            PoolInfos::<T>::insert(pid, &pool_info);

            Self::deposit_event(Event::FarmingPoolEdited { pid });
            Ok(())
        }

        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::gauge_withdraw())]
        pub fn gauge_withdraw(origin: OriginFor<T>, gid: PoolId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut gauge_pool_info =
                GaugePoolInfos::<T>::get(gid).ok_or(Error::<T>::GaugePoolNotExist)?;
            match gauge_pool_info.gauge_state {
                GaugeState::Bonded => {
                    Self::gauge_claim_inner(&who, gid)?;
                }
                GaugeState::Unbond => {
                    let current_block_number: BlockNumberFor<T> =
                        frame_system::Pallet::<T>::block_number();
                    GaugeInfos::<T>::mutate(gid, &who, |maybe_gauge_info| -> DispatchResult {
                        if let Some(gauge_info) = maybe_gauge_info.take() {
                            if gauge_info.gauge_stop_block <= current_block_number {
                                T::MultiCurrency::transfer(
                                    gauge_pool_info.token,
                                    &gauge_pool_info.keeper,
                                    &who,
                                    gauge_info.gauge_amount,
                                )?;
                                gauge_pool_info.total_time_factor = gauge_pool_info
                                    .total_time_factor
                                    .checked_sub(gauge_info.total_time_factor)
                                    .ok_or(ArithmeticError::Overflow)?;
                                GaugePoolInfos::<T>::insert(gid, gauge_pool_info);
                            } else {
                                *maybe_gauge_info = Some(gauge_info);
                            };
                        }
                        Ok(())
                    })?;
                }
            }

            Self::deposit_event(Event::GaugeWithdrawn { who, gid });
            Ok(())
        }

        #[pallet::call_index(13)]
        #[pallet::weight(0)]
        pub fn force_gauge_claim(origin: OriginFor<T>, gid: PoolId) -> DispatchResult {
            T::ControlOrigin::ensure_origin(origin)?;

            let gauge_infos = GaugeInfos::<T>::iter_prefix_values(gid);
            let retire_limit = RetireLimit::<T>::get();
            let mut all_retired = true;
            for (retire_count, gauge_info) in gauge_infos.enumerate() {
                if retire_count.saturated_into::<u32>() >= retire_limit {
                    all_retired = false;
                    break;
                }
                Self::gauge_claim_inner(&gauge_info.who, gid)?;
            }

            if all_retired {
                Self::deposit_event(Event::AllForceGaugeClaimed { gid });
            } else {
                Self::deposit_event(Event::PartiallyForceGaugeClaimed { gid });
            }
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn farming_token_transfer(
        reward_currency: &CurrencyIdOf<T>,
        reward_to_withdraw: BalanceOf<T>,
        who: &T::AccountId,
        from: &T::AccountId,
    ) -> DispatchResult {
        let ed = T::MultiCurrency::minimum_balance(*reward_currency);
        let mut account_to_send = who.clone();

        if reward_to_withdraw < ed {
            let receiver_balance = T::MultiCurrency::total_balance(*reward_currency, who);

            let receiver_balance_after = receiver_balance
                .checked_add(&reward_to_withdraw)
                .ok_or(ArithmeticError::Overflow)?;
            if receiver_balance_after < ed {
                account_to_send = T::TreasuryAccount::get();
            }
        }
        // pay reward to `who`
        T::MultiCurrency::transfer(*reward_currency, from, &account_to_send, reward_to_withdraw)
    }

    pub fn get_farming_rewards(
        who: &T::AccountId,
        pid: PoolId,
    ) -> Result<RewardOf<T>, DispatchError> {
        let share_info =
            SharesAndWithdrawnRewards::<T>::get(pid, who).ok_or(Error::<T>::ShareInfoNotExists)?;
        let pool_info = PoolInfos::<T>::get(pid).ok_or(Error::<T>::PoolDoesNotExist)?;
        let total_shares = pool_info.total_shares;
        let mut result_vec = Vec::<(CurrencyIdOf<T>, BalanceOf<T>)>::new();

        pool_info.rewards.iter().try_for_each(
            |(reward_currency, (total_reward, total_withdrawn_reward))| -> DispatchResult {
                let (_, reward_to_withdraw) = Self::get_reward_amount(
                    &share_info,
                    total_reward,
                    total_withdrawn_reward,
                    total_shares,
                    reward_currency,
                )?;

                if reward_to_withdraw.is_zero() {
                    return Ok(());
                };

                result_vec.push((*reward_currency, reward_to_withdraw));
                Ok(())
            },
        )?;
        Ok(result_vec)
    }

    fn get_reward_amount(
        share_info: &ShareInfoOf<T>,
        total_reward: &BalanceOf<T>,
        total_withdrawn_reward: &BalanceOf<T>,
        total_shares: BalanceOf<T>,
        reward_currency: &CurrencyIdOf<T>,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
        let withdrawn_reward = share_info
            .withdrawn_rewards
            .get(reward_currency)
            .copied()
            .unwrap_or_default();

        let total_reward_proportion: BalanceOf<T> =
            Self::get_reward_inflation(share_info.share, total_reward, total_shares);

        let reward_to_withdraw = total_reward_proportion
            .saturating_sub(withdrawn_reward)
            .min(total_reward.saturating_sub(*total_withdrawn_reward));

        Ok((withdrawn_reward, reward_to_withdraw))
    }

    fn get_reward_inflation(
        amount: BalanceOf<T>,
        total_reward: &BalanceOf<T>,
        total_share: BalanceOf<T>,
    ) -> BalanceOf<T> {
        let total_reward_proportion: BalanceOf<T> =
            U256::from(amount.to_owned().saturated_into::<u128>())
                .saturating_mul(U256::from(total_reward.to_owned().saturated_into::<u128>()))
                .checked_div(U256::from(total_share.to_owned().saturated_into::<u128>()))
                .unwrap_or_default()
                .as_u128()
                .saturated_into();
        total_reward_proportion
    }
}