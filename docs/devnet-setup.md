# Chameleon Network Devnet Setup Guide

This guide provides comprehensive instructions for setting up and running a 5-validator Chameleon Network devnet for development and testing purposes.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Network Overview](#network-overview)
3. [Genesis Configuration](#genesis-configuration)
4. [Validator Configuration](#validator-configuration)
5. [Network Parameters](#network-parameters)
6. [Starting the Network](#starting-the-network)
7. [Monitoring and Management](#monitoring-and-management)
8. [Troubleshooting](#troubleshooting)
9. [Development Workflows](#development-workflows)

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 20.04+ recommended) or macOS
- **CPU**: 4+ cores (8+ recommended for validators)
- **RAM**: 8GB minimum (16GB+ recommended)
- **Storage**: 100GB+ SSD space
- **Network**: Stable internet connection with open ports

### Software Dependencies

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown

# Install additional tools
sudo apt update
sudo apt install -y git clang curl libssl-dev llvm libudev-dev make protobuf-compiler

# Install Docker and Docker Compose (for containerized setup)
sudo apt install -y docker.io docker-compose
sudo usermod -aG docker $USER
```

### Build Chameleon Node

```bash
# Clone the repository
git clone https://github.com/chameleon-network/chameleon.git
cd chameleon

# Build the node binary
cargo build --release --features runtime-benchmarks

# Verify the build
./target/release/manta --version
```

## Network Overview

### Chameleon Devnet Specifications

- **Network Name**: Chameleon Network Devnet
- **Chain ID**: `chameleon-devnet`
- **Parachain ID**: 2105
- **Validators**: 5 nodes across geographic regions
- **Block Time**: 6 seconds
- **Finality**: 2 blocks (~12 seconds)
- **Total Supply**: 100,000,000 CHML

### Validator Distribution

| Index | Name | Region | Stake | Account ID |
|-------|------|--------|-------|------------|
| 0 | Validator-US-East-1 | US-East | 1M CHML | `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY` |
| 1 | Validator-US-West-1 | US-West | 1M CHML | `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty` |
| 2 | Validator-EU-Central-1 | EU-Central | 1M CHML | `5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y` |
| 3 | Validator-Asia-East-1 | Asia-East | 1M CHML | `5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy` |
| 4 | Validator-Asia-Southeast-1 | Asia-Southeast | 1M CHML | `5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc` |

## Genesis Configuration

### Token Allocation (100M CHML Total)

| Allocation | Amount | Percentage | Purpose |
|------------|--------|------------|----------|
| **Validators** | 5M CHML | 5.0% | Network security (5 validators × 1M each) |
| **Liquidity Pool** | 2.5M CHML | 2.5% | Initial DEX liquidity |
| **Treasury** | 2.5M CHML | 2.5% | DAO-controlled strategic reserve |
| **Ecosystem** | 5M CHML | 5.0% | Team & development (36-month vesting) |
| **Presale** | 15M CHML | 15.0% | Public sale (6-month linear vesting) |
| **Airdrop** | 5M CHML | 5.0% | Community distribution (3 stages) |
| **Emission** | 65M CHML | 65.0% | 20-year validator/LP rewards |

### System Accounts

```rust
// Account IDs for system functions
Liquidity Pool:    chml/liq (PalletId)
Treasury:         chml/tre (PalletId)
Ecosystem:        chml/eco (PalletId)
Presale:          chml/pre (PalletId)
Airdrop:          chml/air (PalletId)
Emission:         chml/emi (PalletId)
```

### Emission Schedule

- **Total Rewards**: 65M CHML over 20 years
- **Year 1**: 7.4M CHML (11.38% of pool)
- **Decay Rate**: 10% per year (90% retention)
- **Distribution**: 70% validators, 30% liquidity providers

## Validator Configuration

### Generate Validator Keys

```bash
# Create validator key directories
mkdir -p devnet/validator-keys

# Generate session keys for each validator
for i in {0..4}; do
    ./target/release/manta key generate --scheme Sr25519 --password-interactive \
        --output-file devnet/validator-keys/validator-${i}-session.json \
        --network chameleon
done

# Generate node keys
for i in {0..4}; do
    ./target/release/manta key generate-node-key \
        --file devnet/validator-keys/validator-${i}-node-key
done
```

### Validator Startup Scripts

Create individual startup scripts for each validator:

```bash
#!/bin/bash
# validator-0-start.sh

VALIDATOR_INDEX=0
NODE_NAME="Validator-US-East-1"
P2P_PORT=30333
RPC_PORT=9933
WS_PORT=9944
PROMETHEUS_PORT=9615

./target/release/manta \
    --validator \
    --name "${NODE_NAME}" \
    --chain devnet/chameleon-devnet-spec.json \
    --base-path ./devnet/validator-${VALIDATOR_INDEX}-data \
    --port ${P2P_PORT} \
    --rpc-port ${RPC_PORT} \
    --ws-port ${WS_PORT} \
    --prometheus-port ${PROMETHEUS_PORT} \
    --rpc-cors all \
    --unsafe-rpc-external \
    --unsafe-ws-external \
    --prometheus-external \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooW... \
    -- \
    --chain polkadot-local \
    --port 30343 \
    --rpc-port 9943
```

## Network Parameters

### Chain Specification

```json
{
  "name": "Chameleon Network Devnet",
  "id": "chameleon-devnet",
  "chainType": "Live",
  "bootNodes": [],
  "telemetryEndpoints": null,
  "protocolId": "chameleon",
  "properties": {
    "ss58Format": 99,
    "tokenDecimals": 18,
    "tokenSymbol": "CHML"
  },
  "extensions": {
    "relay_chain": "polkadot",
    "para_id": 2105
  }
}
```

### Runtime Configuration

- **Block Time**: 6 seconds (faster than Manta's 12s)
- **Session Length**: 4 hours (2400 blocks)
- **Unbonding Period**: 14 days (201,600 blocks)
- **Maximum Validators**: 200
- **Minimum Validators**: 5
- **Existential Deposit**: 0.001 CHML
- **Minimum Validator Stake**: 1,750 CHML

## Starting the Network

### Method 1: Docker Compose (Recommended)

```bash
# Start all validators
cd devnet
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f validator-0
```

### Method 2: Manual Binary Execution

```bash
# Generate chain specification
./target/release/manta build-spec \
    --chain chameleon-devnet \
    --raw > devnet/chameleon-devnet-spec.json

# Start validators in separate terminals
./devnet/scripts/validator-0-start.sh &
./devnet/scripts/validator-1-start.sh &
./devnet/scripts/validator-2-start.sh &
./devnet/scripts/validator-3-start.sh &
./devnet/scripts/validator-4-start.sh &
```

### Method 3: Development Mode (Single Node)

```bash
# Quick development setup
./target/release/manta \
    --dev \
    --chain chameleon-dev \
    --tmp \
    --rpc-cors all \
    --unsafe-rpc-external \
    --unsafe-ws-external
```

## Monitoring and Management

### Health Checks

```bash
# Check node status
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
     http://localhost:9933

# Check validator status
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_hasSessionKeys", "params":["0x..."]}' \
     http://localhost:9933

# Check block production
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader", "params":[]}' \
     http://localhost:9933
```

### Prometheus Metrics

Access validator metrics at:
- Validator 0: http://localhost:9615/metrics
- Validator 1: http://localhost:9616/metrics
- Validator 2: http://localhost:9617/metrics
- Validator 3: http://localhost:9618/metrics
- Validator 4: http://localhost:9619/metrics

### Log Analysis

```bash
# Follow validator logs
tail -f devnet/validator-0-data/chains/chameleon-devnet/network/validator-0.log

# Search for specific events
grep "Imported" devnet/validator-*/data/chains/chameleon-devnet/network/*.log
grep "Finalized" devnet/validator-*/data/chains/chameleon-devnet/network/*.log
```

## Troubleshooting

### Common Issues

#### 1. Validators Not Producing Blocks

**Symptoms**: No new blocks, stuck at genesis

**Solutions**:
```bash
# Check validator session keys
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' \
     http://localhost:9933

# Verify validator is in active set
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "session_validators", "params":[]}' \
     http://localhost:9933

# Check staking status
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "state_call", "params":["ParachainStakingApi_validator_state", "0x..."]}' \
     http://localhost:9933
```

#### 2. Network Connectivity Issues

**Symptoms**: Peers not connecting, sync issues

**Solutions**:
```bash
# Check peer connections
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers", "params":[]}' \
     http://localhost:9933

# Add manual peer connections
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_addReservedPeer", "params":["/ip4/127.0.0.1/tcp/30334/p2p/..."]}' \
     http://localhost:9933

# Check firewall settings
sudo ufw status
sudo ufw allow 30333:30337/tcp
```

#### 3. Database Corruption

**Symptoms**: Node crashes, database errors

**Solutions**:
```bash
# Purge chain data
./target/release/manta purge-chain \
    --chain devnet/chameleon-devnet-spec.json \
    --base-path ./devnet/validator-0-data

# Restart with fresh database
rm -rf devnet/validator-*/data/chains/chameleon-devnet/db
```

#### 4. Memory/Performance Issues

**Symptoms**: High memory usage, slow block times

**Solutions**:
```bash
# Monitor resource usage
top -p $(pgrep manta)
htop

# Adjust cache settings
./target/release/manta \
    --validator \
    --pruning archive \
    --state-cache-size 1073741824 \
    --db-cache 2048

# Enable database compression
./target/release/manta \
    --validator \
    --database rocksdb \
    --db-cache 2048
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/manta \
    --validator \
    --chain devnet/chameleon-devnet-spec.json

# Specific module debugging
RUST_LOG=sc_consensus_aura=debug,sc_consensus_slots=debug \
    ./target/release/manta --validator
```

### Network Reset

```bash
# Complete network reset
docker-compose down -v
rm -rf devnet/validator-*/data
rm -rf devnet/validator-*/keys
docker-compose up -d
```

## Development Workflows

### Testing New Features

1. **Build with feature flags**:
   ```bash
   cargo build --release --features runtime-benchmarks,try-runtime
   ```

2. **Deploy to single validator**:
   ```bash
   ./target/release/manta --dev --tmp --chain chameleon-dev
   ```

3. **Test on full devnet**:
   ```bash
   docker-compose restart
   ```

### Runtime Upgrades

1. **Build new runtime**:
   ```bash
   cargo build --release -p manta-runtime
   ```

2. **Generate upgrade proposal**:
   ```bash
   ./target/release/manta build-spec \
       --chain chameleon-devnet \
       --raw > devnet/chameleon-devnet-spec-new.json
   ```

3. **Submit upgrade via governance**:
   ```bash
   # Use Polkadot.js Apps or custom scripts
   ```

### Performance Testing

```bash
# Benchmark runtime
cargo run --release --features runtime-benchmarks -- \
    benchmark pallet \
    --chain chameleon-dev \
    --pallet "*" \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20

# Load testing
./scripts/load-test.sh --validators 5 --transactions 1000
```

### Integration Testing

```bash
# Run integration tests
cargo test --release --workspace

# Specific pallet tests
cargo test --release -p pallet-chameleon-staking
cargo test --release -p pallet-chameleon-pdex
```

## Security Considerations

### Key Management

- Store validator keys securely
- Use hardware security modules (HSM) for production
- Implement key rotation procedures
- Monitor for unauthorized access

### Network Security

- Configure firewalls properly
- Use VPN for validator communication
- Monitor for DDoS attacks
- Implement rate limiting

### Operational Security

- Regular security audits
- Automated monitoring and alerting
- Incident response procedures
- Backup and recovery plans

## Support and Resources

- **Documentation**: https://docs.chameleon.network
- **GitHub**: https://github.com/chameleon-network/chameleon
- **Discord**: https://discord.gg/chameleon
- **Telegram**: https://t.me/chameleonnetwork
- **Email**: devnet-support@chameleon.network

---

**Last Updated**: December 19, 2024  
**Version**: 1.0.0  
**Maintainer**: Chameleon Network Team
