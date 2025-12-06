# Push Instructions for Standalone Node Deployment

## Pre-Push Checklist

- ✅ All 20 phases complete
- ✅ Deployment scripts updated for chameleon-node binary
- ✅ Cleanup script created
- ✅ Chain spec created
- ✅ Documentation complete
- ✅ Old workflow disabled
- ✅ New workflow created

## Push Sequence

### 1. Verify Local Status

```bash
cd /path/to/chameleon-network
git status
git branch  # Should show "develop"
```

### 2. Commit All Changes

```bash
git add -A
git commit -m "feat: Complete standalone node architecture

Week 4-5 Architecture Pivot Complete

NEW:
- Substrate standalone node (no parachain deps)
- 4 custom pallets integrated (MEV, pDEX, Bridge, Staking)
- Clean compilation workflow
- Updated deployment scripts
- Comprehensive documentation

READY FOR:
- Compilation testing
- DigitalOcean deployment
- 5-validator devnet launch

See: STANDALONE_MIGRATION.md, DEPLOYMENT_SEQUENCE.md"
```

### 3. Push to GitHub

```bash
git push origin develop
```

### 4. Monitor GitHub Actions

- Go to: https://github.com/chmldev/chameleon-network/actions
- Watch "Build Standalone Node" workflow
- Expected: ~30 minutes to complete
- Look for: Green checkmark on all 3 jobs

### 5. Download Binary (After Successful Build)

If GitHub Actions succeeds, binary will be in:
- Artifacts section of the workflow run
- OR GitHub Releases (if tagged)

### 6. Manual Compilation Test (Optional but Recommended)

On Contabo management server:

```bash
ssh root@your-contabo-ip

# Clone/update repo
cd ~
git clone https://github.com/chmldev/chameleon-network.git
cd chameleon-network
git checkout develop
git pull

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown

# Install dependencies
apt-get update
apt-get install -y build-essential git clang curl libssl-dev protobuf-compiler

# Build (this will take 30-60 minutes)
cargo build -p chameleon-node --manifest-path Cargo-standalone.toml --release

# If successful, binary is at:
# target/release/chameleon-node
```

### 7. Deploy to DigitalOcean

Once binary exists (from GitHub Actions or manual build):

```bash
# On management server
cd ~/chameleon-network

# Cleanup old deployment on both droplets
ssh root@104.131.167.75 'cd ~/chameleon-network && git pull && ./devnet/deploy/scripts/cleanup-old-deployment.sh --delete-data'
ssh root@64.23.233.36 'cd ~/chameleon-network && git pull && ./devnet/deploy/scripts/cleanup-old-deployment.sh --delete-data'

# Deploy standalone node
./devnet/deploy/deploy-to-digitalocean.sh
```

### 8. Verify Deployment

See `DEPLOYMENT_SEQUENCE.md` Stage 4 for verification steps.

## If Compilation Fails

### Scenario A: GitHub Actions Fails

1. Check workflow logs for specific error
2. Fix error locally
3. Test with:
   ```bash
   cargo check -p chameleon-node --manifest-path Cargo-standalone.toml
   ```
4. Commit fix, push again
5. Repeat until successful

### Scenario B: Manual Build Fails

1. Read error message carefully
2. Common issues:
   - Missing dependencies → Install as shown in error
   - Type mismatches → Check pallet Config traits
   - Import errors → Verify Cargo.toml dependencies
3. Fix, test, commit, push

### Scenario C: Persistent Compilation Issues

If 3+ attempts fail with different errors:

1. Document all errors in `ORCHESTRATOR_STATUS.md`
2. Consider Option C: Use pre-built substrate-node-template
3. Reassess approach

## Expected Timeline

| Step | Duration |
|------|----------|
| Push | Immediate |
| GitHub Actions | 30 minutes |
| Manual test (optional) | 60 minutes |
| Deployment | 15 minutes |
| Verification | 15 minutes |
| **Total** | **2 hours to working devnet** |

## Success Indicators

- ✅ GitHub Actions green
- ✅ Binary artifact uploaded
- ✅ Manual build succeeds (if tested)
- ✅ Deployment script completes
- ✅ 5 validators producing blocks
- ✅ RPC endpoint responding
- ✅ No errors in logs

## Next Session Focus

After successful deployment:

1. **Week 6:** Mobile wallet RPC integration
2. Connect to: `ws://64.23.233.36:9944`
3. Test custom pallet functionality
4. Begin privacy layer work

## Files Modified/Created in This Session

### New Files
- `/node-standalone/` - Complete standalone node
- `/runtime-standalone/` - Runtime with custom pallets
- `/Cargo-standalone.toml` - Workspace configuration
- `/.github/workflows/build-standalone.yml` - CI pipeline
- `/BUILD_INSTRUCTIONS.md` - Build guide
- `/STANDALONE_MIGRATION.md` - Migration docs
- `/DEPLOYMENT_SEQUENCE.md` - Deployment guide
- `/PUSH_INSTRUCTIONS.md` - This file
- `/devnet/deploy/scripts/cleanup-old-deployment.sh` - Cleanup script
- `/node-standalone/chainspec.json` - Chain specification

### Modified Files
- `/pallets/chameleon-*/Cargo.toml` - Removed manta deps
- `/devnet/deploy/deploy-to-digitalocean.sh` - Updated paths
- `/devnet/deploy/scripts/setup-validator-node.sh` - Updated repo
- `/.github/workflows/build-and-test.yml` - Disabled
- `/README.md` - Quick start section
- `/ORCHESTRATOR_STATUS.md` - Pivot documentation

## Summary

**Architecture Pivot Status:** COMPLETE ✅

- 20 phases executed
- Standalone node fully implemented
- All custom pallets integrated
- Deployment infrastructure updated
- Documentation comprehensive

**Ready for:** Push → Build → Deploy → Test
