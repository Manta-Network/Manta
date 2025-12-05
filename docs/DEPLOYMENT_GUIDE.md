# Chameleon Network DigitalOcean Deployment Guide

This guide provides step-by-step instructions for deploying the Chameleon Network devnet across 2 DigitalOcean droplets.

## 📋 Prerequisites

### Infrastructure (Already Complete)

✅ **Droplet 1 (NYC3)**: `104.131.167.75`
- Size: 4GB RAM, 2 vCPU
- OS: Ubuntu 22.04 LTS
- Will run: 3 validator nodes

✅ **Droplet 2 (SFO3)**: `64.23.233.36`
- Size: 4GB RAM, 2 vCPU
- OS: Ubuntu 22.04 LTS
- Will run: 2 validator nodes + 1 public RPC node

### Local Requirements

- SSH client installed
- SSH keys configured for root access to both droplets
- Git repository cloned locally

### SSH Key Setup

Ensure you can SSH to both droplets:

```bash
# Test SSH connectivity
ssh root@104.131.167.75 "echo 'Droplet 1 accessible'"
ssh root@64.23.233.36 "echo 'Droplet 2 accessible'"
```

## 🚀 Deployment Steps

### Step 1: Navigate to Deployment Directory

```bash
cd /path/to/chameleon-network
cd devnet/deploy
```

### Step 2: Make Scripts Executable

```bash
chmod +x deploy-to-digitalocean.sh
chmod +x scripts/setup-validator-node.sh
chmod +x scripts/setup-rpc-node.sh
```

### Step 3: Run Deployment Script

```bash
./deploy-to-digitalocean.sh
```

The script will:
1. ✅ Check prerequisites and SSH connectivity
2. 📦 Copy deployment scripts to both droplets
3. 🔒 Deploy 3 validators on Droplet 1 (NYC3)
4. 🔒 Deploy 2 validators + RPC on Droplet 2 (SFO3)
5. 🌐 Configure P2P network
6. ▶️ Start all services
7. 📊 Check deployment status
8. 📄 Generate deployment report

### Step 4: Verify Deployment

After deployment completes, verify the services:

```bash
# Check Droplet 1 validators
ssh root@104.131.167.75 "chameleon-validator-status"

# Check Droplet 2 validators and RPC
ssh root@64.23.233.36 "chameleon-validator-status"
ssh root@64.23.233.36 "chameleon-rpc-status"
```

## 🔍 Verification Commands

### Service Status Checks

```bash
# Check all validator services on Droplet 1
ssh root@104.131.167.75 "systemctl status chameleon-validator-*"

# Check all services on Droplet 2
ssh root@64.23.233.36 "systemctl status chameleon-validator-* chameleon-rpc"
```

### RPC Endpoint Testing

```bash
# Test RPC health
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  http://64.23.233.36:9944

# Test system info
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_name", "params":[]}' \
  http://64.23.233.36:9944

# Test chain info
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain", "params":[]}' \
  http://64.23.233.36:9944
```

### Log Monitoring

```bash
# Follow validator logs
ssh root@104.131.167.75 "journalctl -u chameleon-validator-1 -f"

# Follow RPC logs
ssh root@64.23.233.36 "journalctl -u chameleon-rpc -f"
```

## 📡 Network Endpoints

### Validator Nodes

**Droplet 1 (NYC3) - `104.131.167.75`:**
- Validator 1: P2P `104.131.167.75:30333`, RPC `104.131.167.75:9944`
- Validator 2: P2P `104.131.167.75:30334`, RPC `104.131.167.75:9945`
- Validator 3: P2P `104.131.167.75:30335`, RPC `104.131.167.75:9946`

**Droplet 2 (SFO3) - `64.23.233.36`:**
- Validator 4: P2P `64.23.233.36:30333`, RPC `64.23.233.36:9944`
- Validator 5: P2P `64.23.233.36:30334`, RPC `64.23.233.36:9945`

### Public RPC Endpoint

🌐 **Primary RPC Endpoint**: `http://64.23.233.36:9944`
🔌 **WebSocket Endpoint**: `ws://64.23.233.36:9944`

## 📱 Mobile App Configuration

Use these settings in the Chameleon mobile app:

```json
{
  "rpcEndpoint": "http://64.23.233.36:9944",
  "chainId": "chameleon_devnet",
  "networkName": "Chameleon Devnet",
  "tokenSymbol": "CHML",
  "tokenDecimals": 18,
  "ss58Format": 99
}
```

## 🛠️ Management Commands

### Validator Management

```bash
# Start all validators on a droplet
ssh root@104.131.167.75 "chameleon-start-all"

# Stop all validators on a droplet
ssh root@104.131.167.75 "chameleon-stop-all"

# Check validator status
ssh root@104.131.167.75 "chameleon-validator-status"
```

### RPC Management

```bash
# Start RPC service
ssh root@64.23.233.36 "systemctl start chameleon-rpc"

# Stop RPC service
ssh root@64.23.233.36 "systemctl stop chameleon-rpc"

# Check RPC status
ssh root@64.23.233.36 "chameleon-rpc-status"

# Test RPC endpoint
ssh root@64.23.233.36 "chameleon-rpc-test"
```

### System Management

```bash
# Restart all services on both droplets
ssh root@104.131.167.75 "systemctl restart chameleon-validator-*"
ssh root@64.23.233.36 "systemctl restart chameleon-validator-* chameleon-rpc"

# Check system resources
ssh root@104.131.167.75 "htop"
ssh root@64.23.233.36 "htop"
```

## 🔧 Troubleshooting

### Common Issues

#### 1. SSH Connection Failed

**Problem**: Cannot connect to droplets

**Solutions**:
- Verify droplet IPs are correct
- Check SSH key permissions: `chmod 600 ~/.ssh/id_rsa`
- Ensure droplets are running in DigitalOcean dashboard
- Check firewall rules allow SSH (port 22)

#### 2. Services Not Starting

**Problem**: Validator or RPC services fail to start

**Solutions**:
```bash
# Check service logs
ssh root@IP "journalctl -u chameleon-validator-1 -n 50"

# Check binary permissions
ssh root@IP "ls -la /usr/local/bin/chameleon-node"

# Verify chain spec
ssh root@IP "cat /etc/chameleon/chainspec.json | jq ."

# Check disk space
ssh root@IP "df -h"
```

#### 3. RPC Not Accessible

**Problem**: Cannot reach RPC endpoint from external

**Solutions**:
```bash
# Check if RPC port is listening
ssh root@64.23.233.36 "netstat -tuln | grep 9944"

# Check firewall rules
ssh root@64.23.233.36 "ufw status"

# Test local RPC
ssh root@64.23.233.36 "curl -s http://localhost:9944"

# Check service configuration
ssh root@64.23.233.36 "systemctl cat chameleon-rpc"
```

#### 4. Validators Not Producing Blocks

**Problem**: Network not progressing

**Solutions**:
```bash
# Check validator logs for errors
ssh root@IP "journalctl -u chameleon-validator-1 -f"

# Verify P2P connectivity
ssh root@IP "netstat -tuln | grep 30333"

# Check peer connections
# (This would require RPC calls to check peer count)
```

### Log Locations

- Setup logs: `/var/log/chameleon-setup.log`
- RPC setup logs: `/var/log/chameleon-rpc-setup.log`
- Service logs: `journalctl -u chameleon-validator-X` or `journalctl -u chameleon-rpc`
- Node data: `/var/lib/chameleon/validator-X/` or `/var/lib/chameleon/rpc-node/`

### Emergency Recovery

```bash
# Stop all services
ssh root@104.131.167.75 "chameleon-stop-all"
ssh root@64.23.233.36 "chameleon-stop-all && systemctl stop chameleon-rpc"

# Clear node data (CAUTION: This will reset the chain)
ssh root@IP "rm -rf /var/lib/chameleon/*/chains/*"

# Restart services
ssh root@104.131.167.75 "chameleon-start-all"
ssh root@64.23.233.36 "chameleon-start-all && systemctl start chameleon-rpc"
```

## ✅ Success Criteria

Your deployment is successful when:

- [ ] All 5 validator services are running (`systemctl status chameleon-validator-*`)
- [ ] RPC service is running (`systemctl status chameleon-rpc`)
- [ ] RPC endpoint responds to health checks
- [ ] All P2P ports are listening
- [ ] Validators are connected to each other
- [ ] Chain is producing blocks
- [ ] Mobile app can connect to RPC endpoint

### Final Verification Checklist

```bash
# 1. Check all services are active
ssh root@104.131.167.75 "systemctl is-active chameleon-validator-{1,2,3}"
ssh root@64.23.233.36 "systemctl is-active chameleon-validator-{1,2} chameleon-rpc"

# 2. Test RPC endpoint
curl -f http://64.23.233.36:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}'

# 3. Check block production
curl -s http://64.23.233.36:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' | jq .result.number

# 4. Verify network connectivity
ssh root@104.131.167.75 "netstat -tuln | grep -E ':(30333|30334|30335) '"
ssh root@64.23.233.36 "netstat -tuln | grep -E ':(30333|30334|9944) '"
```

## 📞 Support

If you encounter issues:

1. Check the troubleshooting section above
2. Review service logs using `journalctl`
3. Verify network connectivity and firewall rules
4. Ensure sufficient system resources (CPU, memory, disk)

---

**🎉 Congratulations!** Your Chameleon Network devnet is now running across 2 DigitalOcean droplets with 5 validators and a public RPC endpoint ready for mobile app integration.