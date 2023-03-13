import { expect } from "chai";
import { MantaPrivateWallet, Environment, Network } from 'manta.js/node';
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
  sync_time: 530000,
};

describe("Full Sync Test", () => {
  it("Check full sync result", async () => {

    const before_rpc = performance.now();
    
    const privateWalletConfig = {
      environment: Environment.Development,
      network: Network.Calamari
    }

    const privateWallet = await MantaPrivateWallet.init(privateWalletConfig);

    expect(await privateWallet.initalWalletSync()).to.be.ok;

    const after_rpc = performance.now();
    const sync_time = after_rpc - before_rpc;
    console.log("Sync time: ", sync_time);
    expect(sync_time < test_config.sync_time).equals(true);

  }).timeout(test_config.timeout);
});
