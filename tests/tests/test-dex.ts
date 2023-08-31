import { expect } from "chai";
import { step } from "mocha-steps";
import {describeWithManta, executeTx, remark} from "./util";
import '@polkadot/api-augment';
import {
    LP_1K, LP_2K,
    LP_USDT_USDC_METADATA, MANTA_1K, USDC_10M, USDC_20M,
    USDC_LOCATION,
    USDC_METADATA,
    USDT_10M, USDT_20M,
    USDT_LOCATION,
    USDT_METADATA
} from "../constants";
import {BN} from "@polkadot/util";
import { delay } from "../test-util";

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

        callData = api.tx.balances.transfer(bob, MANTA_1K);
        await executeTx(context, callData);

        callData = api.tx.assetManager.mintAsset(8, alice, USDT_20M);
        await executeTx(context, callData, true);
        callData = api.tx.assetManager.mintAsset(8, bob, USDT_20M);
        await executeTx(context, callData, true);

        callData = api.tx.assetManager.mintAsset(9, alice, USDC_20M);
        await executeTx(context, callData, true);
        callData = api.tx.assetManager.mintAsset(9, bob, USDC_20M);
        await executeTx(context, callData, true);

        // Alice add liquidity
        callData = api.tx.zenlinkProtocol.addLiquidity([parachainId,2,8], [parachainId,2,9],
            USDT_10M, USDC_10M, USDT_10M, USDC_10M, 1000);
        await executeTx(context, callData);

        let state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        expect(JSON.parse(JSON.stringify(state)).trading.totalSupply.toString()).to.equal(LP_1K);

        // Bob add liquidity
        callData = api.tx.zenlinkProtocol.addLiquidity([parachainId,2,8], [parachainId,2,9],
            USDT_10M, USDC_10M, USDT_10M, USDC_10M, 1000);
        await executeTx(context, callData, false, false);

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        expect(JSON.parse(JSON.stringify(state)).trading.totalSupply.toString()).to.equal(LP_2K);

        state = await api.query.assets.account(10, alice);
        expect(JSON.parse(JSON.stringify(state)).balance.toString()).to.equal(LP_1K);

        state = await api.query.assets.account(10, bob);
        expect(JSON.parse(JSON.stringify(state)).balance.toString()).to.equal(LP_1K);

        callData = api.tx.zenlinkProtocol.removeLiquidity([parachainId,2,8], [parachainId,2,9],
            new BN(LP_1K),
            USDT_10M, USDC_10M,
            alice,
            1000);
        await executeTx(context, callData);

        state = await api.query.zenlinkProtocol.pairStatuses([[parachainId,2,8], [parachainId,2,9]]);
        expect(JSON.parse(JSON.stringify(state)).trading.totalSupply.toString()).to.equal(LP_1K);

        state = await api.query.assets.account(10, alice);
        expect(JSON.parse(JSON.stringify(state))).to.equal(null);

        callData = api.tx.zenlinkProtocol.setNativeSwapFeeFactor(200);
        await executeTx(context, callData, true);

        await delay(12000);

        callData = api.tx.zenlinkProtocol.createPair([parachainId,0,1], [parachainId,2,8]);
        await executeTx(context, callData, true);

        callData = api.tx.zenlinkProtocol.addLiquidity([parachainId,0,1], [parachainId,2,8],
            MANTA_1K, USDT_10M, MANTA_1K, USDT_10M, 1000);
        await executeTx(context, callData);

        callData = api.tx.zenlinkProtocol.swapExactAssetsForAssets(100,10,[[parachainId,0,1],[parachainId,2,8],alice,1000000);
        await executeTx(context, callData);

        await delay(24000);

        const swapFeesBalance = (await api.query.system.account("0x2f73776170706f74000000000000000000000000000000000000000000000000")).data.free.toString();
        console.log("Swap fees pot balance: ", swapFeesBalance);
        const swapFeesBalance2 = (await api.query.system.account("0x2f73776170706f7400000000000000000000000000000000000000000000000000")).data.free.toString();
        console.log("Swap fees pot balance: ", swapFeesBalance2);
		// #[pallet::compact] amount_in: AssetBalance,
		// 	#[pallet::compact] amount_out_min: AssetBalance,
		// 	path: Vec<T::AssetId>,
		// 	recipient: T::AccountId,
		// 	#[pallet::compact] deadline: T::BlockNumber,
    });
});