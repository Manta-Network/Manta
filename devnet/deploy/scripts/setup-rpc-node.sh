#!/bin/bash

# Chameleon Network RPC Node Setup Script
# Sets up a public RPC node for external access
# Usage: ./setup-rpc-node.sh

set -euo pipefail

# Configuration
NODE_USER="chameleon"
NODE_HOME="/var/lib/chameleon"
BINARY_PATH="/usr/local/bin/chameleon-node"
CHAIN_SPEC_PATH="/etc/chameleon/chainspec.json"
RPC_DIR="${NODE_HOME}/rpc-node"
SERVICE_NAME="chameleon-rpc"
RPC_PORT=9944
WS_PORT=9944
P2P_PORT=30333
METRICS_PORT=9615

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Emojis
CHECK="✅"
CROSS="❌"
GEAR="⚙️"
RPC="📡"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} ${GEAR} $1" | tee -a /var/log/chameleon-rpc-setup.log
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} ${CHECK} $1" | tee -a /var/log/chameleon-rpc-setup.log
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} ⚠️  $1" | tee -a /var/log/chameleon-rpc-setup.log
}

log_error() {
    echo -e "${RED}[ERROR]${NC} ${CROSS} $1" | tee -a /var/log/chameleon-rpc-setup.log
}

log_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n" | tee -a /var/log/chameleon-rpc-setup.log
}

# Check prerequisites
check_prerequisites() {
    log_header "Checking Prerequisites"
    
    # Check if chameleon user exists
    if ! id "${NODE_USER}" &>/dev/null; then
        log_error "User ${NODE_USER} does not exist. Run validator setup first."
        exit 1
    fi
    
    # Check if binary exists
    if [[ ! -x "${BINARY_PATH}" ]]; then
        log_error "Node binary not found at ${BINARY_PATH}. Run validator setup first."
        exit 1
    fi
    
    # Check if chain spec exists
    if [[ ! -f "${CHAIN_SPEC_PATH}" ]]; then
        log_error "Chain specification not found at ${CHAIN_SPEC_PATH}. Run validator setup first."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Create RPC node directory
create_rpc_directory() {
    log_header "Creating RPC Node Directory"
    
    # Create RPC node directory
    mkdir -p "${RPC_DIR}/chains"
    mkdir -p "${RPC_DIR}/keystore"
    mkdir -p "${RPC_DIR}/logs"
    
    # Generate node key for RPC node
    openssl rand -hex 32 > "${RPC_DIR}/node-key"
    chmod 600 "${RPC_DIR}/node-key"
    
    # Set proper ownership
    chown -R "${NODE_USER}:${NODE_USER}" "${RPC_DIR}"
    
    log_success "RPC node directory created: ${RPC_DIR}"
}

# Create systemd service for RPC node
create_rpc_service() {
    log_header "Creating RPC Node Systemd Service"
    
    local rpc_name="rpc-node-$(hostname -s)"
    
    log_info "Creating service: ${SERVICE_NAME}"
    log_info "RPC will be accessible on port ${RPC_PORT}"
    
    # Create service file
    cat > "/etc/systemd/system/${SERVICE_NAME}.service" << EOF
[Unit]
Description=Chameleon Network Public RPC Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${NODE_USER}
Group=${NODE_USER}
WorkingDirectory=${RPC_DIR}
ExecStart=${BINARY_PATH} \\
    --name "${rpc_name}" \\
    --chain ${CHAIN_SPEC_PATH} \\
    --base-path ${RPC_DIR} \\
    --port ${P2P_PORT} \\
    --rpc-port ${RPC_PORT} \\
    --ws-port ${WS_PORT} \\
    --prometheus-port ${METRICS_PORT} \\
    --rpc-cors all \\
    --unsafe-rpc-external \\
    --unsafe-ws-external \\
    --prometheus-external \\
    --rpc-methods Safe \\
    --node-key-file ${RPC_DIR}/node-key \\
    --log info,runtime::system=debug

Restart=always
RestartSec=10
KillSignal=SIGINT
TimeoutStopSec=60
KillMode=mixed

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${RPC_DIR} /tmp

# Environment
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd
    systemctl daemon-reload
    
    log_success "RPC service file created: ${SERVICE_NAME}"
}

# Enable RPC service
enable_rpc_service() {
    log_header "Enabling RPC Service"
    
    systemctl enable "${SERVICE_NAME}"
    
    log_success "RPC service enabled: ${SERVICE_NAME}"
}

# Configure firewall for RPC
configure_rpc_firewall() {
    log_header "Configuring Firewall for RPC"
    
    # Allow RPC port from anywhere (public access)
    ufw allow "${RPC_PORT}/tcp"
    log_info "Allowed public access to RPC port: ${RPC_PORT}"
    
    # Allow WebSocket port from anywhere (public access)
    ufw allow "${WS_PORT}/tcp"
    log_info "Allowed public access to WebSocket port: ${WS_PORT}"
    
    # Allow P2P port for RPC node
    ufw allow "${P2P_PORT}/tcp"
    log_info "Allowed P2P port: ${P2P_PORT}"
    
    # Allow Prometheus metrics (restricted to private networks)
    ufw allow from 10.0.0.0/8 to any port "${METRICS_PORT}"
    ufw allow from 172.16.0.0/12 to any port "${METRICS_PORT}"
    ufw allow from 192.168.0.0/16 to any port "${METRICS_PORT}"
    log_info "Allowed metrics port: ${METRICS_PORT} (restricted)"
    
    log_success "Firewall configured for RPC access"
}

# Create RPC management scripts
create_rpc_management_scripts() {
    log_header "Creating RPC Management Scripts"
    
    # Create RPC status script
    cat > /usr/local/bin/chameleon-rpc-status << EOF
#!/bin/bash

echo "=== Chameleon RPC Node Status ==="
echo

service="${SERVICE_NAME}"
echo "RPC Node Service (\$service):"

if systemctl is-active --quiet "\$service"; then
    echo "  Status: ✅ Running"
    
    echo "  RPC Port: ${RPC_PORT}"
    echo "  WebSocket Port: ${WS_PORT}"
    echo "  P2P Port: ${P2P_PORT}"
    echo "  Metrics Port: ${METRICS_PORT}"
    
    # Check if ports are listening
    if netstat -tuln | grep -q ":${RPC_PORT} "; then
        echo "  RPC: ✅ Listening"
    else
        echo "  RPC: ❌ Not listening"
    fi
    
    if netstat -tuln | grep -q ":${P2P_PORT} "; then
        echo "  P2P: ✅ Listening"
    else
        echo "  P2P: ❌ Not listening"
    fi
    
    # Test RPC endpoint
    echo
    echo "Testing RPC endpoint..."
    if curl -s -H "Content-Type: application/json" \\
        -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \\
        http://localhost:${RPC_PORT} >/dev/null 2>&1; then
        echo "  RPC Health: ✅ Responding"
    else
        echo "  RPC Health: ❌ Not responding"
    fi
    
    # Get public IP for external access info
    public_ip=\$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 2>/dev/null || echo "unknown")
    if [[ "\$public_ip" != "unknown" ]]; then
        echo
        echo "External Access:"
        echo "  HTTP RPC: http://\$public_ip:${RPC_PORT}"
        echo "  WebSocket: ws://\$public_ip:${WS_PORT}"
    fi
else
    echo "  Status: ❌ Stopped"
fi

echo
echo "System Resources:"
echo "  CPU: \$(top -bn1 | grep 'Cpu(s)' | awk '{print \$2}' | cut -d'%' -f1)%"
echo "  Memory: \$(free | grep Mem | awk '{printf \"%.1f%%\", \$3/\$2 * 100.0}')"
echo "  Disk: \$(df -h / | awk 'NR==2{printf \"%s\", \$5}')"
EOF
    
    chmod +x /usr/local/bin/chameleon-rpc-status
    
    # Create RPC test script
    cat > /usr/local/bin/chameleon-rpc-test << EOF
#!/bin/bash

echo "=== Chameleon RPC Node Test ==="
echo

# Get public IP
public_ip=\$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 2>/dev/null || echo "localhost")
rpc_url="http://\$public_ip:${RPC_PORT}"

echo "Testing RPC endpoint: \$rpc_url"
echo

# Test system health
echo "1. Testing system health..."
response=\$(curl -s -H "Content-Type: application/json" \\
    -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \\
    "\$rpc_url" 2>/dev/null)

if echo "\$response" | jq -e '.result' >/dev/null 2>&1; then
    echo "   ✅ Health check passed"
    echo "   Response: \$(echo "\$response" | jq -c '.result')"
else
    echo "   ❌ Health check failed"
    echo "   Response: \$response"
fi
echo

# Test system name
echo "2. Testing system name..."
response=\$(curl -s -H "Content-Type: application/json" \\
    -d '{"id":1, "jsonrpc":"2.0", "method": "system_name", "params":[]}' \\
    "\$rpc_url" 2>/dev/null)

if echo "\$response" | jq -e '.result' >/dev/null 2>&1; then
    echo "   ✅ System name: \$(echo "\$response" | jq -r '.result')"
else
    echo "   ❌ System name failed"
    echo "   Response: \$response"
fi
echo

# Test chain info
echo "3. Testing chain info..."
response=\$(curl -s -H "Content-Type: application/json" \\
    -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain", "params":[]}' \\
    "\$rpc_url" 2>/dev/null)

if echo "\$response" | jq -e '.result' >/dev/null 2>&1; then
    echo "   ✅ Chain: \$(echo "\$response" | jq -r '.result')"
else
    echo "   ❌ Chain info failed"
    echo "   Response: \$response"
fi
echo

# Test block number
echo "4. Testing current block..."
response=\$(curl -s -H "Content-Type: application/json" \\
    -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader", "params":[]}' \\
    "\$rpc_url" 2>/dev/null)

if echo "\$response" | jq -e '.result.number' >/dev/null 2>&1; then
    echo "   ✅ Current block: \$(echo "\$response" | jq -r '.result.number')"
else
    echo "   ❌ Block number failed"
    echo "   Response: \$response"
fi
echo

echo "RPC test completed."
EOF
    
    chmod +x /usr/local/bin/chameleon-rpc-test
    
    log_success "RPC management scripts created"
}

# Create RPC info file
create_rpc_info_file() {
    log_header "Creating RPC Information File"
    
    # Get public IP
    local public_ip
    public_ip=$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 2>/dev/null || echo "unknown")
    
    cat > "${RPC_DIR}/rpc-info.json" << EOF
{
  "setup_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "hostname": "$(hostname)",
  "service_name": "${SERVICE_NAME}",
  "data_directory": "${RPC_DIR}",
  "public_ip": "${public_ip}",
  "ports": {
    "rpc": ${RPC_PORT},
    "websocket": ${WS_PORT},
    "p2p": ${P2P_PORT},
    "metrics": ${METRICS_PORT}
  },
  "endpoints": {
    "http_rpc": "http://${public_ip}:${RPC_PORT}",
    "websocket": "ws://${public_ip}:${WS_PORT}",
    "metrics": "http://${public_ip}:${METRICS_PORT}/metrics"
  },
  "configuration": {
    "public_access": true,
    "cors_enabled": true,
    "unsafe_rpc_external": true,
    "unsafe_ws_external": true,
    "rpc_methods": "Safe"
  }
}
EOF
    
    chown "${NODE_USER}:${NODE_USER}" "${RPC_DIR}/rpc-info.json"
    
    log_success "RPC information file created"
}

# Main setup function
main() {
    log_header "${RPC} Chameleon RPC Node Setup"
    
    echo -e "${BLUE}Setting up public RPC node${NC}"
    echo -e "${BLUE}RPC Port: ${RPC_PORT}${NC}"
    echo -e "${BLUE}WebSocket Port: ${WS_PORT}${NC}"
    echo -e "${BLUE}Timestamp: $(date)${NC}"
    echo
    
    # Run setup steps
    check_prerequisites
    create_rpc_directory
    create_rpc_service
    enable_rpc_service
    configure_rpc_firewall
    create_rpc_management_scripts
    create_rpc_info_file
    
    log_success "${CHECK} RPC node setup completed successfully!"
    echo
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Start RPC node: systemctl start ${SERVICE_NAME}"
    echo "  2. Check status: chameleon-rpc-status"
    echo "  3. Test RPC: chameleon-rpc-test"
    echo "  4. View logs: journalctl -u ${SERVICE_NAME} -f"
    echo
    
    # Get public IP for endpoint info
    local public_ip
    public_ip=$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 2>/dev/null || echo "unknown")
    
    if [[ "${public_ip}" != "unknown" ]]; then
        echo -e "${BLUE}Public endpoints:${NC}"
        echo "  HTTP RPC: http://${public_ip}:${RPC_PORT}"
        echo "  WebSocket: ws://${public_ip}:${WS_PORT}"
        echo
        echo -e "${BLUE}Mobile app configuration:${NC}"
        echo "  RPC Endpoint: http://${public_ip}:${RPC_PORT}"
        echo "  Chain ID: chameleon_devnet"
    fi
}

# Handle script errors
trap 'log_error "RPC setup failed at line $LINENO"' ERR

# Run main function
main "$@" 2>&1 | tee -a /var/log/chameleon-rpc-setup.log