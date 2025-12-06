# Standalone Node Build Instructions

## Overview

This guide explains how to build the Chameleon Network standalone node from source.

## Prerequisites

### System Requirements
- **OS:** Ubuntu 20.04+ or similar Linux distribution
- **RAM:** 8GB minimum (16GB recommended for release builds)
- **Disk:** 50GB free space
- **CPU:** Multi-core processor recommended

### Software Requirements
- Rust 1.74.0 or later
- Clang and LLVM
- Protobuf compiler
- CMake

### Install Dependencies (Ubuntu)

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build essentials
sudo apt install -y build-essential git clang curl libssl-dev llvm libudev-dev protobuf-compiler cmake

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify installation
rustc --version
cargo --version
```

## Build Commands

### Clone Repository

```bash
git clone https://github.com/chmldev/chameleon-network.git
cd chameleon-network
git checkout develop
```

### Test Individual Pallets (Quick Check)

```bash
# Check each custom pallet compiles (faster than full build)
cargo check -p pallet-chameleon-mev --manifest-path Cargo-standalone.toml
cargo check -p pallet-chameleon-pdex --manifest-path Cargo-standalone.toml
cargo check -p pallet-chameleon-bridge --manifest-path Cargo-standalone.toml
cargo check -p pallet-chameleon-staking --manifest-path Cargo-standalone.toml
```

### Test Runtime Compilation

```bash
# Check runtime compiles (includes WASM build)
cargo check -p chameleon-runtime --manifest-path Cargo-standalone.toml --release
```

### Build Full Node Binary

```bash
# Release build (optimized, takes 30-60 minutes on first run)
cargo build -p chameleon-node --manifest-path Cargo-standalone.toml --release
```

### Binary Location

After successful build:
```
target/release/chameleon-node
```

## Running the Node

### Development Mode (Single Node)

```bash
./target/release/chameleon-node --dev
```

This starts a single-validator development chain with:
- Alice as the sole validator
- Instant block finalization
- Pre-funded test accounts (Alice, Bob, Charlie, etc.)

### Local Testnet (5 Validators)

```bash
# Terminal 1 - Validator 1 (Alice)
./target/release/chameleon-node \
  --chain local \
  --validator \
  --alice \
  --base-path /tmp/alice \
  --port 30333 \
  --rpc-port 9944

# Terminal 2 - Validator 2 (Bob)
./target/release/chameleon-node \
  --chain local \
  --validator \
  --bob \
  --base-path /tmp/bob \
  --port 30334 \
  --rpc-port 9945 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<ALICE_PEER_ID>

# Repeat for Charlie, Dave, Eve...
```

### Generate Chain Specification

```bash
# Generate raw chain spec
./target/release/chameleon-node build-spec --chain local --raw > chameleon-chain-spec.json
```

## Troubleshooting

### Missing Dependencies

If you see "cannot find type X in this scope":
1. Check the pallet's `lib.rs` imports
2. Verify `Cargo.toml` has required dependencies
3. Ensure workspace dependencies are correct

### Type Mismatches

If you see "expected type X, found type Y":
1. Check Config trait implementations in runtime
2. Verify associated types match pallet requirements
3. Check balance/asset type consistency

### WASM Build Failures

If WASM build fails:
1. Ensure `wasm32-unknown-unknown` target is installed:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Check `substrate-wasm-builder` in `build.rs`
3. Verify no `std` dependencies leak into `no_std` code

### Memory Issues During Build

If build runs out of memory:
```bash
# Limit parallel jobs
cargo build -p chameleon-node --manifest-path Cargo-standalone.toml --release -j 2
```

### Cargo Cache Issues

If you encounter strange errors after git pull:
```bash
# Clear cargo cache
cargo clean --manifest-path Cargo-standalone.toml
```

## Deployment

### Deploy to Server

```bash
# Copy binary to server
scp target/release/chameleon-node user@server:/opt/chameleon/

# Copy chain spec
scp chameleon-chain-spec.json user@server:/opt/chameleon/

# On server: start validator
/opt/chameleon/chameleon-node \
  --chain /opt/chameleon/chameleon-chain-spec.json \
  --validator \
  --name "Validator 1" \
  --base-path /data/chameleon \
  --port 30333 \
  --rpc-port 9944 \
  --rpc-cors all \
  --rpc-external
```

### Systemd Service

Create `/etc/systemd/system/chameleon-validator.service`:

```ini
[Unit]
Description=Chameleon Network Validator
After=network.target

[Service]
Type=simple
User=chameleon
ExecStart=/opt/chameleon/chameleon-node \
  --chain /opt/chameleon/chameleon-chain-spec.json \
  --validator \
  --name "Validator 1" \
  --base-path /data/chameleon \
  --port 30333 \
  --rpc-port 9944 \
  --rpc-cors all
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable chameleon-validator
sudo systemctl start chameleon-validator
```

## Verify Deployment

### Check Node Status

```bash
# Check RPC endpoint
curl -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' http://localhost:9944

# Check block height
curl -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"chain_getHeader","params":[],"id":1}' http://localhost:9944
```

### Expected Response (Healthy Node)

```json
{"jsonrpc":"2.0","result":{"peers":4,"isSyncing":false,"shouldHavePeers":true},"id":1}
```

## Next Steps

After successful build:
1. Run local development node
2. Test with Polkadot.js Apps (https://polkadot.js.org/apps)
3. Deploy to DigitalOcean droplets
4. Configure 5 validators
5. Run 24-hour stability test

See `STANDALONE_MIGRATION.md` for architecture details.
