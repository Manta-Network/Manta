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

//! NFT Utilities

use crate::assets::{AssetIdType, BalanceType, FungibleLedgerError};
use core::marker::PhantomData;
use frame_support::traits::tokens::nonfungibles::{Inspect, Mutate, Transfer};
use frame_system::Config;

///
pub trait NonFungibleLedger: AssetIdType + BalanceType {
    /// Account Id Type
    type AccountId;

    // /// Collection Id Type
    // type CollectionId;

    /// Deposit NFT to `account` of `asset_id`
    fn deposit_minting(
        collection_id: &Self::AssetId,
        asset_id: &Self::AssetId,
        account: &Self::AccountId,
    ) -> Result<(), FungibleLedgerError<Self::AssetId, Self::Balance>>;

    /// Owner of nft
    fn owner(collection_id: &Self::AssetId, asset_id: &Self::AssetId) -> Option<Self::AccountId>;

    /// Performs a NFT transfer to `destination` of `asset_id`
    fn transfer(
        collection_id: &Self::AssetId,
        asset_id: &Self::AssetId,
        destination: &Self::AccountId,
    ) -> Result<(), FungibleLedgerError<Self::AssetId, Self::Balance>>;

    /// Performs a NFT burn of `asset_id`
    fn withdraw_burning(
        collection_id: &Self::AssetId,
        asset_id: &Self::AssetId,
        who: &Self::AccountId,
    ) -> Result<(), FungibleLedgerError<Self::AssetId, Self::Balance>>;

    ///
    fn can_mint(
        asset_id: Self::AssetId,
    ) -> Result<Self::AssetId, FungibleLedgerError<Self::AssetId, Self::Balance>> {
        Ok(asset_id)
    }

    ///
    fn can_burn(
        asset_id: Self::AssetId,
    ) -> Result<Self::AssetId, FungibleLedgerError<Self::AssetId, Self::Balance>> {
        Ok(asset_id)
    }
}

///
pub struct NonFungibleAsset<C, A, B, NFT> {
    ///  Type Parameter Marker
    __: PhantomData<(C, A, B, NFT)>,
}

impl<C, A, B, NFT> AssetIdType for NonFungibleAsset<C, A, B, NFT>
where
    A: Clone + PartialOrd,
    C: Config,
{
    type AssetId = A;
}

impl<C, A, B, NFT> BalanceType for NonFungibleAsset<C, A, B, NFT>
where
    C: Config,
{
    type Balance = B;
}

///
impl<C, A, B, NFT> NonFungibleLedger for NonFungibleAsset<C, A, B, NFT>
where
    C: Config,
    A: Clone + PartialOrd,
    NFT: Inspect<C::AccountId, ItemId = A, CollectionId = A>
        + Mutate<C::AccountId>
        + Transfer<C::AccountId>,
{
    type AccountId = C::AccountId;

    #[inline]
    fn deposit_minting(
        collection_id: &Self::AssetId,
        asset_id: &Self::AssetId,
        account: &C::AccountId,
    ) -> Result<(), FungibleLedgerError<Self::AssetId, B>> {
        NFT::mint_into(collection_id, asset_id, account)
            .map_err(FungibleLedgerError::InvalidMint)?;
        Ok(())
    }

    #[inline]
    fn owner(collection_id: &Self::AssetId, asset_id: &Self::AssetId) -> Option<C::AccountId> {
        NFT::owner(collection_id, asset_id)
    }

    #[inline]
    fn transfer(
        collection_id: &Self::AssetId,
        asset_id: &Self::AssetId,
        destination: &C::AccountId,
    ) -> Result<(), FungibleLedgerError<Self::AssetId, B>> {
        NFT::transfer(collection_id, asset_id, destination)
            .map_err(FungibleLedgerError::InvalidTransfer)?;
        Ok(())
    }

    #[inline]
    fn withdraw_burning(
        collection_id: &Self::AssetId,
        asset_id: &Self::AssetId,
        who: &C::AccountId,
    ) -> Result<(), FungibleLedgerError<Self::AssetId, B>> {
        NFT::burn(collection_id, asset_id, Some(who)).map_err(FungibleLedgerError::InvalidBurn)?;
        Ok(())
    }
}
