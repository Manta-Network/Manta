# Functional Tests for Runtime Node

## Prerequisites

- nodejs >= 16.x
- yarn

## MantaPay

### correctness test for rpc methods

1. setup a relaychain/parachain deployment locally, for example, using `polkadot-launch`.
2. Navigate to [config.json](./config/config.json), ensure `nodeAddress` points to the parachain.
3. Build tests

```
cd tests
yarn
```

4. Register new assets for future testing.

```
yarn init-assets
```

This script will register new assets, and issue some tokens to each assets.

5. Submit some private transactions to generate some Utxos.

```
yarn mint-assets
```

6. Now validate two rpc methods `pull_ledger_diff` and `dense_pull_ledger_diff`, ensure their response is the same.

```
yarn validate-pull-ledger-diff
```

Ensure test case get passed.

### performance tests rpc methods

1. setup a relaychain/parachain deployment locally, for example, using `polkadot-launch`.
2. `yarn install`
3. `yarn performance_test`
   You can pass an optional argument `--address=<some_address>` but the default is set to `ws://127.0.0.1:9921`.

## Runtime Tests

- `runtime_upgrade_test`, which is for do runtime upgrade test.

## Check Lints

Before you add or update test cases, please check ts lints for your change.

```
yarn lint:write
```
