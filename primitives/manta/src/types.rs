// Copyright 2020-2022 Manta Network.
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

//! Types

use sp_core::H256;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, OpaqueExtrinsic,
};

/// Block Number Type
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Signer Type
pub type Signer = <Signature as Verify>::Signer;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <Signer as IdentifyAccount>::AccountId;

/// Account Index Type
///
/// This index is used to look up accounts.
pub type AccountIndex = u32;

/// Calamari Asset Id Type
pub type CalamariAssetId = u128;

/// Dolphin Asset Id Type
pub type DolphinAssetId = u32;

/// Balance of an Account
pub type Balance = u128;

/// Transaction Index Type
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = H256;

/// Block Header Type
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block Type
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// Digest Item Type
pub type DigestItem = generic::DigestItem;

/// Moment Type
pub type Moment = u64;
