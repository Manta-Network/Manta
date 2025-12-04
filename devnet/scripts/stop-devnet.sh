#!/bin/bash

# Chameleon Network Devnet Stop Script

set -e

DEVNET_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🛑 Stopping Chameleon Network Devnet"

cd "$DEVNET_DIR"
docker-compose down

echo "✅ Chameleon Network Devnet stopped successfully!"
