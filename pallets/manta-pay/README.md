# pallet-manta-pay

This is a pallet that enables MantaPay, a multi-asset, decentralized shielded payment protocol.
The best way to use this repo is to invoke it with a manta parachain node (with configuration of Manta/Calamari/Dolphin),
available from [manta](https://github.com/Manta-Network/manta).

__Disclaimer__: The code has not been properly reviewed or audited and is likely to have 
severe bugs or security pitfalls.Use at your own risk!

## Documentations
``` sh
cargo doc --open
```

## Update pre-computed tokens
1. delete `./src/benchmark/precomputed_coins.rs`
2.
``` sh
cargo run --release --features=precompute-coins --bin precompute_coins ./src/benchmark/precomputed_coins.rs
```
Note: This is only needed when the zero-knowledge-proof circuit or asset id used has been changed.

## Benchmark 
1. Compile Manta runtime using `runtime-benchmarks` feature
```sh
cargo build --release --features=runtime-benchmarks
```
2. Benchmark manta-pay related extrinsics
```sh
./target/release/manta benchmark \
--chain=dolphin-dev \
--execution=Wasm \
--wasm-execution=Compiled \
--pallet=pallet_manta_pay \
--extrinsic='*' \
--steps=20 \
--repeat=10 \
--heap-pages=4096
```