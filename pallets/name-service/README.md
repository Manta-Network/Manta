# Name Service Pallet

## Formatting Rules

- dependencies in alphabetical order in the `Cargo.toml` and at the top of each file
- prefer explicit imports to glob import syntax i.e. prefer `use::crate::{Ex1, Ex2, ..};` to `use super::*;`

## Description

Implements Name Service

Users can register usernames connected to their public key.
Only one registered username can be chosen as primary. Primary usernames can be used instead of keys to do transfers.

## Workflow

1. Register -> Pending Register Storage containing block amount wait time
2. accept_register -> Push the pending register name to the usernameRecords if the block number has been passed
3. set_primary_name -> Set registered/owned name as a primary name to be used for transfers

* cancel_pending_register -> cancel a pending register
* remove_register -> "unregister" a name, this would remove it from the primary, leaving the user without a primary

## Benchmark
1. Compile Manta runtime using `runtime-benchmarks` feature
```sh
cargo build --release --features=runtime-benchmarks
```
2. Benchmark manta-pay related extrinsics
```sh
./target/release/manta benchmark \
--chain=calamari-dev \
--execution=Wasm \
--wasm-execution=Compiled \
--pallet=pallet_name_service \
--extrinsic='*' \
--steps=20 \
--repeat=10 \
--heap-pages=4096
```
