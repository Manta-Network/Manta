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

use core::marker::PhantomData;
use frame_support::{
    migrations::migrate_from_pallet_version_to_storage_version,
    pallet_prelude::Weight,
    traits::{Get, OnRuntimeUpgrade},
};

pub struct EnableStorageVersion<T>(PhantomData<T>);
impl<T: frame_system::Config> OnRuntimeUpgrade for EnableStorageVersion<T> {
    fn on_runtime_upgrade() -> Weight {
        migrate_from_pallet_version_to_storage_version<System,ParachainSystem,Timestamp ,   ParachainInfo , TransactionPause , Balances , TransactionPayment , Democracy , Council , CouncilMembership , TechnicalCommittee , TechnicalMembership , Authorship , CollatorSelection , Session , Aura , AuraExt , Treasury , Preimage , Scheduler , XcmpQueue , PolkadotXcm , CumulusXcm , DmpQueue , XTokens , Utility , Multisig , Assets , AssetManager , CalamariVesting>()
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        use frame_support::{migration::{get_storage_value, have_storage_value}, traits::STORAGE_VERSION_STORAGE_KEY_POSTFIX};

        let pallets = (System,ParachainSystem,Timestamp ,   ParachainInfo , TransactionPause , Balances , TransactionPayment , Democracy , Council , CouncilMembership , TechnicalCommittee , TechnicalMembership , Authorship , CollatorSelection , Session , Aura , AuraExt , Treasury , Preimage , Scheduler , XcmpQueue , PolkadotXcm , CumulusXcm , DmpQueue , XTokens , Utility , Multisig , Assets , AssetManager , CalamariVesting);
        for pallet in pallets
        {
        if have_storage_value(pallet, PALLET_VERSION_STORAGE_KEY_POSTFIX, b"") {
            let pVer = get_storage_value(pallet,  PALLET_VERSION_STORAGE_KEY_POSTFIX, b"");
let sVer = pallet::current_storage_version();
            log::info!(target: "OnRuntimeUpgrade", "✅ {pallet} will have palletVersion {pVer} migrated to  to storageversion {sVer}");
            Ok(())
        } else if have_storage_value(pallet, STORAGE_VERSION_STORAGE_KEY_POSTFIX, b""){
            let sVer = pallet::current_storage_version();
            let SVer = get_storage_value(pallet, STORAGE_VERSION_STORAGE_KEY_POSTFIX, b"");

            log::info!(target: "OnRuntimeUpgrade", "✅ {pallet} is already at {sVer} with {SVer} in storage. Noop");
            Ok(())
        } else {
            Err("PALLET_VERSION and STORAGE_VERSION don't exist. This should not happen")
        }}
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
            ensure!(pallet::current_storage_version() ==  pallet::runtime_storage_version());
        todo!()
    }
}
