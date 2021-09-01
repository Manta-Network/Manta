<a href="https://manta.network">
<img width="650" alt="github-banner" src="https://user-images.githubusercontent.com/720571/119246129-f6f39800-bb4c-11eb-8d9f-d68e9fe482e9.png">
</a>

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
docker run -it -p 9944:9944 mantanetwork/manta:latest --dev --tmp --alice --unsafe-ws-external
```

* Run two nodes locally.
```
# Alice node
docker run \
-p 9944:9944 \
-p 30333:30333 \
--name=alice mantanetwork/manta:latest \
--chain=local \
--tmp \
--alice \
--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
--unsafe-ws-external \
--validator

docker run \
-p 9945:9944 \
-p 30334:30333 \
--name=bob mantanetwork/manta:latest \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--chain=local \
--bob \
--unsafe-ws-external \
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

## ci build

[![publish draft releases](https://github.com/Manta-Network/Manta/actions/workflows/publish-draft-releases.yml/badge.svg?branch=manta-pc)](https://github.com/Manta-Network/Manta/actions/workflows/publish-draft-releases.yml)

the [publish draft releases](https://github.com/Manta-Network/Manta/blob/manta-pc/.github/workflows/publish-draft-releases.yml) workflow builds:

* **calamari-pc** the calamari parachain executable (a substrate node)
* wasm runtimes:
  * **manta-pc** the manta parachain wasm runtime
  * **calamari** the calamari parachain wasm runtime

the workflow is triggered whenever a tag containing a semver is pushed to the github repo. if you have a branch derived from the [manta-pc](https://github.com/Manta-Network/Manta/tree/manta-pc) branch, you may trigger a ci-build and create a draft release (only available to Manta-Network org members) with commands similar to the following:

```bash
# clone the repo and checkout the `manta-pc` branch
git clone --branch manta-pc git@github.com:Manta-Network/Manta.git

# create a new branch called `my-awesome-feature`, derived from branch `manta-pc` which contains the ci build workflow
git checkout -b my-awesome-feature manta-pc

# ... add my awesome feature ...
git add .
git commit -m "added my awesome feature"

# create a tag pointing to the last commit that is also named with the semver and latest commit sha `v3.0.0-<short-git-sha>` (eg: `v3.0.0-abcd123`)
git tag -a v3.0.0-$(git rev-parse --short HEAD) -m "manta-pc and my awesome feature"

# push my awesome feature branch **and** my new tag to origin (github)
git push origin my-awesome-feature --tags
```

now you can watch the ci build your awesome feature and publish your draft release on the [actions tab](https://github.com/Manta-Network/Manta/actions/workflows/publish-draft-releases.yml). note that draft [releases](https://github.com/Manta-Network/Manta/releases) become available relatively quickly, but wasm and binary artifacts are only added to the draft release when their ci build completes, which may be an hour or more after your git push.
