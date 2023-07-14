import { ApiPromise, WsProvider } from '@polkadot/api';
import { BN } from '@polkadot/util';
import { Keyring } from '@polkadot/keyring';
import { farming_rpc_api, farming_types } from './types';
import { expect } from 'chai';
import minimist, { ParsedArgs } from 'minimist';
import {execute_transaction} from "./chain-util";

// ./target/release/manta --chain=manta-localdev --alice --ws-port 9800 --rpc-cors all --execution=native
const test_config = {
    ws_address: "ws://127.0.0.1:9800",
    timeout: 2000000
}
const timer = (ms: number) => new Promise(res => setTimeout(res, ms))

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
        const api = await ApiPromise.create({provider: wsProvider,
            types: farming_types,
            rpc: farming_rpc_api
        });
        const alice = new Keyring({type: 'sr25519'}).addFromUri("//Alice");

        const location = {
            V1: {
                parents: 1,
                interior: {
                    X3: [
                        {
                            Parachain: 1000
                        },
                        {
                            PalletInstance: 50
                        },
                        {
                            GeneralIndex: 1984
                        }
                    ]
                }
            }
        };
        const metadata = {
            metadata: {
                name: "Tether USD",
                symbol: "USDT",
                decimals: 6,
                isFrozen: false
            },
            minBalance: 1000,
            isSufficient: true
        };
        let callData = api.tx.assetManager.registerAsset(location, metadata);
        await execute_transaction(api, alice, callData);

        // register lp asset
        const lp_metadata = {
            metadata: {
                name: "LP-MANTA-USDT",
                symbol: "LP",
                decimals: 12,
                isFrozen: false
            },
            minBalance: 1,
            isSufficient: true
        };
        callData = api.tx.assetManager.registerLpAsset(1, 8, lp_metadata);
        await execute_transaction(api, alice, callData);

        // create dex pair
        callData = api.tx.zenlinkProtocol.createPair([2104,0,0], [2104,2,8]);
        await execute_transaction(api, alice, callData);

        callData = api.tx.assetManager.mintAsset(8, alice.address, new BN(10000000000));
        await execute_transaction(api, alice, callData);

        // add liquidity to dex
        callData = api.tx.zenlinkProtocol.addLiquidity([2104,0,0], [2104,2,8],
            new BN("1000000000000000000000"), 1000000000, new BN("1000000000000000000000"), 1000000000, 8);
        await execute_transaction(api, alice, callData, false);

        // create farming pool
        callData = api.tx.farming.createFarmingPool([[9, 1000000000]], [[1, new BN("1000000000000000000")]], null, 100000000000000, 9, 0, 0, 2);
        await execute_transaction(api, alice, callData);

        // charge reward token to farming pool account
        callData = api.tx.farming.charge(0, [[1, new BN("1000000000000000000000")]]);
        await execute_transaction(api, alice, callData, false);

        // mock new block
        callData = api.tx.balances.transfer(alice.address, new BN("1000000000000000000"));
        await execute_transaction(api, alice, callData, false);

        callData = api.tx.balances.transfer(alice.address, new BN("1000000000000000000"));
        await execute_transaction(api, alice, callData, false);

        await timer(1000);
        let state = await api.query.zenlinkProtocol.pairStatuses([[2104,0,0], [2104,2,8]]);
        let json = JSON.parse(JSON.stringify(state));
        expect(new BN(json.trading.totalSupply.toString())).to.deep.equal(new BN("1000000000000000"));

        state = await api.query.farming.poolInfos(0);
        json = JSON.parse(JSON.stringify(state));
        expect(json.state).to.deep.equal("Charged");

        // user deposit lp token to farming pool
        callData = api.tx.farming.deposit(0, new BN("100000000000000"), null);
        await execute_transaction(api, alice, callData, false);

        // mock new block
        callData = api.tx.balances.transfer(alice.address, new BN("1000000000000000000"));
        await execute_transaction(api, alice, callData, false);
        callData = api.tx.balances.transfer(alice.address, new BN("1000000000000000000"));
        await execute_transaction(api, alice, callData, false);

        await timer(1000);
        state = await api.query.farming.poolInfos(0);
        json = JSON.parse(JSON.stringify(state));
        expect(json.state).to.deep.equal("Ongoing");

        state = await api.query.farming.sharesAndWithdrawnRewards(0, alice.address);
        expect(new BN(JSON.parse(JSON.stringify(state)).share.toString())).to.deep.equal(new BN("100000000000000"));

        // get farming reward
        let response = await (api.rpc as any).farming.getFarmingRewards(alice.address, 0);
        expect(new BN(response[0][1].toString())).to.deep.equal(new BN("1000000000000000000"));

        callData = api.tx.balances.transfer(alice.address, new BN("1000000000000000000"));
        await execute_transaction(api, alice, callData, false);

        await timer(1000);
        response = await (api.rpc as any).farming.getFarmingRewards(alice.address, 0);
        expect(new BN(response[0][1].toString())).to.deep.equal(new BN("2000000000000000000"));

        api.disconnect();
    }).timeout(test_config.timeout);
});
