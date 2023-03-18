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

use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use manta_primitives::types::Balance;
use smallvec::smallvec;
pub use sp_runtime::Perbill;

/// The block saturation level. Fees will be updates based on this value.
pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

pub const FEES_PERCENTAGE_TO_AUTHOR: u8 = 10;
pub const FEES_PERCENTAGE_TO_BURN: u8 = 45;
pub const FEES_PERCENTAGE_TO_TREASURY: u8 = 45;

pub const TIPS_PERCENTAGE_TO_AUTHOR: u8 = 100;
pub const TIPS_PERCENTAGE_TO_TREASURY: u8 = 0;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // Refer to the congested_chain_simulation() test for how to come up with the coefficient.
        smallvec![WeightToFeeCoefficient {
            coeff_integer: 50_000u32.into(),
            coeff_frac: Perbill::zero(),
            negative: false,
            degree: 1,
        }]
    }
}

#[cfg(test)]
mod fee_split_tests {
    use super::*;
    #[test]
    fn fee_split_adds_up_to_one() {
        assert_eq!(
            100,
            FEES_PERCENTAGE_TO_AUTHOR + FEES_PERCENTAGE_TO_BURN + FEES_PERCENTAGE_TO_TREASURY
        );
    }
    #[test]
    fn tips_split_adds_up_to_one() {
        assert_eq!(100, TIPS_PERCENTAGE_TO_AUTHOR + TIPS_PERCENTAGE_TO_TREASURY);
    }
}
#[cfg(test)]
mod multiplier_tests {
    use crate::{
        Call, Runtime, RuntimeBlockWeights as BlockWeights, System, TransactionPayment, KMA,
    };
    use codec::Encode;
    use frame_support::{
        dispatch::DispatchInfo,
        weights::{DispatchClass, Weight, WeightToFee},
    };
    use frame_system::WeightInfo;
    use pallet_transaction_payment::{
        CurrencyAdapter, FeeDetails, Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment,
    };
    use runtime_common::{
        AdjustmentVariable, MinimumMultiplier, SlowAdjustingFeeUpdate, TargetBlockFullness,
    };
    use sp_runtime::{
        traits::{Convert, One},
        FixedPointNumber,
    };

    fn run_with_system_weight<F>(w: Weight, mut assertions: F)
    where
        F: FnMut(),
    {
        let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap()
            .into();
        t.execute_with(|| {
            System::set_block_consumed_resources(w, 0);
            assertions()
        });
    }

    // update based on runtime impl.
    fn runtime_multiplier_update(fm: Multiplier) -> Multiplier {
        TargetedFeeAdjustment::<
            Runtime,
            TargetBlockFullness,
            AdjustmentVariable,
            MinimumMultiplier,
        >::convert(fm)
    }

    fn fetch_kma_price() -> Result<f32, &'static str> {
        let body = reqwest::blocking::get(
            "https://api.coingecko.com/api/v3/simple/price?ids=calamari-network&vs_currencies=usd",
        )
        .unwrap();
        let json_reply: serde_json::Value = serde_json::from_reader(body).unwrap();
        if let Some(price) = json_reply["calamari-network"]["usd"].as_f64() {
            // CG API return: {"calamari-network":{"usd": 0.01092173}}
            Ok(price as f32)
        } else {
            Err("KMA price not found in reply from Coingecko. API changed? Check https://www.coingecko.com/en/api/documentation")
        }
    }

    #[test]
    #[ignore]
    fn multiplier_growth_simulator_and_congestion_budget_test() {
        let target_daily_congestion_cost_usd = 250_000;
        let kma_price = fetch_kma_price().unwrap();
        println!("KMA/USD price as read from CoinGecko = {kma_price}");
        let target_daily_congestion_cost_kma =
            (target_daily_congestion_cost_usd as f32 / kma_price * KMA as f32) as u128;

        // assume the multiplier is initially set to its minimum and that each block
        // will be full with a single user extrinsic, which will minimize then length and base fees
        let mut multiplier = MinimumMultiplier::get();
        let block_weight = BlockWeights::get()
            .get(DispatchClass::Normal)
            .max_total
            .unwrap();

        let mut blocks = 0;
        let mut fees_paid = 0;
        let mut fees_after_one_day = 0;

        let info = DispatchInfo {
            weight: block_weight,
            ..Default::default()
        };

        let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap()
            .into();
        // set the minimum
        t.execute_with(|| {
            pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(MinimumMultiplier::get());
        });

        let mut should_fail = false;
        while multiplier <= Multiplier::from_u32(1) {
            t.execute_with(|| {
                frame_system::Pallet::<Runtime>::set_block_consumed_resources(Weight::MAX, 0);
                // let len = <Runtime as frame_system::Config>::BlockLength::get();
                let len = 0;//3_670_016;
                // imagine this tx was called.
                let fee = TransactionPayment::compute_fee(len, &info, 0);
                fees_paid += fee;

                // this will update the multiplier.
                System::set_block_consumed_resources(block_weight, len.try_into().unwrap());
                use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::Hooks;
                TransactionPayment::on_finalize(1);
                let next = TransactionPayment::next_fee_multiplier();

                assert!(next > multiplier, "{:?} !>= {:?}", next, multiplier);
                multiplier = next;

                println!(
                    "block = {} / multiplier {:?} / fee = {:?} / fees so far {:?} / fees so far in USD {:?}",
                    blocks, multiplier, fee, fees_paid, (fees_paid / KMA) as f32 * kma_price as f32
                );

                if blocks == 7200 {
                    fees_after_one_day = fees_paid;
                    if fees_paid < target_daily_congestion_cost_kma {
                        should_fail = true;
                    }
                }
            });
            blocks += 1;
        }

        println!(
            "It will take {:?} days to reach multiplier of 1",
            (blocks * 12) as f32 / (60 * 60 * 24) as f32,
        );

        println!(
            "Cost for 1 day in KMA {:?} and in USD {:?}",
            fees_after_one_day,
            (fees_after_one_day / KMA) as f32 * kma_price as f32
        );

        if should_fail {
            panic!("The cost to fully congest our network should be over the target_daily_congestion_cost_kma after 1 day.");
        }
    }
}
