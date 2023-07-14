import {ApiPromise} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
import {blake2AsHex} from "@polkadot/util-crypto";

/**
 * Execute an extrinsic with Root origin via governance.
 * @param api API object connecting to node.
 * @param keyring keyring to sign extrinsics.
 * @param extrinsicData the callData of the extrinsic that will be executed
 * @param referendumIndexObject the index of the referendum that will be executed
 */
export async function execute_with_root_via_governance(
    api: ApiPromise,
    keyring: KeyringPair,
    extrinsicData: any,
    referendumIndexObject: any
) {
    const encodedCallData = extrinsicData.method.toHex();
    await api.tx.preimage.notePreimage(encodedCallData).signAndSend(keyring, {nonce: -1});
    console.log("Runtime upgrade preimage noted ...");
    let encodedCallDataHash = blake2AsHex(encodedCallData);
    let externalProposeDefault = await api.tx.democracy.externalProposeDefault({
        Legacy: {
            hash: encodedCallDataHash
        }
    });
    const encodedExternalProposeDefault = externalProposeDefault.method.toHex();
    await api.tx.council.propose(1, encodedExternalProposeDefault, encodedExternalProposeDefault.length).signAndSend(keyring, {nonce: -1});
    console.log("Runtime upgrade governance proposed ...");
    let fastTrackCall = await api.tx.democracy.fastTrack(encodedCallDataHash, 1, 1);
    await api.tx.technicalCommittee.propose(1, fastTrackCall, fastTrackCall.encodedLength).signAndSend(keyring, {nonce: -1});
    console.log("Runtime upgrade governance fast tracked ...");
    await api.tx.democracy.vote(referendumIndexObject.referendumIndex, {
        Standard: { balance: 1_000_000_000_000, vote: { aye: true, conviction: 1 } },
    }).signAndSend(keyring, {nonce: -1});
    console.log("Runtime upgrade governanceZ voted on ...");
    referendumIndexObject.referendumIndex++;
}

export async function execute_transaction(
    api: ApiPromise,
    alice: KeyringPair,
    extrinsicData: any,
    sudo: boolean = true
) {
    if (sudo) {
        const rootCall = api.tx.sudo.sudo(extrinsicData);
        await rootCall.signAndSend(alice, {nonce: -1}, async ({ events = [], status, txHash, dispatchError }) => {});
    } else {
        await extrinsicData.signAndSend(alice, {nonce: -1});
    }
}