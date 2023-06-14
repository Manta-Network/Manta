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
pub const FEES_PERCENTAGE_TO_TREASURY: u8 = 90;

pub const TIPS_PERCENTAGE_TO_AUTHOR: u8 = 100;
pub const TIPS_PERCENTAGE_TO_TREASURY: u8 = 0;

#[cfg(test)]
mod fee_split_tests {
    use super::*;
    #[test]
    fn fee_split_adds_up_to_one() {
        assert_eq!(100, FEES_PERCENTAGE_TO_AUTHOR + FEES_PERCENTAGE_TO_TREASURY);
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
        RuntimeBlockWeights as BlockWeights, System, TransactionPayment, MANTA,
    };
    use frame_support::{
        dispatch::{DispatchClass, DispatchInfo},
        weights::{Weight, WeightToFee},
    };
    use manta_primitives::constants::time::DAYS;
    use pallet_transaction_payment::Multiplier;
    use runtime_common::MinimumMultiplier;

    fn fetch_manta_price() -> f32 {
        0.36
    }

    #[test]
    #[ignore] // This test should not fail CI
    fn multiplier_growth_simulator_and_congestion_budget_test() {
        let target_daily_congestion_cost_usd = 100_000;
        let manta_price = fetch_manta_price();
        println!("MANTA/USD price as read from CoinGecko = {manta_price}");
        let target_daily_congestion_cost_manta =
            (target_daily_congestion_cost_usd as f32 / manta_price * MANTA as f32) as u128;

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
            pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
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
                    blocks, multiplier, fee, fees_paid, (fees_paid as f32 / MANTA as f32) * manta_price
                );

                if blocks == 7200 {
                    fees_after_one_day = fees_paid;
                    if fees_paid < target_daily_congestion_cost_manta {
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
            (fees_to_1x as f32 / MANTA as f32) * manta_price
        );

        println!(
            "Cost for 1 day in MANTA {:?} and in USD {:?}",
            fees_after_one_day,
            (fees_after_one_day as f32 / MANTA as f32) * manta_price
        );

        if should_fail {
            panic!("The cost to fully congest our network should be over the target_daily_congestion_cost_manta after 1 day.");
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

    use codec::Encode;
    use frame_system::WeightInfo;
    use sp_runtime::FixedPointNumber;

    #[test]
    fn multiplier_growth_simulator_and_congestion_budget_test2() {
        let target_daily_congestion_cost_usd = 100_000;
        //let kma_price = fetch_kma_price().unwrap();
        let kma_price = 0.36f32;
        println!("MANTA/USD price as read from CoinGecko = {kma_price}");
        let target_daily_congestion_cost_kma =
            (target_daily_congestion_cost_usd as f32 / kma_price * MANTA as f32) as u128;

        // (98974000 * 50000000) / 1000000000000000000 = 0.0049487
        let base_fee = <Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
            &runtime_common::ExtrinsicBaseWeight::get(),
        );

        let remark =
            crate::RuntimeCall::Balances(pallet_balances::Call::<Runtime>::transfer_keep_alive {
                dest: sp_runtime::MultiAddress::Id(crate::AssetManager::account_id()),
                value: 250 * MANTA,
            });
        let transfer_len: u32 = 148; //remark.encode().len() as u32;
        let transfer_len_weight = Weight::from_parts(transfer_len as u64, 0);
        let mut multiplier = MinimumMultiplier::get();
        let transfer_len_fee =
            <Runtime as pallet_transaction_payment::Config>::LengthToFee::weight_to_fee(
                &transfer_len_weight,
            ) as f32
                / MANTA as f32;
        // let adj_len_fee = multiplier.saturating_mul_int(transfer_len_fee);

        println!("transfer len_fee: {:?}", transfer_len_fee);

        let zk_len: u32 = 661; //remark.encode().len() as u32;
        let zk_len_weight = Weight::from_parts(zk_len as u64, 0);
        let zk_len_fee = <Runtime as pallet_transaction_payment::Config>::LengthToFee::weight_to_fee(
            &zk_len_weight,
        ) as f32
            / MANTA as f32;
        println!("zk len_fee: {:?}", zk_len_fee);

        let rtu_len: u32 = 1247928; //remark.encode().len() as u32;
        let rtu_len_weight = Weight::from_parts(rtu_len as u64, 0);
        let rtu_len_fee =
            <Runtime as pallet_transaction_payment::Config>::LengthToFee::weight_to_fee(
                &rtu_len_weight,
            ) as f32
                / MANTA as f32;
        println!("rtu len_fee: {:?}", rtu_len_fee);

        // assume the multiplier is initially set to its minimum and that each block
        // will be full with a single user extrinsic, which will minimize then length and base fees
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

                // println!(
                //     "block = {} / multiplier {:?} / fee = {:?} / fees so far {:?} / fees so far in USD {:?}",
                //     blocks, multiplier, (fee as f32 / MANTA as f32) * kma_price, fees_paid, (fees_paid as f32 / MANTA as f32) * kma_price
                // );

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
            });
            blocks += 1;
        }

        let transfer_percentage = 0.0005f32;
        let zk_percentage = 0.15f32;

        println!(
            "The single block cost at the start is: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?}.
            {:?} for 0.25x at {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},
            {:?} for 0.5x at  {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},
            {:?} for 1x at    {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},
            {:?} for 2x at    {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},
            {:?} for 3x at    {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},
            {:?} for 10x at   {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},
            {:?} for 100x at  {:?} USD, with 1 block: {:?} USD. 1 transfer: {:?} and 1 zk-tx: {:?}, 1 zk-tx in MANTA {:?},",
            (fee_at_0 as f32 / MANTA as f32) * kma_price,
            ((fee_at_0 as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_0 as f32 / MANTA as f32) * kma_price) * zk_percentage,
            (fee_at_0 as f32 / MANTA as f32) * zk_percentage,

            (quarter_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_quarter_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_quarter as f32 / MANTA as f32) * kma_price,
            ((fee_at_quarter as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_quarter as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_quarter as f32 * zk_percentage,

            (half_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_half_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_half as f32 / MANTA as f32) * kma_price,
            ((fee_at_half as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_half as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_half as f32 * zk_percentage,

            (one_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_1x_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_1 as f32 / MANTA as f32) * kma_price,
            ((fee_at_1 as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_1 as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_1 as f32 * zk_percentage,

            (two_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_2x_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_2 as f32 / MANTA as f32) * kma_price,
            ((fee_at_2 as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_2 as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_2 as f32 * zk_percentage,

            (three_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_3x_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_3 as f32 / MANTA as f32) * kma_price,
            ((fee_at_3 as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_3 as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_3 as f32 * zk_percentage,

            (ten_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_10x_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_10 as f32 / MANTA as f32) * kma_price,
            ((fee_at_10 as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_10 as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_10 as f32 * zk_percentage,

            (hundred_block * 12) as f32 / (60 * 60 * 24) as f32,
            (fees_at_100x_multiplier as f32 / MANTA as f32) * kma_price,
            (fee_at_100 as f32 / MANTA as f32) * kma_price,
            ((fee_at_100 as f32 / MANTA as f32) * kma_price) * transfer_percentage,
            ((fee_at_100 as f32 / MANTA as f32) * kma_price) * zk_percentage,
            fee_at_100 as f32 * zk_percentage,
        );

        // for i in 0 .. 20 {
        //     println!(
        //         "{:?} days for {:?}x at a cost {:?} USD, with single block: {:?} USD. Single transfer: {:?} and single zk-tx: {:?}, single zk-tx in MANTA {:?}",
        //         (blocks_at[i] * 12) as f32 / (60 * 60 * 24) as f32,
        //         multipliers[i],
        //         (fees_so_far_at[i] as f32 / MANTA as f32) * kma_price,
        //         (fee_at[i] as f32 / MANTA as f32) * kma_price,
        //         ((fee_at[i] as f32 / MANTA as f32) * kma_price) * transfer_percentage,
        //         ((fee_at[i] as f32 / MANTA as f32) * kma_price) * zk_percentage,
        //         fee_at[i]
        //     );
        // }

        println!(
            "Cost for 1 day in MANTA {:?} and in USD {:?}",
            fees_after_one_day,
            (fees_after_one_day as f32 / MANTA as f32) * kma_price
        );

        if should_fail {
            panic!("The cost to fully congest our network should be over the target_daily_congestion_cost_kma after 1 day.");
        }
    }

    #[test]
    fn multiplier_cool_down_simulator2() {
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
