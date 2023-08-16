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
//
//! # Native Barrier Pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks_instance_pallet, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

use crate::Pallet as Balances;

const SEED: u32 = 0;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;

benchmarks_instance_pallet! {
    // Benchmark `transfer` extrinsic with the worst possible conditions:
    // * Transfer will kill the sender account.
    // * Transfer will create the recipient account.
    transfer {
        let existential_deposit = T::ExistentialDeposit::get();
        let caller = whitelisted_caller();

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&caller, balance);

        // Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account,
        // and reap this user.
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());
        let transfer_amount = existential_deposit.saturating_mul((ED_MULTIPLIER - 1).into()) + 1u32.into();
    }: transfer(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount)
    verify {
        assert_eq!(Balances::<T, I>::free_balance(&caller), Zero::zero());
        assert_eq!(Balances::<T, I>::free_balance(&recipient), transfer_amount);
    }

    // Benchmark `transfer` with the best possible condition:
    // * Both accounts exist and will continue to exist.
    #[extra]
    transfer_best_case {
        let caller = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());

        // Give the sender account max funds for transfer (their account will never reasonably be killed).
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());

        // Give the recipient account existential deposit (thus their account already exists).
        let existential_deposit = T::ExistentialDeposit::get();
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&recipient, existential_deposit);
        let transfer_amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
    }: transfer(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount)
    verify {
        assert!(!Balances::<T, I>::free_balance(&caller).is_zero());
        assert!(!Balances::<T, I>::free_balance(&recipient).is_zero());
    }

    // Benchmark `transfer_keep_alive` with the worst possible condition:
    // * The recipient account is created.
    transfer_keep_alive {
        let caller = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());

        // Give the sender account max funds, thus a transfer will not kill account.
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&caller, T::Balance::max_value());
        let existential_deposit = T::ExistentialDeposit::get();
        let transfer_amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
    }: _(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount)
    verify {
        assert!(!Balances::<T, I>::free_balance(&caller).is_zero());
        assert_eq!(Balances::<T, I>::free_balance(&recipient), transfer_amount);
    }

    // Benchmark `set_balance` coming from ROOT account. This always creates an account.
    set_balance_creating {
        let user: T::AccountId = account("user", 0, SEED);
        let user_lookup = T::Lookup::unlookup(user.clone());

        // Give the user some initial balance.
        let existential_deposit = T::ExistentialDeposit::get();
        let balance_amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&user, balance_amount);
    }: set_balance(RawOrigin::Root, user_lookup, balance_amount, balance_amount)
    verify {
        assert_eq!(Balances::<T, I>::free_balance(&user), balance_amount);
        assert_eq!(Balances::<T, I>::reserved_balance(&user), balance_amount);
    }

    // Benchmark `set_balance` coming from ROOT account. This always kills an account.
    set_balance_killing {
        let user: T::AccountId = account("user", 0, SEED);
        let user_lookup = T::Lookup::unlookup(user.clone());

        // Give the user some initial balance.
        let existential_deposit = T::ExistentialDeposit::get();
        let balance_amount = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&user, balance_amount);
    }: set_balance(RawOrigin::Root, user_lookup, Zero::zero(), Zero::zero())
    verify {
        assert!(Balances::<T, I>::free_balance(&user).is_zero());
    }

    // Benchmark `force_transfer` extrinsic with the worst possible conditions:
    // * Transfer will kill the sender account.
    // * Transfer will create the recipient account.
    force_transfer {
        let existential_deposit = T::ExistentialDeposit::get();
        let source: T::AccountId = account("source", 0, SEED);
        let source_lookup = T::Lookup::unlookup(source.clone());

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&source, balance);

        // Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account, and reap this user.
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());
        let transfer_amount = existential_deposit.saturating_mul((ED_MULTIPLIER - 1).into()) + 1u32.into();
    }: force_transfer(RawOrigin::Root, source_lookup, recipient_lookup, transfer_amount)
    verify {
        assert_eq!(Balances::<T, I>::free_balance(&source), Zero::zero());
        assert_eq!(Balances::<T, I>::free_balance(&recipient), transfer_amount);
    }

    // This benchmark performs the same operation as `transfer` in the worst case scenario,
    // but additionally introduces many new users into the storage, increasing the the merkle
    // trie and PoV size.
    #[extra]
    transfer_increasing_users {
        // 1_000 is not very much, but this upper bound can be controlled by the CLI.
        let u in 0 .. 1_000;
        let existential_deposit = T::ExistentialDeposit::get();
        let caller = whitelisted_caller();

        // Give some multiple of the existential deposit
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&caller, balance);

        // Transfer `e - 1` existential deposits + 1 unit, which guarantees to create one account,
        // and reap this user.
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());
        let transfer_amount = existential_deposit.saturating_mul((ED_MULTIPLIER - 1).into()) + 1u32.into();

        // Create a bunch of users in storage.
        for i in 0 .. u {
            // The `account` function uses `blake2_256` to generate unique accounts, so these
            // should be quite random and evenly distributed in the trie.
            let new_user: T::AccountId = account("new_user", i, SEED);
            let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&new_user, balance);
        }
    }: transfer(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount)
    verify {
        assert_eq!(Balances::<T, I>::free_balance(&caller), Zero::zero());
        assert_eq!(Balances::<T, I>::free_balance(&recipient), transfer_amount);
    }

    // Benchmark `transfer_all` with the worst possible condition:
    // * The recipient account is created
    // * The sender is killed
    transfer_all {
        let caller = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup = T::Lookup::unlookup(recipient.clone());

        // Give some multiple of the existential deposit
        let existential_deposit = T::ExistentialDeposit::get();
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&caller, balance);
    }: _(RawOrigin::Signed(caller.clone()), recipient_lookup, false)
    verify {
        assert!(Balances::<T, I>::free_balance(&caller).is_zero());
        assert_eq!(Balances::<T, I>::free_balance(&recipient), balance);
    }

    force_unreserve {
        let user: T::AccountId = account("user", 0, SEED);
        let user_lookup = T::Lookup::unlookup(user.clone());

        // Give some multiple of the existential deposit
        let existential_deposit = T::ExistentialDeposit::get();
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <Balances<T, I> as Currency<_>>::make_free_balance_be(&user, balance);

        // Reserve the balance
        <Balances<T, I> as ReservableCurrency<_>>::reserve(&user, balance)?;
        assert_eq!(Balances::<T, I>::reserved_balance(&user), balance);
        assert!(Balances::<T, I>::free_balance(&user).is_zero());

    }: _(RawOrigin::Root, user_lookup, balance)
    verify {
        assert!(Balances::<T, I>::reserved_balance(&user).is_zero());
        assert_eq!(Balances::<T, I>::free_balance(&user), balance);
    }

    impl_benchmark_test_suite!(
        Balances,
        crate::tests_composite::ExtBuilder::default().build(),
        crate::tests_composite::Test,
    )
}
