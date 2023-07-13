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
  start_iteration: 14000,
  tests_iterations: 100,
  mint_size: 553,
  transfer_size: 1291,
  reclaim_size: 1001,
  expected_tps: 0.5,
  expected_average_block_time: 13000
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
    let lastBlock = 0;
    
    const firstHeader = await api.rpc.chain.getHeader();
    const firstBlock = firstHeader.number.toNumber();

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
            } else if (status.isFinalized) {
              lastFinalized = true;
              let endTime = performance.now();
              totalTime = endTime - startTime;
              // Convert to seconds
              totalTime = totalTime / 1000;
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
      await delay(12000);

      txsCount += 7;
      console.log("\n Transactions sent: ", txsCount);
    }

    // wait all txs finalized
    for (let i = 0; i < test_config.max_wait_time_sec; i++) {
      await delay(1000);
      if (lastFinalized) {
        let tps = (test_config.tests_iterations * 7) / totalTime;
        console.log("Tps is: ", tps);
        assert(tps >= test_config.expected_tps);
        
        const lastHeader = await api.rpc.chain.getHeader();
        lastBlock = lastHeader.number.toNumber();
        const averageBlockTime = totalTime / (lastBlock - firstBlock);
        console.log("Total time: ", totalTime);
        console.log("Number of blocks: ", (lastBlock - firstBlock));
        console.log("Average block time: ", averageBlockTime);
        assert(averageBlockTime < test_config.expected_average_block_time);

        break;
      }
    }

    assert(lastFinalized);

    if (allSuccesses != test_config.tests_iterations * 7) {
      console.log("allSuccesses Count: ", allSuccesses);
      assert(false);
    }

    api.disconnect();
  }).timeout(test_config.timeout);
});
