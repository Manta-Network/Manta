import { assert, expect } from "chai";
import { ApiPromise } from "@polkadot/api";
import "@polkadot/api-augment";
import { signer } from "../config/config.json";
import { createPromiseApi, readChainSpec } from "../utils/utils";
import { base64Decode, isBase64 } from "@polkadot/util-crypto";
import { $Receivers, $Senders } from "../types";
import { u8aToHex } from "@polkadot/util";

/**
  Purpose: This test case is for testing dense/pull_ledger_diff rpc method,
  but `dense_pull_ledger_diff` response is encoded by base64. So we must decode `dense_pull_ledger_diff`'s response,
  and compare it with `pull_ledger_diff`'s response, ensure both response are equal.
  How to test:
  1. Create some new assets, and issue tokens for these assets.
  2. Make some `to_private`s transactions to generate Utxos.
  3. Send a request to `pull_ledger_diff`, get the response.
  4. Send a request to `dense_pull_ledger_diff`, get the response, and decode it.
  5. Compare both response.
**/

describe("Test dense/pull_ledger_diff rpc method", function () {
  let fullNodeApi: ApiPromise;

  before(async function () {
    // ensure node is health
    const chainSpec = await readChainSpec();
    const wsPort = chainSpec.parachains[0].nodes[0].wsPort;
    const nodeAddress = "ws://127.0.0.1:" + wsPort;

    fullNodeApi = await createPromiseApi(nodeAddress);
    const fullNodeHealth = await fullNodeApi.rpc.system.health();
    assert.isNotTrue(fullNodeHealth.toJSON().isSyncing);
  });

  it("Check dense pull response", async function () {
    // there're only 40 private extrinsic sent, so 128 is big enough to get all Utxos.
    const totalReceivers = 128;
    const totalSenders = 128;

    const shardNumber = 256;
    let checkPoint = {
      receiver_index: new Array<number>(shardNumber).fill(0),
      sender_index: 0,
    };

    const pullResponse = await (
      fullNodeApi.rpc as any
    ).mantaPay.pull_ledger_diff(
      checkPoint,
      BigInt(totalReceivers),
      BigInt(totalSenders)
    );

    const densePullResponse = await (
      fullNodeApi.rpc as any
    ).mantaPay.dense_pull_ledger_diff(
      checkPoint,
      BigInt(totalReceivers),
      BigInt(totalSenders)
    );

    // ensure both fields are base64 based string
    assert.isTrue(isBase64(densePullResponse.receivers.toString()));
    assert.isTrue(isBase64(densePullResponse.senders.toString()));

    // decode densePullResponse, ensure which is equal to pullResponse
    assert.isNotTrue(pullResponse.should_continue);
    assert.isNotTrue(densePullResponse.should_continue);

    const decodedRecievers = $Receivers.decode(
      base64Decode(densePullResponse.receivers.toString())
    );
    const decodedSenders = $Senders.decode(
      base64Decode(densePullResponse.senders.toString())
    );

    // ensure the length of receivers and senders are equal
    assert.equal(decodedRecievers.length, pullResponse.receivers.length);
    assert.equal(decodedSenders.length, pullResponse.senders.length);

    // ensure encoded receivers and senders are equal
    expect(
      u8aToHex(base64Decode(densePullResponse.receivers.toString()))
    ).to.deep.equal(pullResponse.receivers.toHex());
    expect(
      u8aToHex(base64Decode(densePullResponse.senders.toString()))
    ).to.deep.equal(pullResponse.senders.toHex());

    for (let i = 0; i < decodedRecievers.length; ++i) {
      const [utxo, incomingNotes] = decodedRecievers[i];

      // assert utxo
      assert.equal(
        utxo.is_transparent,
        pullResponse.receivers[i][0].is_transparent
      );
      assert.equal(
        u8aToHex(utxo.commitment),
        pullResponse.receivers[i][0].commitment
      );
      assert.equal(
        u8aToHex(utxo.public_asset.id),
        pullResponse.receivers[i][0].public_asset.id
      );
      assert.equal(
        u8aToHex(utxo.public_asset.value),
        pullResponse.receivers[i][0].public_asset.value
      );

      // assert FullIncomingNote
      assert.equal(
        incomingNotes.address_partition,
        pullResponse.receivers[i][1].address_partition
      );
      assert.equal(
        u8aToHex(incomingNotes.incoming_note.ephemeral_public_key),
        pullResponse.receivers[i][1].incoming_note.ephemeral_public_key
      );
      assert.equal(
        u8aToHex(incomingNotes.incoming_note.tag),
        pullResponse.receivers[i][1].incoming_note.tag
      );

      const ciphertext = incomingNotes.incoming_note.ciphertext.map(function (
        c
      ) {
        return u8aToHex(c);
      });
      assert.equal(
        ciphertext[0],
        pullResponse.receivers[i][1].incoming_note.ciphertext[0]
      );
      assert.equal(
        ciphertext[1],
        pullResponse.receivers[i][1].incoming_note.ciphertext[1]
      );
      assert.equal(
        ciphertext[1],
        pullResponse.receivers[i][1].incoming_note.ciphertext[1]
      );

      assert.equal(
        u8aToHex(incomingNotes.light_incoming_note.ephemeral_public_key),
        pullResponse.receivers[i][1].light_incoming_note.ephemeral_public_key
      );

      const _ciphertext = incomingNotes.light_incoming_note.ciphertext.map(
        function (c) {
          return u8aToHex(c);
        }
      );
      assert.equal(
        _ciphertext[0],
        pullResponse.receivers[i][1].light_incoming_note.ciphertext[0]
      );
      assert.equal(
        _ciphertext[1],
        pullResponse.receivers[i][1].light_incoming_note.ciphertext[1]
      );
      assert.equal(
        _ciphertext[1],
        pullResponse.receivers[i][1].light_incoming_note.ciphertext[1]
      );
    }

    // assert senders
    for (let i = 0; i < decodedSenders.length; ++i) {
      const [nullifier, outgoingNotes] = decodedSenders[i];
      assert.equal(u8aToHex(nullifier), pullResponse.senders[i][0]);

      // assert OutgoingNote
      assert.equal(
        u8aToHex(outgoingNotes.ephemeral_public_key),
        pullResponse.senders[i][1].ephemeral_public_key
      );

      const ciphertext = outgoingNotes.ciphertext.map(function (c) {
        return u8aToHex(c);
      });
      assert.equal(ciphertext[0], pullResponse.senders[i][1].ciphertext[0]);
      assert.equal(ciphertext[1], pullResponse.senders[i][1].ciphertext[1]);
    }

    expect(densePullResponse.senders_receivers_total).to.deep.equal(
      pullResponse.senders_receivers_total
    );
    assert.isNull(densePullResponse.next_checkpoint.toJSON());
  });

  after(async function () {
    // Exit the mocha process.
    // If not, the process will pend there.
    await fullNodeApi.disconnect();
  });
});
