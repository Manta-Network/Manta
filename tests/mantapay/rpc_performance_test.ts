import { ApiPromise } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { setup_storage, manta_pay_config } from "./manta_pay";
import minimist, { ParsedArgs } from "minimist";
import { performance } from "perf_hooks";
import { expect } from "chai";
import { createPromiseApi, readChainSpec } from "../utils/utils";

const test_config = {
  mnemonic:
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice",
  storage_prepare_config: {
    utxo_batch_number: 4,
    utxo_batch_size_per_shard: 16,
    utxo_big_batch_number: 1,
    vn_batch_number: 2,
    vn_batch_size: 4096,
  },
  storage_setup_phase_timeout: 7500000,
  sync_iterations: 50,
  expected_average_sync_time: 250,
  testing_phase_timeout_tolerance: 1.5,
};

async function single_rpc_performance(api: ApiPromise, isDense: boolean) {
  let total_sync_time = 0;
  for (let i = 0; i < test_config.sync_iterations; ++i) {
    const receiver_checkpoint = new Array<number>(
      manta_pay_config.shard_number
    );
    receiver_checkpoint.fill(0);
    const before_rpc = performance.now();
    let data;
    if (isDense) {
      data = await (api.rpc as any).mantaPay.dense_pull_ledger_diff(
        {
          receiver_index: new Array<number>(manta_pay_config.shard_number).fill(
            0
          ),
          sender_index: 0,
        },
        BigInt(8192),
        BigInt(8192)
      );
    } else {
      data = await (api.rpc as any).mantaPay.pull_ledger_diff(
        {
          receiver_index: new Array<number>(manta_pay_config.shard_number).fill(
            0
          ),
          sender_index: 0,
        },
        BigInt(8192),
        BigInt(8192)
      );
    }
    const after_rpc = performance.now();
    const sync_time = after_rpc - before_rpc;
    expect(data.receivers.length).to.not.equal(0);
    expect(data.senders.length).to.not.equal(0);
    console.log("ledger diff receiver size: %i", data.receivers.length);
    console.log("ledger diff void number size: %i", data.senders.length);
    console.log("single rpc sync time: %i ms", after_rpc - before_rpc);
    total_sync_time += sync_time;
  }
  const average_sync_time = total_sync_time / test_config.sync_iterations;
  console.log("average sync time: %i ms", average_sync_time);
  expect(average_sync_time < test_config.expected_average_sync_time).equals(
    true
  );
}

describe("MantaPay RPC Performance Test", () => {
  let api: ApiPromise;

  before(async function () {
    // create api
    let nodeAddress = "";
    const args: ParsedArgs = minimist(process.argv.slice(2));
    if (args["address"] == null) {
      const chainSpec = await readChainSpec();
      const wsPort = chainSpec.parachains[0].nodes[0].wsPort;
      nodeAddress = "ws://127.0.0.1:" + wsPort;
    } else {
      nodeAddress = args["address"];
    }
    api = await createPromiseApi(nodeAddress);

    const keyring = new Keyring({ type: "sr25519" });
    const sudo_key_pair = keyring.addFromMnemonic(test_config.mnemonic);
    await setup_storage(
      api,
      sudo_key_pair,
      0,
      test_config.storage_prepare_config
    );
  });
  it("Check RPC pull_ledger_diff Performance result", async () => {
    await single_rpc_performance(api, false);
  }).timeout(
    test_config.storage_setup_phase_timeout +
      test_config.sync_iterations *
        test_config.expected_average_sync_time *
        test_config.testing_phase_timeout_tolerance
  );

  it("Check RPC dense_pull_ledger_diff Performance result", async () => {
    await single_rpc_performance(api, true);
  }).timeout(
    test_config.storage_setup_phase_timeout +
      test_config.sync_iterations *
        test_config.expected_average_sync_time *
        test_config.testing_phase_timeout_tolerance
  );

  after(async function () {
    // Exit the mocha process.
    // If not, the process will pend there.
    await api.disconnect();
  });
});
