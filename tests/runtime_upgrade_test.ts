import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import {execute_with_root_via_governance } from './manta_pay';
import { delay } from './test-util';
import { assert } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import { blake2AsHex } from "@polkadot/util-crypto";
import * as fs from 'fs';

const test_config = {
    ws_address: "ws://127.0.0.1:9800",
    mnemonic: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice',
    timeout: 2000000
}

describe('Node RPC Test', () => { 
    it('Check RPC result', async () => {

        let nodeAddress = "";
        const args: ParsedArgs = minimist(process.argv.slice(2));
        if (args["address"] == null) {
            nodeAddress = test_config.ws_address;
        } else {
            nodeAddress = args["address"];
        }
        console.log("using address %s", nodeAddress);

        const wsProvider = new WsProvider(nodeAddress);
        const api = await ApiPromise.create({ 
            provider: wsProvider,
            types: manta_pay_types,
            rpc: rpc_api});
        const keyring = new Keyring({ type: 'sr25519' });
        const aliceKeyPair = keyring.addFromMnemonic(test_config.mnemonic);
       
        const oldRuntimeVersion = await api.rpc.state.getRuntimeVersion();
        const oldSpecVersion = oldRuntimeVersion["specVersion"];

        const code = fs.readFileSync('calamari.wasm').toString('hex');
        let codeHash = blake2AsHex(`0x${code}`);
        const authorizeUpgradeCallData = api.tx.parachainSystem.authorizeUpgrade(codeHash);
        var referendumIndexObject = { referendumIndex: 0 };
        execute_with_root_via_governance(api, aliceKeyPair, authorizeUpgradeCallData, referendumIndexObject);
        await delay(60000);
        api.tx.parachainSystem.enactAuthorizedUpgrade(`0x${code}`).signAndSend(aliceKeyPair, {nonce: -1});
        await delay(120000);

        let newRuntimeVersions = await api.rpc.state.getRuntimeVersion();
        const newSpecVersion = newRuntimeVersions["specVersion"];
        assert(newSpecVersion > oldSpecVersion);

        let blockNow = await api.rpc.chain.getBlock();
        let blockNumberNow = blockNow.block.header.number;
        await delay(60000);
        let blockLater = await api.rpc.chain.getBlock();
        let blockNumberLater = blockLater.block.header.number;
        assert(blockNumberLater > blockNumberNow);

        api.disconnect();
    }).timeout(test_config.timeout);
});
