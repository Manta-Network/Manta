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

The following checks can be performed after we have frozen our release candidate:

- [ ] Code freeze should typically happen one week prior to release, to ensure we have enough time for related testing.
- [ ] Notify everyone, especially people with merge rights to `manta` (stechu, Dengjianping) that a release is ongoing and no more merges to `manta` should happen until told otherwise
- [ ] Complete the following [manual QA workflow](https://hackmd.io/TbFmorG2RnOPmLuFcg9JOQ?view).
- [ ] Verify Polkadot JS API are up to date with the latest
    runtime changes.
- [ ] Execute runtime upgrade to Como and verify network stability.
- [ ] Execute runtime upgrade to Baikal and verify network stability.
- [ ] Prepare a governance post and submit to our forum with description and motivation for changes.

### Client Releases

- [ ] Verify that each crate's `version` has been bumped from previous release.
- [ ] Check that the new client versions have [burned-in](#burn-in) without issue for at least 12 hours.

### All Releases

- [ ] Check that a draft release has been created at
    https://github.com/Manta-Network/Manta/releases with relevant [release
    notes](#release-notes)
- [ ] Check that build artifacts have been added to the
    draft-release
- [ ] Coordinate with marketing team for documentation updates and other relevant tasks.
- [ ] Update changelog.

Note: Do not publish draft releases from PR branches, because those branches will be deleted when the PR is merged.

### After Runtime Upgrade
- [ ] Notify subscan team. Ensure subscan service can continue to scan calamari blocks.

## Notes

### Burn In

Ensure that Manta DevOps has run the new release on Como and Baikal nodes
for at least 12 hours prior to publishing the release.

### Release notes

The release notes should list:

- The priority of the release (i.e., how quickly users should upgrade) - this is
    based on the max priority of any *client* changes.
- Which native runtimes and their versions are included
- The proposal hashes of the runtimes as built with
    [srtool](https://gitlab.com/chevdor/srtool)
- Any changes in this release that are still awaiting audit

The release notes may also list:

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

There are three benchmarking machines reserved for updating the weights at
release-time. To initialize a benchmark run for each production runtime
(calamari, manta):
* Go to [Calamari Benchmarking Github Action](https://github.com/Manta-Network/Manta/actions/workflows/generate_calamari_weights_files.yml) 
  and [Manta Benchmarking Github Action](https://github.com/Manta-Network/Manta/actions/workflows/generate_manta_weights_files.yml)
* Open `Run workflow` drop-down menu.
* Choose your branch and run the workflow.
* When these jobs have completed (it takes a few hours), custom weights files will
    be available to download as artifacts. 
* Commit the changes to your branch and push to the remote branch for review.
* The weights should be (Currently manually) checked to make sure there are no
    big outliers (i.e., twice or half the weight).
