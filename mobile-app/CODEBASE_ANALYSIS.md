# Chameleon Mobile App - Existing Codebase Analysis

**Analysis Date:** December 12, 2025  
**Source:** https://github.com/Chameleonnetwork/Chameleon-App  
**Imported to:** /app/mobile-app/

---

## Technology Stack

| Component | Technology | Version |
|-----------|------------|--------|
| **Framework** | React Native (Expo) | SDK 52 |
| **Language** | TypeScript | 5.3.3 |
| **Routing** | Expo Router | 4.0.15 |
| **UI Components** | Gluestack UI | Multiple packages |
| **Styling** | NativeWind (Tailwind CSS) | 4.1.23 |
| **Navigation** | React Navigation Bottom Tabs | 7.2.0 |
| **Animations** | React Native Reanimated | 3.16.1 |
| **Storage** | AsyncStorage | 2.1.0 |

---

## Project Structure

```
/mobile-app/
├── app/                        # Expo Router (file-based routing)
│   ├── (tabs)/                 # Tab navigation screens
│   │   ├── _layout.tsx         # Tab bar configuration
│   │   ├── index.tsx           # Home screen (Welcome)
│   │   ├── roadmap.tsx         # Project roadmap timeline
│   │   ├── connect.tsx         # Social links (Telegram, Discord, etc.)
│   │   ├── updates.tsx         # Presale countdown/announcements
│   │   └── explore.tsx         # Placeholder (opens drawer)
│   ├── _layout.tsx             # Root layout (theme, fonts)
│   └── +not-found.tsx          # 404 page
├── components/
│   ├── DrawerComponent.tsx     # Feature exploration drawer
│   ├── CustomCard.tsx          # Reusable card component
│   ├── ThemedText.tsx          # Themed text component
│   ├── ExternalLink.tsx        # External link handler
│   └── ui/                     # Gluestack UI components (30+ files)
│       ├── button/
│       ├── card/
│       ├── drawer/
│       ├── toast/
│       └── ... (extensive UI library)
├── constants/
│   └── Colors.ts               # Light/dark theme colors
├── hooks/
│   └── useColorScheme.ts       # Theme detection hook
├── assets/
│   ├── images/
│   │   ├── Logo.png            # Chameleon logo (green)
│   │   ├── Logo_black.png      # Black variant
│   │   ├── CHAMELEON.png       # Full wordmark
│   │   ├── icon.png            # App icon
│   │   └── logo.svg            # SVG version
│   └── fonts/
│       └── Poppins/            # Primary font family
├── package.json
├── tailwind.config.js          # NativeWind configuration
├── app.json                    # Expo configuration
└── tsconfig.json
```

---

## Existing Features

### ✅ Implemented (Marketing/Info App)

| Feature | Screen | Status |
|---------|--------|--------|
| Welcome/Landing | `index.tsx` | ✅ Complete |
| Project Roadmap | `roadmap.tsx` | ✅ Complete |
| Social Links | `connect.tsx` | ✅ Complete |
| Presale Countdown | `updates.tsx` | ✅ Complete |
| Feature Preview Drawer | `DrawerComponent` | ✅ Complete |
| Bottom Tab Navigation | `_layout.tsx` | ✅ Complete |
| Dark Theme | Throughout | ✅ Complete |
| Branding/Logo | Assets | ✅ Complete |

### ❌ Not Implemented (Wallet Features)

| Feature | Required For | Status |
|---------|--------------|--------|
| Wallet Creation | Week 6 | ❌ Missing |
| Seed Phrase Management | Week 6 | ❌ Missing |
| Balance Display | Week 6 | ❌ Missing |
| Send/Receive | Week 6 | ❌ Missing |
| Transaction History | Week 7 | ❌ Missing |
| RPC Connection | Week 6 | ❌ Missing |
| Staking Interface | Week 8+ | ❌ Missing |
| pDEX Integration | Week 8+ | ❌ Missing |

---

## Visual Design System

### Colors (Extracted from Code)

| Color | Hex | Usage |
|-------|-----|-------|
| **Background** | `#0C0E12` | Main app background |
| **Card Background** | `#1B1B1B` | Cards, elevated surfaces |
| **Primary Green** | `#18BB59` | Buttons, active states |
| **Accent Green** | `#23DE2B` | Highlights, gradients |
| **Teal** | `#13E1BC` | Gradient stops |
| **Text Primary** | `#FFFFFF` | Main text |
| **Text Secondary** | `#CDCDE0` | Inactive tabs, muted text |
| **Text Muted** | `#777777` | Timeline lines, borders |
| **Error/Beta** | `#FF0000` | Beta badge |

### Gradient (Brand Signature)
```css
linear-gradient(
  #13E1BC → #23DE2B → #1DDF61 → #23DE2B → #13E1BC
)
```
Used for: "Chameleon" text, presale cards, accent elements

### Typography

| Font | Usage |
|------|-------|
| **Poppins** | Primary font (headings, body) |
| **SpaceMono** | Monospace (available but unused) |

### Icon Library
- `@expo/vector-icons` (AntDesign, Feather, FontAwesome5/6, Ionicons, MaterialCommunityIcons)

---

## Blockchain Integration Status

### Current State: ❌ None

The existing app is a **marketing/informational app** with:
- No Polkadot.js or Substrate Connect
- No RPC connection
- No wallet/account functionality
- No transaction signing
- No blockchain queries

### Required for Week 6:

```typescript
// Packages to add:
"@polkadot/api": "latest",
"@polkadot/keyring": "latest",
"@polkadot/util-crypto": "latest",
"expo-secure-store": "latest",  // For seed phrase storage
```

---

## Social/External Links (Configured)

| Platform | URL |
|----------|-----|
| Website | https://chml.network/ |
| Forum | https://forum.chml.network/ |
| Telegram | https://telegram.me/Chameleon_Network |
| Twitter/X | https://x.com/CHMLnetwork |
| Discord | https://discord.gg/gV79PXJQGA |
| Presale | https://tinyurl.com/chmlPresale |

---

## Week 6 Development Plan

Based on existing code, we need to:

### 1. Add Blockchain Dependencies
```bash
# In mobile-app directory
yarn add @polkadot/api @polkadot/keyring @polkadot/util-crypto
yarn add expo-secure-store
```

### 2. Create New Screens

| Screen | Path | Purpose |
|--------|------|--------|
| Wallet | `app/(tabs)/wallet.tsx` | Main wallet dashboard |
| Send | `app/send.tsx` | Send CHML |
| Receive | `app/receive.tsx` | Receive/QR code |
| Settings | `app/settings.tsx` | Wallet settings |

### 3. Create Services

| Service | Path | Purpose |
|---------|------|--------|
| RPC | `services/rpc.ts` | Substrate RPC connection |
| Wallet | `services/wallet.ts` | Key management |
| Storage | `services/storage.ts` | Secure storage wrapper |

### 4. Modify Tab Navigation

Update `app/(tabs)/_layout.tsx` to add Wallet tab (or replace existing tab)

---

## Files to Modify

| File | Changes Needed |
|------|---------------|
| `app/(tabs)/_layout.tsx` | Add Wallet tab, reorder tabs |
| `app/(tabs)/index.tsx` | Optional: Add wallet summary card |
| `package.json` | Add Polkadot.js dependencies |
| `constants/Colors.ts` | Add wallet-specific colors if needed |

## Files to Create

| File | Purpose |
|------|--------|
| `app/(tabs)/wallet.tsx` | Main wallet screen |
| `app/send.tsx` | Send transaction screen |
| `app/receive.tsx` | Receive/QR screen |
| `services/rpc.ts` | RPC connection to `http://64.23.233.36:9933` |
| `services/wallet.ts` | Wallet/keyring management |
| `services/storage.ts` | Secure storage for seed phrases |
| `hooks/useBalance.ts` | Balance subscription hook |
| `hooks/useApi.ts` | Polkadot API hook |

---

## RPC Configuration

**Target Endpoint:**
```typescript
const RPC_ENDPOINT = 'http://64.23.233.36:9933';  // HTTP
const WS_ENDPOINT = 'ws://64.23.233.36:9944';     // WebSocket (preferred)
```

---

## Build Commands

```bash
# Development
npx expo start

# iOS
npx expo start --ios

# Android  
npx expo start --android

# Build for production (EAS)
eas build --platform ios
eas build --platform android
```

---

## Summary

The existing mobile app is a **well-designed marketing/information app** with:

✅ **Strengths:**
- Professional UI with Chameleon branding
- Modern tech stack (Expo SDK 52, TypeScript, NativeWind)
- Extensive UI component library (Gluestack)
- Clean project structure
- Dark theme throughout
- Social links and roadmap content

❌ **Gaps for Wallet Functionality:**
- No blockchain integration whatsoever
- No wallet/account screens
- No RPC connection
- No transaction capability

**Week 6 Effort Estimate:** Medium  
The UI foundation is solid. We need to add blockchain connectivity and wallet screens while preserving the existing design system.

---

**Analysis by:** Orchestrator Agent  
**Next Step:** Begin Week 6 wallet integration
