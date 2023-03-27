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

pub use sp_runtime::Perbill;

/// The block saturation level. Fees will be updates based on this value.
pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

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
    use frame_support::{dispatch::DispatchInfo, weights::DispatchClass};
    use pallet_transaction_payment::Multiplier;
    use runtime_common::MinimumMultiplier;
    use sp_runtime::FixedU128;

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
    fn multiplier_growth_simulator_and_congestion_budget_test() {
        let target_daily_congestion_cost_usd = 100_000;
        //let kma_price = fetch_kma_price().unwrap();
        let kma_price = 0.002318f32;
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
        let mut quarter_block = 0;
        let mut half_block = 0;
        let mut one_block = 0;
        let mut two_block = 0;
        let mut three_block = 0;
        let mut ten_block = 0;
        let mut hundred_block = 0;
        let mut fees_at_quarter_multiplier = 0;
        let mut fees_at_half_multiplier = 0;
        let mut fees_at_1x_multiplier = 0;
        let mut fees_at_2x_multiplier = 0;
        let mut fees_at_3x_multiplier = 0;
        let mut fees_at_10x_multiplier = 0;
        let mut fees_at_100x_multiplier = 0;
        let mut fee_at_quarter = 0;
        let mut fee_at_half = 0;
        let mut fee_at_0 = 0;
        let mut fee_at_1 = 0;
        let mut fee_at_2 = 0;
        let mut fee_at_3 = 0;
        let mut fee_at_10 = 0;
        let mut fee_at_100 = 0;

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

        //let mut multipliers = vec![];
        // let base = 0.0002f32;
        // let chunk = (1.0f32 - 0.0002f32) / 20.0f32;
        // let mc = 5000 / 20;
        // for i in 0..20 {
        //     let next = base + i as f32 * chunk;
        //     multipliers.push(5000 - i * mc);
        // }
        // multipliers[19] = 1;

        // let mut multipliers = vec![1_052_620_497_623_580_000u128,
        // 1_111_086_416_598_230_000u128,
        // 1_176_429_061_673_450_000u128,
        // 1_249_937_492_188_440_000u128,
        // 1_333_244_436_149_650_000u128,
        // 1_428_449_092_109_900_000u128,
        // 1_538_295_849_798_860_000u128,
        // 1_666_444_437_968_640_000u128,
        // 1_817_884_494_909_560_000u128,
        // 1_999_600_012_011_200_000u128,
        // 1_999_600_012_011_200_000u128,
        // 2_221_679_056_252_420_000u128,
        // 2_499_250_100_007_490_000u128,
        // 2_856_081_847_217_320_000u128,
        // 3_331_778_248_047_910_000u128,
        // 3_997_601_519_040_610_000u128,
        // 4_996_002_897_921_490_000u128,
        // 6_659_119_087_910_050_000u128,
        // 9_982_031_544_657_100_000u128,
        // 19_924_287_031_853_200_000u128,
        // 5_000_000_000_000_000_000_000u128
        // ];

        let incr = Multiplier::from_u32(1) / 20.into();
        let mut multipliers = vec![Multiplier::from_u32(1) / 5000.into(), incr];
        for i in 2..20 {
            multipliers.push(multipliers[i -1] + incr);
        }
        multipliers[19] = Multiplier::from_u32(1);

        let mut fee_at = vec![0; 20];
        let mut fees_so_far_at = vec![0; 20];
        let mut blocks_at = vec![0; 20];

        let mut should_fail = false;
        while multiplier <= Multiplier::from_u32(100) || blocks <= 7200 {
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

                assert!(next > multiplier, "{:?} !>= {:?}", next, multiplier);
                multiplier = next;

                println!(
                    "block = {} / multiplier {:?} / fee = {:?} / fees so far {:?} / fees so far in USD {:?}",
                    blocks, multiplier, (fee as f32 / KMA as f32) * kma_price, fees_paid, (fees_paid as f32 / KMA as f32) * kma_price
                );
                
                if blocks == 0 {
                    fee_at_0 = fee;
                }
                if blocks == 7200 {
                    fees_after_one_day = fees_paid;
                    if fees_paid < target_daily_congestion_cost_kma {
                        should_fail = true;
                    }
                }
                if multiplier <= Multiplier::from_u32(1) / 4.into() {
                    fees_at_quarter_multiplier = fees_paid;
                    quarter_block = blocks;
                    fee_at_quarter = fee;
                }
                if multiplier <= Multiplier::from_u32(1) / 2.into() {
                    fees_at_half_multiplier = fees_paid;
                    half_block = blocks;
                    fee_at_half = fee;
                }
                if multiplier <= Multiplier::from_u32(1) {
                    fees_at_1x_multiplier = fees_paid;
                    one_block = blocks;
                    fee_at_1 = fee;
                }
                if multiplier <= Multiplier::from_u32(2) {
                    fees_at_2x_multiplier = fees_paid;
                    two_block = blocks;
                    fee_at_2 = fee;
                }
                if multiplier <= Multiplier::from_u32(3) {
                    fees_at_3x_multiplier = fees_paid;
                    three_block = blocks;
                    fee_at_3 = fee;
                }
                if multiplier <= Multiplier::from_u32(10) {
                    fees_at_10x_multiplier = fees_paid;
                    ten_block = blocks;
                    fee_at_10 = fee;
                }
                if multiplier <= Multiplier::from_u32(100) {
                    fees_at_100x_multiplier = fees_paid;
                    hundred_block = blocks;
                    fee_at_100 = fee;
                }

                for i in 0 ..20 {
                    let check = multipliers[i].clone();
                    if multiplier <= check {
                         fee_at[i] = fee;
                         fees_so_far_at[i] = fees_paid;
                         blocks_at[i] = blocks;
                    }
                }
            });
            blocks += 1;
        }

        println!(
            "The single block cost at the start is: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},.
            {:?} for 0.25x at a cost   {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},
            {:?} for 0.5x at a cost of {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},
            {:?} for 1x at a cost of   {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},
            {:?} for 2x at a cost of   {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},
            {:?} for 3x at a cost of   {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},
            {:?} for 10x at a cost of  {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},
            {:?} for 100x at a cost of {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?},",
            (fee_at_0 as f32 / KMA as f32) * kma_price,
            ((fee_at_0 as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_0 as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_0,

            (quarter_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_quarter_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_quarter as f32 / KMA as f32) * kma_price,
            ((fee_at_quarter as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_quarter as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_quarter,

            (half_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_half_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_half as f32 / KMA as f32) * kma_price,
            ((fee_at_half as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_half as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_half,

            (one_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_1x_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_1 as f32 / KMA as f32) * kma_price,
            ((fee_at_1 as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_1 as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_1,

            (two_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_2x_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_2 as f32 / KMA as f32) * kma_price,
            ((fee_at_2 as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_2 as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_2,

            (three_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_3x_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_3 as f32 / KMA as f32) * kma_price,
            ((fee_at_3 as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_3 as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_3,

            (ten_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_10x_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_10 as f32 / KMA as f32) * kma_price,
            ((fee_at_10 as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_10 as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_10,

            (hundred_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_100x_multiplier as f32 / KMA as f32) * kma_price,
            (fee_at_100 as f32 / KMA as f32) * kma_price,
            ((fee_at_100 as f32 / KMA as f32) * kma_price) * 0.0003f32,
            ((fee_at_100 as f32 / KMA as f32) * kma_price) * 0.15f32,
            fee_at_100,
        );

        for i in 0 .. 20 {
            println!(
                "{:?} days for {:?}x at a cost {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in KMA {:?}",
                (blocks_at[i] * 12) as f32 / (60 * 60 * 24) as f32,
                multipliers[i],
                (fees_so_far_at[i] as f32 / KMA as f32) * kma_price,
                (fee_at[i] as f32 / KMA as f32) * kma_price,
                ((fee_at[i] as f32 / KMA as f32) * kma_price) * 0.0003f32,
                ((fee_at[i] as f32 / KMA as f32) * kma_price) * 0.15f32,
                fee_at[i]
            );    
        }

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
        let mut multiplier = Multiplier::from_u32(2);
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
                TransactionPayment::on_finalize(100);
                let next = TransactionPayment::next_fee_multiplier();

                assert!(next < multiplier, "{:?} !>= {:?}", next, multiplier);
                multiplier = next;

                println!(
                    "days = {} / block = {} / multiplier {:?}",
                    (blocks * 12) as f32 / (60 * 60 * 24) as f32,
                    blocks,
                    multiplier,
                );
            });
            blocks += 1;
        }

        let cooldown_target = 10.0f32;
        let days = (blocks * 12) as f32 / (60 * 60 * 24) as f32;
        if days > cooldown_target {
            panic!("It will take more than 10 days to cool down: {:?}", days);
        }
    }
}
