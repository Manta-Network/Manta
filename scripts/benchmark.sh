#!/usr/bin/env bash


cargo +nightly build --features runtime-benchmarks --release

target/release/manta benchmark --pallet pallet_manta_pay --extrinsic init_asset --execution=Wasm --wasm-execution Compiled --repeat 100 --log warn
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic transfer_asset --execution=Wasm --wasm-execution Compiled --repeat 100 --log warn
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic mint_private_asset --execution=Wasm --wasm-execution Compiled --repeat 10 --log warn
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic private_transfer --execution=Wasm --wasm-execution Compiled --repeat 10 --log warn
target/release/manta benchmark --pallet pallet_manta_pay --extrinsic reclaim --execution=Wasm --wasm-execution Compiled --repeat 10 --log warn 
