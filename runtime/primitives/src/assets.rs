// Copyright 2020-2021 Manta Network.
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

///! Manta/Calamari/Dolphin Asset

use xcm::{v1::{MultiLocation, Junctions}, VersionedMultiLocation};
use codec::{Encode, Decode};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, borrow::Borrow};


#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct AssetLocation(pub VersionedMultiLocation);

/// This cannot act as the default before v0.9.16 and need overwrite
/// https://docs.google.com/document/d/1W8y00IcJb0JXPBF59aP4nm-c7DY8Ld02-yIAO7UxR80
impl Default for AssetLocation {
    fn default() -> Self {
        AssetLocation(VersionedMultiLocation::V1(MultiLocation {parents: 0, interior: Junctions::Here}))
    }
}

/// Convert a `MultiLocaiton` to an `AssetLocation`
/// Note: This does not guaranttee the `AssetLocation` is registered (i.e. have an AssetId)
impl From<MultiLocation> for AssetLocation {
	fn from(location: MultiLocation) -> Self {
       AssetLocation(VersionedMultiLocation::V1(location))
	}
}

/// Convert an `AssetLocation` to a MultiLocation
/// If Native, retrun none.
impl Into<Option<MultiLocation>> for AssetLocation {
	fn into(self: Self) -> Option<MultiLocation> {
        // only support specific version
		if let AssetLocation(VersionedMultiLocation::V1(loc)) = self {
            Some(loc)
        } else {
            None
        }
	}
}

/// Defines the trait to obtain a generic AssetId
pub trait AssetIdLocationGetter<AssetId, AssetLocation>{
    // get AssetLocation from AssetId
    fn get_asset_location(asset_id: AssetId) -> Option<AssetLocation>;

    // get AssetId from AssetLocation
    fn get_asset_id(loc: AssetLocation) -> Option<AssetId>;
}

/// Converter struct implementing `Convert`.
/// This enforce the `AssetInfoGetter` implements `AssetIdLocationGetter` 
pub struct AssetIdLocationConvert<AssetId, AssetLocation, AssetInfoGetter>(
    PhantomData<(AssetId, AssetLocation, AssetInfoGetter)>,
);
impl<AssetId, AssetLocation, AssetInfoGetter> xcm_executor::traits::Convert<MultiLocation, AssetId> 
    for AssetIdLocationConvert<AssetId, AssetLocation, AssetInfoGetter>
where
    AssetId: Clone,
    AssetLocation: From<MultiLocation> + Into<Option<MultiLocation>> + Clone,
    AssetInfoGetter: AssetIdLocationGetter<AssetId, AssetLocation>,
{
    fn convert_ref(loc: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
        if let Some(asset_id) = AssetInfoGetter::get_asset_id(loc.borrow().clone().into()){
            Ok(asset_id)
        } else {
            Err(())
        }
    }

    fn reverse_ref(id: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
        if let Some(asset_loc) = AssetInfoGetter::get_asset_location(id.borrow().clone()) {
            if let Some(location) = asset_loc.into() {
                Ok(location)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}


