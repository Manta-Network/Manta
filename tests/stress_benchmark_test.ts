import { ApiPromise, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { manta_pay_types, rpc_api } from "./types";
import { delay } from "./test-util";
import { assert, expect } from "chai";
import minimist, { ParsedArgs } from "minimist";
import { readFile } from "fs/promises";
import { manta_pay_config } from './manta_pay';
import { MantaPrivateWallet, Environment, Network } from 'manta.js';

const test_config = {
  ws_address: "ws://127.0.0.1:9801",
  mnemonic:
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice",
  timeout: 20000000,
  max_wait_time_sec: 100000,
  mints_offset: 2,
  transfers_offset: 4,
  reclaims_offset: 4,
  total_iterations: 15000,
  start_iteration: 13500,
  tests_iterations: 100,
  mint_size: 552,
  transfer_size: 1290,
  reclaim_size: 968,
  expected_tps: 0.5,

  sync_iterations: 50,
  sync_time: 80000,
};

describe("Node RPC Test", () => {
  it("Check RPC result", async () => {
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
      rpc: rpc_api,
    });
    const keyring = new Keyring({ type: "sr25519" });
    const sender = keyring.addFromMnemonic(test_config.mnemonic);

    const mintsContent = await readFile("./data/precomputed_mints");
    const mintsBuffer = mintsContent.subarray(
      test_config.mints_offset,
      test_config.mint_size * test_config.total_iterations
    );
    const full_transfer_size =
      test_config.mint_size * 2 + test_config.transfer_size;
    const transfersContent = await readFile("./data/precomputed_transfers");
    const transfersBuffer = transfersContent.subarray(
      test_config.transfers_offset,
      full_transfer_size * test_config.total_iterations
    );
    const fullReclaimSize = test_config.mint_size * 2 + test_config.reclaim_size;
    const reclaimsContent = await readFile("./data/precomputed_reclaims");
    const reclaimsBuffer = reclaimsContent.subarray(
      test_config.reclaims_offset,
      fullReclaimSize * test_config.total_iterations
    );
    let lastFinalized = false;
    let allSuccesses = 0;
    let txsCount = 0;
    let startTime = performance.now();
    let totalTime = 0;
    
    
    const before_rpc = performance.now();
    let checkpoint = {receiver_index: new Array<number>(manta_pay_config.shard_number).fill(0), sender_index: 0};
    let should_continue = true;
    let max_receivers = 1000;
    let max_senders = 1000;
    

    const privateWalletConfig = {
      environment: Environment.Production,
      network: Network.Calamari
    }

    const privateWallet = await MantaPrivateWallet.init(privateWalletConfig);
    await privateWallet.initalWalletSync();

    const after_rpc = performance.now();
    const sync_time = after_rpc - before_rpc;
    console.log("Sync time: ", sync_time);
    expect(sync_time < test_config.sync_time).equals(true);

    api.disconnect();
  }).timeout(test_config.timeout);
});
