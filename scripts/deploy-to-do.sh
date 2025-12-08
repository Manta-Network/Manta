#!/bin/bash
set -e

echo "=== Chameleon Network - Deploy to DigitalOcean ==="

BINARY_PATH="/root/chameleon-network/node-template/target/release/solochain-template-node"
DROPLET1="104.131.167.75"
DROPLET2="64.23.233.36"

# Verify binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Binary not found at: $BINARY_PATH"
    echo "Run build script first: bash /root/chameleon-network/scripts/contabo-build.sh"
    exit 1
fi

echo "Deploying binary to DigitalOcean droplets..."

# Deploy to Droplet 1 (NYC3 - 3 validators)
echo "Deploying to Droplet 1 ($DROPLET1)..."
scp "$BINARY_PATH" root@$DROPLET1:/usr/local/bin/chameleon-node
ssh root@$DROPLET1 "chmod +x /usr/local/bin/chameleon-node"

# Deploy to Droplet 2 (SFO3 - 2 validators + RPC)
echo "Deploying to Droplet 2 ($DROPLET2)..."
scp "$BINARY_PATH" root@$DROPLET2:/usr/local/bin/chameleon-node
ssh root@$DROPLET2 "chmod +x /usr/local/bin/chameleon-node"

echo ""
echo "✅ DEPLOYMENT COMPLETE"
echo ""
echo "Next steps:"
echo "1. SSH to droplets and start validators"
echo "2. Droplet 1: Run 3 validators (Alice, Bob, Charlie)"
echo "3. Droplet 2: Run 2 validators (Dave, Eve) + RPC node"
echo ""
echo "Start validator example:"
echo "  /usr/local/bin/chameleon-node --validator --name Alice --chain=dev"
