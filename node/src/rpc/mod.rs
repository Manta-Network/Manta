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

//! Parachain-specific RPCs implementation.

use alloc::sync::Arc;
use core::marker::PhantomData;

mod common;
mod dolphin;

pub use common::Common;
pub use dolphin::Dolphin;

/// RPC Extension Type
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// RPC Extension Builder
pub struct Builder<C, P, M = common::Common> {
    /// Client
    client: Arc<C>,

    /// Transaction Poool
    transaction_pool: Arc<P>,

    /// Runtime Marker
    __: PhantomData<M>,
}

impl<C, P, M> Builder<C, P, M> {
    /// Builds a new RPC Extension [`Builder`] from `client` and `transaction_pool`.
    #[inline]
    pub fn new(client: Arc<C>, transaction_pool: Arc<P>) -> Self {
        Self {
            client,
            transaction_pool,
            __: PhantomData,
        }
    }

    /// Converts `self` into a [`Builder`] with the `T` marker.
    #[inline]
    pub fn using<T>(&self) -> Builder<C, P, T> {
        Builder::new(self.client.clone(), self.transaction_pool.clone())
    }
}
