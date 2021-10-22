#!/usr/bin/env bash


cargo build --features runtime-benchmarks --release

target/release/manta benchmark --pallet pallet_manta_pay --extrinsic="*" --execution=wasm --wasm-execution compiled --repeat 100 --log warn

