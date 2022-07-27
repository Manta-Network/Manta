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

use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use manta_primitives::types::Balance;
use smallvec::smallvec;
pub use sp_runtime::Perbill;

/// The block saturation level. Fees will be updates based on this value.
pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

pub const FEES_PERCENTAGE_TO_AUTHOR: u8 = 60;
pub const FEES_PERCENTAGE_TO_TREASURY: u8 = 40;

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
            coeff_integer: 5000u32.into(),
            coeff_frac: Perbill::zero(),
            negative: false,
            degree: 1,
        }]
    }
}

#[cfg(test)]
mod multiplier_tests {
    use crate::{
        Call, Runtime, RuntimeBlockWeights as BlockWeights, System, TransactionPayment, KMA,
    };
    use codec::Encode;
    use frame_support::weights::{DispatchClass, Weight, WeightToFeePolynomial};
    use frame_system::WeightInfo;
    use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
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

    // Consider the daily cost to fully congest our network to be defined as:
    // `target_daily_congestion_cost_usd = inclusion_fee * blocks_per_day * kma_price`
    // Where:
    // `inclusion_fee = fee_adjustment * (weight_to_fee_coeff * (block_weight ^ degree)) + base_fee + (len_to_fee_coeff * length_fee)`
    // Where:
    // `fee_adjustment`, `weight_to_fee_coeff` and `len_to_fee_coeff` are configurable in a runtime via:
    // `FeeMultiplierUpdate` and `WeightToFee` and `LengthToFee`
    // `fee_adjustment` is also variable depending on previous block's fullness
    // We are also assuming `length_fee` is negligible for small TXs like a remark or a transfer.
    // This test loops 1 day of parachain blocks (7200) and calculates accumulated fee if every block is almost full
    #[test]
    fn congested_chain_simulation() {
        // Configure the target cost depending on the current state of the network.
        let target_daily_congestion_cost_usd = 250000;
        let kma_price = fetch_kma_price().unwrap();
        println!("KMA/USD price as read from CoinGecko = {}", kma_price);
        let target_daily_congestion_cost_kma =
            (target_daily_congestion_cost_usd as f32 * kma_price * KMA as f32) as u128;

        // `cargo test --package calamari-runtime --lib -- fee::multiplier_tests::congested_chain_simulation --exact --nocapture` to get some insight.
        // almost full. The entire quota of normal transactions is taken.
        let block_weight = BlockWeights::get()
            .get(DispatchClass::Normal)
            .max_total
            .unwrap()
            - 10;

        // remark extrinsic is chosen arbitrarily for benchmark as a small, constant size TX.
        let remark = Call::System(frame_system::Call::<Runtime>::remark_with_event {
            remark: vec![1, 2, 3],
        });
        let len: u32 = remark.encode().len() as u32;
        let remark_weight: Weight =
            <Runtime as frame_system::Config>::SystemWeightInfo::remark(len);
        let max_number_of_remarks_per_block = (block_weight / remark_weight) as u128;
        let len_fee = max_number_of_remarks_per_block.saturating_mul(
            <Runtime as pallet_transaction_payment::Config>::LengthToFee::calc(&(len as Weight)),
        );

        let base_fee = max_number_of_remarks_per_block
            * <Runtime as pallet_transaction_payment::Config>::WeightToFee::calc(
                &frame_support::weights::constants::ExtrinsicBaseWeight::get(),
            );

        run_with_system_weight(block_weight, || {
            // initial value configured on module
            let mut fee_adjustment = Multiplier::one();
            assert_eq!(fee_adjustment, TransactionPayment::next_fee_multiplier());
            let mut accumulated_fee: u128 = 0;
            // Simulates 1 day of parachain blocks (12 seconds each)
            for iteration in 0..7200 {
                let next = runtime_multiplier_update(fee_adjustment);
                // if no change or less, panic. This should never happen in this case.
                if fee_adjustment >= next {
                    println!("final fee_adjustment: {}", fee_adjustment);
                    println!("final next: {}", next);
                    panic!("The fee should ever increase");
                }
                fee_adjustment = next;
                let fee = <Runtime as pallet_transaction_payment::Config>::WeightToFee::calc(
                    &block_weight,
                );

                // base_fee and len_fee are not adjusted:
                // https://docs.substrate.io/main-docs/build/tx-weights-fees/#:~:text=A%20closer%20look%20at%20the%20inclusion%20fee
                let adjusted_fee = fee_adjustment.saturating_mul_acc_int(fee) + base_fee + len_fee;
                accumulated_fee += adjusted_fee;
                println!(
                    "Iteration {}, New fee_adjustment = {:?}. Adjusted Fee: {} KMA, Total Fee: {} KMA, Dollar Value: {}",
                    iteration,
                    fee_adjustment,
                    adjusted_fee / KMA,
                    accumulated_fee / KMA,
                    (accumulated_fee / KMA) as f32 * kma_price,
                );
            }

            if accumulated_fee < target_daily_congestion_cost_kma {
                panic!("The cost to fully congest our network should be over the target_daily_congestion_cost_kma after 1 day.");
            }
        });
    }

    #[test]
    fn multiplier_can_grow_from_zero() {
        let minimum_multiplier = MinimumMultiplier::get();
        let target = TargetBlockFullness::get()
            * BlockWeights::get()
                .get(DispatchClass::Normal)
                .max_total
                .unwrap();
        // if the min is too small, then this will not change, and we are doomed forever.
        // the weight is 1/100th bigger than target.
        run_with_system_weight(target * 101 / 100, || {
            let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
            assert!(
                next > minimum_multiplier,
                "{:?} !>= {:?}",
                next,
                minimum_multiplier
            );
        })
    }

    #[test]
    #[ignore] // test runs for a very long time
    fn multiplier_growth_simulator() {
        // assume the multiplier is initially set to its minimum. We update it with values twice the
        //target (target is 25%, thus 50%) and we see at which point it reaches 1.
        let mut multiplier = MinimumMultiplier::get();
        let block_weight = TargetBlockFullness::get()
            * BlockWeights::get()
                .get(DispatchClass::Normal)
                .max_total
                .unwrap()
            * 2;
        let mut blocks = 0;
        while multiplier <= Multiplier::one() {
            run_with_system_weight(block_weight, || {
                let next = SlowAdjustingFeeUpdate::<Runtime>::convert(multiplier);
                // ensure that it is growing as well.
                assert!(next > multiplier, "{:?} !>= {:?}", next, multiplier);
                multiplier = next;
            });
            blocks += 1;
            println!("block = {} multiplier {:?}", blocks, multiplier);
        }
    }
}
