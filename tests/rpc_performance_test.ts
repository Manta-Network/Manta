import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { StoragePrepareConfig, setup_storage, manta_pay_config} from './manta_pay';
import minimist, { ParsedArgs } from 'minimist';
import { performance } from 'perf_hooks';
import { expect } from 'chai';

const test_config = {
    ws_address: "ws://127.0.0.1:9800",
    mnemonic: 'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice',
    storage_prepare_config: {
        utxo_batch_number: 4,
        utxo_batch_size_per_shard: 16,
        utxo_big_batch_number: 1,
        vn_batch_number: 2,
        vn_batch_size: 4096,
    },
    storage_setup_phase_timeout: 750000,
    sync_iterations: 50,
    expected_average_sync_time: 1500,
    testing_phase_timeout_tolerance: 1.5
}

async function single_rpc_performance(api:ApiPromise) {
    let total_sync_time = 0;
    for(let i = 0; i < test_config.sync_iterations; ++i){
        const receiver_checkpoint = new Array<number>(manta_pay_config.shard_number);
        receiver_checkpoint.fill(0);
        const before_rpc = performance.now();
        const data = await (api.rpc as any).mantaPay.pull_ledger_diff(
            {receiver_index: new Array<number>(manta_pay_config.shard_number).fill(0), sender_index: 0},
            BigInt(8192), BigInt(8192));
        const after_rpc = performance.now();
        const sync_time = after_rpc - before_rpc;
        console.log("ledger diff receiver size: %i", data.receivers.length);
        console.log("ledger diff void number size: %i", data.senders.length);
        console.log("single rpc sync time: %i ms", after_rpc - before_rpc);
        total_sync_time+=sync_time;
    }
    const average_sync_time = total_sync_time / test_config.sync_iterations;
    console.log("average sync time: %i ms", average_sync_time);
    expect(average_sync_time < test_config.expected_average_sync_time).equals(true);
}

describe('Node RPC Performance Test', () => { 
    it('Check RPC Performance result', async () => {
    
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
        const sudo_key_pair = keyring.addFromMnemonic(test_config.mnemonic);
        await setup_storage(api, sudo_key_pair, 0, test_config.storage_prepare_config);
        await single_rpc_performance(api);

        api.disconnect();
    }).timeout(test_config.storage_setup_phase_timeout + (test_config.sync_iterations * test_config.expected_average_sync_time * test_config.testing_phase_timeout_tolerance));
});
