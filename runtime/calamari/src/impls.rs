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

use crate::{
    fee::{
        FEES_PERCENTAGE_TO_AUTHOR, FEES_PERCENTAGE_TO_BURN, FEES_PERCENTAGE_TO_TREASURY,
        TIPS_PERCENTAGE_TO_AUTHOR, TIPS_PERCENTAGE_TO_TREASURY,
    },
    Authorship, Balances, NegativeImbalance, Treasury,
};
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use sp_arithmetic::Percent;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        if let Some(author) = Authorship::author() {
            Balances::resolve_creating(&author, amount);
        }
    }
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            const REMAINING_PERCENTAGE_AFTER_COLLATOR_SPLIT: u8 = 100 - FEES_PERCENTAGE_TO_AUTHOR;
            let (mut to_author, rest) = fees.ration(
                FEES_PERCENTAGE_TO_AUTHOR as u32,
                REMAINING_PERCENTAGE_AFTER_COLLATOR_SPLIT as u32,
            );

            // NOTE: `from_rational` always rounds DOWN, so if these don't divide cleanly, we'll burn more and distribute less
            let to_treasury_from_rest = Percent::from_rational(
                FEES_PERCENTAGE_TO_TREASURY,
                REMAINING_PERCENTAGE_AFTER_COLLATOR_SPLIT,
            )
            .deconstruct();
            let to_burn_from_rest = Percent::from_rational(
                FEES_PERCENTAGE_TO_BURN,
                REMAINING_PERCENTAGE_AFTER_COLLATOR_SPLIT,
            )
            .deconstruct();
            let (mut to_treasury, _to_burn) =
                rest.ration(to_treasury_from_rest as u32, to_burn_from_rest as u32);

            if let Some(tips) = fees_then_tips.next() {
                let (tips_to_treasury, tips_to_author) = tips.ration(
                    TIPS_PERCENTAGE_TO_TREASURY as u32,
                    TIPS_PERCENTAGE_TO_AUTHOR as u32,
                );
                tips_to_treasury.merge_into(&mut to_treasury);
                tips_to_author.merge_into(&mut to_author);
            }
            Treasury::on_unbalanced(to_treasury);
            Author::on_unbalanced(to_author);
        }
    }
}
