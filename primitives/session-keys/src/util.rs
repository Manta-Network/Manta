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

//! Key Generation Utilities

use manta_primitives::types::{AccountId, Signer};
use sp_core::{crypto::CryptoType, Pair};
use sp_runtime::traits::IdentifyAccount;

/// Public Key Type
pub type PublicKey<T> = <<T as CryptoType>::Pair as Pair>::Public;

/// Derives [`PublicKey`] from `seed` for the corresponding crypto type `T` without checking that
/// the `seed` is valid.
#[inline]
pub fn unchecked_public_key<T>(seed: &str) -> PublicKey<T>
where
    T: CryptoType,
{
    T::Pair::from_string(&format!("//{seed}"), None)
        .expect("The validity of the seed is unchecked.")
        .public()
}

/// Derives [`AccountId`] from `seed` for the corresponding crypto type `T` without checking that
/// the `seed` is valid.
#[inline]
pub fn unchecked_account_id<T>(seed: &str) -> AccountId
where
    T: CryptoType,
    Signer: From<PublicKey<T>>,
{
    Signer::from(unchecked_public_key::<T>(seed)).into_account()
}
