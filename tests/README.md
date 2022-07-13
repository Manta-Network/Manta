Functional Tests for Runtime Node
=================================

## Run unit tests
1. setup a relaychain/parachain deployment locally, for example, using `polkadot-launch`. 
Make sure the websocket address of the local parachain node is set to  `ws://127.0.0.1:9800`.
2. `yarn install`
3. `yarn correctness_test`

## Run rpc performance tests
1. setup a relaychain/parachain deployment locally, for example, using `polkadot-launch`. 
You can pass an optional argumetn `--address=<some_address>` but the default is set to  `ws://127.0.0.1:9800`.
2. `yarn install`
3. `yarn performance_test`