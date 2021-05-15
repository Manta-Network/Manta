#!/usr/bin/env bash


cargo +nightly build --features runtime-benchmarks --release

target/release/manta benchmark --pallet pallet_manta_pay --extrinsic init_asset --wasm-execution Interpreted --repeat 100
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic transfer_asset --wasm-execution Interpreted --repeat 100
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic mint_private_asset --wasm-execution Interpreted --repeat 100
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic private_transfer --wasm-execution Interpreted --repeat 100
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic reclaim --wasm-execution Interpreted --repeat 100