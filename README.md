<a href="https://manta.network">
<img width="650" alt="github-banner" src="https://user-images.githubusercontent.com/98164067/154848582-58988e81-6a89-4c5f-bdae-ec83478e245c.png">
</a>

<br>
<br>

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/gpl-3.0)
![Tests on manta](https://img.shields.io/github/actions/workflow/status/Manta-Network/Manta/check_tests.yml?branch=manta)
[![Twitter](https://img.shields.io/badge/-Twitter-5c5c5c?style=flat-square&logo=Twitter)](https://twitter.com/mantanetwork)
[![Discord](https://img.shields.io/badge/Discord-gray?style=flat-square&logo=discord)](https://discord.gg/n4QFj4n5vg)
[![Forum](https://img.shields.io/discourse/status?server=https%3A%2F%2Fforum.manta.network&style=flat-square)](https://forum.manta.network)
[![Telegram](https://img.shields.io/badge/Telegram-gray?style=flat-square&logo=telegram)](https://t.me/mantanetworkofficial)
[![Medium](https://img.shields.io/badge/Medium-gray?style=flat-square&logo=medium)](https://mantanetwork.medium.com/)


Manta is the privacy layer for Web 3. Manta's goal is to protect Web 3 users' fundamental privacy from the first principle.

Disclaimer: The code currently hasn't been properly security audited (work in progress), use it at your own risk.

:point_right: Learn more about [Manta Network](https://manta.network). <br>
:point_right: Check out our [technical documentation](https://docs.manta.network). <br>
:point_right: Get involved in [Manta Community](https://forum.manta.network/). <br>

## Manta/Calamari/Dolphin
This is the mono-repo for Manta/Calamari/Dolphin nodes.
* Manta: Manta's Polkadot parachain network
* Calamari: Manta's canary network on Kusama
* Dolphin: Manta's testnet

## Build Manta/Calamari/Dolphin Node
1. Setup environment
  ```bash
  chmod u+x ./scripts/init.sh
  ./scripts/init.sh
  ```
2. Build node binary in production setting
  ```bash
  cargo b --profile production
  ```
> Tips: The binary will be generated under `target/production/manta`. For less performance critical build, `cargo build --release` is recommended for faster build time.

## Semantic Versioning
Manta/Calamari/Dolphin's version number:
`v<x>.<y>.<z>`

where:

* `<x>` is the major version, i.e. major product release.
* `<y>` is the middle verison, i.e. adding major features.
* `<z>` is the minor version, i.e. performance improvement and bug fixes.

## Contributing
* please submit your code through PR.
* please run `cargo +nightly fmt` before pushing your code.

## Minimum supported rust compiler

This project's MSRV is `rustc 1.62`
