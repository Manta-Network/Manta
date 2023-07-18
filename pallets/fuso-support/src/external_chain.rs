// Copyright 2021-2023 UINB Technologies Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use codec::{Codec, Decode, Encode};
use core::fmt::Debug;
use frame_support::dispatch::Dispatchable;
use scale_info::TypeInfo;
use sp_std::{prelude::*, vec::Vec};

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum XToken<Balance> {
    // symbol, contract_address, total, stable, decimals
    NEP141(Vec<u8>, Vec<u8>, Balance, bool, u8),
    ERC20(Vec<u8>, Vec<u8>, Balance, bool, u8),
    BEP20(Vec<u8>, Vec<u8>, Balance, bool, u8),
    // symbol, total
    FND10(Vec<u8>, Balance),
    POLYGON(Vec<u8>, Vec<u8>, Balance, bool, u8),
}

impl<Balance> XToken<Balance> {
    pub fn is_stable(&self) -> bool {
        match self {
            XToken::NEP141(_, _, _, stable, _)
            | XToken::ERC20(_, _, _, stable, _)
            | XToken::POLYGON(_, _, _, stable, _)
            | XToken::BEP20(_, _, _, stable, _) => *stable,
            XToken::FND10(_, _) => false,
        }
    }

    pub fn symbol(&self) -> Vec<u8> {
        match self {
            XToken::NEP141(symbol, _, _, _, _)
            | XToken::ERC20(symbol, _, _, _, _)
            | XToken::POLYGON(symbol, _, _, _, _)
            | XToken::BEP20(symbol, _, _, _, _)
            | XToken::FND10(symbol, _) => symbol.clone(),
        }
    }

    pub fn contract(&self) -> Vec<u8> {
        match self {
            XToken::NEP141(_, contract, _, _, _)
            | XToken::ERC20(_, contract, _, _, _)
            | XToken::POLYGON(_, contract, _, _, _)
            | XToken::BEP20(_, contract, _, _, _) => contract.clone(),
            XToken::FND10(_, _) => Vec::new(),
        }
    }
}

pub type ChainId = u16;

pub trait ExternalSignWrapper<T: frame_system::Config> {
    fn extend_payload<W: Dispatchable<RuntimeOrigin = T::RuntimeOrigin> + Codec>(
        nonce: T::Index,
        tx: Box<W>,
    ) -> Vec<u8>;
}

pub mod chainbridge {
    use crate::ChainId;
    use alloc::string::{String, ToString};
    use sp_std::vec::Vec;

    pub type DepositNonce = u64;
    pub type ResourceId = [u8; 32];
    pub type EvmHash = [u8; 32];
    pub type EthereumCompatibleAddress = [u8; 20];

    /// [len, ..., 01, 01, 00]
    pub fn derive_resource_id(chain: ChainId, dex: u8, id: &[u8]) -> Result<ResourceId, String> {
        let mut r_id: ResourceId = [0; 32];
        let id_len = id.len();
        if id_len > 28 || id_len < 1 {
            return Err("contract length error".to_string());
        }
        r_id[30..].copy_from_slice(&chain.to_le_bytes()[..]);
        r_id[29] = dex;
        r_id[29 - id_len..29].copy_from_slice(&id[..]);
        r_id[0] = id_len as u8;
        Ok(r_id)
    }

    pub fn decode_resource_id(r_id: ResourceId) -> Result<(ChainId, u8, Vec<u8>), String> {
        let chainid = ChainId::from_le_bytes(r_id[30..].try_into().unwrap());
        let dex = r_id[29];
        let id_len = r_id[0];
        if id_len > 28 || id_len < 1 {
            return Err("contract length error".to_string());
        }
        let v: &[u8] = &r_id[29 - id_len as usize..29];
        Ok((chainid, dex, v.to_vec()))
    }

    pub trait AssetIdResourceIdProvider<TokenId> {
        type Err;

        fn try_get_asset_id(
            chain_id: ChainId,
            contract_id: impl AsRef<[u8]>,
        ) -> Result<TokenId, Self::Err>;
    }
}
