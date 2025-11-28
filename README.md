# Chameleon Network

**Privacy-first blockchain with MEV protection for retail users**

Chameleon is a next-generation privacy blockchain built on Substrate, designed to protect retail users from MEV (Miner Extractable Value) exploitation while delivering a mobile-first user experience.

## 🎯 Key Features

- 🔒 **Privacy-Preserving Transactions** - zkSNARK-based shielded transactions
- 🛡️ **MEV Protection** - Encrypted mempool prevents front-running and sandwich attacks
- 📱 **Mobile-First Experience** - Native iOS and Android wallet for seamless UX
- 💱 **Privacy DEX (pDEX)** - Trade with full privacy guarantees
- 🌉 **Cross-Chain Bridges** - Connect to Ethereum, Bitcoin, Solana, and more
- 🏛️ **Community Governance** - Token holders control protocol upgrades

## 🔧 Built On

Chameleon is built on the [Manta Network](https://github.com/Manta-Network/Manta) codebase, inheriting battle-tested privacy primitives and Substrate infrastructure. We extend Manta's foundation with:

- MEV-resistant transaction ordering
- Retail-focused tokenomics (100M CHML fixed supply)
- Mobile-optimized RPC endpoints and wallet experience
- Community-first governance from day one

**We are deeply grateful to the Manta Network team for their pioneering work in privacy-preserving blockchain technology.**

## 🌐 Network Information

- **Token:** CHML (Chameleon)
- **Total Supply:** 100,000,000 CHML (fixed, non-inflationary)
- **Consensus:** Proof-of-Stake (PoS) with 20-year declining emission
- **Block Time:** ~6 seconds
- **Minimum Validator Stake:** 1,750 CHML

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (stable)
- Node.js 18+ (for wallet development)
- Substrate development environment

### Building from Source
```bash
# Clone the repository
git clone https://github.com/chmldev/chameleon-network.git
cd chameleon-network

# Install Rust and dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown

# Build the node (first build takes 30-60 minutes)
cargo build --release

# Run local development node
./target/release/manta --dev --tmp
```

### Running a Validator

Documentation for running a Chameleon validator will be published during testnet launch. Stay tuned!

## 📱 Mobile Wallet

The Chameleon mobile wallet is under active development. Beta testing will begin in Week 13 of our development roadmap.

- **iOS:** Coming soon
- **Android:** Coming soon

## 🔗 Testnet & Mainnet

- **Testnet Launch:** Estimated Week 15 (Q1 2026)
- **Mainnet Launch:** TBD (dependent on testnet results and audits)

## 📚 Documentation

- **Tokenomics:** [View Tokenomics](docs/tokenomics.md) *(coming soon)*
- **Whitepaper:** [Read Whitepaper](docs/whitepaper.md) *(coming soon)*
- **Developer Docs:** [API Documentation](docs/developers.md) *(coming soon)*
- **Validator Guide:** [Run a Validator](docs/validators.md) *(coming soon)*

## 🤝 Community

- **Discord:** [Join our Discord](#) *(link coming soon)*
- **Telegram:** [Join Telegram](#) *(link coming soon)*
- **Twitter:** [@ChameleonChain](#) *(link coming soon)*
- **Website:** [chameleonnetwork.io](#) *(coming soon)*

## 🛣️ Roadmap

**Phase 1: Foundation (Weeks 1-6)**
- ✅ Repository setup and fork
- ⏳ Token implementation (CHML)
- ⏳ Emission schedule logic
- ⏳ Staking mechanism
- ⏳ Privacy primitives integration

**Phase 2: Core Features (Weeks 7-10)**
- ⏳ Mobile wallet MVP
- ⏳ pDEX integration
- ⏳ MEV protection implementation
- ⏳ Ethereum bridge

**Phase 3: Testnet (Weeks 11-14)**
- ⏳ Security audits
- ⏳ Testnet infrastructure deployment
- ⏳ Mobile wallet beta program
- ⏳ Public testnet launch

**Phase 4: Mainnet (Week 19+)**
- ⏳ Presale
- ⏳ Token Generation Event (TGE)
- ⏳ Mainnet launch

## 🔐 Security

Security is our top priority. We have:
- Internal security audits (ongoing)
- External audits planned before mainnet (CertiK, Quantstamp, Trail of Bits)
- Bug bounty program (500K CHML pool)

**Found a security issue?** Please email security@chml.network (once set up) or open a confidential issue.

## 🧪 Testing
```bash
# Run all tests
cargo test --all

# Run specific test suite
cargo test -p manta-runtime

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

## 📜 License

This project is licensed under **GPL-3.0** (same as Manta Network).

By contributing to this project, you agree to license your contributions under the same license.

## 🙏 Acknowledgments

Chameleon Network is built on the shoulders of giants:

- **Manta Network** - For their groundbreaking work on privacy infrastructure
- **Substrate/Polkadot** - For the robust blockchain framework
- **Zcash** - For pioneering zkSNARK technology
- **Ethereum** - For demonstrating the power of decentralized applications

## 📞 Contact

- **General Inquiries:** hello@chameleonnetwork.io *(coming soon)*
- **Development:** dev@chameleonnetwork.io *(coming soon)*
- **Partnerships:** partnerships@chameleonnetwork.io *(coming soon)*

---

**⚠️ Note:** Chameleon Network is in active development. Testnet is expected in Q1 2026. Use at your own risk. This is not financial advice.

**Built with 💜 for privacy and freedom**
