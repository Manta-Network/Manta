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

use crate::{benchmark::precomputed_coins::TO_PRIVATE, Box, Call, Config, Pallet, TransferPost};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use scale_codec::Decode;

mod precomputed_coins;

benchmarks! {
    to_private {
        let caller: T::AccountId = whitelisted_caller();
        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::ReservePrice::get() * factor.into());
        Pallet::<T>::reserve_sbt(RawOrigin::Signed(caller.clone()).into())?;
        let mint_post = TransferPost::decode(&mut &*TO_PRIVATE).unwrap();
    }: to_private (
        RawOrigin::Signed(caller.clone()),
        Box::new(mint_post),
        vec![0].try_into().unwrap()
    )

    reserve_sbt {
        let caller: T::AccountId = whitelisted_caller();
        let factor = 1_000u32;
        <T as crate::Config>::Currency::make_free_balance_be(&caller, T::ReservePrice::get() * factor.into());
    }: reserve_sbt (
        RawOrigin::Signed(caller)
    )
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
