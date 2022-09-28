# CHANGELOG

## v3.4.0
### Added
- [\#745](https://github.com/Manta-Network/Manta/pull/745) Workflow to check for labels
- [\#758](https://github.com/Manta-Network/Manta/pull/758) All-benchmarks script and CI workflow improvements
- [\#724](https://github.com/Manta-Network/Manta/pull/724) Nimbus Stage 2 - Enable permissionless staking on Calamari [CA]

### Changed
- [\#770](https://github.com/Manta-Network/Manta/pull/770) Update fees splits, 50% burned, 50% to treasury [CADO]
- [\#766](https://github.com/Manta-Network/Manta/pull/766) Change QA workflow link to internal Notion
- [\#743](https://github.com/Manta-Network/Manta/pull/743) Split all testing from `publish_draft_release` workflow and filter execution by labels on the PRs [CADO]
- [\#781](https://github.com/Manta-Network/Manta/pull/781) Proper XCM weights benchmark [CADO]
- [\#782](https://github.com/Manta-Network/Manta/pull/782) Run manta-pay randomized tests in a loop 10 times [CADO]
- [\#678](https://github.com/Manta-Network/Manta/pull/678) Minor: Update PULL_REQUEST_TEMPLATE.md
- [\#792](https://github.com/Manta-Network/Manta/pull/792) Adapt 45/45/10 TX fee split for KMA, 100% to author for DOL [CADO]

### Fixed
- [\#783](https://github.com/Manta-Network/Manta/pull/783) Fix calamari bootnode names in chain-spec and docker [CADO]
- [\#791](https://github.com/Manta-Network/Manta/pull/791) Fix CI by ignoring failure of `stop-` jobs

## v3.3.0
### Added
- [\#717](https://github.com/Manta-Network/Manta/pull/717) Dolphin-2085 on Baikal genesis [DO]
- [\#712](https://github.com/Manta-Network/Manta/pull/712) Add RPC for latest checkpoint
- [\#763](https://github.com/Manta-Network/Manta/pull/763) Support verification of historic Aura blocks

### Changed
- [\#681](https://github.com/Manta-Network/Manta/pull/681) CI Ledger RPC Tests
- [\#682](https://github.com/Manta-Network/Manta/pull/682) Use `LengthToFee` in the `congested_chain_simulation`'s fee calculation
- [\#695](https://github.com/Manta-Network/Manta/pull/695) Refactor fungible ledger mint/burn
- [\#715](https://github.com/Manta-Network/Manta/pull/715) Update xcm-onboarding and release templates
- [\#701](https://github.com/Manta-Network/Manta/pull/701) switch runtime to wasm only
- [\#720](https://github.com/Manta-Network/Manta/pull/720) Update deps from v0.9.22 to v0.9.26
- [\#726](https://github.com/Manta-Network/Manta/pull/726) support STORAGE_VERSION for our pallets
- [\#738](https://github.com/Manta-Network/Manta/pull/738) Add changelog verification. Remove old changelog workflow
- [\#582](https://github.com/Manta-Network/Manta/pull/582) Consensus migration stage 1: Enable Nimbus-Aura [CADO]
- [\#752](https://github.com/Manta-Network/Manta/pull/752) v3.3.0 bump versions and weights

### Fixed
- [\#694](https://github.com/Manta-Network/Manta/pull/694) Use u128::MAX in fungible ledger transfer test
- [\#703](https://github.com/Manta-Network/Manta/pull/703) Fix double spend reclaim test
- [\#723](https://github.com/Manta-Network/Manta/pull/723) fix: upgrade to `manta-rs` v0.5.4

### Removed
- [\#737](https://github.com/Manta-Network/Manta/pull/737) Remove v3.2.1 SessionKey migration code [CADO]

## v3.2.1
### Breaking changes
- [Dolphin] [\#628](https://github.com/Manta-Network/Manta/pull/628) Improve RPC performance, add `max_receivers` and `max_senders` fields in the RPC request.

### Features
- [\#646](https://github.com/Manta-Network/Manta/pull/646) Add collator session keys for future nimbus consensus and a vrf placeholder.

### Improvements
- [\#449](https://github.com/Manta-Network/Manta/pull/449) Remove strip from CI, and add strip profile to production.
- [\#571](https://github.com/Manta-Network/Manta/pull/571) Update upstream dependencies to v0.9.22.
- [\#563](https://github.com/Manta-Network/Manta/pull/563) Re-implement the `TransactAsset` trait with the unified interface of `FungibleLedger` trait, and `AssetConfig` trait.
- [\#576](https://github.com/Manta-Network/Manta/pull/576) Unfilter xtokens.transfer_multicurrencies and bump MaxAssetsForTransfer to 2.
- [\#607](https://github.com/Manta-Network/Manta/pull/607) Turn node client code into library for CLI project.
- [\#614](https://github.com/Manta-Network/Manta/pull/614) Remove `OnRuntimeUpgrade` from calamari-runtime.
- [\#619](https://github.com/Manta-Network/Manta/pull/619) Add CI runtime upgrade test for Dolphin and improve test scenario.
- [Dolphin] [\#622](https://github.com/Manta-Network/Manta/pull/622) Update parameter path from `sdk` to `manta-parameters`.
- [\#636](https://github.com/Manta-Network/Manta/pull/636) Equalize Barrier impl between Calamari/Dolphin production and XCM tests mock parachain.
- [\#638](https://github.com/Manta-Network/Manta/pull/638) `existence_requirement` argument for `FungibleLedger` trait functions.
- [\#652](https://github.com/Manta-Network/Manta/pull/652) Reduce CI failure rate by switching AWS CI runners from AMD to Intel.
- [\#653](https://github.com/Manta-Network/Manta/pull/653) Add concurrency groups for pull request CI builds to reduce CI costs.
- [\#657](https://github.com/Manta-Network/Manta/pull/657) retire `manta-pc-launch` with `polkadot-launch`.

### Bug fixes
- [\#671](https://github.com/Manta-Network/Manta/pull/671) polkadot-v0.9.22 syn breakage workaround.
- [\#677](https://github.com/Manta-Network/Manta/pull/677) Fix CI failure by building the runtime with stable Rust.

## v3.2.0
### Breaking changes

### Features
- [Dolphin] [\#529](https://github.com/Manta-Network/Manta/pull/529) Add RPC for MantaPay to synchronize with latest ledger state

### Improvements
- [\#481](https://github.com/Manta-Network/Manta/pull/481) Update upstream dependencies to v0.9.18.
- [\#491](https://github.com/Manta-Network/Manta/pull/491) Revamp collator-selection.
- [\#493](https://github.com/Manta-Network/Manta/pull/493) Dedupe mock-xcm tests (part 1).
- [\#505](https://github.com/Manta-Network/Manta/pull/505) Proper bare-metal instances for benchmarking workflows.
- [\#507](https://github.com/Manta-Network/Manta/pull/507) Add issue template for Calamari xcm onboarding of other parachains.
- [\#519](https://github.com/Manta-Network/Manta/pull/519) Concrete fungible ledger integration tests.
- [\#523](https://github.com/Manta-Network/Manta/pull/523) Move xcm and assets related runtime configurations to own files.
- [\#531](https://github.com/Manta-Network/Manta/pull/531) Clean up AssetManager migration code.
- [\#541](https://github.com/Manta-Network/Manta/pull/541) Skip build on too tiny change.
- [\#542](https://github.com/Manta-Network/Manta/pull/542) Update xcm integrations template issue.
- [Calamari] [\#550](https://github.com/Manta-Network/Manta/pull/550) Remove sudo pallet from calamari runtime.
- [\#560](https://github.com/Manta-Network/Manta/pull/560) Bump srtool to v0.4.0.
- [Dolphin] [\#583](https://github.com/Manta-Network/Manta/pull/583) Remove checkpoint from RPC API when synchronizing with MantaPay.
- [\#583](https://github.com/Manta-Network/Manta/pull/583) Remove checkpoint from RPC API when synchronizing with MantaPay

### Bug fixes
- [\#558](https://github.com/Manta-Network/Manta/pull/558) Fix try runtime and metadata diff ci workflows.
- [\#567](https://github.com/Manta-Network/Manta/pull/567) Fix file structure of relay chian specs.
- [\#570](https://github.com/Manta-Network/Manta/pull/570) Revert hard-coded branch of yamllint github action.

## v3.1.5-1
### Breaking changes

### Features

### Improvements
- [\#475](https://github.com/Manta-Network/Manta/pull/475) New workflow for comparing runtime metadata before and after runtime upgrade.
- [\#485](https://github.com/Manta-Network/Manta/pull/485) XCM Fees now accrue to the Treasury instead of AssetManager.
- [\#509](https://github.com/Manta-Network/Manta/pull/509) OnRuntimeUpgrade hook for AssetManager to properly set initial configurations.
- [\#510](https://github.com/Manta-Network/Manta/pull/510) Automate publishing of Dolphin release artifacts.
- [\#513](https://github.com/Manta-Network/Manta/pull/513) Update the release issues template.

### Bug fixes

## v3.1.5
### Breaking changes

### Features
- [\#484](https://github.com/Manta-Network/Manta/pull/484) Update to [latest MantaPay circuits](https://github.com/Manta-Network/manta-rs/pull/50)
- [\#436](https://github.com/Manta-Network/Manta/pull/436) Dolphin XCM Integration
- [\#430](https://github.com/Manta-Network/Manta/pull/430) Add private payment to dolphin runtime.
- [\#419](https://github.com/Manta-Network/Manta/pull/419) Add asset manager and XCM support.
- [\#416](https://github.com/Manta-Network/Manta/pull/416) Automatic Collator removal enabled for Calamari
- [\#383](https://github.com/Manta-Network/Manta/pull/383) Calamari & Manta support `cargo build --features=fast-runtime`, setting most configurable timers to 2 or 5 minutes (instead of days)
- [\#358](https://github.com/Manta-Network/Manta/pull/358) Underperforming collators are automatically removed from the collator set after each session

### Improvements
- [\#476](https://github.com/Manta-Network/Manta/pull/476) Set the version of feature resolver as 2.
- [\#472](https://github.com/Manta-Network/Manta/pull/472) Improve asset manager.
- [\#457](https://github.com/Manta-Network/Manta/pull/457) Add manual `try-runtime` CI workflow test against Calamari mainnet.
- [\#455](https://github.com/Manta-Network/Manta/pull/455) Calamari: Integrate new collator eviction.
- [\#447](https://github.com/Manta-Network/Manta/pull/447) Dolphin parachain testnet genesis.
- [\#445](https://github.com/Manta-Network/Manta/pull/445) Clean up readme.
- [\#441](https://github.com/Manta-Network/Manta/pull/441) Dolphin benchmarking workflow.
- [\#439](https://github.com/Manta-Network/Manta/pull/439) Instructions for DCO.
- [\#435](https://github.com/Manta-Network/Manta/pull/435) Expose 9945 for checking relaychain's block number on parachain.
- [\#426](https://github.com/Manta-Network/Manta/pull/426) DCO for community PRs.
- [\#411](https://github.com/Manta-Network/Manta/pull/411) Add a corner case about increasing/decreasing candidate bond in collator-selection.
- [\#410](https://github.com/Manta-Network/Manta/pull/410) Add to-do item to update CHANGELOG.md.
- [\#409](https://github.com/Manta-Network/Manta/pull/409) Update banner.
- [\#406](https://github.com/Manta-Network/Manta/pull/406) Adjust treasury and preimage pallets' deposits.
- [\#405](https://github.com/Manta-Network/Manta/pull/405) Migrate to new method of declaring constants.
- [\#404](https://github.com/Manta-Network/Manta/pull/404) Reduce PreimageMaxSize to 3.5MB.
- [\#401](https://github.com/Manta-Network/Manta/pull/401) Customize cargo profiles. Add production profile.
- [\#393](https://github.com/Manta-Network/Manta/pull/393) CI runtime upgrade test and github templates improvements.
- [\#373](https://github.com/Manta-Network/Manta/pull/373) Expose more ports in dockerfile.

### Bug fixes
- [\#470](https://github.com/Manta-Network/Manta/pull/470) Fix: move deserialization of manta-pay types into the extrinsic.
- [\#467](https://github.com/Manta-Network/Manta/pull/467) Fix ssl compilation issue in CI.
- [\#461](https://github.com/Manta-Network/Manta/pull/461) Fix AssetManager's `update_asset_metadata` to update the underlying assets storage.
- [\#421](https://github.com/Manta-Network/Manta/pull/421) Fix CI integration test false negatives.

## v3.1.4-1
### Breaking changes

### Features

### Improvements
- Bump spec version to **3141**.
- [\#403](https://github.com/Manta-Network/Manta/pull/403) Remove pallet_scheduler v3 migration after 3140 runtime upgrade.
- [\#407](https://github.com/Manta-Network/Manta/pull/407) Update substrate dependencies to fix some low hanging fruit in democracy pallet.

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
- [\#312](https://github.com/Manta-Network/Manta/pull/312) Enable collator-selection.
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
