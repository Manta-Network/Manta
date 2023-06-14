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

use crate::{
    benchmark::precomputed_coins::TO_PRIVATE, AccountId, Box, Call, Config, Pallet,
    Pallet as MantaSBTPallet, TransferPost,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use scale_codec::Decode;
use sp_core::H160;
use sp_io::hashing::keccak_256;

mod precomputed_coins;

fn alice() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

benchmarks! {
    where_clause {  where T::AccountId: From<AccountId> + Into<AccountId> }
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::ReservePrice::get() * factor.into());
        Pallet::<T>::reserve_sbt(RawOrigin::Signed(caller.clone()).into(), None)?;
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE).unwrap();
        MantaSBTPallet::<T>::new_mint_info(
            RawOrigin::Root.into(),
            0_u32.into(),
            None,
            vec![].try_into().unwrap(),
            true,
        )?;
        let bab_id = 1;
    }: to_private (
        RawOrigin::Signed(caller.clone()),
        Some(bab_id),
        None,
        None,
        Box::new(mint_post),
        vec![0].try_into().unwrap()
    )

    reserve_sbt {
        let caller: T::AccountId = whitelisted_caller();
        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::ReservePrice::get() * factor.into());
    }: reserve_sbt (
        RawOrigin::Signed(caller),
        None
    )

    change_allowlist_account{
        let caller: T::AccountId = whitelisted_caller();
    }: change_allowlist_account (
        RawOrigin::Root,
        Some(caller)
    )

    allowlist_evm_account {
        let caller: T::AccountId = whitelisted_caller();
        MantaSBTPallet::<T>::change_allowlist_account(
            RawOrigin::Root.into(),
            Some(caller.clone())
        )?;
        MantaSBTPallet::<T>::new_mint_info(
            RawOrigin::Root.into(),
            0_u32.into(),
            None,
            vec![].try_into().unwrap(),
            true
        )?;
        let bab_id = 1;
    }: allowlist_evm_account (
        RawOrigin::Signed(caller),
        bab_id,
        H160::default()
    )

    new_mint_info {
    }: new_mint_info (
        RawOrigin::Root,
        5u32.into(),
        Some(10u32.into()),
        vec![].try_into().unwrap(),
        true
    )

    update_mint_info {
        MantaSBTPallet::<T>::new_mint_info(
            RawOrigin::Root.into(),
            0_u32.into(),
            None,
            vec![].try_into().unwrap(),
            true,
        )?;
    }: update_mint_info (
        RawOrigin::Root,
        1,
        5u32.into(),
        None,
        vec![].try_into().unwrap(),
        false
    )

    mint_sbt_eth {
        let bab_id = 1;
        let caller: T::AccountId = whitelisted_caller();
        MantaSBTPallet::<T>::change_allowlist_account(
            RawOrigin::Root.into(),
            Some(caller.clone())
        )?;
        let bab_alice = MantaSBTPallet::<T>::eth_address(&alice());
        MantaSBTPallet::<T>::new_mint_info(
            RawOrigin::Root.into(),
            0_u32.into(),
            None,
            vec![].try_into().unwrap(),
            true,
        )?;

        MantaSBTPallet::<T>::allowlist_evm_account(
            RawOrigin::Signed(caller.clone()).into(),
            bab_id,
            bab_alice,
        )?;
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE).unwrap();

        let signature = MantaSBTPallet::<T>::eth_sign(&alice(), &mint_post.proof, 0);
    }: mint_sbt_eth(
        RawOrigin::Signed(caller),
        Box::new(mint_post),
        0,
        signature,
        bab_id,
        Some(0),
        Some(0),
        Some(vec![0].try_into().unwrap())
    )

    change_free_reserve_account {
        let caller = whitelisted_caller();
    }: change_allowlist_account(
        RawOrigin::Root,
        Some(caller)
    )

    remove_allowlist_evm_account {
        let caller: T::AccountId = whitelisted_caller();
        MantaSBTPallet::<T>::change_allowlist_account(
            RawOrigin::Root.into(),
            Some(caller.clone())
        )?;
        MantaSBTPallet::<T>::new_mint_info(
            RawOrigin::Root.into(),
            0_u32.into(),
            None,
            vec![].try_into().unwrap(),
            true
        )?;
        let bab_id = 1;

        MantaSBTPallet::<T>::allowlist_evm_account(
            RawOrigin::Signed(caller).into(),
            bab_id,
            H160::default()
        )?;
    }: remove_allowlist_evm_account(
        RawOrigin::Root,
        bab_id,
        H160::default()
    )
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
