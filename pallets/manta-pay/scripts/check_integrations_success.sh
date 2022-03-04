#!/usr/bin/env bash

cd ..

mkdir breaking_change_check

cd breaking_change_check

BRANCH_STRING="some_sync_branch_name"

########################## Check Manta-Node ##########################

git clone -b $BRANCH_STRING https://github.com/Manta-Network/Manta.git

cd Manta/

cargo build
cargo build --all-features

# Check Wasm benchmarking 
target/debug/manta benchmark --pallet pallet_manta_pay --extrinsic init_asset --execution=Wasm --wasm-execution Compiled --repeat 100 
target/debug/manta benchmark --pallet pallet_manta_pay --extrinsic transfer_asset --execution=Wasm --wasm-execution Compiled --repeat 100
target/debug/manta benchmark --pallet pallet_manta_pay --extrinsic mint_private_asset --execution=Wasm --wasm-execution Compiled --repeat 10
target/debug/manta benchmark --pallet pallet_manta_pay --extrinsic private_transfer --execution=Wasm --wasm-execution Compiled --repeat 10
target/debug/manta benchmark --pallet pallet_manta_pay --extrinsic reclaim --execution=Wasm --wasm-execution Compiled --repeat 10 

cd ..