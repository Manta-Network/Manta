# Manta
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

Manta is a privacy preserving DeFi stack on Polkadot/Substrate. The code currently hasn't been properly security audited (work in progress), use it at your own risk. 

## Compile Manta
```
./scripts/init.sh
cargo build --release -p manta
```

## Manta Developement
Currently, there are two developing branches:
* `manta`: Manta Network's testnet/mainnet node
* `manta-pc`: Manta Network's parachain node

## Using Docker
You can run manta nodes using docker.

* Pull latest image.
```
docker pull mantanetwork/manta:latest
```

* Run one dev node locally.
```
docker run -it -p 9944:9944 mantanetwork/manta:latest --dev --unsafe-ws-external
```

* Run two nodes locally.
```
# Alice node
docker run \
-p 9944:9944 \
-p 30333:30333 \
--name=alice mantanetwork/manta:latest \
--unsafe-ws-external \
--tmp \
--alice \
--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
--validator

# Bob node
docker run \
-p 9945:9944 \
--name=bob mantanetwork/manta:latest \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--unsafe-ws-external \
--bob \
--validator
```
Normally, both nodes will produce and finalize blocks.

* Connect to manta testnet.
```
docker run mantanetwork/manta:latest --chain manta-testnet --name "ILoveManta"
```

## Contributing
* use `[Manta]` as the prefix to submit a PR to `manta` branch, use `[Manta-PC]` as the prefix to submit a PR to `manta-pc` branch.
* please submit your code through PR.
* please run `cargo +nightly fmt` before pushing your code.
