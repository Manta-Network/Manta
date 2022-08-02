export const manta_pay_types = {
    Checkpoint: {
        receiver_index: '[u64; 256]',
        sender_index: 'u64'
    },
    EncryptedNote: {
        ephemeral_public_key: '[u8; 32]',
        ciphertext: '[u8; 68]'
    },
    PullResponse: {
        should_continue: 'bool',
        receivers: 'Vec<([u8; 32], EncryptedNote)>',
        senders: 'Vec<[u8; 32]>',
        senders_receivers_total: 'u128',
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
                    name: 'max_receiver',
                    type: 'u64'
                },
                {
                    name: 'max_sender',
                    type: 'u64'
                }
            ],
            type: 'PullResponse'
        }
    }
}