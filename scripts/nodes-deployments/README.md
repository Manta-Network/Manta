### Manta Nodes Deployment

1. Build chain spec.
```shell
./target/release/manta build-spec --chain manta-testnet --raw --disable-default-bootnode > manta-raw.json
```

2. Start nodes by script.
```shell
# node1
./start-node1.sh

# node2
./start-node2.sh

# node3
./start-node3.sh

# node4
./start-node4.sh

# node5
./start-node5.sh
```
> Tips: Remember that nodes link to the right bootnodes(peer id).

3. Insert session keys to keystore respectively. Replace babe_mnemonics and gran_mnemonics with the exact ones respectively.
```
./insert-keys.sh
```
> Tips: Ensure that these ports which are showed in these scripts are open.