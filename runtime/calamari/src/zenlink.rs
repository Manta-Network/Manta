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
use crate::{assets_config::CalamariConcreteFungibleLedger, MantaCurrencies};
use frame_support::{parameter_types, traits::ExistenceRequirement, PalletId};
use manta_primitives::{
    assets::{AssetIdLpMap, FungibleLedger},
    types::CalamariAssetId,
};
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, DispatchError};
use zenlink_protocol::{
    AssetBalance, AssetId as ZenlinkAssetId, GenerateLpAssetId, LocalAssetHandler,
    ZenlinkMultiAssets, LOCAL,
};
use zenlink_stable_amm::traits::{StablePoolLpCurrencyIdGenerate, ValidateCurrency};

// Normal Coin AMM
parameter_types! {
    pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
    pub SelfParaId: u32 = ParachainInfo::parachain_id().into();
    pub MantaNativeAssetId: CalamariAssetId = 1;
    pub ZenlinkNativeAssetId: u64 = 0;
}

type MultiAssets = ZenlinkMultiAssets<ZenlinkProtocol, Balances, LocalAssetAdaptor>;

impl zenlink_protocol::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MultiAssetsHandler = MultiAssets;
    type PalletId = ZenlinkPalletId;
    type SelfParaId = SelfParaId;
    type AssetId = ZenlinkAssetId;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type LpGenerate = AssetManagerLpGenerate;
    #[cfg(feature = "runtime-benchmarks")]
    type LpGenerate = mock_benchmark::MockAssetManagerLpGenerate;
    type WeightInfo = crate::weights::zenlink_protocol::SubstrateWeight<Runtime>;
}

pub struct AssetManagerLpGenerate;
impl GenerateLpAssetId<ZenlinkAssetId> for AssetManagerLpGenerate {
    fn generate_lp_asset_id(
        asset_0: ZenlinkAssetId,
        asset_1: ZenlinkAssetId,
    ) -> Option<ZenlinkAssetId> {
        if asset_0 == asset_1 {
            return None;
        }
        // LP asset id is registered on AssetManager based on two asset id.
        let asset_id_0 = LocalAssetAdaptor::asset_id_convert(asset_0);
        let asset_id_1 = LocalAssetAdaptor::asset_id_convert(asset_1);
        match (asset_id_0, asset_id_1) {
            (Some(asset_id0), Some(asset_id1)) => {
                let lp_asset_id: Option<CalamariAssetId> =
                    <AssetManager as AssetIdLpMap>::lp_asset_id(&asset_id0, &asset_id1);
                lp_asset_id.map(|lp_asset| ZenlinkAssetId {
                    chain_id: SelfParaId::get(),
                    asset_type: LOCAL,
                    asset_index: lp_asset as u64,
                })
            }
            _ => None,
        }
    }
}

pub struct LocalAssetAdaptor;

impl LocalAssetAdaptor {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn asset_id_convert(asset_id: ZenlinkAssetId) -> Option<CalamariAssetId> {
        // Notice: Manta native asset id is 1, but Zenlink native asset id is 0.
        if asset_id.asset_index == ZenlinkNativeAssetId::get() {
            // When Zenlink asset index is 0, the asset type need to be NATIVE(0).
            return if asset_id.asset_type != zenlink_protocol::NATIVE {
                None
            } else {
                Some(MantaNativeAssetId::get())
            };
        }
        let manta_asset_id = asset_id.asset_index as CalamariAssetId;
        Some(manta_asset_id)
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn asset_id_convert(asset_id: ZenlinkAssetId) -> Option<CalamariAssetId> {
        mock_benchmark::asset_id_convert(asset_id)
    }
}

impl LocalAssetHandler<sp_runtime::AccountId32> for LocalAssetAdaptor {
    fn local_balance_of(asset_id: ZenlinkAssetId, who: &sp_runtime::AccountId32) -> AssetBalance {
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        if let Some(manta_asset_id) = manta_asset_id {
            <CalamariConcreteFungibleLedger as FungibleLedger>::balance(manta_asset_id, who)
        } else {
            AssetBalance::zero()
        }
    }

    fn local_total_supply(asset_id: ZenlinkAssetId) -> AssetBalance {
        let manta_asset_id = LocalAssetAdaptor::asset_id_convert(asset_id);
        if let Some(manta_asset_id) = manta_asset_id {
            <CalamariConcreteFungibleLedger as FungibleLedger>::supply(manta_asset_id)
        } else {
            AssetBalance::zero()
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
            .map_err(|_e| DispatchError::Other("deposit lp asset error"))?;
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
            .map_err(|_e| DispatchError::Other("withdraw lp asset error"))?;
            Ok(amount)
        } else {
            Err(DispatchError::Other("unknown asset in local withdraw"))
        }
    }
}

// Stable Coin AMM
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

// Router: Hybrid AMM
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
        <MantaCurrencies as MultiCurrency<<Runtime as frame_system::Config>::AccountId>>::total_issuance(_currency_id) > 0
    }
}

pub struct PoolLpGenerate;

impl StablePoolLpCurrencyIdGenerate<CalamariAssetId, CalamariAssetId> for PoolLpGenerate {
    fn generate_by_pool_id(pool_id: CalamariAssetId) -> CalamariAssetId {
        <AssetManager as AssetIdLpMap>::lp_asset_pool(&pool_id).expect("must find asset id")
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod mock_benchmark {
    use super::super::*;
    use crate::{
        zenlink::{MantaNativeAssetId, SelfParaId, ZenlinkNativeAssetId},
        ZenlinkAssetId,
    };
    use manta_primitives::{
        assets::{AssetLocation, AssetRegistryMetadata, AssetStorageMetadata},
        types::{Balance, CalamariAssetId},
    };
    use xcm::{
        latest::MultiLocation,
        prelude::{GeneralIndex, PalletInstance, Parachain, X3},
        VersionedMultiLocation,
    };
    use zenlink_protocol::{GenerateLpAssetId, LOCAL, NATIVE};

    pub struct MockAssetManagerLpGenerate;
    impl GenerateLpAssetId<ZenlinkAssetId> for MockAssetManagerLpGenerate {
        fn generate_lp_asset_id(
            _asset_0: ZenlinkAssetId,
            asset_1: ZenlinkAssetId,
        ) -> Option<ZenlinkAssetId> {
            Some(ZenlinkAssetId {
                chain_id: SelfParaId::get(),
                asset_type: LOCAL,
                asset_index: asset_1.asset_index + 1u64,
            })
        }
    }

    pub fn mock_asset(name: &str, index: u128) -> (AssetRegistryMetadata<Balance>, AssetLocation) {
        let metadata = AssetRegistryMetadata {
            metadata: AssetStorageMetadata {
                name: name.as_bytes().to_vec(),
                symbol: name.as_bytes().to_vec(),
                decimals: 12,
                is_frozen: false,
            },
            min_balance: 1u128,
            is_sufficient: true,
        };
        let location = AssetLocation(VersionedMultiLocation::V1(MultiLocation::new(
            1,
            X3(
                Parachain(SelfParaId::get()),
                PalletInstance(45),
                GeneralIndex(index),
            ),
        )));
        (metadata, location)
    }

    pub fn asset_id_convert(asset_id: ZenlinkAssetId) -> Option<CalamariAssetId> {
        // Notice: Manta native asset id is 1, but Zenlink native asset id is 0.
        if asset_id.asset_index == ZenlinkNativeAssetId::get() {
            // When Zenlink asset index is 0, the asset type need to be NATIVE(0).
            return if asset_id.asset_type != NATIVE {
                None
            } else {
                Some(MantaNativeAssetId::get())
            };
        }
        // Manual create asset if not exist to make sure deposit_mint is fine.
        let (metadata1, location1) = mock_asset("Asset0", 8);
        let (metadata2, location2) = mock_asset("Asset1", 9);
        let (metadata3, _location3) = mock_asset("LPAsset01", 10);
        let (metadata4, location4) = mock_asset("Asset2", 11);
        let (metadata5, _location5) = mock_asset("LPAsset12", 12);
        let _ = AssetManager::do_register_asset(Some(&location1), &metadata1);
        let _ = AssetManager::do_register_asset(Some(&location2), &metadata2);
        let _ = AssetManager::do_register_asset(None, &metadata3);
        let _ = AssetManager::do_register_asset(Some(&location4), &metadata4);
        let _ = AssetManager::do_register_asset(None, &metadata5);
        Some(asset_id.asset_index as CalamariAssetId)
    }
}
