#!/bin/bash
set -e

echo "=== Chameleon Network - Contabo Build Script ==="
echo "Starting build at: $(date)"

# Navigate to repository
cd /root/chameleon-network || {
    echo "Error: Repository not found at /root/chameleon-network"
    echo "Please clone first: git clone https://github.com/chmldev/chameleon-network.git"
    exit 1
}

# Pull latest from develop
echo "Pulling latest from develop branch..."
git checkout develop
git pull origin develop

# Navigate to node template
cd node-template

# Clean previous builds (optional, saves space)
echo "Cleaning previous builds..."
cargo clean

# Build in release mode
echo "Building node binary (this takes 30-45 minutes)..."
cargo build --release

# Verify binary exists
if [ -f "target/release/solochain-template-node" ]; then
    echo ""
    echo "✅ BUILD SUCCESSFUL!"
    echo "Binary location: $(pwd)/target/release/solochain-template-node"
    echo "Binary size: $(du -h target/release/solochain-template-node | cut -f1)"
    echo ""
    echo "Next steps:"
    echo "1. Test locally: ./target/release/solochain-template-node --dev"
    echo "2. Deploy to DO: bash /root/chameleon-network/scripts/deploy-to-do.sh"
else
    echo ""
    echo "❌ BUILD FAILED - Binary not found"
    exit 1
fi

echo "Build completed at: $(date)"
