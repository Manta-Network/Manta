# Chameleon Wallet - Mobile App

A beautiful, privacy-first mobile wallet for the Chameleon Network blockchain.

## Features

### 🔐 Privacy First
- **Shielded Transactions**: Zero-knowledge proofs hide transaction details
- **Private by Default**: All transactions are private unless explicitly made public
- **No Metadata Leakage**: IP addresses and timing information are protected

### ⚡ Lightning Fast
- **6-Second Block Times**: Faster than most blockchains
- **Instant Confirmations**: See transactions immediately
- **Optimized Mobile Performance**: <3 second load times

### 🛡️ MEV Protected
- **Encrypted Mempool**: No front-running or sandwich attacks
- **Fair Ordering**: First-come, first-served transaction processing
- **Retail User Protection**: No exploitation by MEV bots

### 📱 Mobile-First Design
- **Intuitive UX**: No Web3 jargon, designed for everyone
- **Biometric Security**: Face ID, Touch ID, and fingerprint support
- **Offline Capability**: View balances and history without internet

## Tech Stack

- **Framework**: React Native 0.73+
- **Language**: TypeScript (strict mode)
- **Navigation**: React Navigation 6
- **State Management**: Zustand + React Context
- **Blockchain**: Polkadot.js API
- **Security**: React Native Keychain
- **Storage**: Encrypted AsyncStorage

## Project Structure

```
src/
├── components/          # Reusable UI components
│   ├── Button.tsx
│   ├── Card.tsx
│   ├── Balance.tsx
│   └── TransactionItem.tsx
├── screens/             # App screens
│   ├── WelcomeScreen.tsx
│   ├── CreateWalletScreen.tsx
│   ├── ImportWalletScreen.tsx
│   ├── HomeScreen.tsx
│   └── PlaceholderScreen.tsx
├── navigation/          # Navigation setup
│   └── index.tsx
├── services/            # Business logic
│   ├── wallet.ts        # Wallet management
│   ├── rpc.ts          # Blockchain communication
│   └── storage.ts      # Secure storage
├── hooks/              # Custom React hooks
│   ├── useWallet.ts
│   ├── useTransactions.ts
│   └── useStaking.ts
├── context/            # React Context providers
│   └── WalletContext.tsx
├── utils/              # Utility functions
│   ├── crypto.ts       # Cryptographic operations
│   └── format.ts       # Number/text formatting
├── types/              # TypeScript definitions
│   └── index.ts
├── constants/          # App constants
│   └── index.ts
├── theme/              # Design system
│   └── index.ts
└── App.tsx             # Root component
```

## Getting Started

### Prerequisites

- Node.js 18+
- React Native CLI
- iOS: Xcode 14+
- Android: Android Studio with SDK 31+

### Installation

1. **Install dependencies**:
   ```bash
   cd mobile
   yarn install
   ```

2. **iOS setup**:
   ```bash
   cd ios && pod install && cd ..
   ```

3. **Start Metro bundler**:
   ```bash
   yarn start
   ```

4. **Run on device**:
   ```bash
   # iOS
   yarn ios
   
   # Android
   yarn android
   ```

### Development

- **Linting**: `yarn lint`
- **Type checking**: `yarn tsc`
- **Testing**: `yarn test`
- **Clean build**: `yarn clean`

## Key Components

### Wallet Management
- **Seed Generation**: BIP39 12/24-word phrases
- **HD Wallets**: Hierarchical deterministic key derivation
- **Secure Storage**: iOS Keychain / Android Keystore
- **Biometric Auth**: Face ID, Touch ID, Fingerprint

### Privacy Features
- **Shield/Unshield**: Convert between public and private balances
- **Private Transfers**: zkSNARK-based hidden transactions
- **Stealth Addresses**: One-time receive addresses
- **Ring Signatures**: Hide transaction origins

### Blockchain Integration
- **Polkadot.js API**: Native Substrate integration
- **Custom RPC**: Chameleon-specific endpoints
- **Real-time Updates**: WebSocket subscriptions
- **Offline Mode**: Cached data when disconnected

## Security

### Key Storage
- **Seed Phrases**: Encrypted in device keychain
- **Private Keys**: Never stored in plain text
- **PIN/Biometric**: Required for sensitive operations
- **Auto-lock**: Configurable timeout periods

### Network Security
- **TLS Encryption**: All network communication encrypted
- **Certificate Pinning**: Prevent man-in-the-middle attacks
- **No Telemetry**: No user data collection
- **Local Processing**: All crypto operations on-device

## User Experience

### Design Principles
1. **No Jargon**: Use plain English, not crypto terms
2. **Sensible Defaults**: Privacy on, secure by default
3. **Fast Performance**: <3 second load times
4. **Beautiful UI**: Modern, delightful interactions
5. **Accessible**: Works for non-technical users

### Onboarding Flow
1. Welcome screen with feature highlights
2. Create new wallet or import existing
3. Secure seed phrase backup
4. PIN/biometric setup
5. Ready to use!

## Testing

### Unit Tests
- Crypto utilities
- Formatting functions
- Validation logic
- Storage operations

### Integration Tests
- Wallet creation flow
- Transaction sending
- Balance updates
- Navigation flows

### E2E Tests (Detox)
- Complete user journeys
- Cross-platform compatibility
- Performance benchmarks
- Security validations

## Deployment

### iOS (TestFlight)
1. Build release version
2. Upload to App Store Connect
3. Submit for TestFlight review
4. Distribute to beta testers

### Android (Play Console)
1. Generate signed APK/AAB
2. Upload to Play Console
3. Internal testing track
4. Gradual rollout to users

## Roadmap

### Phase 1: MVP (Current)
- ✅ Wallet creation and import
- ✅ Balance display
- ✅ Basic navigation
- 🚧 Send/receive transactions
- 🚧 Transaction history

### Phase 2: Privacy Features
- 🔄 Shield/unshield functionality
- 🔄 Private transactions
- 🔄 Stealth addresses
- 🔄 Ring signatures

### Phase 3: Advanced Features
- 📋 Staking interface
- 📋 pDEX integration
- 📋 Cross-chain bridges
- 📋 Governance voting

### Phase 4: Ecosystem
- 📋 DApp browser
- 📋 NFT support
- 📋 Multi-wallet management
- 📋 Hardware wallet support

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open Pull Request

### Code Style
- Use TypeScript strict mode
- Follow React Native best practices
- Write tests for new features
- Update documentation

## License

GPL-3.0 - See [LICENSE](../LICENSE) file for details.

## Support

- **Documentation**: [docs.chameleon.network](https://docs.chameleon.network)
- **Discord**: [discord.gg/chameleon](https://discord.gg/chameleon)
- **Issues**: [GitHub Issues](https://github.com/chmldev/chameleon-network/issues)
- **Email**: support@chameleon.network

---

**Built with ❤️ for the Chameleon Network community**
