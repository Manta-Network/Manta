# Manta's zombienet examples

## Docs and examples
- docs: https://paritytech.github.io/zombienet/
- examples: 
    - https://github.com/paritytech/zombienet/tree/main/examples
    - https://github.com/paritytech/cumulus/tree/master/zombienet
    - https://github.com/paritytech/polkadot/tree/master/zombienet_tests

- Demo docs: [demo](https://www.notion.so/mantanetwork/Zombienet-Demo-fa7e38568b474498b73f5e13adf906f9?pvs=4)

## Install zombienet

According to your platform, download the latest pre-compiled binary from [zombinet](https://github.com/paritytech/zombienet/releases) repository.

## Start network

### Start one paracahin network locally by zombienet
1. Go to [small-network](./small-network.toml), make the `command` point to the polkadot binary and manta binary.
2. Start the network.
    ```shell
    zombienet spawn --provider native small-network.toml
    ```
    After a while, the network will be started.

### Start two parachains
1. Go to [two-parachains](./two-parachains.toml), make the `command` point to the polkadot binary and manta binary.
2. Start the network.
    ```shell
    zombienet spawn --provider native two-parachains.toml
    ```
    After a while, two paracahins(`2084` and `2104`) will be started.

## Testing

Zombienet supports testing as well, but you have to write test cases with a special dsl: `zndsl`. 
Please take a look at this [doc](https://paritytech.github.io/zombienet/cli/test-dsl-definition-spec.html) to see more details about how to write test cases.

### Runtime upgrade(not ready)
1. Go to [runtime-upgrade](./runtime-upgrade.toml), make the `command` point to the polkadot binary and manta binary. And go to [runtime upgrade test case](./runtime-upgrade.zndsl), make sure `line 6` point to the correct wasm binary.
2. Run runtime upgrade.
    ```shell
    zombienet -f --provider native test runtime-upgrade.zndsl
    ```
    This test case would take minutes to be finished.

### Run you own customized test script
1. Go to [custom-script](./custom-script.toml), make the `command` point to the polkadot binary and manta binary. 
2. Define and implement a function named `run` in your script.
    ```ts
    async function run(nodeName, networkInfo, args) {
        return 2084;
    }
    ```
3. Go to the [custom-script test case](./custom-script.zndsl), make sure `line 8` point to your script, and compare the expected value.
    ```
    Dave: js-script ./custom-script.js return is equal to 2084 within 300 seconds
    ```
4. Start the test.
    ```shell
    zombienet -f --provider native test custom-script.zndsl
    ```

## Tips:
1. When run the network, please do not use the same node name for relaychain and parachain.
For example: if one relaychain node takes `Alice`, so you cannot use `Alice` for any parachain nodes, but you can use `alice`.
2. Please be careful the node name in your test case, you must understand what you want to test.
For example, if one relaychain node takes `Alice` as node, you can use `alice` for one of paracahin nodes, the `Alice` will test relaychain node, but `alice` is for parachain only.
    ```
    alice: parachain 2084 is registered within 225 seconds
    Alice: reports block height is at least 5 within 250 seconds
    ```
    The first line will check parachain's block production, the second line will check relaychain's.
    Because relaychain and parachain use the same node name, the zombienet's test framework won't know who is `Alice`.
