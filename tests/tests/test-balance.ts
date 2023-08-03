import { expect } from "chai";
import { step } from "mocha-steps";
import {describeWithManta, remark} from "./util";
import '@polkadot/api-augment';
import {MANTA_1B} from "../constants";

describeWithManta("Manta RPC (Balance)", (context) => {
    step("genesis balance is setup correctly", async function () {
        expect((await context.api.query.system.number()).toString()).to.equal("0");

        const aliceBalance = (await context.api.query.system.account(context.alice.address)).data.free.toString();
        expect(aliceBalance.toString()).to.equal(MANTA_1B);

        await remark(context);

        expect((await context.api.query.system.number()).toString()).to.equal("1");
    });
});