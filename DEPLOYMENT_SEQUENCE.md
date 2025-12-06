# Standalone Node Deployment Sequence

## Overview

Step-by-step guide for deploying Chameleon standalone node to DigitalOcean.

## Prerequisites

- 2 DigitalOcean droplets provisioned
  - Droplet 1: 104.131.167.75 (NYC3)
  - Droplet 2: 64.23.233.36 (SFO3)
- SSH access configured
- Contabo management server (optional, or use local machine)

## Stage 1: Push and Build

### 1.1 Push to GitHub

```bash
cd /path/to/chameleon-network
git checkout develop
git add -A
git commit -m "feat: Standalone node complete - ready for deployment"
git push origin develop
```

### 1.2 Wait for GitHub Actions

- Monitor: https://github.com/chmldev/chameleon-network/actions
- Wait for "Build Standalone Node" workflow to complete (~30 minutes)
- Verify artifacts uploaded (chameleon-node binary)

## Stage 2: Cleanup DO Droplets

### 2.1 SSH to Droplet 1 (NYC3)

```bash
ssh root@104.131.167.75
```

### 2.2 Download and Run Cleanup Script

```bash
cd ~
git clone https://github.com/chmldev/chameleon-network.git || (cd chameleon-network && git pull)
cd chameleon-network
./devnet/deploy/scripts/cleanup-old-deployment.sh --delete-data
```

### 2.3 SSH to Droplet 2 (SFO3)

```bash
ssh root@64.23.233.36
```

### 2.4 Run Cleanup on Droplet 2

```bash
cd ~
git clone https://github.com/chmldev/chameleon-network.git || (cd chameleon-network && git pull)
cd chameleon-network
./devnet/deploy/scripts/cleanup-old-deployment.sh --delete-data
```

## Stage 3: Deploy Standalone Node

### 3.1 On Management Server (Contabo or Local)

```bash
cd /path/to/chameleon-network
./devnet/deploy/deploy-to-digitalocean.sh
```

This will:
1. Download chameleon-node binary from GitHub releases
2. Copy to both droplets
3. Set up 3 validators on Droplet 1
4. Set up 2 validators + RPC on Droplet 2
5. Start all services

### 3.2 Expected Output

```
Deploying to Droplet 1 (104.131.167.75)...
  ✓ Binary copied
  ✓ Validator 1 started
  ✓ Validator 2 started
  ✓ Validator 3 started

Deploying to Droplet 2 (64.23.233.36)...
  ✓ Binary copied
  ✓ Validator 4 started
  ✓ Validator 5 started
  ✓ RPC node started

Deployment complete! 🎉
```

## Stage 4: Verification

### 4.1 Check Validator Status (Droplet 1)

```bash
ssh root@104.131.167.75

# Check services
systemctl status chameleon-validator-1
systemctl status chameleon-validator-2
systemctl status chameleon-validator-3

# Check logs
journalctl -u chameleon-validator-1 -f
```

Look for: "💤 Idle (X peers)" or "🎁 Prepared block"

### 4.2 Check Validator Status (Droplet 2)

```bash
ssh root@64.23.233.36

# Check validator services
systemctl status chameleon-validator-4
systemctl status chameleon-validator-5

# Check RPC service
systemctl status chameleon-rpc

# Check RPC logs
journalctl -u chameleon-rpc -f
```

### 4.3 Test RPC Endpoint

```bash
# From anywhere
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://64.23.233.36:9944
```

Expected response:
```json
{
  "jsonrpc":"2.0",
  "result": {
    "peers": 4,
    "isSyncing": false,
    "shouldHavePeers": true
  },
  "id":1
}
```

### 4.4 Test Block Production

```bash
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getBlock"}' \
  http://64.23.233.36:9944
```

Should return latest block info. Run multiple times - block number should increase.

## Stage 5: 24-Hour Stability Test

### 5.1 Monitor Block Production

```bash
# On management server
watch -n 10 'curl -s -H "Content-Type: application/json" \
  -d "{\"id\":1, \"jsonrpc\":\"2.0\", \"method\": \"chain_getHeader\"}" \
  http://64.23.233.36:9944 | jq .result.number'
```

Should show increasing block numbers every 6 seconds.

### 5.2 Monitor Logs for Errors

```bash
# On each droplet
journalctl -u "chameleon-*" -f | grep -i "error\|panic\|fatal"
```

Should see minimal output (only normal operational messages).

### 5.3 Check Resource Usage

```bash
# On each droplet
htop

# Check disk
df -h /var/lib/chameleon
```

## Troubleshooting

### Validators Not Producing Blocks

1. Check if all 5 validators are running:
   ```bash
   systemctl list-units | grep chameleon
   ```
2. Check peer connections: Look for "X peers" in logs
3. Verify chain spec matches across all nodes
4. Check firewall:
   ```bash
   ufw status  # ports 30333-30335 should be open
   ```

### RPC Not Responding

1. Check service:
   ```bash
   systemctl status chameleon-rpc
   ```
2. Check port binding:
   ```bash
   netstat -tlnp | grep 9944
   ```
3. Check firewall:
   ```bash
   ufw allow 9944/tcp
   ```
4. Test locally:
   ```bash
   curl http://localhost:9944
   ```

### High CPU/Memory Usage

1. Check if WASM compilation is happening (normal on first run)
2. Monitor with `htop` and `iotop`
3. Check logs for spam or loops

### Nodes Not Syncing

1. Verify all nodes using same genesis (chain spec)
2. Check bootnodes configuration
3. Verify P2P port accessibility
4. Check time sync:
   ```bash
   timedatectl
   ```

## Success Criteria

- ✅ All 5 validators running (systemctl status green)
- ✅ Blocks being produced every 6 seconds
- ✅ RPC endpoint responding to queries
- ✅ All nodes have 4+ peers
- ✅ No errors in logs
- ✅ Resource usage stable (<50% CPU, <2GB RAM)
- ✅ 24-hour uptime achieved

## Next Steps After Successful Deployment

1. **Week 6:** Mobile wallet RPC integration
2. Connect app to: `ws://64.23.233.36:9944`
3. Test balance queries and transactions
4. Begin feature development

## Rollback Plan (If Deployment Fails)

If standalone node has critical issues:

1. Stop all services:
   ```bash
   systemctl stop chameleon-*
   ```
2. Investigate compilation errors
3. Fix issues on develop branch
4. Rebuild and redeploy
5. If unfixable within 4 hours → Reassess approach

## Contact

Issues during deployment? Document in `ORCHESTRATOR_STATUS.md` under "Deployment Issues" section.
