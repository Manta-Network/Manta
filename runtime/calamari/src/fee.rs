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

pub const FEES_PERCENTAGE_TO_AUTHOR: u8 = 10;
pub const FEES_PERCENTAGE_TO_BURN: u8 = 45;
pub const FEES_PERCENTAGE_TO_TREASURY: u8 = 45;

pub const TIPS_PERCENTAGE_TO_AUTHOR: u8 = 100;
pub const TIPS_PERCENTAGE_TO_TREASURY: u8 = 0;

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
        sp_api_hidden_includes_construct_runtime::hidden_include::traits::Hooks, Runtime,
        RuntimeBlockWeights as BlockWeights, System, TransactionPayment, KMA,
    };
    use frame_support::dispatch::{DispatchClass, DispatchInfo};
    use manta_primitives::constants::time::DAYS;
    use pallet_transaction_payment::Multiplier;
    use runtime_common::MinimumMultiplier;

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
    #[ignore] // This test should not fail CI
    fn multiplier_growth_simulator_and_congestion_budget_test() {
        let target_daily_congestion_cost_usd = 100_000;
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
        let mut fees_to_1x = 0;
        let mut blocks_to_1x = 0;

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
        while multiplier <= Multiplier::from_u32(1) || blocks <= 7200 {
            t.execute_with(|| {
                // Give the attacker super powers to not pay tx-length fee
                // The maximum length fo a block is 3_670_016 on Calamari
                let len = 0;
                // Imagine this tx was called. This means that he will pay a single base-fee,
                // which is additional super powers for the attacker
                let fee = TransactionPayment::compute_fee(len, &info, 0);
                fees_paid += fee;

                // this will update the multiplier.
                System::set_block_consumed_resources(block_weight, len.try_into().unwrap());
                TransactionPayment::on_finalize(1);
                let next = TransactionPayment::next_fee_multiplier();

                assert!(next > multiplier, "{next:?} !>= {multiplier:?}");
                multiplier = next;

                println!(
                    "block = {} / multiplier {:?} / fee = {:?} / fees so far {:?} / fees so far in USD {:?}",
                    blocks, multiplier, fee, fees_paid, (fees_paid as f32 / KMA as f32) * kma_price
                );

                if blocks == 7200 {
                    fees_after_one_day = fees_paid;
                    if fees_paid < target_daily_congestion_cost_kma {
                        should_fail = true;
                    }
                }

                if multiplier <= Multiplier::from_u32(1u32) {
                    fees_to_1x = fees_paid;
                    blocks_to_1x = blocks;
                }
            });
            blocks += 1;
        }

        println!(
            "It will take {:?} days to reach multiplier of 1x at a total cost of USD {:?}",
            blocks_to_1x as f32 / DAYS as f32,
            (fees_to_1x as f32 / KMA as f32) * kma_price
        );

        println!(
            "Cost for 1 day in KMA {:?} and in USD {:?}",
            fees_after_one_day,
            (fees_after_one_day as f32 / KMA as f32) * kma_price
        );

        if should_fail {
            panic!("The cost to fully congest our network should be over the target_daily_congestion_cost_kma after 1 day.");
        }
    }

    #[test]
    fn multiplier_cool_down_simulator() {
        // Start from multiplier of 1 to see how long it will take to cool-down to the minimum
        let mut multiplier = Multiplier::from_u32(1);
        let mut blocks = 0;

        let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap()
            .into();

        t.execute_with(|| {
            pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
        });

        while multiplier > MinimumMultiplier::get() {
            t.execute_with(|| {
                // this will update the multiplier.
                TransactionPayment::on_finalize(1);
                let next = TransactionPayment::next_fee_multiplier();

                assert!(next < multiplier, "{next:?} !>= {multiplier:?}");
                multiplier = next;

                println!("block = {blocks} / multiplier {multiplier:?}");
            });
            blocks += 1;
        }

        let cooldown_target = 10f32;
        let days = blocks as f32 / DAYS as f32;
        if days > cooldown_target {
            panic!("It will take more than 10 days to cool down: {days:?}");
        }
    }
}
