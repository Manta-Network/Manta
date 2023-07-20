import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { delay } from './test-util';
import { assert } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import { blake2AsHex } from "@polkadot/util-crypto";
import * as fs from 'fs';
import {execute_with_root_via_governance} from "./chain-util";

const test_config = {
    ws_address: "ws://127.0.0.1:9801",
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
        console.log("Old spec version: ", oldSpecVersion.toString());
        const code = fs.readFileSync('calamari.wasm').toString('hex');
        let codeHash = blake2AsHex(`0x${code}`);
        const authorizeUpgradeCallData = api.tx.parachainSystem.authorizeUpgrade(codeHash);
        execute_with_root_via_governance(api, aliceKeyPair, authorizeUpgradeCallData);
        await delay(60000);
        api.tx.parachainSystem.enactAuthorizedUpgrade(`0x${code}`).signAndSend(aliceKeyPair, {nonce: -1});
        await delay(120000);

        let newRuntimeVersions = await api.rpc.state.getRuntimeVersion();
        const newSpecVersion = newRuntimeVersions["specVersion"];
        console.log("New spec version: ", newSpecVersion.toString());
        assert(newSpecVersion > oldSpecVersion);

        let blockNow = await api.rpc.chain.getBlock();
        let blockNumberNow = blockNow.block.header.number;
        console.log("Block number before upgrade is enacted: ", blockNumberNow.toString());
        await delay(60000);
        let blockLater = await api.rpc.chain.getBlock();
        let blockNumberLater = blockLater.block.header.number;
        console.log("Block number after upgrade is enacted: ", blockNumberLater.toString());
        assert(blockNumberLater > blockNumberNow);

        api.disconnect();
    }).timeout(test_config.timeout);
});
