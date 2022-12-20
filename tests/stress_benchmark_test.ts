import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { delay } from './test-util';
import { assert } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import { blake2AsHex } from "@polkadot/util-crypto";
import * as fs from 'fs';
import { readFile } from 'fs/promises';

const test_config = {
    ws_address: "ws://127.0.0.1:9801",
    mnemonic: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice',
    timeout: 20000000,
    max_wait_time_sec: 100000,
    expected_tps: 1.9
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
        const sender = keyring.addFromMnemonic(test_config.mnemonic);
       
        let mints_offset = 2;
        let transfers_offset = 4;
        let reclaims_offset = 4;
        let total_iterations = 15000;
        let start_iteration = 13500;
        let tests_iterations = 100;

        let mint_size = 552;
        const mints_content = await readFile("./precomputed_mints");
        const mints_buffer = mints_content.subarray(
          mints_offset,
          mint_size * total_iterations
        );
        let transfer_size = 1290;
        let full_transfer_size = (mint_size * 2 + transfer_size);
        const transfers_content = await readFile("./precomputed_transfers");
        const transfers_buffer = transfers_content.subarray(
          transfers_offset,
          full_transfer_size * total_iterations
        );
        let reclaim_size = 968;
        let full_reclaim_size = (mint_size * 2 + reclaim_size);
        const reclaims_content = await readFile("./precomputed_reclaims");
        const reclaims_buffer = reclaims_content.subarray(
          reclaims_offset,
          full_reclaim_size * total_iterations
        );
        let lastFinalized = false;
        let finalizedCount = 0;
        let allSuccesses = 0;
        let txs_sent = 0;
        var startTime = performance.now()
        let totalTime = 0;
        for(let i = start_iteration; i < start_iteration + tests_iterations; ++i){
            await api.tx.mantaPay.toPrivate(mints_buffer.subarray(mint_size * i, mint_size * (i + 1))).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                if (status.isInBlock) {
                    events.forEach(({ event: { data, method, section }, phase }) => {
                      if ('mantaPay.ToPrivate' == section + '.' + method ) {
                        allSuccesses++;
                      }
                    });
                }
             });
    
            let transfers_start = i * (2 * mint_size + transfer_size);
            await api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start, transfers_start + mint_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                if (status.isInBlock) {
                    events.forEach(({ event: { data, method, section }, phase }) => {
                      if ('mantaPay.ToPrivate' == section + '.' + method ) {
                        allSuccesses++;
                      }
                    });
                }
             });
             await api.tx.mantaPay.toPrivate(transfers_buffer.subarray(transfers_start + mint_size, transfers_start + 2 * mint_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                if (status.isInBlock) {
                    events.forEach(({ event: { data, method, section }, phase }) => {
                      if ('mantaPay.ToPrivate' == section + '.' + method ) {
                        allSuccesses++;
                      }
                    });
                }
             });
             await api.tx.mantaPay.privateTransfer(transfers_buffer.subarray(transfers_start + 2 * mint_size, transfers_start + 2 * mint_size + transfer_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                if (status.isInBlock) {
                    events.forEach(({ event: { data, method, section }, phase }) => {
                      if ('mantaPay.ToPrivate' == section + '.' + method ) {
                        allSuccesses++;
                      }
                    });
                }
             });
    
            let reclaims_start = i * (2 * mint_size + reclaim_size);
            await api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start, reclaims_start + mint_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                if (status.isInBlock) {
                    events.forEach(({ event: { data, method, section }, phase }) => {
                      if ('mantaPay.ToPrivate' == section + '.' + method ) {
                        allSuccesses++;
                      }
                    });
                }
             });
            await api.tx.mantaPay.toPrivate(reclaims_buffer.subarray(reclaims_start + mint_size, reclaims_start + 2 * mint_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                if (status.isInBlock) {
                    events.forEach(({ event: { data, method, section }, phase }) => {
                      if ('mantaPay.ToPrivate' == section + '.' + method ) {
                        allSuccesses++;
                      }
                    });
                }
             });
            if (i == start_iteration + tests_iterations - 1) {
                const unsub = await api.tx.mantaPay.toPublic(reclaims_buffer.subarray(reclaims_start + 2 * mint_size, reclaims_start + 2 * mint_size + reclaim_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                    if (status.isInBlock) {
                        events.forEach(({ event: { data, method, section }, phase }) => {
                            let event = section + '.' + method;
                            console.log('method: ', event);
                            if ('mantaPay.ToPublic' == event ) {
                              allSuccesses++;
                            }
                          });
                    } else if (status.isFinalized) {
                         finalizedCount++;
                         lastFinalized = true;
                         var endTime = performance.now()
                         totalTime = endTime - startTime;
                         totalTime = totalTime / 1000;
                         unsub();
                    }
                 });
            } else {
                await api.tx.mantaPay.toPublic(reclaims_buffer.subarray(reclaims_start + 2 * mint_size, reclaims_start + 2 * mint_size + reclaim_size)).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
                    if (status.isInBlock) {
                        console.log('Included at block hash', status.asInBlock.toHex());
                        console.log('Events:');
                        events.forEach(({ event: { data, method, section }, phase }) => {
                          let event = section + '.' + method;
                          console.log('method: ', event);
                          if ('mantaPay.ToPublic' == event ) {
                            allSuccesses++;
                        }
                        });
                      } else if (status.isFinalized) {
                        console.log('Finalized block hash', status.asFinalized.toHex());
                      }
                 });
            }
            await new Promise(resolve => setTimeout(resolve, 12000));
    
            txs_sent += 7;
            console.log("\n Transactions sent: ", txs_sent);
        }
        
        // wait all txs finalized
        for(let i = 0; i < test_config.max_wait_time_sec; i++){
            await delay(1000);
            if (lastFinalized) {
                let tps = totalTime / (tests_iterations * 7);
                console.log("Tps is: ", tps);
                assert(tps <= test_config.expected_tps);
                break;
            }
        }

        if(lastFinalized == false) {
            assert(false);
        }

        if (allSuccesses != tests_iterations * 7) {
            console.log("allSuccesses Count: ", allSuccesses);
            assert(false);
        }

        api.disconnect();
    }).timeout(test_config.timeout);
});
