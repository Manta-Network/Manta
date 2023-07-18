/// All error kinds related to the light client.
use crate::alloc::string::String;
#[derive(Clone, Debug)]
pub enum Kind {
    // #[error("invalid data length while converting slice to fixed-size array type ({current} != {expected}")]
    InvalidDataLength { current: usize, expected: usize },

    // #[error("rlp decode error")]
    RlpDecodeError,

    // #[error("aggregated seal does not aggregate enough seals, num_seals: {current}, minimum quorum size: {expected}")]
    MissingSeals { current: usize, expected: usize },

    // #[error("BLS verify error")]
    BlsVerifyError,

    // #[error("BLS invalid signature")]
    BlsInvalidSignature,

    // #[error("BLS invalid public key")]
    BlsInvalidPublicKey,

    // #[error("header verification failed: {msg}")]
    HeaderVerificationError { msg: &'static str },

    HeaderError,

    // #[error("unknown error occurred")]
    Unknown,
    Other { msg: String },
}
