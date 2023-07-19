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

//! Manta Currencies Implementation

use crate::assets::AssetConfig;
use codec::{FullCodec, MaxEncodedLen};
use frame_support::{
    ensure,
    traits::{
        fungible, fungibles,
        fungibles::{Mutate, Transfer},
        Currency, ExistenceRequirement, WithdrawReasons,
    },
    Parameter,
};
use frame_system::Config;
use orml_traits::{arithmetic::CheckedSub, MultiCurrency};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member, Zero},
    ArithmeticError, DispatchError, DispatchResult,
};
use sp_std::{fmt::Debug, marker::PhantomData};

/// Currencies implements for orml_traits::MultiCurrency.
pub struct Currencies<C, A, Native, NonNative>(PhantomData<(C, A, Native, NonNative)>);

impl<C, A, Native, NonNative> MultiCurrency<C::AccountId> for Currencies<C, A, Native, NonNative>
where
    C: Config,
    A: AssetConfig<C>,
    A::AssetId:
        Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + TypeInfo + MaxEncodedLen,
    A::Balance: AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default
        + scale_info::TypeInfo
        + MaxEncodedLen,
    Native: fungible::Inspect<C::AccountId, Balance = A::Balance>
        + fungible::Mutate<C::AccountId>
        + Currency<C::AccountId, Balance = A::Balance>,
    NonNative: fungibles::Inspect<C::AccountId, AssetId = A::AssetId, Balance = A::Balance>
        + Mutate<C::AccountId>
        + Transfer<C::AccountId>,
{
    type CurrencyId = A::AssetId;
    type Balance = A::Balance;

    fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
        if currency_id == A::NativeAssetId::get() {
            <Native as fungible::Inspect<C::AccountId>>::minimum_balance()
        } else {
            NonNative::minimum_balance(currency_id)
        }
    }

    fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
        if currency_id == A::NativeAssetId::get() {
            <Native as fungible::Inspect<C::AccountId>>::total_issuance()
        } else {
            NonNative::total_issuance(currency_id)
        }
    }

    fn total_balance(currency_id: Self::CurrencyId, who: &C::AccountId) -> Self::Balance {
        if currency_id == A::NativeAssetId::get() {
            Native::balance(who)
        } else {
            NonNative::balance(currency_id, who)
        }
    }

    fn free_balance(currency_id: Self::CurrencyId, who: &C::AccountId) -> Self::Balance {
        if currency_id == A::NativeAssetId::get() {
            Native::free_balance(who)
        } else {
            NonNative::balance(currency_id, who)
        }
    }

    fn ensure_can_withdraw(
        currency_id: Self::CurrencyId,
        who: &C::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if amount == Zero::zero() {
            return Ok(());
        }
        let new_balance = Self::free_balance(currency_id, who)
            .checked_sub(&amount)
            .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
        if currency_id == A::NativeAssetId::get() {
            Native::ensure_can_withdraw(who, amount, WithdrawReasons::empty(), new_balance)
        } else {
            ensure!(
                new_balance >= Self::minimum_balance(currency_id),
                DispatchError::Other("balance too low")
            );
            Ok(())
        }
    }

    fn transfer(
        currency_id: Self::CurrencyId,
        from: &C::AccountId,
        to: &C::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if currency_id == A::NativeAssetId::get() {
            Native::transfer(from, to, amount, ExistenceRequirement::AllowDeath)?;
        } else {
            NonNative::transfer(currency_id, from, to, amount, false)?;
        }
        Ok(())
    }

    fn deposit(
        currency_id: Self::CurrencyId,
        who: &C::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if currency_id == A::NativeAssetId::get() {
            Native::deposit_creating(who, amount);
            Ok(())
        } else {
            NonNative::mint_into(currency_id, who, amount)
        }
    }

    fn withdraw(
        currency_id: Self::CurrencyId,
        who: &C::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if currency_id == A::NativeAssetId::get() {
            Native::withdraw(
                who,
                amount,
                WithdrawReasons::empty(),
                ExistenceRequirement::AllowDeath,
            )?;
        } else {
            NonNative::burn_from(currency_id, who, amount)?;
        }
        Ok(())
    }

    fn can_slash(currency_id: Self::CurrencyId, who: &C::AccountId, value: Self::Balance) -> bool {
        if value == Zero::zero() {
            return true;
        }
        Self::free_balance(currency_id, who) >= value
    }

    fn slash(
        currency_id: Self::CurrencyId,
        who: &C::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        if currency_id == A::NativeAssetId::get() {
            <Native as fungible::Mutate<C::AccountId>>::slash(who, amount)
                .expect("slash should not failed")
        } else {
            NonNative::slash(currency_id, who, amount).expect("slash should not failed")
        }
    }
}
