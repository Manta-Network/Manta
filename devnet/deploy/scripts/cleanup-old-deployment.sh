#!/bin/bash
# Cleanup script for removing old Manta placeholder deployment
# Run this before deploying the new standalone node

set -e

echo "==================================="
echo "Cleaning Up Old Deployment"
echo "==================================="

# Stop all old services
echo "Stopping old validator services..."
for i in {1..5}; do
    if systemctl is-active --quiet chameleon-validator-${i} 2>/dev/null; then
        sudo systemctl stop chameleon-validator-${i}
        sudo systemctl disable chameleon-validator-${i}
        echo "  ✓ Stopped validator-${i}"
    fi
done

# Stop old RPC service
if systemctl is-active --quiet chameleon-rpc 2>/dev/null; then
    sudo systemctl stop chameleon-rpc
    sudo systemctl disable chameleon-rpc
    echo "  ✓ Stopped RPC node"
fi

# Remove old service files
echo "Removing old service files..."
sudo rm -f /etc/systemd/system/chameleon-validator-*.service
sudo rm -f /etc/systemd/system/chameleon-rpc.service
sudo systemctl daemon-reload
echo "  ✓ Service files removed"

# Remove old binaries
echo "Removing old binaries..."
sudo rm -f /usr/local/bin/manta
sudo rm -f /usr/local/bin/chameleon-node
sudo rm -f /usr/local/bin/chameleon-validator-*
echo "  ✓ Old binaries removed"

# Clean up old data directories (OPTIONAL - keeps blockchain data)
if [[ "${1}" == "--delete-data" ]]; then
    echo "Removing blockchain data..."
    sudo rm -rf /var/lib/chameleon/validator-*
    sudo rm -rf /var/lib/chameleon/rpc
    sudo rm -rf /var/lib/chameleon/rpc-node
    echo "  ✓ Blockchain data removed"
else
    echo "  ⊘ Keeping blockchain data (use --delete-data to remove)"
    echo "  Note: Old data will be incompatible with new chain"
fi

# Remove old logs
echo "Removing old logs..."
sudo rm -f /var/log/chameleon-validator-*.log
sudo rm -f /var/log/chameleon-rpc.log
sudo rm -f /var/log/chameleon-setup.log
sudo rm -f /var/log/chameleon-rpc-setup.log
echo "  ✓ Old logs removed"

# Clean up placeholder scripts
echo "Removing placeholder scripts..."
sudo rm -f /usr/local/bin/validator-placeholder-*
echo "  ✓ Placeholder scripts removed"

# Clean up temp deploy directory
echo "Cleaning temp files..."
sudo rm -rf /tmp/chameleon-deploy
echo "  ✓ Temp files removed"

# Remove old chain spec
echo "Removing old chain spec..."
sudo rm -f /etc/chameleon/chainspec.json
echo "  ✓ Old chain spec removed"

echo ""
echo "==================================="
echo "Cleanup Complete!"
echo "==================================="
echo ""
echo "System is now ready for fresh standalone node deployment."
echo ""
echo "Next steps:"
echo "  1. Pull latest code: cd ~/chameleon-network && git pull"
echo "  2. Run deployment: ./devnet/deploy/deploy-to-digitalocean.sh"
echo ""
