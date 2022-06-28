import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { manta_pay_types, rpc_api } from './types';
import { StoragePrepareConfig, setup_storage, manta_pay_config} from './manta_pay';
//import { expect } from 'chai';

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
    timeout: 120000
}

describe('Node RPC Test', () => { 
    it('Check RPC result', async () => {
        const wsProvider = new WsProvider(test_config.ws_address);
        const api = await ApiPromise.create({ 
            provider: wsProvider,
            types: manta_pay_types,
            rpc: rpc_api});
        const keyring = new Keyring({ type: 'sr25519' });
        const sudo_key_pair = keyring.addFromMnemonic(test_config.mnemonic);
        await setup_storage(api, sudo_key_pair, 0, test_config.storage_prepare_config);
    }).timeout(test_config.timeout);
});
