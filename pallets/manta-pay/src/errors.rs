use crate::types::AssetValue;
use crate::{Config, Error};
use manta_accounting::transfer;
use manta_accounting::transfer::receiver::ReceiverPostError;
use manta_accounting::transfer::sender::SenderPostError;
use manta_accounting::transfer::{
    InvalidAuthorizationSignature, InvalidSinkAccount, InvalidSourceAccount,
};
use manta_pay::{config, manta_accounting::transfer::ProofSystemError};
use manta_primitives::assets;
use manta_primitives::types::StandardAssetId;
use sp_std::marker::PhantomData;

/// Fungible Ledger Error
pub type FungibleLedgerError = assets::FungibleLedgerError<StandardAssetId, AssetValue>;

impl<T> From<InvalidAuthorizationSignature> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(_: InvalidAuthorizationSignature) -> Self {
        Self::InvalidAuthorizationSignature
    }
}

impl<T> From<InvalidSourceAccount<config::Config, T::AccountId>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(_: InvalidSourceAccount<config::Config, T::AccountId>) -> Self {
        Self::InvalidSourceAccount
    }
}

impl<T> From<InvalidSinkAccount<config::Config, T::AccountId>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(_: InvalidSinkAccount<config::Config, T::AccountId>) -> Self {
        Self::InvalidSinkAccount
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
            SenderPostError::UnexpectedError(_) => Self::InternalLedgerError,
        }
    }
}

impl<T> From<ReceiverPostError<ReceiverLedgerError<T>>> for Error<T>
where
    T: Config,
{
    #[inline]
    fn from(err: ReceiverPostError<ReceiverLedgerError<T>>) -> Self {
        match err {
            ReceiverPostError::AssetRegistered => Self::AssetRegistered,
            ReceiverPostError::UnexpectedError(e) => {
                match e {
                    ReceiverLedgerError::ChecksumError
                    | ReceiverLedgerError::MerkleTreeCapacityError => {
                        // T::Suspender::suspend_manta_pay_execution();
                    }
                    _ => {}
                };
                Self::InternalLedgerError
            }
        }
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
            FungibleLedgerError::EncodeError => Self::EncodeError,
        }
    }
}
use manta_util::codec;

/// Verification Context Decode Error Type
pub type VerifyingContextError = codec::DecodeError<
    <&'static [u8] as codec::Read>::Error,
    <config::VerifyingContext as codec::Decode>::Error,
>;

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
            TransferLedgerError::UnknownAsset => Self::InvalidProof,
            err => {
                match err {
                    TransferLedgerError::ChecksumError
                    | TransferLedgerError::VerifyingContextDecodeError(_) => {
                        // T::Suspender::suspend_manta_pay_execution();
                    }
                    _ => {}
                };
                Self::UnexpectedError(err)
            }
        }
    }
}

/// Transfer Post Error
pub type TransferPostError<T> = transfer::TransferPostError<
    config::Config,
    <T as frame_system::Config>::AccountId,
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
            TransferPostError::<T>::UnexpectedError(_) => err.into(),
        }
    }
}

/// Merkle Tree Parameters Decode Error Type
pub type MTParametersError = codec::DecodeError<
    <&'static [u8] as codec::Read>::Error,
    <config::UtxoAccumulatorModel as codec::Decode>::Error,
>;

/// Utxo Accumulator Item Hash Decode Error Type
pub type UtxoItemHashError = codec::DecodeError<
    <&'static [u8] as codec::Read>::Error,
    <config::utxo::UtxoAccumulatorItemHash as codec::Decode>::Error,
>;

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

impl<T> From<ReceiverLedgerError<T>> for ReceiverPostError<ReceiverLedgerError<T>>
where
    T: Config,
{
    #[inline]
    fn from(value: ReceiverLedgerError<T>) -> Self {
        if let ReceiverLedgerError::AssetRegistered = value {
            Self::AssetRegistered
        } else {
            match value {
                ReceiverLedgerError::ChecksumError
                | ReceiverLedgerError::MerkleTreeCapacityError => {
                    // T::Suspender::suspend_manta_pay_execution();
                }
                _ => {}
            };
            Self::UnexpectedError(value)
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

impl<T> From<ReceiverLedgerError<T>> for TransferLedgerError<T>
where
    T: Config,
{
    #[inline]
    fn from(err: ReceiverLedgerError<T>) -> Self {
        match err {
            ReceiverLedgerError::ChecksumError | ReceiverLedgerError::MerkleTreeCapacityError => {
                // T::Suspender::suspend_manta_pay_execution();
            }
            _ => {}
        };
        TransferLedgerError::ReceiverLedgerError(err)
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
