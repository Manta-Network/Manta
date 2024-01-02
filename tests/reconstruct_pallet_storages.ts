import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { farming_rpc_api, farming_types } from './types';
import { assert } from 'chai';
import {execute_transaction, execute_via_governance } from "./chain-util";

const test_config = {
    timeout: 2000000
}

// A helper about reconstructing mainnet pallet'chain states for local testnet,
// then test storage migration on local testnet.

describe('Register Assets', () => {
    it('Check Assets', async () => {
        let nodeAddress = "ws://127.0.0.1:9921";
        console.log("using address %s", nodeAddress);

        const wsProvider = new WsProvider(nodeAddress);
        const api = await ApiPromise.create({provider: wsProvider,
            types: farming_types,
            rpc: farming_rpc_api
        });
        const alice = new Keyring({type: 'sr25519'}).addFromUri("//Alice");

        const mantaProvider = new WsProvider('wss://a1.manta.systems');
        const mantaApi = await ApiPromise.create({provider: mantaProvider,
            types: farming_types,
            rpc: farming_rpc_api
        });

        const mainnetAssetIdLocationEntries = await mantaApi.query.assetManager.assetIdLocation.entries();
        console.log('The length of assetIdLocation:', mainnetAssetIdLocationEntries.length)
        const mainnetAssetIdLocations = new Array(mainnetAssetIdLocationEntries.length);
        mainnetAssetIdLocations.fill(null);

        for (const state of mainnetAssetIdLocationEntries) {
          const [{ args: [assetId] }, value] = state;
          const i = parseInt(assetId.toString()) - 8; // only insert non native assets' location
          if (i >= 0) {
            const m = await mantaApi.query.assetManager.assetIdMetadata(assetId);
            mainnetAssetIdLocations[i] = [value.toJSON(), m.toJSON()];
          }
        }

        let txs = [];
        for (let i = 0; i < mainnetAssetIdLocations.length; ++i) {
          if (mainnetAssetIdLocations[i] == null) continue;
          let callData = api.tx.assetManager.registerAsset(mainnetAssetIdLocations[i][0], mainnetAssetIdLocations[i][1]);
          txs.push(callData);
        }
        let batch = api.tx.utility.batch(txs);
        // if sudo, use this line of code
        // await execute_transaction(api, alice, batch, true);
        // if no sudo, use this line of code
        await execute_via_governance(api, alice, batch);

        // compare two states
        const localAssetIdLocationEntries = await api.query.assetManager.assetIdLocation.entries();
        const mainnetLocationAssetIdEntries = await mantaApi.query.assetManager.locationAssetId.entries();
        const localLocationAssetIdEntries = await api.query.assetManager.locationAssetId.entries();
        assert(localAssetIdLocationEntries.length===mainnetAssetIdLocationEntries.length)
        assert(mainnetLocationAssetIdEntries.length===localLocationAssetIdEntries.length)
        
        // check whether the storages are identical or not
        console.log(`Compare mainnet's assetIdLocation to local testnet's assetIdLocation`);
        for (const assetIdLocation of mainnetAssetIdLocationEntries) {
          const [{ args: [assetId] }, location] = assetIdLocation;
          const localLocation = await api.query.assetManager.assetIdLocation(assetId);
          assert(location.toString() === localLocation.toString());
        }

        console.log(`Compare mainnet's locationAssetId to local testnet's locationAssetId`)
        for (const state of mainnetLocationAssetIdEntries) {
          const [{ args: [location] }, assetId] = state;
          const localAssetId = await api.query.assetManager.locationAssetId(location);
          assert(assetId.toString() === localAssetId.toString());
        }

        api.disconnect();
    }).timeout(test_config.timeout);
});
