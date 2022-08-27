---
name: Release issue template
about: Tracking issue for new releases
title: Manta {{ SET_VERSION }} Release checklist
---
# Release Checklist

Most of the following checks should be completed before officially publishing the new release
of the Calamari/Manta runtime or client. Some need to be completed after the new code is deployed.

### Runtime Releases

These checks should be performed on the codebase prior to freezing our release candidate:

- [ ] Verify [`spec_version`](#spec-version) has been incremented since the
    last release for any native runtimes from any existing use on public
    (non-private/test) networks. If the runtime was published (release or pre-release), either
    the `spec_version` or `impl` must be bumped.
- [ ] Verify pallet and [extrinsic ordering](#extrinsic-ordering) has stayed
    the same. Bump `transaction_version` if not.
- [ ] Verify new extrinsics have been correctly whitelisted/blacklisted
- [ ] Verify [benchmarks](#benchmarks) have been updated for any modified
    runtime logic.
- [ ] Check for any upstream storage migrations and perform tests with `try-runtime`, if any.
- [ ] Update hard-coded URLs to polkadot/manta binaries/runtimes in `publish_draft_releases.yml` CI workflow.

The following checks can be performed after we have frozen our release candidate:

- [ ] Code freeze should typically happen one week prior to release, to ensure we have enough time for related testing.
- [ ] Notify everyone, especially people with merge rights to `manta` (stechu, Dengjianping) that a release is ongoing and no more merges to `manta` should happen until told otherwise
- [ ] Complete the following [manual QA workflow](https://hackmd.io/TbFmorG2RnOPmLuFcg9JOQ?view).
- [ ] Verify Polkadot JS API are up to date with the latest
    runtime changes.
- [ ] Execute runtime upgrade to Baikal relay and verify network stability.
- [ ] Execute runtime upgrade to Calamari @ Baikal and verify network stability.
- [ ] Execute runtime upgrade to Calamari @ Moonbase-Relay and verify network stability.
- [ ] Execute runtime upgrade to Dolphin @ Baikal and verify network stability.
- [ ] Prepare a governance post and submit to our forum with description and motivation for changes.

Note: Usually update client first then runtime.

### Client Releases

- [ ] Verify that each crate's `version` has been bumped from previous release.
- [ ] Update client of Baikal relay nodes.
- [ ] Update client of Calamari-Testnet @ Baikal nodes.
- [ ] Update client of Calamari-Testnet @ Moonbase-Relay nodes.
- [ ] Update client of Dolphin @ Baikal nodes.

### All Releases

- [ ] Check that a draft release has been created at
    https://github.com/Manta-Network/Manta/releases with relevant [release
    notes](#release-notes)
- [ ] Check that build artifacts have been added to the
    draft-release
- [ ] Coordinate with marketing team for documentation updates and other relevant tasks.
- [ ] Update changelog.
- [ ] If the release contains any changes that break/change functionality used in https://github.com/Manta-Network/sdk (e.g. RPC changes, see also [extrinsic ordering](#extrinsic-ordering)), raise a PR there and **block this release** until your PR has been merged and incorporated in a new SDK release.
- [ ] Check that the new client and/or runtime versions have [burned-in](#burn-in) without issue for at least 3 days.
- [ ] Before declaring a successful burn-in make sure to check for anomalies in our nodes' memory, cpu, disk and network usage via the [Grafana Node Explorer](https://grafana.pulse.pelagos.systems/d/rYdddlPWk/node-exporter-full). These would include but are not limited to: memory leaks, cpu spikes, spike in tcp sockets waiting to close, etc. Make sure to take a look at all of the available graphs, because some problems might only show up in the collapsed views.
- [ ] Keep an eye out on the [manta status dashboard](https://status.manta.network/) for additional metrics like outages.

Note: Do not publish draft releases from PR branches, because those branches will be deleted when the PR is merged.

### After Runtime Upgrade
- [ ] Notify subscan team. Ensure subscan service can continue to scan calamari blocks.

## Notes

### Burn In

Ensure that Manta DevOps has run the new release on Baikal nodes
for at least 3 days prior to publishing the release.

### Release notes

The release notes **MUST** contain:

- The priority of the release (i.e., how quickly users should upgrade) - this is
    based on the max priority of any *client* changes.
- The version of [Manta SDK](https://github.com/Manta-Network/sdk) that is compatible with this release
- Which native runtimes and their versions are included
- The proposal hashes of the runtimes as built with [srtool](https://gitlab.com/chevdor/srtool)
- (After auditing starts:) Any changes in this release that are still awaiting audit

The release notes **MAY** also list:

- Free text at the beginning of the notes mentioning anything important
    regarding this release
- Notable changes (those labelled with B[1-9]-* labels) separated into sections

### Spec Version

A runtime upgrade must bump the spec number. This may follow a pattern with the
client release

### Extrinsic Ordering

Offline signing libraries depend on a consistent ordering of call indices and
functions. Compare the metadata of the current and new runtimes and ensure that
the `module index, call index` tuples map to the same set of functions. To generate a diff report you can do the following:
* Go to the [metadata diff tool page](https://github.com/Manta-Network/Manta/actions/workflows/metadata_diff.yml).
* Open `Run workflow` drop-down menu.
* Choose the branch you want to test and other inputs if the defaults are not applicable.
* Run the workflow.
* Wait for the job to complete and inspect the output. The things to look for in the `output.txt` are lines like:
  - `[Identity] idx 28 -> 25 (calls 15)` - indicates the index for `Identity` has changed
  - `[+] Society, Recovery` - indicates the new version includes 2 additional modules/pallets.
  - If no indices have changed, every modules line should look something like `[Identity] idx 25 (calls 15)`

 In case of a breaking change, bump the `transaction_version`.

Note: Adding new functions to the runtime does not constitute a breaking change
as long as the indexes did not change.

### Benchmarks

There is a manually deployed github action that runs all benchmarks on a bare-metal AWS machine. In order to use go to :
* Go to [Run All Benchmarks Github Action](https://github.com/Manta-Network/Manta/actions/workflows/run_all_benchmarks.yml) 
* Open `Run workflow` drop-down menu.
* Choose your branch.
* Choose a chain-spec. You'll have to run the workflow multiple times usually with `dolphin-dev`, `calamari-dev`, `manta-dev`.
* Optionally choose whether you want to provide a chain database snapshot to benchmark storage performance.
* When these jobs have completed (it takes a few hours), all the benchmarks outputs files will be available to download as workflow artifacts.
* Commit the changes to your branch and push to the remote branch for review.
* The weights should be (Currently manually) checked to make sure there are no big outliers (i.e., twice or half the weight).

### Security Audit

Before release, run a `Security Audit`

* Go to [Security Audit](https://github.com/Manta-Network/Manta/actions/workflows/audit.yml).
* Open `Run workflow` drop-down menu.
* Choose your branch and run the workflow.
* An audit report will be generated.
* Address any reported findings before release. If it cannot be fixed, please file an issue.
