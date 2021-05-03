# Manta

Manta is a privacy preserving DeFi stack on Polkadot/Substrate.

## Compile Manta
```
./scripts/init.sh
cargo build --release -p manta
```

## Manta Developement
Currently, there are two developing branches:
* `manta`: Manta Network's testnet/mainnet node
* `manta-pc`: Manta Network's parachain node

## Contributing
* use `[Manta]` as the prefix to submit a PR to `manta` branch, use `[Manta-PC]` as the prefix to submit a PR to `manta-pc` branch.
* please submit your code through PR.
* please run `cargo +nightly fmt` before pushing your code.
