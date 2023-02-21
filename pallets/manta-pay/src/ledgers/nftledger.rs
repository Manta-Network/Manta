extern crate alloc;

use crate::common::{PreprocessedEvent, Wrap, WrapPair};
use crate::errors::{
    FungibleLedgerError, ReceiverLedgerError, SenderLedgerError, TransferLedgerError,
};
use crate::ledger::ProxyLedger;
use crate::types::{asset_value_encode, fp_encode, Asset, AssetValue};
use crate::{pallet::*, Config};
use alloc::vec::Vec;
use manta_accounting::transfer;
use manta_pay::{
    config::{self},
    manta_accounting::transfer::{
        canonical::TransferShape, receiver::ReceiverLedger, sender::SenderLedger,
        InvalidSinkAccount, InvalidSourceAccount, SinkPostingKey, SourcePostingKey, TransferLedger,
        TransferLedgerSuperPostingKey, TransferPostingKeyRef,
    },
    manta_parameters::{self, Get as _},
    manta_util::codec::Decode as _,
};
use manta_primitives::{assets::IsFungible, nft::NonFungibleLedger as _};

pub use crate::types::{Checkpoint, PullResponse, RawCheckpoint};

/// Fungible Token Ledger
pub struct NFTTransferLedger<T>
where
    T: Config,
{
    pub ledger: ProxyLedger<T>,
}

impl<T> SenderLedger<config::Parameters> for NFTTransferLedger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxoAccumulatorOutput = Wrap<config::UtxoAccumulatorOutput>;
    type ValidNullifier = Wrap<config::Nullifier>;
    type Error = SenderLedgerError;

    fn is_unspent(
        &self,
        nullifier: config::Nullifier,
    ) -> Result<Self::ValidNullifier, Self::Error> {
        self.ledger.is_unspent(nullifier)
    }

    #[inline]
    fn has_matching_utxo_accumulator_output(
        &self,
        output: config::UtxoAccumulatorOutput,
    ) -> Result<Self::ValidUtxoAccumulatorOutput, Self::Error> {
        self.ledger.has_matching_utxo_accumulator_output(output)
    }

    #[inline]
    fn spend_all<I>(
        &mut self,
        super_key: &Self::SuperPostingKey,
        iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxoAccumulatorOutput, Self::ValidNullifier)>,
    {
        self.ledger.spend_all(super_key, iter)
    }
}

impl<T> ReceiverLedger<config::Parameters> for NFTTransferLedger<T>
where
    T: Config,
{
    type SuperPostingKey = (Wrap<()>, ());
    type ValidUtxo = Wrap<config::Utxo>;
    type Error = ReceiverLedgerError<T>;

    #[inline]
    fn is_not_registered(&self, utxo: config::Utxo) -> Result<Self::ValidUtxo, Self::Error> {
        self.ledger.is_not_registered(utxo)
    }

    #[inline]
    fn register_all<I>(
        &mut self,
        super_key: &Self::SuperPostingKey,
        iter: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = (Self::ValidUtxo, config::Note)>,
    {
        self.ledger.register_all(super_key, iter)
    }
}

impl<T> TransferLedger<config::Config> for NFTTransferLedger<T>
where
    T: Config,
{
    type SuperPostingKey = ();
    type AccountId = T::AccountId;
    type Event = PreprocessedEvent<T>;
    type ValidSourceAccount = WrapPair<Self::AccountId, AssetValue>;
    type ValidSinkAccount = WrapPair<Self::AccountId, AssetValue>;
    type ValidProof = Wrap<()>;
    type Error = TransferLedgerError<T>;

    #[inline]
    fn check_source_accounts<I>(
        &self,
        _asset_id: &config::AssetId,
        sources: I,
    ) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<config::Config, Self::AccountId>>
    where
        I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        // let metadata = Pallet::<T>::get_metadata(*asset_id).expect("Metadata get failed");
        sources
            .map(move |(account_id, withdraw)| Ok(WrapPair(account_id.clone(), withdraw)))
            .collect()
    }

    #[inline]
    fn check_sink_accounts<I>(
        &self,
        _asset_id: &config::AssetId,
        sinks: I,
    ) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<config::Config, Self::AccountId>>
    where
        I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        // let metadata = Pallet::<T>::get_metadata(*asset_id).expect("Metadata get failed");
        sinks
            .map(move |(account_id, deposit)| Ok(WrapPair(account_id.clone(), deposit)))
            .collect()
    }

    #[inline]
    fn is_valid(
        &self,
        posting_key: TransferPostingKeyRef<config::Config, Self>,
    ) -> Result<(Self::ValidProof, Self::Event), TransferLedgerError<T>> {
        let (mut verifying_context, event) = match TransferShape::from_posting_key_ref(&posting_key)
            .expect("todo remove")
        {
            TransferShape::ToPrivate => (
                manta_parameters::pay::verifying::ToPrivate::get()
                    .expect("Checksum did not match."),
                PreprocessedEvent::<T>::ToPrivate {
                    asset: Asset::new(
                        fp_encode(posting_key.asset_id.or(None).expect("todo: remove"))
                            .ok()
                            .expect("todo: remove"),
                        asset_value_encode(posting_key.sources[0].1),
                    ),
                    source: posting_key.sources[0].0.clone(),
                },
            ),
            TransferShape::PrivateTransfer => (
                manta_parameters::pay::verifying::PrivateTransfer::get()
                    .expect("Checksum did not match."),
                PreprocessedEvent::<T>::PrivateTransfer,
            ),
            TransferShape::ToPublic => (
                manta_parameters::pay::verifying::ToPublic::get().expect("Checksum did not match."),
                PreprocessedEvent::<T>::ToPublic {
                    asset: Asset::new(
                        fp_encode(posting_key.asset_id.or(None).expect("todo: remove"))
                            .ok()
                            .expect("todo: remove"),
                        asset_value_encode(posting_key.sinks[0].1),
                    ),
                    sink: posting_key.sinks[0].0.clone(),
                },
            ),
        };
        // if let Some(res) =
        let verification = posting_key
            .has_valid_proof(
                &config::VerifyingContext::decode(&mut verifying_context)
                    .map_err(TransferLedgerError::VerifyingContextDecodeError)?,
            )
            .map_err(TransferLedgerError::ProofSystemError)?;
        if verification {
            Ok((Wrap(()), event))
        } else {
            Err(TransferLedgerError::InvalidProof)
        }

        // Ok(res)
    }

    #[inline]
    fn update_public_balances(
        &mut self,
        _asset_type: transfer::AssetType,
        super_key: &TransferLedgerSuperPostingKey<config::Config, Self>,
        asset_id: config::AssetId,
        sources: Vec<SourcePostingKey<config::Config, Self>>,
        sinks: Vec<SinkPostingKey<config::Config, Self>>,
        proof: Self::ValidProof,
    ) -> Result<(), TransferLedgerError<T>> {
        let _ = (proof, super_key);
        let metadata = Pallet::<T>::get_metadata(asset_id)?;
        let (collection_id, item_id) = metadata
            .get_non_fungible_id()
            .ok_or(TransferLedgerError::UnknownAsset)?;
        for WrapPair(_account_id, _withdraw) in sources {
            NonFungibleLedger::<T>::transfer(collection_id, item_id, &Pallet::<T>::account_id())
                .expect("todo: remove");
        }
        for WrapPair(account_id, _deposit) in sinks {
            NonFungibleLedger::<T>::transfer(collection_id, item_id, &account_id)
                .expect("todo: remove");
        }
        Ok(())
    }
}
