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

//! Error Handling for MantaPay

use super::*;

impl<T> From<InvalidAuthorizationSignature> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(_: InvalidAuthorizationSignature) -> Self {
        Self::InvalidAuthorizationSignature
    }
}

impl<T, Id> From<InvalidSourceAccount<config::Config, Id>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(_: InvalidSourceAccount<config::Config, Id>) -> Self {
        Self::InvalidSourceAccount
    }
}

impl<T, Id> From<InvalidSinkAccount<config::Config, Id>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(_: InvalidSinkAccount<config::Config, Id>) -> Self {
        Self::InvalidSinkAccount
    }
}

/// Receiver Ledger Error
pub enum ReceiverLedgerError<T>
where
    T: Config,
{
    /// Utxo Decoding Error
    UtxoDecodeError(scale_codec::Error),

    /// Wrong Checksum Error
    ChecksumError,

    /// Merkle Tree Parameters Decoding Error
    MTParametersDecodeError(MTParametersError),

    /// Utxo Accumulator Item Hash Decoding Error
    UtxoAccumulatorItemHashDecodeError(UtxoItemHashError),

    /// Merkle Tree Out of Capacity Error
    MerkleTreeCapacityError,

    /// Field Element Encoding Error
    FpEncodeError(scale_codec::Error),

    /// Field Element Encoding Error
    FpDecodeError(scale_codec::Error),

    /// Path Decoding Error
    PathDecodeError(scale_codec::Error),

    /// Full Incoming Note Decoding Error
    FullNoteDecodeError(scale_codec::Error),

    /// Asset Registered Error
    ///
    /// The asset has already been registered with the ledger.
    AssetRegistered,

    /// Type Marker Parameter
    Marker(PhantomData<T>),
}

impl<T> From<ReceiverPostError<ReceiverLedgerError<T>>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(err: ReceiverPostError<ReceiverLedgerError<T>>) -> Self {
        match err {
            ReceiverPostError::AssetRegistered => Self::AssetRegistered,
            ReceiverPostError::UnexpectedError(e) => match e {
                ReceiverLedgerError::<T>::AssetRegistered => Self::AssetRegistered,
                ReceiverLedgerError::<T>::UtxoDecodeError(_) => {
                    Self::ReceiverLedgerUtxoDecodeFailed
                }
                ReceiverLedgerError::<T>::ChecksumError => Self::ReceiverLedgerChecksumError,
                ReceiverLedgerError::<T>::MTParametersDecodeError(_) => {
                    Self::ReceiverLedgerMTParametersDecodeError
                }
                ReceiverLedgerError::<T>::UtxoAccumulatorItemHashDecodeError(_) => {
                    Self::ReceiverLedgerUtxoAccumulatorItemHashDecodeError
                }
                ReceiverLedgerError::<T>::MerkleTreeCapacityError => {
                    Self::ReceiverLedgerMerkleTreeCapacityError
                }
                ReceiverLedgerError::<T>::FpEncodeError(_) => Self::ReceiverLedgerFpEncodeError,
                ReceiverLedgerError::<T>::FpDecodeError(_) => Self::ReceiverLedgerFpDecodeError,
                ReceiverLedgerError::<T>::PathDecodeError(_) => Self::ReceiverLedgerPathDecodeError,
                ReceiverLedgerError::<T>::FullNoteDecodeError(_) => {
                    Self::ReceiverLedgerFullNoteDecodeError
                }
                ReceiverLedgerError::<T>::Marker(_) => Self::Marker,
            },
        }
    }
}

impl<T> From<ReceiverLedgerError<T>> for ReceiverPostError<ReceiverLedgerError<T>>
where
    T: Config,
{
    #[inline]
    fn from(value: ReceiverLedgerError<T>) -> Self {
        if let ReceiverLedgerError::AssetRegistered = value {
            Self::AssetRegistered
        } else {
            Self::UnexpectedError(value)
        }
    }
}

/// Sender Ledger Error
pub enum SenderLedgerError {
    /// Field Element Encoding Error
    FpEncodeError(scale_codec::Error),

    /// Outgoing Note Decoding Error
    OutgoingNoteDecodeError(scale_codec::Error),

    /// Asset Spent Error
    ///
    /// The asset has already been spent.
    AssetSpent,

    /// Invalid UTXO Accumulator Output Error
    ///
    /// The sender was not constructed under the current state of the UTXO accumulator.
    InvalidUtxoAccumulatorOutput,
}

impl<T> From<SenderPostError<SenderLedgerError>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(err: SenderPostError<SenderLedgerError>) -> Self {
        match err {
            SenderPostError::AssetSpent => Self::AssetSpent,
            SenderPostError::InvalidUtxoAccumulatorOutput => Self::InvalidUtxoAccumulatorOutput,
            SenderPostError::UnexpectedError(err) => match err {
                SenderLedgerError::AssetSpent => Self::AssetSpent,
                SenderLedgerError::InvalidUtxoAccumulatorOutput => {
                    Self::InvalidUtxoAccumulatorOutput
                }
                SenderLedgerError::FpEncodeError(_) => Self::SenderLedgerFpEncodeError,
                SenderLedgerError::OutgoingNoteDecodeError(_) => {
                    Self::SenderLedgerOutgoingNodeDecodeFailed
                }
            },
        }
    }
}

impl From<SenderLedgerError> for SenderPostError<SenderLedgerError> {
    #[inline]
    fn from(value: SenderLedgerError) -> Self {
        match value {
            SenderLedgerError::AssetSpent => Self::AssetSpent,
            SenderLedgerError::InvalidUtxoAccumulatorOutput => Self::InvalidUtxoAccumulatorOutput,
            SenderLedgerError::FpEncodeError(err) => {
                Self::UnexpectedError(SenderLedgerError::FpEncodeError(err))
            }
            SenderLedgerError::OutgoingNoteDecodeError(err) => {
                Self::UnexpectedError(SenderLedgerError::OutgoingNoteDecodeError(err))
            }
        }
    }
}

impl<T> From<SenderLedgerError> for TransferLedgerError<T>
where
    T: Config,
{
    #[inline]
    fn from(err: SenderLedgerError) -> Self {
        TransferLedgerError::SenderLedgerError(err)
    }
}

impl<T> From<ReceiverLedgerError<T>> for TransferLedgerError<T>
where
    T: Config,
{
    #[inline]
    fn from(err: ReceiverLedgerError<T>) -> Self {
        TransferLedgerError::ReceiverLedgerError(err)
    }
}

impl<T> From<FungibleLedgerError> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(err: FungibleLedgerError) -> Self {
        match err {
            FungibleLedgerError::InvalidAssetId(_) => Self::PublicUpdateInvalidAssetId,
            FungibleLedgerError::BelowMinimum => Self::PublicUpdateBelowMinimum,
            FungibleLedgerError::CannotCreate => Self::PublicUpdateCannotCreate,
            FungibleLedgerError::UnknownAsset => Self::PublicUpdateUnknownAsset,
            FungibleLedgerError::Overflow => Self::PublicUpdateOverflow,
            FungibleLedgerError::CannotWithdrawMoreThan(_) => Self::PublicUpdateCannotWithdraw,
            FungibleLedgerError::InvalidMint(_) => Self::PublicUpdateInvalidMint,
            FungibleLedgerError::InvalidBurn(_) => Self::PublicUpdateInvalidBurn,
            FungibleLedgerError::InvalidTransfer(_) => Self::PublicUpdateInvalidTransfer,
            FungibleLedgerError::EncodeError => Self::FungibleLedgerEncodeError,
        }
    }
}

/// Transfer Post Error
pub type TransferPostError<T> = transfer::TransferPostError<
    config::Config,
    AccountId,
    SenderLedgerError,
    ReceiverLedgerError<T>,
    TransferLedgerError<T>,
>;

impl<T> From<TransferPostError<T>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(err: TransferPostError<T>) -> Self {
        match err {
            TransferPostError::<T>::InvalidShape => Self::InvalidShape,
            TransferPostError::<T>::InvalidAuthorizationSignature(err) => err.into(),
            TransferPostError::<T>::InvalidSourceAccount(err) => err.into(),
            TransferPostError::<T>::InvalidSinkAccount(err) => err.into(),
            TransferPostError::<T>::Sender(err) => err.into(),
            TransferPostError::<T>::Receiver(err) => err.into(),
            TransferPostError::<T>::DuplicateMint => Self::DuplicateRegister,
            TransferPostError::<T>::DuplicateSpend => Self::DuplicateSpend,
            TransferPostError::<T>::InvalidProof => Self::InvalidProof,
            TransferPostError::<T>::UnexpectedError(e) => match e {
                TransferLedgerError::ChecksumError => Self::TransferLedgerChecksumError,
                TransferLedgerError::VerifyingContextDecodeError(_) => {
                    Self::TransferLedgerVerifyingContextDecodeError
                }
                TransferLedgerError::FpEncodeError(_) => Self::TransferLedgerFpEncodeError,
                TransferLedgerError::FungibleLedgerError(err) => err.into(),
                TransferLedgerError::UnknownAsset => Self::TransferLedgerUnknownAsset,
                TransferLedgerError::InvalidTransferShape => Self::InvalidShape,
                TransferLedgerError::ProofSystemError(_) => Self::TransferLedgerProofSystemFailed,
                TransferLedgerError::InvalidProof => Self::InvalidProof,
                TransferLedgerError::Marker(_) => Self::Marker,
                TransferLedgerError::SenderLedgerError(err) => SenderPostError::from(err).into(),
                TransferLedgerError::InvalidAssetId => Self::InvalidAssetId,
                TransferLedgerError::ReceiverLedgerError(err) => {
                    ReceiverPostError::from(err).into()
                }
            },
        }
    }
}

/// Transfer Ledger Error
pub enum TransferLedgerError<T>
where
    T: Config,
{
    /// Wrong Checksum Error
    ChecksumError,

    /// Verifying Context Decoding Error
    VerifyingContextDecodeError(VerifyingContextError),

    /// Field Element Encoding Error
    FpEncodeError(scale_codec::Error),

    /// Unknown Asset Error
    UnknownAsset,

    /// Fungible Ledger Error
    FungibleLedgerError(FungibleLedgerError),

    /// Sender Ledger Error
    SenderLedgerError(SenderLedgerError),

    /// Receiver Ledger Error
    ReceiverLedgerError(ReceiverLedgerError<T>),

    /// Invalid Transfer Shape
    InvalidTransferShape,

    /// Proof System Error
    ProofSystemError(ProofSystemError<config::Config>),

    /// Invalid Transfer Proof Error
    ///
    /// Validity of the transfer could not be proved by the ledger.
    InvalidProof,

    /// Invalid AssetId, cannot have a value of zero
    InvalidAssetId,

    /// Type Marker Parameter
    Marker(PhantomData<T>),
}

impl<T> From<TransferLedgerError<T>> for TransferPostError<T>
where
    T: Config,
{
    #[inline]
    fn from(value: TransferLedgerError<T>) -> Self {
        match value {
            TransferLedgerError::InvalidProof => Self::InvalidProof,
            TransferLedgerError::InvalidTransferShape => Self::InvalidShape,
            TransferLedgerError::SenderLedgerError(err) => Self::Sender(err.into()),
            TransferLedgerError::ReceiverLedgerError(err) => Self::Receiver(err.into()),
            err => Self::UnexpectedError(err),
        }
    }
}
