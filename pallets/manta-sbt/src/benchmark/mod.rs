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
    AccountId, Box, Call, Config, EvmAddress, Pallet, Pallet as MantaSBTPallet, TransferPost,
};
use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use scale_codec::Decode;
use sp_core::H160;
use sp_io::hashing::keccak_256;

const MINTS_OFFSET: usize = 4;
const MINT_SIZE: usize = 553;

fn alice() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn read_mint_coins() -> &'static [u8; 22120004] {
    core::include_bytes!("../../../../tests/data/mantaSbt_mints")
}

benchmarks! {
    where_clause {  where T::AccountId: From<AccountId> + Into<AccountId> }
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::ReservePrice::get() * factor.into());
        // 0..360000 asset ids have been inserted into mantasbt.
        // so next sbt id should be 360000, or related private txs won't get passed.
        crate::NextSbtId::<T>::put(360_000);
        Pallet::<T>::reserve_sbt(RawOrigin::Signed(caller.clone()).into(), None)?;

        let mint_coins = read_mint_coins();
        let mints_start = MINTS_OFFSET;
        let to_private_coin = &mint_coins[mints_start..mints_start + MINT_SIZE];
        let mint_post = TransferPost::decode(&mut &*to_private_coin).unwrap();
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
        crate::NextSbtId::<T>::put(360_000);
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
        let mint_coins = read_mint_coins();
        let mints_start = MINTS_OFFSET;
        let to_private_coin = &mint_coins[mints_start..mints_start + MINT_SIZE];
        let mint_post = TransferPost::decode(&mut &*to_private_coin).unwrap();

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

    set_next_sbt_id {
    }: set_next_sbt_id(
        RawOrigin::Root,
        Some(100)
    )

    force_to_private {
        let caller: T::AccountId = whitelisted_caller();
        let mint_coins = read_mint_coins();
        let mints_start = MINTS_OFFSET;
        let to_private_coin = &mint_coins[mints_start..mints_start + MINT_SIZE];
        MantaSBTPallet::<T>::change_force_account(
            RawOrigin::Root.into(),
            Some(caller.clone())
        )?;
        MantaSBTPallet::<T>::set_next_sbt_id(RawOrigin::Root.into(), Some(20_000_000))?;
        crate::NextSbtId::<T>::put(20_000_000);
        let mint_post = TransferPost::decode(&mut &*to_private_coin).unwrap();
    }: force_to_private(
        RawOrigin::Signed(caller.clone()),
        Box::new(mint_post),
        0,
        vec![].try_into().unwrap(),
        caller.clone()
    )

    force_mint_sbt_eth {
        let caller: T::AccountId = whitelisted_caller();
        MantaSBTPallet::<T>::change_force_account(
            RawOrigin::Root.into(),
            Some(caller.clone())
        )?;
        MantaSBTPallet::<T>::set_next_sbt_id(RawOrigin::Root.into(), Some(100))?;
        crate::NextSbtId::<T>::put(20_000_000);

        let mint_coins = read_mint_coins();
        let mints_start = MINTS_OFFSET;
        let to_private_coin = &mint_coins[mints_start..mints_start + MINT_SIZE];
        let mint_post = TransferPost::decode(&mut &*to_private_coin).unwrap();
    }: force_mint_sbt_eth(
        RawOrigin::Signed(caller.clone()),
        Box::new(mint_post),
        0,
        EvmAddress::default(),
        None,
        None,
        vec![].try_into().unwrap(),
        caller.clone()
    )

    change_force_account {
        let caller = whitelisted_caller();
    }: change_force_account(
        RawOrigin::Root,
        Some(caller)
    )
}
