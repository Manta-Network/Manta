#!/bin/bash

# Chameleon Network Devnet Reset Script
# This script completely resets the devnet by removing all data

set -e

DEVNET_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🔄 Resetting Chameleon Network Devnet"
echo "⚠️  This will delete all validator data and blockchain state!"

read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Reset cancelled."
    exit 1
fi

cd "$DEVNET_DIR"

# Stop containers
echo "🛑 Stopping containers..."
docker-compose down -v

# Remove validator data
echo "🗑️  Removing validator data..."
for i in {0..4}; do
    if [ -d "validator-$i-data" ]; then
        rm -rf "validator-$i-data"
        echo "   Removed validator-$i-data"
    fi
done

# Remove generated files
if [ -f "chameleon-devnet-spec.json" ]; then
    rm "chameleon-devnet-spec.json"
    echo "   Removed chain specification"
fi

echo "✅ Chameleon Network Devnet reset complete!"
echo "Run './scripts/start-devnet.sh' to start fresh."
