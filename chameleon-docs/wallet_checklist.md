### **1\. Wallet Capabilities**

* Private wallet generation (master seed \+ mnemonic backup).

* Multiple accounts/addresses via keychain.

* Stealth addresses / one-time receive addresses.

* Local-only private key storage, encrypted.

* Shielded token transfers (send/receive with ring signatures \+ confidential amounts).

* Transaction history (privacy-friendly: masked amounts/addresses, optional reveal).

* Fee management (CHML gas fees, shield/unshield fees shown transparently).

* QR code scanning & sharing.

* Local address book (stored on device, not synced).

### **2\. Shielding & Unshielding (Cross-Chain Bridges)**

* Shielding flow:

  * Deposit external assets (BTC, ETH/ERC20, BSC, Solana).

  * Detect confirmations → mint private tokens (pBTC, pETH, etc.).

* Unshielding flow:

  * Burn pTokens in Chameleon → release public assets to external wallet.

* Progress tracking (e.g., “0/6 BTC confirmations,” “Minting complete”).

* Transparent fees: \~0.01% protocol fee \+ network-specific fees.

* Custodian-based model for BTC/Non-EVM, trustless vaults for EVM chains.

* Error handling (expired deposits, rejected transactions).

### **3\. Validator Staking & Node Management**

* **vNodes (virtual validators):**

  * Stake 1,750 CHML per validator.

  * Node linking via Validator Key \+ IP.

  * Node lifecycle: Waiting → Active → Earning → Unstaking → Unstaked.

  * Unstake with cooldown (\~7 days).

  * Claimable CHML rewards (not auto-distributed).

* **Monitoring:**

  * Node status (online/offline, synced, earning).

  * Committee participation.

  * Uptime % and downtime history.

  * Blocks validated per epoch.

  * Alerts (offline, missed blocks, desync).

* **pNodes (hardware validators):**

  * Plug-and-play device support.

  * Local discovery & pairing with mobile app.

  * Same staking/earning flow as vNodes.

### **4\. pDEX (Private Decentralized Exchange)**

* In-app privacy-preserving DEX.

* Swap UI (like Uniswap): Token In → Token Out.

* Real-time rate/slippage/fee display.

* Execution via shielded contracts (ring sigs, stealth addresses, confidential amounts).

* Liquidity pool exploration:

  * Show TVL, 24h volume, APY, price.

  * Sort/filter pools by asset or APY.

* User liquidity positions:

  * Pool share %, tokens contributed, accrued earnings.

  * Add/remove liquidity with warnings (impermanent loss, ratio requirements).

* Withdrawals return shielded tokens \+ earned fees.

* Privacy note: All swaps/liquidity ops must be shielded at the DEX level, not just bridge level.

### **5\. Liquidity Mining**

* Provide liquidity to pairs (e.g., CHML/pBTC, CHML/pETH).

* LP tokens minted on deposit, burned on withdrawal.

* Earnings:

  * Trading fees (in-kind, e.g., pBTC \+ CHML).

  * CHML liquidity mining rewards.

* Rewards dashboard:

  * Pending CHML claimable manually.

  * Breakdown: trading fees vs CHML rewards.

  * Pool-level stats: TVL, 24h volume, APY.

  * Impermanent loss indicator.

* **Future Extension:** Single-sided liquidity (deposit only pAsset, paired with CHML reserves).

### **6\. Incentive Model**

* **Validator Rewards:**

  * Block rewards in CHML, claimable.

  * Show pending rewards \+ APY.

* **Liquidity Mining Rewards:**

  * Claimable CHML incentives.

  * Show per-pool rewards & APY.

* **Custodian Fees:**

  * Custodians earn \~0.01% fee on BTC/non-EVM shielding/unshielding.

  * Transparent breakdown to users.

* **Rewards Dashboard:**

  * Aggregate staking, liquidity, and custodian rewards (if applicable).

  * Show totals, pending rewards, historical earnings.

### **7\. Sanction Prevention**

* Block sanctioned wallets from both shielding **and** unshielding.

* Integrate sanction lists (OFAC, UN, EU, etc.) → auto-refresh daily.

* Custodians enforce compliance for BTC/non-EVM.

* Clear notifications if a transaction is blocked.

### **8\. Testnet & Mainnet Support**

* Separate **Testnet (CHMLTest)** and **Mainnet (CHML).**

* Easy switch inside app (settings).

* Faucet for CHMLTest tokens.

* Clear visual indicators (themes, labels) to avoid confusion.

* All major features must work on Testnet first before Mainnet.

### **9\. Security & Privacy Features**

* Always-on privacy for all transactions.

* Ring signatures, stealth addresses, confidential transactions.

* No transparent transfers inside Chameleon (only on unshield).

* Local key storage (encrypted, never leaves device).

* Device security: PIN, biometrics, auto-lock, screenshot blocking.

* Secure comms: all app-node communication encrypted.

* View-only key export (read-only, no spending rights).

* Mandatory security audits before mainnet launch.

