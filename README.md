<a href="https://manta.network">
<img width="650" alt="github-banner" src="https://user-images.githubusercontent.com/720571/119246129-f6f39800-bb4c-11eb-8d9f-d68e9fe482e9.png">
</a>

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Manta-Network/Manta/Check%20Self?label=Github%20Actions&logo=Github%20Actions)
[![Twitter](https://img.shields.io/badge/-Twitter-5c5c5c?logo=Twitter)](https://twitter.com/mantanetwork)
[![Discord](https://img.shields.io/badge/Discord-gray?logo=discord)](https://discord.gg/n4QFj4n5vg)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/mantanetworkofficial)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://mantanetwork.medium.com/)

Manta is a privacy preserving DeFi stack on Polkadot/Substrate. The code currently hasn't been properly security audited (work in progress), use it at your own risk. 

## Compile Calamari
```
./scripts/init.sh
cargo build --release --features calamari
```

## Manta Developement
Currently, there are two developing branches:
* `manta`: Manta Network's testnet/mainnet node
* `manta-pc`: Manta Network's parachain node

## Contributing
* use `[Manta]` as the prefix to submit a PR to `manta` branch, use `[Manta-PC]` as the prefix to submit a PR to `manta-pc` branch.
* please submit your code through PR.
* please run `cargo +nightly fmt` before pushing your code.
