# Build Chameleon Network on Contabo

## Context

After 39 iterations, we've proven that GitHub Actions cannot reliably build Substrate projects due to dependency resolution issues in CI environments.

**Solution:** Build on Contabo server (local git clone), then deploy binaries to DigitalOcean.

## Prerequisites

- Contabo server accessible via SSH
- Repository cloned to `/root/chameleon-network`
- Rust toolchain installed on Contabo

## Build Process

### 1. SSH to Contabo
```bash
ssh root@your-contabo-ip
```

### 2. First-Time Setup (if needed)
```bash
# Clone repository
cd /root
git clone https://github.com/chmldev/chameleon-network.git
cd chameleon-network

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install protobuf
apt-get update && apt-get install -y protobuf-compiler
```

### 3. Build Node Binary
```bash
# Run build script
bash /root/chameleon-network/scripts/contabo-build.sh
```

**Expected:**
- Duration: 30-45 minutes
- Output: Binary at `node-template/target/release/solochain-template-node`
- Size: ~150MB

### 4. Deploy to DigitalOcean
```bash
# Run deployment script
bash /root/chameleon-network/scripts/deploy-to-do.sh
```

**This will:**
- Copy binary to both DO droplets
- Set executable permissions
- Verify deployment

### 5. Start Validators

**On Droplet 1 (104.131.167.75) - 3 validators:**
```bash
# Start Alice
/usr/local/bin/chameleon-node --validator --name Alice --chain=dev --port 30333 --rpc-port 9944

# Start Bob (separate terminal)
/usr/local/bin/chameleon-node --validator --name Bob --chain=dev --port 30334 --rpc-port 9945

# Start Charlie (separate terminal)
/usr/local/bin/chameleon-node --validator --name Charlie --chain=dev --port 30335 --rpc-port 9946
```

**On Droplet 2 (64.23.233.36) - 2 validators + RPC:**
```bash
# Start Dave
/usr/local/bin/chameleon-node --validator --name Dave --chain=dev --port 30333 --rpc-port 9944

# Start Eve
/usr/local/bin/chameleon-node --validator --name Eve --chain=dev --port 30334 --rpc-port 9945

# Start RPC node
/usr/local/bin/chameleon-node --name RPC --chain=dev --rpc-external --rpc-cors all --port 30336 --rpc-port 9933
```

## Verify Deployment

**Check blocks are being produced:**
```bash
# On any droplet
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getBlock"}' \
  http://localhost:9944
```

## Development Workflow

**When you make code changes:**

1. **Edit code using Emergent** (on limited resources machine)
2. **Commit and push to GitHub**
3. **SSH to Contabo** and pull changes:
   ```bash
   cd /root/chameleon-network
   git pull origin develop
   ```
4. **Rebuild:**
   ```bash
   bash scripts/contabo-build.sh
   ```
5. **Redeploy:**
   ```bash
   bash scripts/deploy-to-do.sh
   ```
6. **Restart validators on DO droplets**

## Why This Works

- ✅ Local git clone preserves dependency resolution context
- ✅ Cargo resolves git dependencies properly
- ✅ Proven approach (all Substrate projects do this)
- ✅ No CI containerization issues
- ✅ Same binary works on all DO droplets

## Troubleshooting

**If build fails:**
- Check Rust version: `rustc --version` (should be 1.80+)
- Update Rust: `rustup update`
- Clean build: `cd node-template && cargo clean`

**If deployment fails:**
- Verify SSH access to DO droplets
- Check binary path is correct
- Ensure DO droplets have space (~200MB needed)
