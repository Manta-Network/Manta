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

use super::{
    AssetManager, Balances, ParachainInfo, Runtime, RuntimeEvent, Timestamp, ZenlinkProtocol,
    ZenlinkStableAMM,
};
use crate::{
    assets_config::DolphinConcreteFungibleLedger, xcm_config::RelayNetwork, MantaCurrencies,
};
use frame_support::{parameter_types, traits::ExistenceRequirement, PalletId};
use manta_primitives::{
    assets::{AssetIdLocationMap, AssetIdLpMap, FungibleLedger},
    types::{AccountId, DolphinAssetId, PoolId},
};
use orml_traits::MultiCurrency;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::DispatchError;
use sp_std::prelude::*;
use xcm::latest::prelude::*;
use xcm_builder::{AccountId32Aliases, SiblingParachainConvertsVia};
use zenlink_protocol::{
    make_x2_location, AssetBalance, AssetId as ZenlinkAssetId, AssetIdConverter, GenerateLpAssetId,
    LocalAssetHandler, ZenlinkMultiAssets, LOCAL,
};
use zenlink_stable_amm::traits::{StablePoolLpCurrencyIdGenerate, ValidateCurrency};

parameter_types! {
    pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
    pub const GetExchangeFee: (u32, u32) = (3, 1000);   // 0.3%
    pub SelfParaId: u32 = ParachainInfo::parachain_id().into();

    pub const AnyNetwork: NetworkId = NetworkId::Any;
    pub ZenlinkRegistedParaChains: Vec<(MultiLocation, u128)> = vec![
        // Dolphin local and live, 0.01 DOL
        (make_x2_location(2084), 10_000_000_000),
        // Karura local and live, 0.01 KAR
        (make_x2_location(2000), 10_000_000_000),
        // Bifrost local and live, 0.01 BNC
        (make_x2_location(2001), 10_000_000_000),
    ];
}

type MultiAssets = ZenlinkMultiAssets<ZenlinkProtocol, Balances, LocalAssetAdaptor>;

pub type ZenlinkLocationToAccountId = (
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

impl zenlink_protocol::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MultiAssetsHandler = MultiAssets;
    type PalletId = ZenlinkPalletId;
    type SelfParaId = SelfParaId;
    type TargetChains = ZenlinkRegistedParaChains;
    type AssetId = ZenlinkAssetId;
    type LpGenerate = AssetManagerLpGenerate;
    type AccountIdConverter = ZenlinkLocationToAccountId;
    type AssetIdConverter = AssetIdConverter;
    type XcmExecutor = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const StringLimit: u32 = 50;
    pub const StableAmmPalletId: PalletId = PalletId(*b"mt/stamm");
}

impl zenlink_stable_amm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CurrencyId = DolphinAssetId;
    type MultiCurrency = MantaCurrencies;
    type PoolId = u32;
    type TimeProvider = Timestamp;
    type EnsurePoolAsset = StableAmmVerifyPoolAsset;
    type LpGenerate = PoolLpGenerate;
    type PoolCurrencySymbolLimit = StringLimit;
    type PalletId = StableAmmPalletId;
    type WeightInfo = ();
}

impl zenlink_swap_router::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StablePoolId = u32;
    type Balance = u128;
    type StableCurrencyId = DolphinAssetId;
    type NormalCurrencyId = ZenlinkAssetId;
    type NormalAmm = ZenlinkProtocol;
    type StableAMM = ZenlinkStableAMM;
    type WeightInfo = ();
}

pub struct StableAmmVerifyPoolAsset;

impl ValidateCurrency<DolphinAssetId> for StableAmmVerifyPoolAsset {
    fn validate_pooled_currency(_currencies: &[DolphinAssetId]) -> bool {
        true
    }

    fn validate_pool_lp_currency(_currency_id: DolphinAssetId) -> bool {
        if <MantaCurrencies as MultiCurrency<<Runtime as frame_system::Config>::AccountId>>::total_issuance(_currency_id) > 0 {
            return false;
        }
        true
    }
}

pub struct PoolLpGenerate;

impl StablePoolLpCurrencyIdGenerate<DolphinAssetId, PoolId> for PoolLpGenerate {
    fn generate_by_pool_id(pool_id: PoolId) -> DolphinAssetId {
        <AssetManager as AssetIdLpMap>::lp_asset_pool(&pool_id).expect("must find asset id")
    }
}

pub struct AssetManagerLpGenerate;
impl GenerateLpAssetId<ZenlinkAssetId> for AssetManagerLpGenerate {
    fn generate_lp_asset_id(
        asset_0: ZenlinkAssetId,
        asset_1: ZenlinkAssetId,
    ) -> Option<ZenlinkAssetId> {
        let asset_0 = asset_0.asset_index as DolphinAssetId;
        let asset_1 = asset_1.asset_index as DolphinAssetId;
        let lp_asset_id = <AssetManager as AssetIdLpMap>::lp_asset_id(&asset_0, &asset_1);
        if let Some(lp_asset_id) = lp_asset_id {
            Some(ZenlinkAssetId {
                chain_id: SelfParaId::get(),
                asset_type: LOCAL,
                asset_index: lp_asset_id as u64,
            })
        } else {
            None
        }
    }
}

/// Zenlink protocol Asset adaptor for orml_traits::MultiCurrency.
pub struct LocalAssetAdaptor;

impl LocalAssetAdaptor {
    /// Ignore ZenlinkAssetId's chain and type, use asset_index as asset id.
    fn asset_id_convert(asset_id: ZenlinkAssetId) -> Option<DolphinAssetId> {
        let asset_index = asset_id.asset_index as DolphinAssetId;
        let location = AssetManager::location(&asset_index);
        if location.is_none() {
            return None;
        }
        Some(asset_index)
    }
}

impl LocalAssetHandler<sp_runtime::AccountId32> for LocalAssetAdaptor {
    fn local_balance_of(asset_id: ZenlinkAssetId, who: &sp_runtime::AccountId32) -> AssetBalance {
        let asset_index = LocalAssetAdaptor::asset_id_convert(asset_id);
        log::info!(
            "local balance asset id convert:{:?},{:?},{:?}, index:{:?}",
            asset_id.chain_id,
            asset_id.asset_type,
            asset_id.asset_index,
            asset_index
        );
        if let Some(asset_id) = asset_index {
            <DolphinConcreteFungibleLedger as FungibleLedger>::balance(asset_id, who)
        } else {
            AssetBalance::default()
        }
    }

    fn local_total_supply(asset_id: ZenlinkAssetId) -> AssetBalance {
        let asset_index = LocalAssetAdaptor::asset_id_convert(asset_id);
        log::info!(
            "local supply asset id convert:{:?},{:?},{:?}, index:{:?}",
            asset_id.chain_id,
            asset_id.asset_type,
            asset_id.asset_index,
            asset_index
        );
        if let Some(asset_id) = asset_index {
            <DolphinConcreteFungibleLedger as FungibleLedger>::supply(asset_id)
        } else {
            AssetBalance::default()
        }
    }

    fn local_is_exists(asset_id: ZenlinkAssetId) -> bool {
        let asset_index = LocalAssetAdaptor::asset_id_convert(asset_id);
        log::info!(
            "local exists asset id convert:{:?},{:?},{:?}, index:{:?}",
            asset_id.chain_id,
            asset_id.asset_type,
            asset_id.asset_index,
            asset_index
        );
        if let Some(_asset_id) = asset_index {
            true
        } else {
            false
        }
    }

    fn local_deposit(
        asset_id: ZenlinkAssetId,
        origin: &sp_runtime::AccountId32,
        amount: AssetBalance,
    ) -> Result<AssetBalance, DispatchError> {
        let asset_index = LocalAssetAdaptor::asset_id_convert(asset_id);
        log::info!(
            "local deposit asset id convert:{:?},{:?},{:?}, index:{:?}, account:{:?}",
            asset_id.chain_id,
            asset_id.asset_type,
            asset_id.asset_index,
            asset_index,
            origin
        );

        if let Some(asset_id) = asset_index {
            <DolphinConcreteFungibleLedger as FungibleLedger>::deposit_minting(
                asset_id, origin, amount,
            )
            .map_err(|_e| zenlink_protocol::Error::<Runtime>::ExecutionFailed)?;
            Ok(amount)
        } else {
            Err(DispatchError::Other("unknown asset in local deposit"))
        }
    }

    fn local_withdraw(
        asset_id: ZenlinkAssetId,
        origin: &sp_runtime::AccountId32,
        amount: AssetBalance,
    ) -> Result<AssetBalance, DispatchError> {
        let asset_index = LocalAssetAdaptor::asset_id_convert(asset_id);
        log::info!(
            "local withdraw asset id convert:{:?},{:?},{:?}, index:{:?}, account:{:?}",
            asset_id.chain_id,
            asset_id.asset_type,
            asset_id.asset_index,
            asset_index,
            origin
        );

        if let Some(asset_id) = asset_index {
            <DolphinConcreteFungibleLedger as FungibleLedger>::withdraw_burning(
                asset_id,
                origin,
                amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_e| zenlink_protocol::Error::<Runtime>::ExecutionFailed)?;
            Ok(amount)
        } else {
            Err(DispatchError::Other("unknown asset in local withdraw"))
        }
    }
}
