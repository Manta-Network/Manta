# CHANGELOG

## Pending

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
