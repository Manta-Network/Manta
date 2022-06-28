Functional Tests for Runtime Node
=================================

## Run unit tests
1. setup a relaychain/parachain deployment locally, for example, using `polkadot-launch`. 
Make sure the websocket address of the local parachain node is set to  `ws://127.0.0.1:9800`.
2. `yarn install`
3. `yarn test`

## Run rpc performance tests
1. setup a relaychain/parachain deployment locally, for example, using `polkadot-launch`. 
Make sure the websocket address of the local parachain node is set to  `ws://127.0.0.1:9800`.
2. `yarn install`
3. `yarn ts-node rpc_performance.ts`