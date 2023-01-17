import { bool } from "@polkadot/types";
import * as $ from "scale-codec";

export const manta_pay_types = {
  Checkpoint: {
    receiver_index: "[u64; 256]",
    sender_index: "u64",
  },
  FullIncomingNote: {
    address_partition: "u8",
    incoming_note: "IncomingNote",
    light_incoming_note: "LightIncomingNote",
  },
  IncomingNote: {
    ephemeral_public_key: "[u8; 32]",
    tag: "[u8; 32]",
    ciphertext: "[[u8;32]; 3]",
  },
  LightIncomingNote: {
    ephemeral_public_key: "[u8; 32]",
    ciphertext: "[[u8;32]; 3]",
  },
  Utxo: {
    is_transparent: "bool",
    public_asset: "Asset",
    commitment: "[u8; 32]",
  },
  Asset: {
    id: "[u8; 32]",
    value: "[u8; 16]",
  },
  OutgoingNote: {
    ephemeral_public_key: "[u8; 32]",
    ciphertext: "[[u8;32]; 2]",
  },
  PullResponse: {
    should_continue: "bool",
    receivers: "Vec<(Utxo, FullIncomingNote)>",
    senders: "Vec<([u8; 32], OutgoingNote)>",
    senders_receivers_total: "[u8; 16]",
  },
  DensePullResponse: {
    should_continue: "bool",
    receivers: "String",
    senders: "String",
    senders_receivers_total: "[u8; 16]",
    next_checkpoint: "Option<Checkpoint>",
  },
};

export const rpc_api = {
  mantaPay: {
    pull_ledger_diff: {
      description: "pull from mantaPay",
      params: [
        {
          name: "checkpoint",
          type: "Checkpoint",
        },
        {
          name: "max_receivers",
          type: "u64",
        },
        {
          name: "max_senders",
          type: "u64",
        },
      ],
      type: "PullResponse",
    },
    dense_pull_ledger_diff: {
      description: "pull from mantaPay",
      params: [
        {
          name: "checkpoint",
          type: "Checkpoint",
        },
        {
          name: "max_receivers",
          type: "u64",
        },
        {
          name: "max_senders",
          type: "u64",
        },
      ],
      type: "DensePullResponse",
    },
  },
};

/**
  Why define these types?
  There's a rpc method `dense_pull_ledger_diff` in mantaPay, but its response is encoded by base64.
  From frontend side, it doesn't know how to decode `Uint8Array` to concrete types. So we have to define all related types which
  are used by `DensePullResponse`.

  Which package to define types? https://github.com/paritytech/scale-ts
  Example: https://github.com/paritytech/scale-ts#example

  How do we use scale-ts?
  Please define these types carefully, ensure they're identical with related rust based types.
  Once you define these types, then you can decode raw data to a concrete type, like:
  ```ts
  import { $Receivers, $Senders } from "../types";
  ...
  
  const densePullResponse = await (fullNodeApi.rpc as any).mantaPay.dense_pull_ledger_diff(...);
  const decodedRecievers = $Receivers.decode(
    base64Decode(densePullResponse.receivers.toString())
  ); 
  ```
**/

const $Asset = $.object(
  $.field("id", $.sizedUint8Array(32)),
  $.field("value", $.sizedUint8Array(16))
);

const $Utxo = $.object(
  $.field("is_transparent", $.bool),
  $.field("public_asset", $Asset),
  $.field("commitment", $.sizedUint8Array(32))
);

const $IncomingNote = $.object(
  $.field("ephemeral_public_key", $.sizedUint8Array(32)),
  $.field("tag", $.sizedUint8Array(32)),
  $.field("ciphertext", $.sizedArray($.sizedUint8Array(32), 3))
);

const $LightIncomingNote = $.object(
  $.field("ephemeral_public_key", $.sizedUint8Array(32)),
  $.field("ciphertext", $.sizedArray($.sizedUint8Array(32), 3))
);

const $FullIncomingNote = $.object(
  $.field("address_partition", $.u8),
  $.field("incoming_note", $IncomingNote),
  $.field("light_incoming_note", $LightIncomingNote)
);

const $OutgoingNote = $.object(
  $.field("ephemeral_public_key", $.sizedUint8Array(32)),
  $.field("ciphertext", $.sizedArray($.sizedUint8Array(32), 2))
);

export const $Receivers = $.array($.tuple($Utxo, $FullIncomingNote));
export const $Senders = $.array($.tuple($.sizedUint8Array(32), $OutgoingNote));
