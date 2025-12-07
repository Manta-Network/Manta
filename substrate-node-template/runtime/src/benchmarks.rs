//! Benchmarking setup for the Chameleon solochain runtime.

frame_benchmarking::define_benchmarks!(
    [frame_benchmarking, BaselineBench::<Runtime>]
    [frame_system, SystemBench::<Runtime>]
    [pallet_balances, Balances]
    [pallet_timestamp, Timestamp]
);

use crate::*;

use frame_benchmarking::baseline::Pallet as BaselineBench;
use frame_system_benchmarking::Pallet as SystemBench;
