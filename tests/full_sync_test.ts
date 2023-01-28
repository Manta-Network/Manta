import { expect } from "chai";
import minimist, { ParsedArgs } from "minimist";
import { MantaPrivateWallet, Environment, Network } from 'manta.js';
import fetch from 'node-fetch';

// @ts-ignore
global.fetch = fetch;
// @ts-ignore
global.Headers = fetch.Headers;
// @ts-ignore
global.Request = fetch.Request;
// @ts-ignore
global.Response = fetch.Response;

const test_config = {
  timeout: 20000000,
  sync_iterations: 50,
  sync_time: 80000,
};

describe("Full Sync Test", () => {
  it("Check full sync result", async () => {

    const before_rpc = performance.now();
    
    const privateWalletConfig = {
      environment: Environment.Production,
      network: Network.Dolphin
    }

    const privateWallet = await MantaPrivateWallet.init(privateWalletConfig);
    console.log("check1");
    await privateWallet.initalWalletSync();
    
    const after_rpc = performance.now();
    const sync_time = after_rpc - before_rpc;
    console.log("Sync time: ", sync_time);
    expect(sync_time < test_config.sync_time).equals(true);

  }).timeout(test_config.timeout);
});
