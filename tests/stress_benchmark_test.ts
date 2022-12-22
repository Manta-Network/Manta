import { ApiPromise, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { manta_pay_types, rpc_api } from "./types";
import { delay } from "./test-util";
import { assert } from "chai";
import minimist, { ParsedArgs } from "minimist";
import { readFile } from "fs/promises";

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
  expected_tps: 1.85,
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

    const mintsContent = await readFile("./precomputed_mints");
    const mintsBuffer = mintsContent.subarray(
      test_config.mints_offset,
      test_config.mint_size * test_config.total_iterations
    );
    let full_transfer_size =
      test_config.mint_size * 2 + test_config.transfer_size;
    const transfersContent = await readFile("./precomputed_transfers");
    const transfersBuffer = transfersContent.subarray(
      test_config.transfers_offset,
      full_transfer_size * test_config.total_iterations
    );
    let fullReclaimSize = test_config.mint_size * 2 + test_config.reclaim_size;
    const reclaimsContent = await readFile("./precomputed_reclaims");
    const reclaimsBuffer = reclaimsContent.subarray(
      test_config.reclaims_offset,
      fullReclaimSize * test_config.total_iterations
    );
    let lastFinalized = false;
    let allSuccesses = 0;
    let txsCount = 0;
    let startTime = performance.now();
    let totalTime = 0;

    for (
      let i = test_config.start_iteration;
      i < test_config.start_iteration + test_config.tests_iterations;
      ++i
    ) {
      await api.tx.mantaPay
        .toPrivate(
          mintsBuffer.subarray(
            test_config.mint_size * i,
            test_config.mint_size * (i + 1)
          )
        )
        .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
              let event = section + "." + method;
              console.log("event: ", event);
              if ("mantaPay.ToPrivate" == event) {
                allSuccesses++;
              }
            });
          }
        });

      let transfersStart =
        i * (2 * test_config.mint_size + test_config.transfer_size);
      await api.tx.mantaPay
        .toPrivate(
          transfersBuffer.subarray(
            transfersStart,
            transfersStart + test_config.mint_size
          )
        )
        .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
              let event = section + "." + method;
              console.log("event: ", event);
              if ("mantaPay.ToPrivate" == event) {
                allSuccesses++;
              }
            });
          }
        });
      await api.tx.mantaPay
        .toPrivate(
          transfersBuffer.subarray(
            transfersStart + test_config.mint_size,
            transfersStart + 2 * test_config.mint_size
          )
        )
        .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
              let event = section + "." + method;
              console.log("event: ", event);
              if ("mantaPay.ToPrivate" == event) {
                allSuccesses++;
              }
            });
          }
        });
      await api.tx.mantaPay
        .privateTransfer(
          transfersBuffer.subarray(
            transfersStart + 2 * test_config.mint_size,
            transfersStart +
              2 * test_config.mint_size +
              test_config.transfer_size
          )
        )
        .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
              let event = section + "." + method;
              console.log("event: ", event);
              if ("mantaPay.PrivateTransfer" == event) {
                allSuccesses++;
              }
            });
          }
        });
      let reclaimsStart =
        i * (2 * test_config.mint_size + test_config.reclaim_size);
      await api.tx.mantaPay
        .toPrivate(
          reclaimsBuffer.subarray(
            reclaimsStart,
            reclaimsStart + test_config.mint_size
          )
        )
        .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
              let event = section + "." + method;
              console.log("event: ", event);
              if ("mantaPay.ToPrivate" == event) {
                allSuccesses++;
              }
            });
          }
        });
      await api.tx.mantaPay
        .toPrivate(
          reclaimsBuffer.subarray(
            reclaimsStart + test_config.mint_size,
            reclaimsStart + 2 * test_config.mint_size
          )
        )
        .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
          if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
              let event = section + "." + method;
              console.log("event: ", event);
              if ("mantaPay.ToPrivate" == event) {
                allSuccesses++;
              }
            });
          }
        });
      if (i == test_config.start_iteration + test_config.tests_iterations - 1) {
        const unsub = await api.tx.mantaPay
          .toPublic(
            reclaimsBuffer.subarray(
              reclaimsStart + 2 * test_config.mint_size,
              reclaimsStart +
                2 * test_config.mint_size +
                test_config.reclaim_size
            )
          )
          .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
            if (status.isInBlock) {
              console.log("Included at block hash", status.asInBlock.toHex());
              console.log("Events:");
              events.forEach(({ event: { data, method, section }, phase }) => {
                let event = section + "." + method;
                console.log("event: ", event);
                if ("mantaPay.ToPublic" == event) {
                  allSuccesses++;
                }
              });
            } else if (status.isFinalized) {
              lastFinalized = true;
              let endTime = performance.now();
              totalTime = endTime - startTime;
              // Convert to seconds
              totalTime = totalTime / 1000;
              unsub();
            }
          });
      } else {
        await api.tx.mantaPay
          .toPublic(
            reclaimsBuffer.subarray(
              reclaimsStart + 2 * test_config.mint_size,
              reclaimsStart +
                2 * test_config.mint_size +
                test_config.reclaim_size
            )
          )
          .signAndSend(sender, { nonce: -1 }, ({ events = [], status }) => {
            if (status.isInBlock) {
              console.log("Included at block hash", status.asInBlock.toHex());
              console.log("Events:");
              events.forEach(({ event: { data, method, section }, phase }) => {
                let event = section + "." + method;
                console.log("event: ", event);
                if ("mantaPay.ToPublic" == event) {
                  allSuccesses++;
                }
              });
            }
          });
      }
      await new Promise((resolve) => setTimeout(resolve, 9000));

      txsCount += 7;
      console.log("\n Transactions sent: ", txsCount);
    }

    console.log("Reached");

    // wait all txs finalized
    for (let i = 0; i < test_config.max_wait_time_sec; i++) {
      await delay(1000);
      if (lastFinalized) {
        let tps = (test_config.tests_iterations * 7) / totalTime;
        console.log("Tps is: ", tps);
        assert(tps >= test_config.expected_tps);
        break;
      }
    }

    if (lastFinalized == false) {
      assert(false);
    }

    if (allSuccesses != test_config.tests_iterations * 7) {
      console.log("allSuccesses Count: ", allSuccesses);
      assert(false);
    }

    api.disconnect();
  }).timeout(test_config.timeout);
});
