<a href="https://manta.network">
<img width="650" alt="github-banner" src="https://user-images.githubusercontent.com/720571/119246129-f6f39800-bb4c-11eb-8d9f-d68e9fe482e9.png">
</a>

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
![GitHub Workflow Status (branch)](https://img.shields.io/github/workflow/status/Manta-Network/Manta/Check%20Calamari-PC/manta-pc)
[![Twitter](https://img.shields.io/badge/-Twitter-5c5c5c?logo=Twitter)](https://twitter.com/mantanetwork)
[![Discord](https://img.shields.io/badge/Discord-gray?logo=discord)](https://discord.gg/n4QFj4n5vg)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/mantanetworkofficial)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://mantanetwork.medium.com/)

Manta is a privacy preserving DeFi stack on Polkadot/Substrate. The code currently hasn't been properly security audited (work in progress), use it at your own risk. 

## Build Manta/Calamari Node
```
./scripts/init.sh
cargo build --release
```

## Manta Developement
Currently, there are two developing branches:
* `manta-pc`: Manta Network/Calamari Network's parachain runtime
* `dolphin`: Dolphin testnet runtime (a standlone testnet runs its own consensus)

## Semantic Versioning
Manta/Calamari's version number:
`v<x>.<y>.<z>-<relay-id>.<para-id>`

where:

* `<x>` is the major version, i.e. major product release.
* `<y>` is the middle verison, i.e. adding major features.
* `<z>` is the minor version, i.e. performance improvement and bug fixes.
* `<relay-id>` is the relay chain name, i.e. kusama or polkadot
* `<para-id>` is the parachain name, i.e. clamari or manta

## Contributing
* please submit your code through PR.
* please run `cargo +nightly fmt` before pushing your code.

## ci build

[![publish draft releases](https://github.com/Manta-Network/Manta/actions/workflows/publish-draft-releases.yml/badge.svg?branch=manta-pc)](https://github.com/Manta-Network/Manta/actions/workflows/publish-draft-releases.yml)

the [publish draft releases](https://github.com/Manta-Network/Manta/blob/manta-pc/.github/workflows/publish-draft-releases.yml) workflow builds:

* **manta** the manta/calamari parachain executable
* wasm runtimes:
  * **manta** the manta parachain wasm runtime
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
