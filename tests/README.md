Functional Tests for Manta
==========================

Get data from `mantaPay_pull` rpc methods:
```bash
yarn ts-node manta_pay_rpc.ts --address=some_address
```
The `address` is optional, with default `ws://127.0.0.1:9801`

The test will insert some random but correct data to the Shards and VoidNumberSetInsertionOrder storage items.
Currently 9 shards entries and 9 VN insertion order set entries, dispatched via the sudo account.
This means sending 1 transaction per block with some delays in between, so the test can take a few minutes.
The final result should print those 18 entries that were inserted:

```
{
  should_continue: false,
  receivers: [
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ],
    [
      '0x83590b405cf760cb1660fc295f7810d428fb27d946f2bba38cb9ca5b7d4ed643',
      [Object]
    ]
  ],
  senders: [
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071',
    '0xefe34cfd4418c9b1c04e555965e479d00ec4814ed0cd94641df1a8c6f9fa1071'
  ]
}
```