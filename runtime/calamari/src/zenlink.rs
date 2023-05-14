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
    assets_config::CalamariConcreteFungibleLedger, xcm_config::RelayNetwork, MantaCurrencies,
};
use frame_support::{parameter_types, traits::ExistenceRequirement, PalletId};
use manta_primitives::{
    assets::{AssetIdLocationMap, AssetIdLpMap, AssetLocation, FungibleLedger},
    types::{AccountId, CalamariAssetId},
};
use orml_traits::MultiCurrency;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::DispatchError;
use sp_std::prelude::*;
use xcm::latest::prelude::*;
use xcm_builder::{AccountId32Aliases, SiblingParachainConvertsVia};
use zenlink_protocol::{
    AssetBalance, AssetId as ZenlinkAssetId, ConvertMultiLocation, GenerateLpAssetId,
    LocalAssetHandler, ZenlinkMultiAssets, LOCAL, NATIVE,
};
use zenlink_stable_amm::traits::{StablePoolLpCurrencyIdGenerate, ValidateCurrency};

parameter_types! {
    pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
    pub const GetExchangeFee: (u32, u32) = (3, 1000);   // 0.3%
    pub SelfParaId: u32 = ParachainInfo::parachain_id().into();
    pub MantaNativeAssetId: CalamariAssetId = 1;
    pub ZenlinkNativeAssetId: u64 = 0;

    pub const AnyNetwork: NetworkId = NetworkId::Any;
    // Not allowed parachain token transfer on zenlink protocol
    pub ZenlinkRegistedParaChains: Vec<(MultiLocation, u128)> = vec![];
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
    type AssetIdConverter = MantaAssetIdConverter;
    type LpGenerate = AssetManagerLpGenerate;
    type AccountIdConverter = ZenlinkLocationToAccountId;
    type XcmExecutor = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const StringLimit: u32 = 50;
    pub const StableAmmPalletId: PalletId = PalletId(*b"mt/stamm");
}

impl zenlink_stable_amm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CurrencyId = CalamariAssetId;
    type MultiCurrency = MantaCurrencies;
    type PoolId = CalamariAssetId;
    type TimeProvider = Timestamp;
    type EnsurePoolAsset = StableAmmVerifyPoolAsset;
    type LpGenerate = PoolLpGenerate;
    type PoolCurrencySymbolLimit = StringLimit;
    type PalletId = StableAmmPalletId;
    type WeightInfo = ();
}

impl zenlink_swap_router::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StablePoolId = CalamariAssetId;
    type Balance = u128;
    type StableCurrencyId = CalamariAssetId;
    type NormalCurrencyId = ZenlinkAssetId;
    type NormalAmm = ZenlinkProtocol;
    type StableAMM = ZenlinkStableAMM;
    type WeightInfo = ();
}

pub struct StableAmmVerifyPoolAsset;

impl ValidateCurrency<CalamariAssetId> for StableAmmVerifyPoolAsset {
    fn validate_pooled_currency(_currencies: &[CalamariAssetId]) -> bool {
        true
    }

    fn validate_pool_lp_currency(_currency_id: CalamariAssetId) -> bool {
        if <MantaCurrencies as MultiCurrency<<Runtime as frame_system::Config>::AccountId>>::total_issuance(_currency_id) > 0 {
            return false;
        }
        true
    }
}

pub struct PoolLpGenerate;

impl StablePoolLpCurrencyIdGenerate<CalamariAssetId, CalamariAssetId> for PoolLpGenerate {
    fn generate_by_pool_id(pool_id: CalamariAssetId) -> CalamariAssetId {
        <AssetManager as AssetIdLpMap>::lp_asset_pool(&pool_id).expect("must find asset id")
    }
}

pub struct MantaAssetIdConverter;
impl ConvertMultiLocation<ZenlinkAssetId> for MantaAssetIdConverter {
    fn chain_id(asset_id: &ZenlinkAssetId) -> u32 {
        asset_id.chain_id
    }

    fn make_x3_location(asset_id: &ZenlinkAssetId) -> MultiLocation {
        let asset_index = asset_id.asset_index;
        if asset_index == ZenlinkNativeAssetId::get() {
            // Notice: native asset is register as (1, Parachain(id)) location now.
            return MultiLocation::new(1, X1(Parachain(SelfParaId::get())));
        }
        let asset = asset_index as CalamariAssetId;
        let asset_location: AssetLocation =
            AssetManager::location(&asset).expect("Asset should have Location!");
        asset_location
            .0
            .try_into()
            .expect("Location convert should not failed!")
    }
}

pub struct AssetManagerLpGenerate;
impl GenerateLpAssetId<ZenlinkAssetId> for AssetManagerLpGenerate {
    fn generate_lp_asset_id(
        asset_0: ZenlinkAssetId,
        asset_1: ZenlinkAssetId,
    ) -> Option<ZenlinkAssetId> {
        // LP asset id is registered on AssetManager based on two asset id.
        let asset_id_0 = LocalAssetAdaptor::asset_id_convert(asset_0);
        let asset_id_1 = LocalAssetAdaptor::asset_id_convert(asset_1);
        if asset_id_0.is_none() || asset_id_1.is_none() {
            return None;
        }
        let lp_asset_id: Option<CalamariAssetId> =
            <AssetManager as AssetIdLpMap>::lp_asset_id(&asset_id_0.unwrap(), &asset_id_1.unwrap());
        lp_asset_id.map(|lp_asset| ZenlinkAssetId {
            chain_id: SelfParaId::get(),
            asset_type: LOCAL,
            asset_index: lp_asset as u64,
        })
    }
}

/// Zenlink protocol Asset adaptor for orml_traits::MultiCurrency.
pub struct LocalAssetAdaptor;

impl LocalAssetAdaptor {
    fn asset_id_convert(asset_id: ZenlinkAssetId) -> Option<CalamariAssetId> {
        // Notice: Manta native asset id is 1, but Zenlink native asset id is 0.
        if asset_id.asset_index == ZenlinkNativeAssetId::get() {
            // When Zenlink asset index is 0, the asset type need to be NATIVE(0).
            return if asset_id.asset_type != NATIVE {
                None
            } else {
                Some(MantaNativeAssetId::get())
            };
        }
        let manta_asset_id = asset_id.asset_index as CalamariAssetId;

        // Must have location mapping of asset id
        let location = AssetManager::location(&manta_asset_id);
        location.map(|_| manta_asset_id)
    }
}

impl LocalAssetHandler<sp_runtime::AccountId32> for LocalAssetAdaptor {
    fn local_balance_of(asset_id: ZenlinkAssetId, who: &sp_runtime::AccountId32) -> AssetBalance {
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        if let Some(manta_asset_id) = manta_asset_id {
            <CalamariConcreteFungibleLedger as FungibleLedger>::balance(manta_asset_id, who)
        } else {
            AssetBalance::default()
        }
    }

    fn local_total_supply(asset_id: ZenlinkAssetId) -> AssetBalance {
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        if let Some(manta_asset_id) = manta_asset_id {
            <CalamariConcreteFungibleLedger as FungibleLedger>::supply(manta_asset_id)
        } else {
            AssetBalance::default()
        }
    }

    fn local_is_exists(asset_id: ZenlinkAssetId) -> bool {
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        manta_asset_id.is_some()
    }

    fn local_deposit(
        asset_id: ZenlinkAssetId,
        origin: &sp_runtime::AccountId32,
        amount: AssetBalance,
    ) -> Result<AssetBalance, DispatchError> {
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        if let Some(manta_asset_id) = manta_asset_id {
            <CalamariConcreteFungibleLedger as FungibleLedger>::deposit_minting(
                manta_asset_id,
                origin,
                amount,
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
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        if let Some(manta_asset_id) = manta_asset_id {
            <CalamariConcreteFungibleLedger as FungibleLedger>::withdraw_burning(
                manta_asset_id,
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
