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

## Manta/Calamari
This is the mono-repo for Manta/Calamari nodes.
* Manta: Manta's Polkadot parachain network
* Calamari: Manta's canary network on Kusama
* Dolphin: Manta's testnet

## Build Manta/Calamari Node
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
3. Run standalone dev chain, useful for local development
  ```bash
  cargo run -- --chain=calamari-localdev --alice --tmp
  ```
> Tip: The chain only produces blocks when you submit extrinsics

## Semantic Versioning
Manta/Calamari's version number:
`v<x>.<y>.<z>`

where:

* `<x>` is the major version, i.e. major product release.
* `<y>` is the middle version, i.e. adding major features.
* `<z>` is the minor version, i.e. performance improvement and bug fixes.

## Project Structure
```
├── scripts/         # Setup and deployment scripts
├── src/            # Source code
├── target/         # Compiled files
└── tests/          # Test files
```

## System Requirements
- Rust 1.74 or higher
- Linux/MacOS/Windows
- Minimum 2 GB RAM
- Minimum 20 GB free disk space

## Development

### Development Environment Setup
1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install required components:
```bash
rustup default stable
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Testing
Run tests:
```bash
cargo test --all
```

### Linting and Formatting
```bash
cargo +nightly fmt
cargo clippy
```

## Contribution Process
1. Fork the repository
2. Create a new branch for your changes
3. Make changes and ensure all tests pass
4. Submit a PR with description of your changes

## Useful Resources
- [Development Guide](https://docs.manta.network/docs/guides/Development)
- [API Documentation](https://docs.manta.network/docs/api)
- [Developer Discord Channel](https://discord.gg/n4QFj4n5vg)
