import {ApiPromise} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
import {blake2AsHex} from "@polkadot/util-crypto";
import {BN} from "@polkadot/util";

export const timer = (ms: number) => new Promise(res => setTimeout(res, ms))

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
    console.log("Runtime upgrade governance voted on ...");
    referendumIndexObject.referendumIndex++;
}

export async function execute_via_governance(
    api: ApiPromise,
    keyring: KeyringPair,
    extrinsicData: any,
    referendumIndexObject: any
) {
    const encodedCallData = extrinsicData.method.toHex();
    await api.tx.preimage.notePreimage(encodedCallData).signAndSend(keyring, {nonce: -1});

    let encodedCallDataHash = blake2AsHex(encodedCallData);
    let externalProposeDefault = await api.tx.democracy.externalProposeDefault({
        Legacy: {
            hash: encodedCallDataHash
        }
    });
    const encodedExternalProposeDefault = externalProposeDefault.method.toHex();
    await api.tx.council.propose(1, encodedExternalProposeDefault, encodedExternalProposeDefault.length).signAndSend(keyring, {nonce: -1});

    let fastTrackCall = await api.tx.democracy.fastTrack(encodedCallDataHash, 3, 2);
    await api.tx.technicalCommittee.propose(1, fastTrackCall, fastTrackCall.encodedLength).signAndSend(keyring, {nonce: -1});

    // vote balance based on current network
    const parachainId = Number(await api.query.parachainInfo.parachainId());
    let balance = new BN("1000000000000"); // Calamari: 12
    if (parachainId != 2084) {
        balance = new BN("1000000000000000000"); // Manta: 18
    }

    await api.tx.democracy.vote(referendumIndexObject.referendumIndex, {
        Standard: { balance, vote: { aye: true, conviction: 1 } },
    }).signAndSend(keyring, {nonce: -1});

    referendumIndexObject.referendumIndex++;
    for (let i = 0; i < 5; i++) {
        await api.tx.system.remark("0x00").signAndSend(keyring, {nonce: -1});
        await timer(13000);
    }
}

export async function execute_transaction(
    api: ApiPromise,
    alice: KeyringPair,
    extrinsicData: any,
    sudo: boolean = true
) {
    if (sudo) {
        const rootCall = api.tx.sudo.sudo(extrinsicData);
        await rootCall.signAndSend(alice, {nonce: -1}, async ({ events = [], status, txHash, dispatchError }) => {
            if (dispatchError) {
                console.log(`sudo extrinsic has error: ${dispatchError.toString()}`);
            }
        });
    } else {
        // @ts-ignore
        await extrinsicData.signAndSend(alice, {nonce: -1}, async ({ events = [], status, txHash, dispatchError }) => {
            if (dispatchError) {
                console.log(`extrinsic has error: ${dispatchError.toString()}, hex:${extrinsicData.toHex()}`);
            }
        });
    }
}