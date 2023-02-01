use crate::{Config, Event};
use crate::types::Asset;

/// Wrap Type
#[derive(Clone, Copy)]
pub struct Wrap<T>(pub T);

impl<T> AsRef<T> for Wrap<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// Wrap Pair Type
#[derive(Clone, Copy)]
pub struct WrapPair<L, R>(pub L, pub R);

impl<L, R> AsRef<R> for WrapPair<L, R> {
    #[inline]
    fn as_ref(&self) -> &R {
        &self.1
    }
}

/// Preprocessed Event
pub enum PreprocessedEvent<T>
    where
        T: Config,
{
    /// To Private Event
    ToPrivate {
        /// Asset Minted
        asset: Asset,

        /// Source Account
        source: T::AccountId,
    },

    /// Private Transfer Event
    PrivateTransfer,

    /// To Public Event
    ToPublic {
        /// Asset Reclaimed
        asset: Asset,

        /// Sink Account
        sink: T::AccountId,
    },
}

impl<T> PreprocessedEvent<T>
    where
        T: Config,
{
    /// Converts a [`PreprocessedEvent`] with into an [`Event`] using the given `origin` for
    /// [`PreprocessedEvent::PrivateTransfer`].
    #[inline]
    pub fn convert(self, origin: Option<T::AccountId>) -> Event<T> {
        match self {
            Self::ToPrivate { asset, source } => Event::ToPrivate { asset, source },
            Self::PrivateTransfer => Event::PrivateTransfer { origin },
            Self::ToPublic { asset, sink } => Event::ToPublic { asset, sink },
        }
    }
}
