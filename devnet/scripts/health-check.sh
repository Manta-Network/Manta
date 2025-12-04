#!/bin/bash

# Chameleon Network Devnet Health Check Script

set -e

DEVNET_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🏥 Chameleon Network Devnet Health Check"
echo "======================================"

# Check Docker containers
echo "📦 Container Status:"
docker-compose ps
echo ""

# Check validator health
echo "🔍 Validator Health:"
for i in {0..4}; do
    port=$((9933 + i))
    echo -n "   Validator $i (port $port): "
    
    if curl -s -f "http://localhost:$port/health" > /dev/null 2>&1; then
        echo "✅ Healthy"
    else
        echo "❌ Unhealthy"
    fi
done
echo ""

# Check block production
echo "⛓️  Block Production:"
for i in {0..4}; do
    port=$((9933 + i))
    echo -n "   Validator $i: "
    
    response=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader", "params":[]}' \
        "http://localhost:$port" 2>/dev/null || echo "error")
    
    if [ "$response" != "error" ]; then
        block_number=$(echo "$response" | jq -r '.result.number' 2>/dev/null || echo "unknown")
        echo "Block #$block_number"
    else
        echo "❌ No response"
    fi
done
echo ""

# Check peer connections
echo "🌐 Peer Connections:"
for i in {0..4}; do
    port=$((9933 + i))
    echo -n "   Validator $i: "
    
    response=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers", "params":[]}' \
        "http://localhost:$port" 2>/dev/null || echo "error")
    
    if [ "$response" != "error" ]; then
        peer_count=$(echo "$response" | jq -r '.result | length' 2>/dev/null || echo "unknown")
        echo "$peer_count peers"
    else
        echo "❌ No response"
    fi
done
echo ""

# Check monitoring services
echo "📊 Monitoring Services:"
echo -n "   Prometheus: "
if curl -s -f "http://localhost:9090/-/healthy" > /dev/null 2>&1; then
    echo "✅ Healthy"
else
    echo "❌ Unhealthy"
fi

echo -n "   Grafana: "
if curl -s -f "http://localhost:3000/api/health" > /dev/null 2>&1; then
    echo "✅ Healthy"
else
    echo "❌ Unhealthy"
fi

echo -n "   Polkadot.js Apps: "
if curl -s -f "http://localhost:3001" > /dev/null 2>&1; then
    echo "✅ Healthy"
else
    echo "❌ Unhealthy"
fi

echo ""
echo "🏁 Health check complete!"
