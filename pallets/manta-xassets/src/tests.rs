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

use crate as manta_xassets;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use manta_primitives::currency_id::{CurrencyId, TokenSymbol};
use xcm_simulator::TestExt;

#[test]
fn send_para_tokens_to_sibling_para() {
	TestNet::reset();

	let currency_id = CurrencyId::Token(TokenSymbol::MANTA);
	let amount = 50;
	MantaPara::execute_with(|| {
		assert_ok!(parachain::MantaXassets::transfer_to_parachain(
			parachain::Origin::signed(ALICE),
			2084.into(),
			BOB,
			currency_id,
			amount
		));
		// assert_eq!(manta_xassets::Pallet::<parachain::Runtime>::xtokens(currency_id, ALICE), 1_000);
	});

	CalamariPara::execute_with(|| {
		assert_eq!(
			manta_xassets::Pallet::<parachain::Runtime>::xtokens(currency_id, BOB),
			1_000
		);
	});
}
