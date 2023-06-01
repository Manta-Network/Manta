---
name: Release issue template
about: Tracking issue for new releases
title: Manta {{ SET_VERSION }} Release checklist
---

# Release Checklist

Most of the following checks should be completed before officially publishing the new release
of the Calamari/Manta runtime or client. Some need to be completed after the new code is deployed.

These checks should be performed on the codebase prior to freezing our release candidate:

- [ ] Announce code freeze (typically one week prior to release), to ensure we have enough time for related testing. Notify everyone @Runtime that a release is ongoing and no more merges to `manta` should happen until told otherwise
- On a branch named `release-vX.Y.Z` or `release-vX.Y.Z-something` ( something could be e.g. `alpha` or `rc1` ). Substitute X Y Z with the release version number.
  - [ ] Verify that each crate's `version` has been bumped from previous release.
  - [ ] Verify [`spec_version`](#spec-version) has been incremented since the
        last release for any native runtimes from any existing use on public
        (non-private/test) networks. If the runtime was published (release or pre-release), either
        the `spec_version` or `impl` must be bumped.
  - [ ] Verify pallet and [extrinsic ordering](#extrinsic-ordering) has stayed
        the same. Bump `transaction_version` if not.
  - [ ] Verify new extrinsics have been correctly whitelisted/blacklisted
  - [ ] Grep github actions files for `https://github.com/paritytech/polkadot/releases/download/v` URLs and align them with current mainnet states
  - polkadot: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpolkadot.api.onfinality.io%2Fpublic-ws#/explorer
  - kusama: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fkusama-public-rpc.blockops.network%2Fws#/explorer
  - [ ] Verify [benchmarks](#benchmarks) have been updated for any modified runtime logic.
  - [ ] Check for any upstream storage migrations and perform tests with `try-runtime`, if any.
  - [ ] Execute the manual runtime upgrade CI workflow and verify that it succeeds.
  - [ ] Generate new changelog using `dev-tools` repo
  - [ ] Merge when green and reviewed
  - [ ] Tag the release with the same version you used on the release PR. IMPORTANT: Use the `manta` branch commit for the tag, NOT a `release-` or other branch
  - [ ] Wait for CI to succeed running on the tag
  - [ ] Check that a draft release has been created on the [release page](https://github.com/Manta-Network/Manta/releases) and add relevant [release notes](#release-notes)
  - [ ] Check that build artifacts have been added to the draft-release
  - [ ] Promote the draft to a Pre-Release on github
  - [ ] If there's any new extrinsic or pallet introduced, please add it to [runtime/calamari/src/diff_tx_fees.rs](../../runtime/calamari/src/diff_tx_fees.rs), then follow [tx-fees-data/README](../../runtime/calamari/tx-fees-data/README.md) to generate a new tx fees sheet.

# Deploy to internal testnets ( fast runtime )

- [ ] Verify Polkadot JS API are up to date with the latest
      runtime changes.
- Unless this release specifies a special upgrade process:
  - [ ] Execute client upgrade on Baikal relaychain nodes
  - [ ] Execute runtime upgrade to Baikal relaychain and verify network stability.
  - [ ] Execute client upgrade on Calamari @ Baikal nodes if needed
  - [ ] Execute runtime upgrade to Calamari @ Baikal and verify network stability.
- [ ] Complete the [manual QA workflow](https://www.notion.so/mantanetwork/d55be01354bb4f579b16d6e34df9e2e1?v=dcfa54e2b4a343ad9b899574ddb94a1c).
- [ ] If the release contains any changes that break/change functionality used in the [Manta SDK](https://github.com/Manta-Network/sdk) (e.g. RPC changes, see also [extrinsic ordering](#extrinsic-ordering)), raise a PR there and **block this release** until your PR has been merged and incorporated in a new SDK release.

# Deploy to public testnet

- Unless this release specifies a special upgrade process:
  - [ ] Upgrade client binary on [manta @ polkadot-internal](https://github.com/Manta-Network/testnet-deployment/tree/master/polkadot-internal%28paleblue%29/manta) if needed
  - [ ] Execute runtime upgrade to manta @ polkadot-internal and verify network stability.
  - [ ] Upgrade client binary on [calamari @ kusama-internal](https://github.com/Manta-Network/testnet-deployment/tree/master/kusama-internal%28seabird%29/calamari) if needed
  - [ ] Execute runtime upgrade to calamari @ kusama-internal and verify network stability.
- [ ] Check network health metrics like average block times, block authors, etc with this [parachain utilities tool](https://parachain-utilities.vercel.app/)
- [ ] Coordinate with the full stack team to deploy and test the wallet-extension, dApp or any other application that depends on the runtime against the staging testnet.
- [ ] Coordinate with marketing team for documentation updates and other relevant tasks.
- [ ] Monitor [Grafana Node Explorer](https://grafana.pulse.pelagos.systems/d/rYdddlPWk/node-exporter-full) for anomalies in our nodes' memory, cpu, disk and network usage. These would include but are not limited to: memory leaks, cpu spikes, spike in tcp sockets waiting to close, etc. Make sure to take a look at all of the available graphs because some problems might only be visible in views that are collapsed by default
- [ ] Check that the new client and/or runtime versions have burned-in without issue for at least 3 days.
- [ ] Keep an eye out on the [manta status dashboard](https://status.manta.network/) for additional metrics like outages.

# Deploy to mainnet

### Before Runtime Upgrade Vote

- [ ] Prepare a governance post with description and motivation for changes.
- [ ] Promote the Pre-Release to a full (latest) Release on github.
- [ ] Start the governance motion to `authorize_upgrade`.
- [ ] Submit governance post to our forum with description and motivation for changes.

### During Runtime Upgrade Vote

- Notify all external users of `manta`, include the block number the upgrade is enacted!
  - [ ] [Calamari Network Forum](https://forum.manta.network/c/calamari-network-governance/6)
  - [ ] Manta/Calamari Discord Announcement
  - [ ] [Exchange Integration Teams](https://www.notion.so/mantanetwork/Exchanges-3rd-Infrastructures-b089e136a14b430ea405400311b362cb)
  - [ ] Subscan team. Ensure subscan service can continue to scan calamari blocks.
- Unless this release specifies a special upgrade process:
  - [ ] Execute client upgrade on company mainnet nodes

## Notes

### Release Notes

The release notes **MUST** contain:

- The priority of the release (i.e., how quickly users should upgrade) - this is
  based on the max priority of any _client_ changes.
- The version of [Manta SDK](https://github.com/Manta-Network/sdk) that is compatible with this release
- Which native runtimes and their versions are included
- The proposal hashes of the runtimes as built with [srtool](https://gitlab.com/chevdor/srtool)
- (After auditing starts:) Any changes in this release that are still awaiting audit

The release notes **MAY** also list:

- Free text at the beginning of the notes mentioning anything important
  regarding this release
- Notable changes (those labelled with B[1-9]-\* labels) separated into sections

### Spec Version

A runtime upgrade must bump the spec number. This may follow a pattern with the
client release

### Extrinsic Ordering

Offline signing libraries depend on a consistent ordering of call indices and
functions. Compare the metadata of the current and new runtimes and ensure that
the `module index, call index` tuples map to the same set of functions. To generate a diff report you can do the following:

- Go to the [metadata diff tool page](https://github.com/Manta-Network/Manta/actions/workflows/metadata_diff.yml).
- Open `Run workflow` drop-down menu.
- Choose the branch you want to test and other inputs if the defaults are not applicable.
- Run the workflow.
- Wait for the job to complete and inspect the output. The things to look for in the `output.txt` are lines like:
  - `[Identity] idx 28 -> 25 (calls 15)` - indicates the index for `Identity` has changed
  - `[+] Society, Recovery` - indicates the new version includes 2 additional modules/pallets.
  - If no indices have changed, every modules line should look something like `[Identity] idx 25 (calls 15)`

In case of a breaking change, bump the `transaction_version`.

Note: Adding new functions to the runtime does not constitute a breaking change
as long as the indexes did not change.

### Benchmarks

There is a manually deployed github action that runs all benchmarks on a bare-metal AWS machine. In order to use go to :

- Go to [Run All Benchmarks Github Action](https://github.com/Manta-Network/Manta/actions/workflows/run_all_benchmarks.yml)
- Open `Run workflow` drop-down menu.
- Choose your branch.
- Choose a chain-spec. You'll have to run the workflow multiple times usually with `calamari-dev`, `manta-dev`.
- Optionally choose whether you want to provide a chain database snapshot to benchmark storage performance.
- When these jobs have completed (it takes a few hours), all the benchmarks outputs files will be available to download as workflow artifacts.
- Commit the changes to your branch and push to the remote branch for review.
- The weights should be (Currently manually) checked to make sure there are no big outliers (i.e., twice or half the weight).

### Security Audit

Before release, run a `Security Audit`

- Go to [Security Audit](https://github.com/Manta-Network/Manta/actions/workflows/audit.yml).
- Open `Run workflow` drop-down menu.
- Choose your branch and run the workflow.
- An audit report will be generated.
- Address any reported findings before release. If it cannot be fixed, please file an issue.
