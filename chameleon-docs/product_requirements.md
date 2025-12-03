# CHAMELEON NETWORK: PRODUCT REQUIREMENTS DOCUMENT
## For Agentic Development Platform (Emergent)

**Version:** 1.0  
**Date:** October 25, 2025  
**Target:** Testnet Launch (Week 15 - February 2026)  
**Development Approach:** AI Agent Orchestration with Parallel Execution

---

## 🎯 EXECUTIVE SUMMARY

**Project:** Chameleon Network - Privacy-first blockchain with MEV protection  
**Codebase Foundation:** Fork of Manta Network (Substrate-based)  
**Key Differentiators:** MEV protection, mobile-first UX, community governance  
**Total Supply:** 100,000,000 CHML (fixed)  
**Timeline:** 16 weeks to public testnet  
**Development Method:** 6 parallel AI agents working on feature branches

---

## 📋 PROJECT OVERVIEW

### Mission
Build a privacy-preserving blockchain that protects retail users from MEV exploitation while delivering a mobile-first user experience.

### Core Value Propositions
1. **Privacy by Default:** zkSNARK-based shielded transactions
2. **MEV Protection:** Encrypted mempool prevents front-running
3. **Mobile-First:** Native iOS/Android wallet, not web3 browser dependency
4. **Fair DeFi:** Privacy DEX without exploitative MEV
5. **Community-Owned:** Token holders control protocol from day one

### Technical Foundation
- **Framework:** Substrate 3.0+
- **Consensus:** Nominated Proof-of-Stake (NPoS)
- **Privacy:** zkSNARKs (inherited from Manta)
- **Language:** Rust (core), TypeScript (frontend), React Native (mobile)
- **Repository:** https://github.com/chmldev/chameleon-network

---

## 🏗️ SYSTEM ARCHITECTURE

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                    CHAMELEON NETWORK                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │   Mobile       │  │   Privacy      │  │    MEV         │ │
│  │   Wallet       │  │   DEX (pDEX)   │  │  Protection    │ │
│  │  (iOS/Android) │  │                │  │                │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │
│                                                               │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │  Cross-Chain   │  │   Validator    │  │   Governance   │ │
│  │   Bridges      │  │    Network     │  │     (DAO)      │ │
│  │  (ETH/BTC/SOL) │  │                │  │                │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│              SUBSTRATE RUNTIME (Rust)                        │
│  • Token Logic (CHML)      • Staking & Rewards              │
│  • Privacy Primitives      • Emission Schedule              │
│  • MEV-Resistant Mempool   • Consensus (NPoS)              │
├─────────────────────────────────────────────────────────────┤
│              SUBSTRATE NODE (P2P Network)                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 💰 TOKENOMICS REQUIREMENTS

### Token Specifications
- **Symbol:** CHML
- **Name:** Chameleon Network Token
- **Total Supply:** 100,000,000 CHML (fixed, non-inflationary)
- **Decimals:** 18
- **Type:** Native blockchain token (not ERC-20)

### Allocation Breakdown

| Allocation | Amount (CHML) | Percentage | Status |
|-----------|---------------|------------|--------|
| Validator & LP Rewards | 65,000,000 | 65.0% | Emitted over 20 years |
| Public Presale | 15,000,000 | 15.0% | Vested (50% TGE, 50% 6mo) |
| Community Airdrop | 5,000,000 | 5.0% | Staged distribution |
| Staking Infrastructure | 5,000,000 | 5.0% | Locked in genesis validators |
| Ecosystem Development | 5,000,000 | 5.0% | 36mo vesting, 6mo cliff |
| Initial DEX Liquidity | 2,500,000 | 2.5% | Deployed at TGE |
| Treasury Reserve | 2,500,000 | 2.5% | DAO-controlled |

### Emission Schedule (20 Years, Declining)

**Formula:** 10% year-over-year reduction

| Year | % of Pool | Total Rewards | Validator (70%) | LP (30%) |
|------|-----------|---------------|----------------|-----------|
| 1 | 11.38% | 7,400,000 | 5,180,000 | 2,220,000 |
| 2 | 10.24% | 6,660,000 | 4,662,000 | 1,998,000 |
| 3 | 9.22% | 6,000,000 | 4,200,000 | 1,800,000 |
| 4 | 8.29% | 5,390,000 | 3,773,000 | 1,617,000 |
| 5 | 7.46% | 4,840,000 | 3,388,000 | 1,452,000 |
| ... | ... | ... | ... | ... |
| 20 | 1.53% | 1,000,000 | 700,000 | 300,000 |

**Implementation Notes:**
- Block rewards decrease by 10% each year
- Calculated at genesis block, hardcoded in runtime
- Validator rewards distributed proportionally to stake
- LP rewards distributed via dynamic yield optimization

### Staking Requirements
- **Minimum Validator Stake:** 1,750 CHML
- **Unbonding Period:** 14 days
- **Delegation:** Supported (delegators earn proportional rewards)
- **Slashing Conditions:**
  - Extended downtime (>12 hours): 0.1% stake penalty
  - Double-signing: 5% stake penalty
  - Consistent poor performance: Removal from active set

---

## 🔒 PRIVACY REQUIREMENTS

### zkSNARK Integration (From Manta)
**Objective:** Inherit Manta's battle-tested privacy primitives

**Requirements:**
1. **Shielded Transactions:**
   - Users can shield public CHML into private balance
   - Private transfers hide sender, receiver, and amount
   - Unshielding converts private CHML back to public
   
2. **zkSNARK Proof System:**
   - Use Groth16 or PLONK (same as Manta)
   - Proof generation: <10 seconds on consumer hardware
   - Verification: <1 second on-chain
   
3. **Privacy Guarantees:**
   - Transaction graph analysis resistant
   - No correlation between shielded and unshielded identities
   - Forward and backward privacy (past transactions stay private)

**Technical Specifications:**
- **Trusted Setup:** Use Manta's existing ceremony results (or conduct new one)
- **Proving Keys:** Pre-generated and distributed with node software
- **Circuits:** Reuse Manta's circuits for compatibility, optimize later

---

## 🛡️ MEV PROTECTION REQUIREMENTS

### Problem Statement
Retail users lose $300M-$900M annually to MEV extraction (front-running, sandwich attacks, etc.) on public blockchains. Chameleon must prevent this.

### Solution: Encrypted Mempool

**Core Mechanism:**
1. **Encrypted Transaction Pool:**
   - Transactions encrypted when submitted to mempool
   - Only decrypted when included in block
   - Prevents validators from seeing pending transactions
   
2. **Fair Ordering:**
   - Transactions ordered by timestamp (first-come-first-served)
   - No priority gas fees (all fees uniform)
   - Validators cannot reorder transactions for profit

**Implementation Requirements:**
1. **Threshold Encryption:**
   - Use threshold cryptography (t-of-n scheme)
   - Validators collectively decrypt after ordering
   - No single validator can decrypt alone
   
2. **Commit-Reveal Scheme:**
   - Block N: Validators commit to transaction order (encrypted)
   - Block N+1: Reveal decryption keys, execute transactions
   - 2-block finality delay (acceptable tradeoff)
   
3. **Mempool Privacy:**
   - P2P layer encrypts transaction gossip
   - Only destination node (validator) can decrypt
   - Prevents snooping by non-validator nodes

**Performance Considerations:**
- Encryption overhead: <50ms per transaction
- Decryption at block time: <200ms total
- Does not impact throughput (100+ TPS target)

---

## 📱 MOBILE WALLET REQUIREMENTS

### Objective
Build a mobile-first wallet that doesn't require users to understand Web3 complexity.

### Platforms
- **iOS:** 15.0+ (SwiftUI for native components if needed)
- **Android:** 11+ (Kotlin for native modules if needed)
- **Framework:** React Native (cross-platform codebase)

### Core Features (MVP)

**1. Wallet Creation & Management**
- Generate 12/24-word seed phrase (BIP39 compatible)
- Secure seed storage (iOS Keychain / Android Keystore)
- Biometric authentication (Face ID / Touch ID / Fingerprint)
- PIN code backup authentication
- Multiple accounts from one seed (HD derivation)

**2. Balance Display**
- Public CHML balance (transparent)
- Private CHML balance (shielded)
- USD equivalent (live price feed)
- Transaction history (both public and private)

**3. Send & Receive**
- Send public CHML (normal transaction)
- Send private CHML (shielded transaction)
- Receive via QR code
- Address book (save frequent contacts)
- Transaction memo field (optional)

**4. Shield & Unshield**
- One-tap shield (public → private)
- One-tap unshield (private → public)
- Show current shielded balance
- Privacy toggle (default to private)

**5. Staking**
- View available validators
- Delegate CHML to validator
- View staking rewards (accumulated)
- Unbond (with 14-day countdown timer)

**6. pDEX Integration**
- Swap tokens within wallet
- View liquidity pools
- Add/remove liquidity
- View LP rewards

### UX Principles
- **No jargon:** Don't say "zkSNARK" or "mempool" - say "Private Send"
- **Sensible defaults:** Private by default, public opt-in
- **Fast:** <3 second load times, <5 second transaction confirmations
- **Beautiful:** Modern UI, smooth animations, delightful micro-interactions
- **Onboarding:** In-app tutorials, educational tooltips

### Technical Requirements
- **RPC Connection:** Custom endpoints optimized for mobile (batch requests)
- **Local Storage:** Encrypted SQLite for transaction cache
- **Push Notifications:** Transaction confirmations, staking rewards
- **Offline Mode:** Show cached balances when offline
- **Auto-Updates:** Over-the-air updates (CodePush for React Native)

---

## 💱 PRIVACY DEX (pDEX) REQUIREMENTS

### Objective
Enable private token swaps without revealing trade details to validators or chain observers.

### Core Features

**1. Liquidity Pools**
- Automated Market Maker (AMM) model (Uniswap v2 style)
- Constant product formula: x × y = k
- Support for CHML/ETH, CHML/USDC, CHML/WBTC pairs
- Dynamic fee structure (0.3% default, adjustable by governance)

**2. Private Swaps**
- Swap amounts hidden via zkSNARKs
- Input/output token types visible (e.g., CHML → ETH)
- Slippage protection (user-defined max slippage %)
- Price impact warning for large trades

**3. Liquidity Provision**
- Add liquidity to pools (deposit both tokens)
- LP tokens minted to represent share of pool
- Remove liquidity (burn LP tokens, receive underlying assets)
- View LP position value and impermanent loss

**4. Rewards Distribution**
- 30% of validator rewards go to LP providers (19.5M CHML over 20 years)
- Dynamic yield optimization (pools with higher volume get higher rewards)
- Rewards claimable anytime
- 50% instant, 50% vested over 90 days (prevent mercenary capital)

### Technical Specifications
- **Smart Contract:** Substrate pallet (not EVM)
- **Privacy Layer:** zkSNARK proofs for swap amounts
- **Oracle:** Chainlink-style price feeds for USD values (display only)
- **Front-Running Protection:** Integrated with MEV protection mechanism

---

## 🌉 CROSS-CHAIN BRIDGE REQUIREMENTS

### Objective
Enable users to move assets between Chameleon and other blockchains securely.

### Phase 1: Ethereum Bridge

**Supported Assets:**
- ETH (wrapped as wETH on Chameleon)
- USDC (wrapped as cUSDC on Chameleon)
- USDT (wrapped as cUSDT on Chameleon)
- WBTC (wrapped as cWBTC on Chameleon)

**Bridge Mechanism:**
1. **Lock & Mint:**
   - User locks ETH on Ethereum smart contract
   - Validators verify lock transaction (majority consensus)
   - Equivalent wETH minted on Chameleon
   
2. **Burn & Unlock:**
   - User burns wETH on Chameleon
   - Validators sign unlock transaction
   - ETH released on Ethereum to user's address

**Security Requirements:**
- **Multi-Sig Contract:** Ethereum side uses 5-of-9 validator multi-sig
- **Validator Rotation:** Multi-sig keys rotate every 6 months
- **Fraud Proofs:** Challenge period for withdrawals (6 hours)
- **Insurance Fund:** 5% of bridged value held in reserve

### Phase 2: Additional Bridges (Post-Testnet)
- Bitcoin (via threshold signatures)
- Solana (via Wormhole integration)
- Binance Smart Chain
- Avalanche

---

## 🏛️ GOVERNANCE REQUIREMENTS

### On-Chain Governance (DAO)

**Voting Power:**
- 1 CHML = 1 vote
- Staked CHML = 2x voting power (incentivize long-term holders)
- Delegated voting supported

**Proposal Types:**
1. **Runtime Upgrades:** Change blockchain logic (requires 60% approval)
2. **Parameter Changes:** Adjust fees, staking requirements, etc. (requires 50% approval)
3. **Treasury Spending:** Allocate treasury funds (requires 50% approval)
4. **Emergency Actions:** Pause protocol, fix critical bugs (requires 75% approval)

**Proposal Process:**
1. **Submission:** Any user with 10,000 CHML can submit proposal
2. **Discussion Period:** 7 days (off-chain: Discord/Forum)
3. **Voting Period:** 7 days (on-chain)
4. **Execution:** Automatic if passed, 7-day timelock before execution
5. **Veto:** Core team can veto in first 6 months only (decentralizes after)

---

## 🔧 TECHNICAL SPECIFICATIONS

### Blockchain Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Block Time | 6 seconds | Balance between speed and finality |
| Finality | 12 seconds (2 blocks) | Fast enough for good UX |
| Max Block Size | 5 MB | Supports 100+ TPS |
| Max Validators | Unlimited (initially) | Encourage decentralization |
| Validator Election | NPoS (Nominated Proof-of-Stake) | Proven by Polkadot |
| Session Length | 4 hours | Validator set updates every 4 hours |

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Transactions Per Second (TPS) | 100+ | Average throughput |
| Transaction Confirmation | <5 seconds | Time to 1 block confirmation |
| Transaction Finality | <15 seconds | Time to irreversible finality |
| Proof Generation (zkSNARK) | <10 seconds | Client-side on modern hardware |
| Proof Verification | <1 second | On-chain verification time |
| Node Sync Time (Full) | <2 hours | From genesis to current block |

### Hardware Requirements

**Validator Node:**
- CPU: 4+ cores (8+ recommended)
- RAM: 16 GB minimum (32 GB recommended)
- Storage: 500 GB SSD (NVMe preferred)
- Network: 100 Mbps+ (1 Gbps recommended)
- Uptime: 99%+ required (slashed below 95%)

**Full Node (Non-Validator):**
- CPU: 2+ cores
- RAM: 8 GB minimum
- Storage: 250 GB SSD
- Network: 50 Mbps+

---

## 📂 FILE STRUCTURE & KEY LOCATIONS

### Repository Organization

```
chameleon-network/
├── node/                       # Node binary and configuration
│   ├── src/
│   │   ├── chain_spec.rs      # Genesis configuration ⭐ CRITICAL
│   │   ├── cli.rs             # Command-line interface
│   │   └── service.rs         # Node service setup
│   └── Cargo.toml
│
├── runtime/                    # Core blockchain logic
│   ├── src/
│   │   ├── lib.rs             # Main runtime config ⭐ CRITICAL
│   │   ├── constants.rs       # Network constants ⭐ TOKEN CONFIG
│   │   ├── weights.rs         # Performance weights
│   │   └── apis.rs            # Runtime APIs
│   └── Cargo.toml
│
├── pallets/                    # Substrate pallets (modules)
│   ├── chameleon-token/       # CHML token logic ⭐ CREATE NEW
│   ├── chameleon-staking/     # Custom staking logic ⭐ MODIFY
│   ├── chameleon-emission/    # Reward emission ⭐ CREATE NEW
│   ├── manta-pay/             # Privacy functionality (keep from Manta)
│   ├── chameleon-mev/         # MEV protection ⭐ CREATE NEW
│   └── chameleon-pdex/        # Privacy DEX ⭐ CREATE NEW
│
├── mobile/                     # Mobile wallet ⭐ CREATE NEW
│   ├── ios/                   # iOS-specific code
│   ├── android/               # Android-specific code
│   ├── src/
│   │   ├── screens/           # UI screens
│   │   ├── components/        # Reusable components
│   │   ├── services/          # RPC, storage, etc.
│   │   └── utils/             # Helper functions
│   └── package.json
│
├── bridges/                    # Cross-chain bridges ⭐ CREATE NEW
│   ├── ethereum/              # ETH bridge contracts
│   └── scripts/               # Deployment scripts
│
└── docs/                       # Documentation
    ├── tokenomics.md
    ├── architecture.md
    └── api/
```

---

## 🎯 DEVELOPMENT MILESTONES (16 WEEKS)

### Phase 1: Foundation (Weeks 1-6)

**Week 1:**
- Repository fork complete ✅
- Token constants implemented (CHML, 100M supply)
- Chain specification configured

**Week 2:**
- Genesis block configuration
- Validator stake requirements (1,750 CHML)

**Week 3:**
- Emission schedule logic (20-year declining)
- Validator reward distribution

**Week 4:**
- Staking mechanism (delegate, unbond)
- Slashing conditions

**Week 5:**
- Local devnet deployment (5 validators)
- Network stability testing

**Week 6:**
- zkSNARK integration (from Manta)
- Privacy transaction testing

**Deliverable:** Functional local devnet with privacy transactions and staking

---

### Phase 2: Core Features (Weeks 7-10)

**Week 7:**
- Mobile wallet MVP (React Native setup)
- Wallet creation and seed management

**Week 8:**
- pDEX liquidity pools
- Basic swap functionality

**Week 9:**
- MEV protection implementation
- Encrypted mempool testing

**Week 10:**
- Ethereum bridge (testnet)
- wETH wrapping/unwrapping

**Deliverable:** Mobile wallet beta + pDEX + ETH bridge on devnet

---

### Phase 3: Testnet Preparation (Weeks 11-14)

**Week 11:**
- Internal security audit
- Vulnerability fixes

**Week 12:**
- Testnet infrastructure (30 genesis validators)
- Block explorer deployment

**Week 13:**
- Mobile wallet beta program (100 users)
- Bug fixes from feedback

**Week 14:**
- Final testnet preparations
- Documentation and guides

**Deliverable:** Ready for public testnet launch

---

### Phase 4: Public Testnet (Weeks 15-18)

**Week 15:**
- Public testnet launch 🚀
- Community onboarding

**Week 16:**
- Stress testing and performance monitoring
- Issue resolution

**Week 17:**
- Bug bounty program launch (500K CHML)
- External security audits begin

**Week 18:**
- Testnet feedback integration
- Mainnet preparation

**Deliverable:** Battle-tested testnet ready for mainnet

---

## ✅ ACCEPTANCE CRITERIA

### Overall Success Metrics

**Phase 1 (Devnet):**
- ✅ 5 validator nodes running for 72+ hours continuously
- ✅ 1,000+ test transactions processed successfully
- ✅ Privacy transactions working (zkSNARK proofs valid)
- ✅ Staking rewards distributed correctly
- ✅ No critical bugs or crashes

**Phase 2 (Features):**
- ✅ Mobile wallet creates/imports seeds successfully
- ✅ Users can send/receive CHML on mobile
- ✅ pDEX swaps execute with <5 second confirmation
- ✅ ETH bridge successfully transfers testnet ETH
- ✅ MEV protection prevents front-running in tests

**Phase 3 (Testnet Prep):**
- ✅ Security audit findings addressed
- ✅ 30 genesis validators operational
- ✅ Block explorer showing real-time data
- ✅ Mobile wallet beta tested by 100+ users
- ✅ All critical/high severity bugs fixed

**Phase 4 (Public Testnet):**
- ✅ 1,000+ unique wallet addresses created
- ✅ 100+ community validators online
- ✅ 10,000+ transactions processed
- ✅ Network uptime >99.5%
- ✅ No critical exploits found

---

## 🚨 CRITICAL CONSTRAINTS

### Non-Negotiable Requirements

1. **GPL-3.0 License Compliance:**
   - Must maintain GPL-3.0 license (same as Manta)
   - Must credit Manta Network in README and docs
   - Cannot relicense to proprietary

2. **Security:**
   - All cryptographic code must use audited libraries
   - No custom crypto implementations without expert review
   - Multi-sig for all privileged operations

3. **Privacy:**
   - zkSNARK proofs must be mathematically sound
   - No metadata leakage (IP addresses, timing, etc.)
   - Forward and backward privacy guaranteed

4. **Performance:**
   - Must achieve 100+ TPS minimum
   - Transaction confirmation <5 seconds
   - Mobile wallet responsive (<3 second load times)

5. **User Experience:**
   - No Web3 jargon in user-facing UI
   - Mobile-first design (not desktop-first adapted)
   - Onboarding flow <2 minutes for new users

---

## 📚 REFERENCE DOCUMENTATION

### Manta Network Resources
- **GitHub:** https://github.com/Manta-Network/Manta
- **Docs:** https://docs.manta.network
- **Whitepaper:** https://www.manta.network/resources

### Substrate Resources
- **Documentation:** https://docs.substrate.io
- **Rust Docs:** https://paritytech.github.io/substrate/master/
- **Polkadot Wiki:** https://wiki.polkadot.network

### Privacy & Cryptography
- **zkSNARKs:** https://z.cash/technology/zksnarks/
- **Groth16:** https://eprint.iacr.org/2016/260.pdf
- **Threshold Encryption:** https://en.wikipedia.org/wiki/Threshold_cryptosystem

---

## 🎯 SUCCESS DEFINITION

**Chameleon is successful if:**

1. **Testnet Launch (Week 15):**
   - 1,000+ community members actively testing
   - 100+ validators online
   - Zero critical security vulnerabilities

2. **User Experience:**
   - Non-technical users can create wallet and transact in <5 minutes
   - 80%+ beta testers rate wallet 4+ stars
   - Privacy features work seamlessly (no failed zkSNARK proofs)

3. **Performance:**
   - Consistent 100+ TPS
   - 99.5%+ network uptime
   - <5 second transaction confirmations

4. **Community:**
   - Active Discord/Telegram (1,000+ members)
   - 50+ GitHub contributors
   - Positive sentiment in crypto community

**If all above achieved → Proceed to Presale & Mainnet**

---

## 📝 NOTES FOR AGENT EXECUTION

### Code Quality Standards
- **Rust:** Follow Substrate best practices, use `cargo clippy`
- **TypeScript:** Use strict mode, ESLint, Prettier
- **Testing:** Minimum 80% code coverage
- **Documentation:** Every public function must have docs

### Git Workflow
- **Commit Messages:** Use conventional commits (feat:, fix:, docs:, etc.)
- **PR Description:** Include what, why, how, and testing done
- **Reviews:** All code must pass automated tests before merge

### Communication
- **Blockers:** Report immediately in Discord
- **Progress:** Daily updates on assigned tasks
- **Questions:** Ask in #dev-support channel

---

**This PRD is a living document and will be updated as requirements evolve.**

**Version History:**
- v1.0 (Oct 25, 2025) - Initial comprehensive requirements

**Last Updated:** October 25, 2025