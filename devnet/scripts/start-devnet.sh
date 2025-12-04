#!/bin/bash

# Chameleon Network Devnet Startup Script
# This script initializes and starts the 5-validator Chameleon devnet

set -e

DEVNET_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="$(cd "$DEVNET_DIR/.." && pwd)"

echo "🦎 Starting Chameleon Network Devnet"
echo "Devnet directory: $DEVNET_DIR"
echo "Root directory: $ROOT_DIR"

# Check if binary exists
if [ ! -f "$ROOT_DIR/target/release/manta" ]; then
    echo "❌ Manta binary not found. Please build the project first:"
    echo "   cd $ROOT_DIR && cargo build --release"
    exit 1
fi

# Create necessary directories
echo "📁 Creating validator directories..."
for i in {0..4}; do
    mkdir -p "$DEVNET_DIR/validator-$i-data"
    mkdir -p "$DEVNET_DIR/validator-$i-keys"
done

# Generate chain specification if it doesn't exist
if [ ! -f "$DEVNET_DIR/chameleon-devnet-spec.json" ]; then
    echo "🔧 Generating chain specification..."
    "$ROOT_DIR/target/release/manta" build-spec \
        --chain chameleon-devnet \
        --raw > "$DEVNET_DIR/chameleon-devnet-spec.json"
fi

# Generate validator keys if they don't exist
echo "🔑 Checking validator keys..."
for i in {0..4}; do
    if [ ! -f "$DEVNET_DIR/validator-$i-keys/session.json" ]; then
        echo "Generating keys for validator $i..."
        "$ROOT_DIR/target/release/manta" key generate \
            --scheme Sr25519 \
            --output-file "$DEVNET_DIR/validator-$i-keys/session.json" \
            --network chameleon
    fi
    
    if [ ! -f "$DEVNET_DIR/validator-$i-keys/node-key" ]; then
        echo "Generating node key for validator $i..."
        "$ROOT_DIR/target/release/manta" key generate-node-key \
            --file "$DEVNET_DIR/validator-$i-keys/node-key"
    fi
done

# Start Docker Compose
echo "🐳 Starting Docker containers..."
cd "$DEVNET_DIR"
docker-compose up -d

echo "✅ Chameleon Network Devnet started successfully!"
echo ""
echo "📊 Access points:"
echo "   - Validator 0 RPC: http://localhost:9933"
echo "   - Validator 0 WebSocket: ws://localhost:9944"
echo "   - Prometheus: http://localhost:9090"
echo "   - Grafana: http://localhost:3000 (admin/chameleon)"
echo "   - Polkadot.js Apps: http://localhost:3001"
echo ""
echo "🔍 Monitor logs with:"
echo "   docker-compose logs -f validator-0"
echo ""
echo "🛑 Stop the network with:"
echo "   docker-compose down"
