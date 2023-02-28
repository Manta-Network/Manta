export const manta_pay_types = {
    Checkpoint: {
        receiver_index: '[u64; 256]',
        sender_index: 'u64'
    },
    FullIncomingNote: {
        address_partition: 'u8',
        incoming_note: 'IncomingNote',
        light_incoming_note: 'LightIncomingNote',
    },
    IncomingNote: {
        ephemeral_public_key: '[u8; 32]',
        tag: '[u8; 32]',
        ciphertext: '[[u8;32]; 3]',
    },
    LightIncomingNote: {
        ephemeral_public_key: '[u8; 32]',
        ciphertext: '[[u8;32]; 3]',
    },
    Utxo: {
        is_transparent: 'bool',
        public_asset: 'Asset',
        commitment: '[u8; 32]',
    },
    Asset: {
        id: '[u8; 32]',
        value: '[u8; 16]',
    },
    OutgoingNote: {
        ephemeral_public_key: '[u8; 32]',
        ciphertext: '[[u8;32]; 2]',
    },
    PullResponse: {
        should_continue: 'bool',
        receivers: 'Vec<(Utxo, FullIncomingNote)>',
        senders: 'Vec<([u8; 32], OutgoingNote)>',
        senders_receivers_total: '[u8; 16]',
    }
};

export const rpc_api = {
    mantaPay: {
        pull_ledger_diff: {
            description: 'pull from mantaPay',
            params: [
                {
                    name: 'checkpoint',
                    type: 'Checkpoint'
                },
                {
                    name: 'max_receivers',
                    type: 'u64'
                },
                {
                    name: 'max_senders',
                    type: 'u64'
                }
            ],
            type: 'PullResponse'
        }
    }
}