## Update pre-computed tokens
1. delete `./src/benchmark/precomputed_coins.rs`
2.
```sh
cargo run --release --features=precompute-coins --bin precompute_coins ./src/benchmark/precomputed_coins.rs
```
Note: This is only needed when the zero-knowledge-proof circuit or asset id used has been changed.
