---
name: Calamari XCM testing and onboarding checklist.
about: Perform cross chain transfer tests in order to open HRMP channel with Calamari!

---

# ⛓ Calamari XCM Testing And Onboarding Checklist

## Process Overview:
- [ ] Open channel for communication.

- [ ] Test XCM between parachains locally with polkadot-launch.

- [ ] Become a parachain on Rococo.

- [ ] Calculate and fund your parachain's sovereign account on Rococo.

- [ ] Open HRMP channels with Calamari.

- [ ] Assets registrations.

- [ ] Complete XCM tests between parachains.

- [ ] Next steps discussion.

* Manta Team Contact:
    - Georgi (XCM Eng): @Ghz (Telegram)
    - Shumo (Co-founder, Tech.): @xstec (Telegram)

## Open communication channel

- Ideally we should create a channel for direct messaging between or teams.
- We can communicate on Discord, Element or Telegram.
- Please fill out this [form](https://forms.gle/SPitZjuiir6fVkrn8) and someone from our team will contact you to setup the chat room.

## Local XCM Integration

- As a first step we insist that both teams first run all tests on a local network.
- For that you can download the latest manta binary from the Releases page.
- Then use polkadot-launch to launch a `calamari-local` or `calamari-dev` network for testing.
- You will also need to launch a `rococo-local` relay chain using the latest release of Polkadot.
- Here's a reference polkadot-launch config for [calamari-dev](#Example-Polkadot-Launch-Config).
- You can add HRMP channels directly in the config.
- Please let us know if there's a specific branch of your codebase that we should test with.

## XCM Integration on Rococo

- The next step of the Calamari integration is to integrate with our parachain (Dolphin) on the official Rococo relay chain. As part of this integration, you’ll need to register an HRMP channel with Dolphin. On the Rococo ecosystem you can also test integrations with a number of other parachains, as most Kusama chains have also deployed on Rococo.

### Rococo Ecosystem Data

- [Rococo endpoint](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/explorer)
- [Dolphin endpoint](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Feddie.rococo.dolphin.engineering#/explorer)
- [Rococo faucet guide](https://wiki.polkadot.network/docs/build-pdk#obtaining-roc)
- [Dolphin faucet](https://discord.gg/UvXpxuyg)

### Sync Node & Open Rococo Slot Request

- To sync your node, you can use the following [relay chain spec](https://raw.githubusercontent.com/paritytech/polkadot/master/node/service/chain-specs/rococo.json) (note: relay chain is Rococo based, and will probably take a few hours to sync)
- Register your parachain on Rococo. For that you will need to open a [Rococo Slot Request](https://github.com/paritytech/subport/issues) issue and follow the instructions.

## Calculate and Fund your Parachain's Sovereign Account

- To calculate your Parachain’s Sovereign account, you can use this [simple xcm tool](https://github.com/Manta-Network/Dev-Tools/tree/main/caclulate-sovereign-account)
    
- Make sure you run the command by providing the parachain ID (flag  –paraid NUMBER) that you’ve selected on Rococo. For example, Dolphin’s Sovereign account for both the relay chain and other parachains can be obtained with:

```
ts-node calculateSovereignAddress.ts --paraid 2084
```
- The result will be:
```
Sovereign Account Address on Relay: 0x7061726124080000000000000000000000000000000000000000000000000000
```

- Once you’ve got your `Sovereign Account`’s address, please fund it using the [Rococo faucet](https://wiki.polkadot.network/docs/build-pdk#obtaining-roc). The sovereign account will be used to execute some transactions  on the relay chain on behalf of your parachain and will need to reserve some funds for those transactions. Specifically funds will be needed for:
1. `Transact` instruction and encoded inner calls to HRMP pallet to open/accept HRMP channels.
2. Reserves for the HRMP channels, which are chain specific configurations, and can be checked with `configuration.activeConfig()` - `hrmpSenderDeposit` and `hrmpRecipientDeposit` 

## Create HRMP Channel with Dolphin
### Get the Relay Encoded Call Data to Open HRMP Channel.

- Once your parachain is onboard, you need to create the HRMP channel between your Parachain and Dolphin.
- The first step is to get an encoded call data from the relay chain. The extrinsic contains the target parachain ID, max number of messages, and max message size, described in the next bullet.
- In PolkadotJS app, switch to the Rococo network and go to Developer -> [Javascript section](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/js). Then run the following code, and replace the demo recipient para id with your own:

```
const tx = api.tx.hrmp.hrmpInitOpenChannel(receiverParaId, hrmpChannelMaxCapacity, hrmpChannelMaxMessageSize);
console.log(tx.toHex());
```

- With arguments `2084, 9, 1024` respectively the result will be:

`0x3c041700240800000800000000040000`, remove the leading hex `3c04`, and so the final encoded result is:
 ```
 0x1700240800000800000000040000
 ```

 **Note:** that `hrmpChannelMaxCapacity` and `hrmpChannelMaxMessageSize` need to be in the range of the relay chain's configuration and can be checked with `configuration.activeConfig()`

### Send XCM to Relay Chain

- The next step is to build and send an XCM message to the relay chain that will request a channel to be opened through the relay chain. This XCM message needs to be sent from the root account (either SUDO or via governance). The message can be broken down in the following elements:
    1. Withdraw asset: take funds out of the Sovereign Account of the origin parachain (in the relay chain) to a holding state
    2. Buy execution: buys execution time from the relay chain, to execute the XCM message
    3. Transact: provides the call data to be executed
    4. Refund surplus: refunds any surplus weight from step 2.
    5. Deposit asset (optional): refunds the leftover funds after the execution. If this is not provided, no refunds will be carried out
- Therefore, to build/send this XCM, you need to:
    1. Polkadot.js Apps in your parachain -> extrinsics
    2. Set the following parameters: polkadotXcm -> send
    3. The destination has to be the relay chain, for dest (V1) set:
    `{ parents:1, interior: Here }`
    4. For the message (V2), you’ll be adding 4 items (described before):
        1. `WithdrawAsset { id: Concrete { parents: 0, interior: Here}, Fungible: 1000000000000 }`
        2. `BuyExecution { id: Concrete: {parents: 0, interior: Here}, Fungible: 1000000000000, weightLimit: Unlimited }`
        3. `Transact { originType: Native, requireWeightAtMost: 1000000000, call: XcmDoubleEncoded: { encoded: RelayEncodedCallData } }`

        **Note:** you need to provide the encoded call data obtained before

        4. `RefundSurplus`
        5. `DepositAsset: { assets: Wild { Wild: All }, maxAssets: 1, beneficiary: { parents: 0, interior: X1 { X1: AccountId32 { network: Any, id: SovereignAccountonRelay } } } }`
    
    **Note:** The values used above are for reference to be used in this testing environment, do not use these values in production!

    **Note:** Verify you have enough funds to execute the `WithdrawAsset` instruction **and** to reserve the required amount encoded in the HRMP calls.

    **Note:** Ensure the `Transact`'s `requireWeightAtMost` is appropriate for the encoded call, especially if you are batching multiple HRMP extrinsics.
    
- Once this message is sent, the relay chain should execute the content and the request to open the channel with Dolphin
- **Please let us know once you’ve requested opening the channel because the request needs to be accepted by Dolphin.**

Here's an example of the fully formed extrinsic:

![https://i.imgur.com/GUN8qJd.png](https://i.imgur.com/GUN8qJd.png)

![https://i.imgur.com/3ONY21d.png](https://i.imgur.com/3ONY21d.png)

## Accepting HRMP Channel from Dolphin

- Channels are one way. This means that if you open a channel with Dolphin, it will allow you only to send tokens from your parachain to Dolphin. There needs to be a channel that Dolphin will request to send back tokens, and you need to accept.
- The process of accepting the channel is similar to the one for opening, meaning that you have to construct an encoded call data in the relay chain, and then get it executed via an XCM from your parachain.

### Get the Relay Encoded Call Data to Accept HRMP Channel

- To get an encoded call data from the relay chain, to accept a channel request with a target parachain, take the following steps:
- In PolkadotJS app, switch to the live Polkadot/Kusama network. Go to Developer -> [Javascript section](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/js). Then run the following code, and replace the demo recipient para id with your own:
```
const tx = api.tx.hrmp.hrmpAcceptOpenChannel(2084);
console.log(tx.toHex());
```

- The result will be like:

`0x1c04170124080000`, remove the leading hex `1c04`, and the final encoded result is:
```
0x170124080000
```

### Send XCM to Relay Chain

- The steps are the same as before (when making the request to open a channel). The main difference is in the `Transact` item, where you need to provide the encoded call data calculated above. This XCM message needs to be sent from the root account (either SUDO or via governance):
```
Transact { originType: Native, requireWeightAtMost: 1000000000, call: XcmDoubleEncoded: { encoded: RelayEncodedCallData } }
```
    
**Note**: The values used above are for reference to be used in this testing environment, do not use these values in production!
    
## Assets Registrations

### Registering your Asset on Dolphin

- Once the channel is opened, we need to register the asset that will be transferred to Dolphin. For that, we need the following information:
    1. `MultiLocation` of your asset (as seen by Dolphin). Please indicate parachain ID and the interior (if you use any pallet index, general index, etc)
    2. `Asset Name`
    3. `Asset symbol`
    4. `Number of decimals`
	5. `Min balance`
	6. `Self sufficient`
- Please write this information as a comment in this issue and we will confirm once the asset is registered.
- After the asset is successfully registered, you can try transferring tokens from your parachain to Dolphin.
- For testing, please also provide your Parachain WS Endpoint so we can connect to it. Lastly, we would need some funds to the following account:
    
    `5CacAW3K4gq3Ufv2dAqUFYWKoqJcQaFu346ahesmt4sua7Xx`
    
- If you need DOL tokens (the native token for Dolphin) to use your parachain's asset, you can get some from our Discord Bot - We can also provide you with some if you give us your address

- We will also set an arbitrary `UnitsPerSecond` value, which is the number of tokens charged per second of execution of the XCM message. This can be arbitrary on the testnet, but on Calamari we're targeting a $0.1 cost for transfers.

### Registering Calamari’s Token on your Parachain

- To register our DOL token on your parachain, you can use the following MultiLocation:

`{ "parents": 1, "interior": {"X1": { "Parachain": 2084 }}`

- And the following metadata:

```
Name: Dolphin
Symbol: DOL
Decimals: 18
Min Balance: 1
Self sufficient: true
```

- Note: Calamari MultiLocation and metadata are different!

## Complete cross chain transfer tests on Dolphin

* The following items must have been completed and fully tested in the Rococo Ecosystem with Dolphin before proceeding with an XCM integration on Calamari (and Manta in the future):
    1. Bi-directional HRMP channels between Dolphin and your parachain
    2. Bi-directional asset registration (DOL token and the token of your parachain)
    3. Both teams must have successfully tested asset transfers through Polkadot.js Apps

* We would advise to test at least the following scenarios:
	1. Transferring a parachain's native token to another parachain and back.
	2. Transferring a non-native token from one parachain to another and back.

## Next Steps - Calamari & Manta

Once everything is successful we can plan for:
* Cross marketing initiatives between our teams.
* Product integrations if relevant.
* Governance proposals to open HRMP channels and register assets.

### Example Polkadot Launch Config

```
{
	"relaychain": {
		"bin": "./polkadot",
		"chain": "rococo-local",
		"nodes": [
			{
				"name": "alice",
				"wsPort": 9944,
				"port": 30444,
				"flags": [
					"--rpc-cors=all",
					"--execution=wasm",
					"--wasm-execution=compiled",
				]
			},
			{
				"name": "bob",
				"wsPort": 9955,
				"port": 30555,
				"flags": [
					"--rpc-cors=all",
					"--execution=wasm",
					"--wasm-execution=compiled",
				]
			},
			{
				"name": "charlie",
				"wsPort": 9966,
				"port": 30666,
				"flags": [
					"--rpc-cors=all",
					"--execution=wasm",
					"--wasm-execution=compiled",
				]
			}
		],
		"genesis": {
			"runtime": {
				"runtime_genesis_config": {
					"configuration": {
						"config": {
							"validation_upgrade_frequency": 10,
							"validation_upgrade_delay": 10
						}
					}
				}
			}
		}
	},
	"parachains": [
		{
			"bin": "./manta",
			"chain": "calamari-dev",
			"nodes": [
				{
					"wsPort": 9801,
					"port": 31201,
					"name": "alice",
					"flags": [
						"--rpc-cors=all",
						"--rpc-port=9971",
						"--execution=wasm",
						"--wasm-execution=compiled"
					]
				}
			]
		}
	],
	"hrmpChannels": [
	],
	"types": {},
	"finalization": false
}
```