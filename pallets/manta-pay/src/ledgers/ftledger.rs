extern crate alloc;

use crate::common::{PreprocessedEvent, Wrap, WrapPair};
use crate::errors::{
    FungibleLedgerError, ReceiverLedgerError, SenderLedgerError, TransferLedgerError,
};
use crate::ledger::ProxyLedger;
use crate::types::{asset_value_encode, fp_encode, Asset, AssetValue};
pub use crate::types::{Checkpoint, PullResponse, RawCheckpoint};
use crate::{pallet::*, Config};
use alloc::vec::Vec;
use frame_support::traits::tokens::ExistenceRequirement;
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
use manta_primitives::assets::{FungibleLedger as _, IsFungible};

/// Fungible Token Ledger
pub struct FTTransferLedger<T>
where
    T: Config,
{
    pub ledger: ProxyLedger<T>,
}

impl<T> SenderLedger<config::Parameters> for FTTransferLedger<T>
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

impl<T> ReceiverLedger<config::Parameters> for FTTransferLedger<T>
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

impl<T> TransferLedger<config::Config> for FTTransferLedger<T>
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
        asset_id: &config::AssetId,
        sources: I,
    ) -> Result<Vec<Self::ValidSourceAccount>, InvalidSourceAccount<config::Config, Self::AccountId>>
    where
        I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        let metadata = Pallet::<T>::get_metadata(*asset_id).expect("Metadata get failed");
        let id = metadata.get_fungible_id().expect("Asset Id get failed");
        sources
            .map(move |(account_id, withdraw)| {
                FungibleLedger::<T>::can_withdraw(
                    *id,
                    &account_id,
                    &withdraw,
                    ExistenceRequirement::KeepAlive,
                )
                .map(|_| WrapPair(account_id.clone(), withdraw))
                .map_err(|_| InvalidSourceAccount {
                    account_id,
                    asset_id: *asset_id,
                    withdraw,
                })
            })
            .collect()
    }

    #[inline]
    fn check_sink_accounts<I>(
        &self,
        asset_id: &config::AssetId,
        sinks: I,
    ) -> Result<Vec<Self::ValidSinkAccount>, InvalidSinkAccount<config::Config, Self::AccountId>>
    where
        I: Iterator<Item = (Self::AccountId, config::AssetValue)>,
    {
        let metadata = Pallet::<T>::get_metadata(*asset_id).expect("Metadata get failed");
        // NOTE: Existence of accounts is type-checked so we don't need to do anything here, just
        // pass the data forward.
        let id = metadata.get_fungible_id().expect("Asset Id get failed");
        sinks
            .map(move |(account_id, deposit)| {
                FungibleLedger::<T>::can_deposit(*id, &account_id, deposit, false)
                    .map(|_| WrapPair(account_id.clone(), deposit))
                    .map_err(|_| InvalidSinkAccount {
                        account_id,
                        asset_id: *asset_id,
                        deposit,
                    })
            })
            .collect()
    }

    #[inline]
    fn is_valid(
        &self,
        posting_key: TransferPostingKeyRef<config::Config, Self>,
    ) -> Result<(Self::ValidProof, Self::Event), TransferLedgerError<T>> {
        let transfer_shape = TransferShape::from_posting_key_ref(&posting_key);
        let (mut verifying_context, event) = match transfer_shape
            .ok_or(TransferLedgerError::InvalidTransferShape)?
        {
            TransferShape::ToPrivate => {
                if let Some(asset_id) = posting_key.asset_id.or(None) {
                    let asset_id =
                        fp_encode(asset_id).map_err(TransferLedgerError::FpEncodeError)?;
                    (
                        manta_parameters::pay::verifying::ToPrivate::get()
                            .ok_or(TransferLedgerError::ChecksumError)?,
                        PreprocessedEvent::<T>::ToPrivate {
                            asset: Asset::new(
                                asset_id,
                                asset_value_encode(posting_key.sources[0].1),
                            ),
                            source: posting_key.sources[0].0.clone(),
                        },
                    )
                } else {
                    return Err(TransferLedgerError::UnknownAsset);
                }
            }
            TransferShape::PrivateTransfer => (
                manta_parameters::pay::verifying::PrivateTransfer::get()
                    .ok_or(TransferLedgerError::ChecksumError)?,
                PreprocessedEvent::<T>::PrivateTransfer,
            ),
            TransferShape::ToPublic => {
                if let Some(asset_id) = posting_key.asset_id.or(None) {
                    let asset_id =
                        fp_encode(asset_id).map_err(TransferLedgerError::FpEncodeError)?;
                    (
                        manta_parameters::pay::verifying::ToPublic::get()
                            .ok_or(TransferLedgerError::ChecksumError)?,
                        PreprocessedEvent::<T>::ToPublic {
                            asset: Asset::new(asset_id, asset_value_encode(posting_key.sinks[0].1)),
                            sink: posting_key.sinks[0].0.clone(),
                        },
                    )
                } else {
                    return Err(TransferLedgerError::UnknownAsset);
                }
            }
        };
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
        let id = metadata
            .get_fungible_id()
            .ok_or(FungibleLedgerError::UnknownAsset)?;
        for WrapPair(account_id, withdraw) in sources {
            FungibleLedger::<T>::transfer(
                id.clone(),
                &account_id,
                &Pallet::<T>::account_id(),
                withdraw,
                ExistenceRequirement::KeepAlive,
            )?;
        }
        for WrapPair(account_id, deposit) in sinks {
            FungibleLedger::<T>::transfer(
                id.clone(),
                &Pallet::<T>::account_id(),
                &account_id,
                deposit,
                ExistenceRequirement::KeepAlive,
            )?;
        }
        Ok(())
    }
}
