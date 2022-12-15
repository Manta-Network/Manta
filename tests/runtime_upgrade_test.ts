import { ApiPromise, WsProvider } from '@polkadot/api';
import { numberToU8a } from '@polkadot/util';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import {execute_with_root_via_governance } from './manta_pay';
import { delay } from './test-util';
import { expect } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import { blake2AsHex } from "@polkadot/util-crypto";
import * as fs from 'fs';
import { democracy } from '@polkadot/types/interfaces/definitions';

const test_config = {
    ws_address: "ws://127.0.0.1:9800",
    mnemonic: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice',
    storage_prepare_config: {
        utxo_batch_number: 1,
        utxo_batch_size_per_shard: 4,
        utxo_big_batch_number: 1,
        vn_batch_number: 1,
        vn_batch_size: 1024,
    },
    timeout: 200000
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
        const callData = api.tx.parachainSystem.authorizeUpgrade(`0x${code}`);
        var referendumIndexObject = { referendumIndex: 0 };
        execute_with_root_via_governance(api, aliceKeyPair, callData, referendumIndexObject);
        delay(60000);
        // Perform the actual chain upgrade via the sudo module
        api.tx.parachainSystem.enactAuthorizedUpgrade(`0x${code}`).signAndSend(aliceKeyPair, {nonce: -1});
        delay(60000);

        let new_runtime_versions = await api.rpc.state.getRuntimeVersion();
        const newSpecVersion = new_runtime_versions["specVersion"];

        expect(newSpecVersion).to.equal(oldSpecVersion as any + 1);
        
        api.disconnect();
    }).timeout(test_config.timeout);
});
