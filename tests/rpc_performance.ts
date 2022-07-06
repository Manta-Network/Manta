import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { StoragePrepareConfig, setup_storage, manta_pay_config} from './manta_pay';
import minimist, { ParsedArgs } from 'minimist';
import { performance } from 'perf_hooks';

async function single_rpc_performance(api:ApiPromise) {
    const before_rpc = performance.now();
    const receiver_checkpoint = new Array<number>(manta_pay_config.shard_number);
    receiver_checkpoint.fill(0);
    const data = await (api.rpc as any).mantaPay.pull_ledger_diff(
        {receiver_index: new Array<number>(manta_pay_config.shard_number).fill(0), sender_index: 0},
        BigInt(16384), BigInt(16384));
    const after_rpc = performance.now();
    console.log("ledger diff receiver size: %i", data.receivers.length);
    console.log("ledger diff void number size: %i", data.senders.length);
    console.log("single rpc sync time: %i ms", after_rpc - before_rpc);
}

async function main(){
    
    let nodeAddress = "";
    const args: ParsedArgs = minimist(process.argv.slice(2));
    if (args["address"] == null) {
        nodeAddress = "ws://127.0.0.1:9800";
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
    const sudo_key_pair = keyring.addFromMnemonic('bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice');
    const storage_prepare_config: StoragePrepareConfig = {
        utxo_batch_number: 4,
        utxo_batch_size_per_shard: 16,
        utxo_big_batch_number: 1,
        vn_batch_number: 2,
        vn_batch_size: 4096,
    }
    await setup_storage(api, sudo_key_pair, 0, storage_prepare_config);
    await single_rpc_performance(api);
}

main().catch(console.error).finally(() => process.exit());