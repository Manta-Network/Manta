import { expect } from "chai";
import { step } from "mocha-steps";
import {describeWithManta, executeTx, remark} from "./util";
import '@polkadot/api-augment';
import {LP_USDT_USDC_METADATA, USDC_LOCATION, USDC_METADATA, USDT_LOCATION, USDT_METADATA} from "../constants";
import {BN} from "@polkadot/util";

describeWithManta("Manta RPC (Dex)", (context) => {
    step("dex add liquidity should work", async function () {
        const api = context.api;
        const alice = context.alice.address;
        const bob = context.bob.address;

        const parachainId = Number(await api.query.parachainInfo.parachainId());
        console.log(new Date() + " parachain:" + parachainId);

        let callData = await api.tx.assetManager.registerAsset(USDT_LOCATION, USDT_METADATA);
        await executeTx(context, callData, true);

        callData = await api.tx.assetManager.registerAsset(USDC_LOCATION, USDC_METADATA);
        await executeTx(context, callData, true);

        callData = api.tx.assetManager.registerLpAsset(8, 9, LP_USDT_USDC_METADATA);
        await executeTx(context, callData, true);

        callData = api.tx.zenlinkProtocol.createPair([parachainId,2,8], [parachainId,2,9]);
        await executeTx(context, callData, true);

        callData = api.tx.balances.transfer(bob, new BN("1000000000000000000000"));
        await executeTx(context, callData);

        callData = api.tx.assetManager.mintAsset(8, alice, new BN("20000000000000"));
        await executeTx(context, callData, true);
        callData = api.tx.assetManager.mintAsset(8, bob, new BN("20000000000000"));
        await executeTx(context, callData, true);

        callData = api.tx.assetManager.mintAsset(9, alice, new BN("200000000000000000"));
        await executeTx(context, callData, true);
        callData = api.tx.assetManager.mintAsset(9, bob, new BN("200000000000000000"));
        await executeTx(context, callData, true);

        callData = api.tx.zenlinkProtocol.addLiquidity([parachainId,2,8], [parachainId,2,9],
            new BN("10000000000000"), new BN("100000000000000000"),
            new BN("10000000000000"), new BN("100000000000000000"), 1000);
        await executeTx(context, callData);

        let state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        expect(JSON.parse(JSON.stringify(state)).trading.totalSupply.toString()).to.equal("1000000000000000");

        callData = api.tx.zenlinkProtocol.addLiquidity([parachainId,2,8], [parachainId,2,9],
            new BN("10000000000000"), new BN("100000000000000000"),
            new BN("10000000000000"), new BN("100000000000000000"), 1000);
        await executeTx(context, callData, false, false);

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        expect(JSON.parse(JSON.stringify(state)).trading.totalSupply.toString()).to.equal("2000000000000000");

        state = await api.query.assets.account(10, alice);
        expect(JSON.parse(JSON.stringify(state)).balance.toString()).to.equal("1000000000000000");

        state = await api.query.assets.account(10, bob);
        expect(JSON.parse(JSON.stringify(state)).balance.toString()).to.equal("1000000000000000");

        callData = api.tx.zenlinkProtocol.removeLiquidity([parachainId,2,8], [parachainId,2,9],
            new BN("1000000000000000"),
            new BN("10000000000000"), new BN("100000000000000000"),
            alice,
            1000);
        await executeTx(context, callData);

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        expect(JSON.parse(JSON.stringify(state)).trading.totalSupply.toString()).to.equal("1000000000000000");

        state = await api.query.assets.account(10, alice);
        expect(JSON.parse(JSON.stringify(state))).to.equal(null);
    });
});