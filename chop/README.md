## Do runtime upgrade by chopsticks

- chopsticks: https://github.com/AcalaNetwork/chopsticks
- manta config: [manta](./manta.yml), which configure Alice as sudo account.
- polkadot config: [polkadot](./polkadot.yml)

1. Update the recent block number for polkadot and manta.
```
POLKADOT_BLOCK_NUMBER=
MANTA_BLOCK_NUMBER=
```

2. Start both forked chains with chopsticks
```
npx @acala-network/chopsticks@latest xcm --r=polkadot.yml --p=manta.yml
```
manta will take port `8000`, and ``8001` for polkadot by default.

3. Please authorize runtime upgrade first on manta chain.
![authorize](./pics/authorize.png)

4. Then enact runtime upgrade
![enact](./pics/enact.png)

5. Advance some blocks on manta and polkadot chain by this scripts
```
cd tests
yarn advance-blocks
```
Ideally, you will see runtime upgraded event, that means the upgrade will have been finished.
