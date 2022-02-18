# CHANGELOG

## Pending

## v3.1.4-1
### Breaking changes

### Features

### Improvements
- Bump spec version to **3141**.
- [\#403](https://github.com/Manta-Network/Manta/pull/403) Remove pallet_scheduler v3 migration after 3140 runtime upgrade.
- [\407](https://github.com/Manta-Network/Manta/pull/407) Update substrate dependencies to fix some low hanging fruit in democracy pallet.

### Bug fixes

## v3.1.4
### Breaking changes

### Features

### Improvements
- Bump spec version to **3140**.
- [\#377](https://github.com/Manta-Network/Manta/pull/377) Update upstream dependencies to v0.9.16.
- [\#359](https://github.com/Manta-Network/Manta/pull/359) Update upstream dependencies to v0.9.15.
- [\#337](https://github.com/Manta-Network/Manta/pull/337) Add a congested_chain_simulation test in Calamari.
- [\#341](https://github.com/Manta-Network/Manta/pull/341) Create Release Checklist Issue Template.
- [\#350](https://github.com/Manta-Network/Manta/pull/350) Setting minValidatorCount to a default value on runtime upgrade.

### Bug fixes

## v3.1.2
### Breaking changes

### Features
- [\#311](https://github.com/Manta-Network/Manta/pull/311) Enable LTO for native binary build.
- [\#312](https://github.com/Manta-Network/Manta/pull/312) Enable collaor-selection.
- [\#313](https://github.com/Manta-Network/Manta/pull/313) Add treasury to Calamari runtime.

### Improvements
- Bump spec version to **3120**, transaction version to **3**.
- Performance improvement. Re-benchmark all unfiltered pallets due to [\#313](https://github.com/Manta-Network/Manta/pull/313) and [\#329](https://github.com/Manta-Network/Manta/pull/329). So all weights are 20% ~ 40% less than release 3.1.1
- [\#318](https://github.com/Manta-Network/Manta/pull/318) Update copyright year.
- [\#329](https://github.com/Manta-Network/Manta/pull/329) Use bare metal instance for Calamari/Manta benchmarking workflows.
- [\#353](https://github.com/Manta-Network/Manta/pull/353) Update dockerfile.

### Bug fixes
- [\#317](https://github.com/Manta-Network/Manta/pull/317) Revert workaround for failing rococo-local runtime upgrade tests in CI.

## v3.1.1
### Breaking changes

### Features
- [\#275](https://github.com/Manta-Network/Manta/pull/275) Deposit all TX fees to block authors.

### Improvements
- [\#280](https://github.com/Manta-Network/Manta/pull/280) Update README.md.
- [\#283](https://github.com/Manta-Network/Manta/pull/283) CI runtime upgrade test for manta parachain.
- [\#288](https://github.com/Manta-Network/Manta/pull/288) Update PR template.
- [\#294](https://github.com/Manta-Network/Manta/pull/294) Integrate `v0.9.13` upstream changes.
- [\#296](https://github.com/Manta-Network/Manta/pull/296) Adjust `weight_2_fee` calculation to increase TX fees and improve DDoS protection.

### Bug fixes
- [\#284](https://github.com/Manta-Network/Manta/pull/284) Unfilter utility for batched token transfer.
- [\#302](https://github.com/Manta-Network/Manta/pull/302) Better CI runtime upgrade test success criteria.

## v3.1.0
### Breaking changes

### Features
- [\#221](https://github.com/Manta-Network/Manta/pull/221) Add calamari-vesting pallet.
- [\#263](https://github.com/Manta-Network/Manta/pull/263) Calamari/Manta docker image and integration tests.
- [\#265](https://github.com/Manta-Network/Manta/pull/265) Integrate pallet-tx-pause in Manta/Calamari giving SUDO the ability to rapidly halt further execution of any extrinsic in the runtime.

### Improvements
- Bump spec version to 3100
- [\#260](https://github.com/Manta-Network/Manta/pull/260) Update weight for `pallet_democracy`/`pallet_collective`/`pallet_membership`/`pallet_scheduler`/`pallet_balances`/`calamari-vesting`.
- [\#270](https://github.com/Manta-Network/Manta/pull/270) Whitelist `frame_system` calls and integrate custom `multisig` weights in Manta/Calamari runtimes.
- [\#279](https://github.com/Manta-Network/Manta/pull/279) CI improvements and custom weights for `pallet_session`, `pallet_timestamp`, `frame_system`.

### Bug fixes

## v3.0.9

### Breaking changes

### Features

### Improvements
- [\#250](https://github.com/Manta-Network/Manta/pull/250) Manta initial release
- [\#242](https://github.com/Manta-Network/Manta/pull/242) Update upstream dependencies to `0.9.12`. Various XCM safeguards. Bump runtime version to 5
- [\#244](https://github.com/Manta-Network/Manta/pull/244) Align benchmarking work flow with polkadot/kusama
- [\#245](https://github.com/Manta-Network/Manta/pull/245) Unify manta and calamari client.

### Bug fixes
- [\#233](https://github.com/Manta-Network/Manta/pull/233) Fix dockerfile so that build args are available at runtime and container entrypoint is correctly executed

## v3.0.8

### Breaking changes

### Features
- [\#190](https://github.com/Manta-Network/Manta/pull/190) Governance configurations for calamari runtime.

### Improvements
- Bump spec version to 4

### Bug fixes

## v3.0.7

### Breaking changes

### Features

### Improvements
- [\#225](https://github.com/Manta-Network/Manta/pull/225) split MA and KMA definitions.
- Bump spec version to 3

### Bug fixes

## v3.0.6

### Breaking changes
- [\#211](https://github.com/Manta-Network/Manta/pull/211) Update Parity dependencies to `v0.9.11`.
- [Support Metadata V14](https://github.com/paritytech/cumulus/pull/623)

### Features
- [Support XCM V2](https://github.com/paritytech/polkadot/pull/3629)
- Split KMA and MA currencies into 18 decimal precision and 12 decimal precision

### Improvements
- [Follow Rework Transaction Priority calculation](https://github.com/paritytech/substrate/pull/9834)
- Refactor Node Rpc Service.
- Remove some unused dependencies.

### Bug fixes

## v3.0.5

### Breaking changes
- [\#195](https://github.com/Manta-Network/Manta/pull/195) Update Parity dependencies to `v0.9.10`.

### Features

### Improvements
- [\#197](https://github.com/Manta-Network/Manta/pull/197) Migrate CI compilation checks to self-hosted runners.
- [\#198](https://github.com/Manta-Network/Manta/pull/198) Improve CI/CD. Always trigger integration tests. Conditionally trigger runtime upgrade tests. Conditionally trigger release publish.

### Bug fixes
