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

use crate::{Authorship, Balances, NegativeImbalance};
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		Balances::resolve_creating(&Authorship::author(), amount);
	}
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 0% to treasury, 100% to author
			let mut split = fees.ration(0, 100);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 0% to treasury, 100% to author (though this can be anything)
				tips.ration_merge_into(0, 100, &mut split);
			}
			// Todo, we should deposit fees to treasury as well once we enable treasury.
			// Like 80% to treasury, 20% to block author
			// Treasury::on_unbalanced(split.0);
			Author::on_unbalanced(split.1);
		}
	}
}
