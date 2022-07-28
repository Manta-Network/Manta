import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { setup_storage, generate_utxo, manta_pay_config, generate_void_number} from './manta_pay';
import { expect } from 'chai';
import minimist, { ParsedArgs } from 'minimist';

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
        const sudo_key_pair = keyring.addFromMnemonic(test_config.mnemonic);
        await setup_storage(api, sudo_key_pair, 0, test_config.storage_prepare_config);
        const data = await (api.rpc as any).mantaPay.pull_ledger_diff(
            {receiver_index: new Array<number>(manta_pay_config.shard_number).fill(0), sender_index: 0},
            BigInt(1024), BigInt(1024));
        data.receivers.forEach((value: any, index:number) => {
            let payload = new Uint8Array([
                ...value[0],
                ...value[1].ephemeral_public_key,
                ...value[1].ciphertext
            ]);
            let size_per_shard = test_config.storage_prepare_config.utxo_batch_size_per_shard;
            // this uses the fact that the RPC request is filled greedly
            expect(payload).to.deep.equal(generate_utxo(~~(index/size_per_shard), index % size_per_shard));
        });
        data.senders.forEach((value: any, index: number)=>{
            expect(value).to.deep.equal(generate_void_number(index));
        });
        api.disconnect();
    }).timeout(test_config.timeout);
});
