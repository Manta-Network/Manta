# pallet-manta-pay

This is a pallet that enables MantaPay, a multi-asset, decentralized shielded payment protocol.
The best way to use this repo is to invoke it with a `manta-runtime`,
available from [manta](https://github.com/Manta-Network/manta).

__Disclaimer__: The code has not been properly reviewed or audited and is likely to have 
severe bugs or security pitfalls.Use at your own risk!

## Documentations
``` sh
cargo doc --open
```

## Update pre-computed tokens
``` sh
cargo run --release --bin precompute_coins --features=precompute-coins > ./src/benchmark/precomputed_coins.rs
```
Note: This is only needed when the zero-knowledge-proof circuit has been changed.

## Test coverage
* install [grcov](https://github.com/mozilla/grcov):
```
cargo install grcov
```
* build and run test (extremely slow)
``` sh
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
cargo +nightly test
```
* generate the report 
``` sh
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
open target/debug/coverage/index.html
```
