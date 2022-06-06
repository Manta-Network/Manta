// Copyright 2019-2021 PureStake Inc.
// This file is part of Nimbus.

// Nimbus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Nimbus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Nimbus.  If not, see <http://www.gnu.org/licenses/>.

use parity_scale_codec::Encode;
use sp_inherents::{InherentData, InherentIdentifier};

/// The InherentIdentifier for nimbus's author inherent
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

/// A thing that an outer node could use to inject the inherent data.
/// This should be used in simple uses of the author inherent (eg permissionless authoring)
/// When using the full nimbus system, we are manually inserting the inherent.
pub struct InherentDataProvider<AuthorId>(pub AuthorId);

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl<AuthorId: Encode + Send + Sync> sp_inherents::InherentDataProvider
	for InherentDataProvider<AuthorId>
{
	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
	}

	async fn try_handle_error(
		&self,
		identifier: &InherentIdentifier,
		_error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		// Dont' process modules from other inherents
		if *identifier != INHERENT_IDENTIFIER {
			return None;
		}

		// All errors with the author inehrent are fatal
		Some(Err(sp_inherents::Error::Application(Box::from(
			String::from("Error processing author inherent"),
		))))
	}
}
