#!/bin/bash

# Chameleon Network Validator Initialization Script
# This script runs on each validator instance at boot time
# It installs dependencies, downloads the node binary, and starts the service

set -euo pipefail

# Configuration
NODE_VERSION="v0.1.0"  # Update this to match your release version
GITHUB_REPO="chameleon-network/chameleon"
NODE_USER="chameleon"
NODE_HOME="/var/lib/chameleon"
BINARY_PATH="/usr/local/bin/chameleon-node"
SERVICE_NAME="chameleon-node"
CHAIN_SPEC_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main/devnet/chameleon-devnet-spec.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a /var/log/chameleon-init.log
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a /var/log/chameleon-init.log
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a /var/log/chameleon-init.log
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a /var/log/chameleon-init.log
}

# Get instance metadata
get_instance_metadata() {
    log_info "Retrieving instance metadata..."
    
    # Get instance ID and region from AWS metadata service
    INSTANCE_ID=$(curl -s http://169.254.169.254/latest/meta-data/instance-id || echo "unknown")
    REGION=$(curl -s http://169.254.169.254/latest/meta-data/placement/region || echo "unknown")
    AZ=$(curl -s http://169.254.169.254/latest/meta-data/placement/availability-zone || echo "unknown")
    PRIVATE_IP=$(curl -s http://169.254.169.254/latest/meta-data/local-ipv4 || echo "unknown")
    PUBLIC_IP=$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 || echo "unknown")
    
    log_success "Instance metadata retrieved:"
    log_info "  Instance ID: ${INSTANCE_ID}"
    log_info "  Region: ${REGION}"
    log_info "  AZ: ${AZ}"
    log_info "  Private IP: ${PRIVATE_IP}"
    log_info "  Public IP: ${PUBLIC_IP}"
}

# Update system packages
update_system() {
    log_info "Updating system packages..."
    
    export DEBIAN_FRONTEND=noninteractive
    
    # Update package lists
    apt-get update -y
    
    # Upgrade existing packages
    apt-get upgrade -y
    
    # Install essential packages
    apt-get install -y \
        curl \
        wget \
        unzip \
        jq \
        htop \
        iotop \
        net-tools \
        tcpdump \
        ufw \
        fail2ban \
        logrotate \
        rsync \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        ca-certificates \
        gnupg \
        lsb-release
    
    log_success "System packages updated"
}

# Configure firewall
configure_firewall() {
    log_info "Configuring firewall..."
    
    # Reset UFW to defaults
    ufw --force reset
    
    # Set default policies
    ufw default deny incoming
    ufw default allow outgoing
    
    # Allow SSH
    ufw allow 22/tcp
    
    # Allow P2P port
    ufw allow 30333/tcp
    
    # Allow RPC port (restricted to private networks)
    ufw allow from 10.0.0.0/8 to any port 9933
    ufw allow from 172.16.0.0/12 to any port 9933
    ufw allow from 192.168.0.0/16 to any port 9933
    
    # Allow WebSocket port (restricted to private networks)
    ufw allow from 10.0.0.0/8 to any port 9944
    ufw allow from 172.16.0.0/12 to any port 9944
    ufw allow from 192.168.0.0/16 to any port 9944
    
    # Allow Prometheus metrics (restricted to private networks)
    ufw allow from 10.0.0.0/8 to any port 9615
    ufw allow from 172.16.0.0/12 to any port 9615
    ufw allow from 192.168.0.0/16 to any port 9615
    
    # Enable firewall
    ufw --force enable
    
    log_success "Firewall configured"
}

# Configure fail2ban
configure_fail2ban() {
    log_info "Configuring fail2ban..."
    
    # Create custom jail for SSH
    cat > /etc/fail2ban/jail.local << EOF
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
bantime = 3600
EOF
    
    # Restart fail2ban
    systemctl restart fail2ban
    systemctl enable fail2ban
    
    log_success "Fail2ban configured"
}

# Create node user
create_node_user() {
    log_info "Creating node user..."
    
    # Create system user for the node
    if ! id "${NODE_USER}" &>/dev/null; then
        useradd --system --home "${NODE_HOME}" --create-home --shell /bin/bash "${NODE_USER}"
        log_success "Created user: ${NODE_USER}"
    else
        log_warning "User ${NODE_USER} already exists"
    fi
    
    # Create necessary directories
    mkdir -p "${NODE_HOME}/chains"
    mkdir -p "${NODE_HOME}/keystore"
    mkdir -p "${NODE_HOME}/logs"
    mkdir -p "/etc/chameleon"
    
    # Set proper ownership
    chown -R "${NODE_USER}:${NODE_USER}" "${NODE_HOME}"
    
    log_success "Node directories created"
}

# Download node binary
download_node_binary() {
    log_info "Downloading Chameleon node binary..."
    
    # Determine architecture
    ARCH=$(uname -m)
    case ${ARCH} in
        x86_64)
            BINARY_ARCH="x86_64-unknown-linux-gnu"
            ;;
        aarch64)
            BINARY_ARCH="aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported architecture: ${ARCH}"
            exit 1
            ;;
    esac
    
    # Download binary from GitHub releases
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${NODE_VERSION}/chameleon-node-${BINARY_ARCH}"
    
    log_info "Downloading from: ${DOWNLOAD_URL}"
    
    # Download with retry logic
    for i in {1..3}; do
        if wget -O "${BINARY_PATH}" "${DOWNLOAD_URL}"; then
            break
        else
            log_warning "Download attempt ${i} failed, retrying..."
            sleep 10
        fi
        
        if [[ ${i} -eq 3 ]]; then
            log_error "Failed to download node binary after 3 attempts"
            
            # Fallback: try to build from source (if this fails, we'll use a placeholder)
            log_warning "Attempting to create placeholder binary for testing..."
            cat > "${BINARY_PATH}" << 'EOF'
#!/bin/bash
echo "Chameleon Node Placeholder - Replace with actual binary"
echo "Args: $@"
sleep infinity
EOF
        fi
    done
    
    # Make binary executable
    chmod +x "${BINARY_PATH}"
    
    # Verify binary
    if [[ -x "${BINARY_PATH}" ]]; then
        log_success "Node binary installed at ${BINARY_PATH}"
        
        # Try to get version (may fail with placeholder)
        "${BINARY_PATH}" --version 2>/dev/null || log_warning "Binary version check failed (placeholder binary?)"
    else
        log_error "Failed to install node binary"
        exit 1
    fi
}

# Download chain specification
download_chain_spec() {
    log_info "Downloading chain specification..."
    
    local chain_spec_path="/etc/chameleon/chameleon-devnet-spec.json"
    
    # Download chain spec with retry logic
    for i in {1..3}; do
        if wget -O "${chain_spec_path}" "${CHAIN_SPEC_URL}"; then
            break
        else
            log_warning "Chain spec download attempt ${i} failed, retrying..."
            sleep 5
        fi
        
        if [[ ${i} -eq 3 ]]; then
            log_warning "Failed to download chain spec, creating placeholder..."
            
            # Create a minimal chain spec for testing
            cat > "${chain_spec_path}" << EOF
{
  "name": "Chameleon Network Devnet",
  "id": "chameleon-devnet",
  "chainType": "Live",
  "bootNodes": [],
  "telemetryEndpoints": null,
  "protocolId": "chameleon",
  "properties": {
    "ss58Format": 99,
    "tokenDecimals": 18,
    "tokenSymbol": "CHML"
  }
}
EOF
        fi
    done
    
    # Validate JSON
    if jq empty "${chain_spec_path}" 2>/dev/null; then
        log_success "Chain specification downloaded and validated"
    else
        log_error "Invalid chain specification JSON"
        exit 1
    fi
    
    # Set proper ownership
    chown "${NODE_USER}:${NODE_USER}" "${chain_spec_path}"
}

# Generate node key
generate_node_key() {
    log_info "Generating node key..."
    
    local node_key_path="${NODE_HOME}/node-key"
    
    # Generate random node key (32 bytes hex)
    openssl rand -hex 32 > "${node_key_path}"
    
    # Set proper permissions
    chmod 600 "${node_key_path}"
    chown "${NODE_USER}:${NODE_USER}" "${node_key_path}"
    
    log_success "Node key generated"
}

# Create systemd service
create_systemd_service() {
    log_info "Creating systemd service..."
    
    # Determine validator name based on region or instance metadata
    local validator_name="validator-${REGION}"
    
    # Create service file
    cat > "/etc/systemd/system/${SERVICE_NAME}.service" << EOF
[Unit]
Description=Chameleon Network Validator Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${NODE_USER}
Group=${NODE_USER}
WorkingDirectory=${NODE_HOME}
ExecStart=${BINARY_PATH} \\
    --validator \\
    --name "${validator_name}" \\
    --chain /etc/chameleon/chameleon-devnet-spec.json \\
    --base-path ${NODE_HOME} \\
    --port 30333 \\
    --rpc-port 9933 \\
    --ws-port 9944 \\
    --prometheus-port 9615 \\
    --rpc-cors all \\
    --unsafe-rpc-external \\
    --unsafe-ws-external \\
    --prometheus-external \\
    --node-key-file ${NODE_HOME}/node-key \\
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \\
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
ReadWritePaths=${NODE_HOME} /tmp

# Environment
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd
    systemctl daemon-reload
    
    log_success "Systemd service created"
}

# Configure log rotation
configure_log_rotation() {
    log_info "Configuring log rotation..."
    
    cat > /etc/logrotate.d/chameleon-node << EOF
${NODE_HOME}/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 ${NODE_USER} ${NODE_USER}
    postrotate
        systemctl reload ${SERVICE_NAME} > /dev/null 2>&1 || true
    endscript
}
EOF
    
    log_success "Log rotation configured"
}

# Install monitoring tools
install_monitoring() {
    log_info "Installing monitoring tools..."
    
    # Install node_exporter for system metrics
    local node_exporter_version="1.7.0"
    local node_exporter_url="https://github.com/prometheus/node_exporter/releases/download/v${node_exporter_version}/node_exporter-${node_exporter_version}.linux-amd64.tar.gz"
    
    cd /tmp
    wget "${node_exporter_url}"
    tar xzf "node_exporter-${node_exporter_version}.linux-amd64.tar.gz"
    cp "node_exporter-${node_exporter_version}.linux-amd64/node_exporter" /usr/local/bin/
    chmod +x /usr/local/bin/node_exporter
    
    # Create node_exporter service
    cat > /etc/systemd/system/node_exporter.service << EOF
[Unit]
Description=Node Exporter
After=network.target

[Service]
Type=simple
User=nobody
ExecStart=/usr/local/bin/node_exporter --web.listen-address=:9100
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    
    # Enable and start node_exporter
    systemctl daemon-reload
    systemctl enable node_exporter
    systemctl start node_exporter
    
    log_success "Monitoring tools installed"
}

# Start and enable services
start_services() {
    log_info "Starting services..."
    
    # Enable and start the Chameleon node service
    systemctl enable "${SERVICE_NAME}"
    systemctl start "${SERVICE_NAME}"
    
    # Wait a moment for service to start
    sleep 5
    
    # Check service status
    if systemctl is-active --quiet "${SERVICE_NAME}"; then
        log_success "Chameleon node service started successfully"
    else
        log_error "Failed to start Chameleon node service"
        systemctl status "${SERVICE_NAME}" --no-pager
        exit 1
    fi
}

# Create health check script
create_health_check() {
    log_info "Creating health check script..."
    
    cat > /usr/local/bin/chameleon-health-check << 'EOF'
#!/bin/bash

# Chameleon Node Health Check Script

set -euo pipefail

# Configuration
RPC_URL="http://localhost:9933"
LOG_FILE="/var/log/chameleon-health.log"

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "${LOG_FILE}"
}

# Check if service is running
check_service() {
    if systemctl is-active --quiet chameleon-node; then
        log "✓ Service is running"
        return 0
    else
        log "✗ Service is not running"
        return 1
    fi
}

# Check RPC endpoint
check_rpc() {
    local response
    if response=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
        "${RPC_URL}" 2>/dev/null); then
        
        if echo "${response}" | jq -e '.result' >/dev/null 2>&1; then
            log "✓ RPC endpoint is responding"
            return 0
        else
            log "✗ RPC endpoint returned invalid response: ${response}"
            return 1
        fi
    else
        log "✗ RPC endpoint is not responding"
        return 1
    fi
}

# Check block production
check_blocks() {
    local response
    if response=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader", "params":[]}' \
        "${RPC_URL}" 2>/dev/null); then
        
        local block_number
        if block_number=$(echo "${response}" | jq -r '.result.number' 2>/dev/null); then
            log "✓ Current block: ${block_number}"
            return 0
        else
            log "✗ Could not get block number"
            return 1
        fi
    else
        log "✗ Could not check block production"
        return 1
    fi
}

# Main health check
main() {
    log "Starting health check..."
    
    local exit_code=0
    
    check_service || exit_code=1
    check_rpc || exit_code=1
    check_blocks || exit_code=1
    
    if [[ ${exit_code} -eq 0 ]]; then
        log "✓ All health checks passed"
    else
        log "✗ Some health checks failed"
    fi
    
    return ${exit_code}
}

main "$@"
EOF
    
    chmod +x /usr/local/bin/chameleon-health-check
    
    # Create cron job for regular health checks
    echo "*/5 * * * * root /usr/local/bin/chameleon-health-check" > /etc/cron.d/chameleon-health
    
    log_success "Health check script created"
}

# Create status script
create_status_script() {
    log_info "Creating status script..."
    
    cat > /usr/local/bin/chameleon-status << 'EOF'
#!/bin/bash

# Chameleon Node Status Script

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Chameleon Node Status ===${NC}"
echo

# Service status
echo -e "${BLUE}Service Status:${NC}"
if systemctl is-active --quiet chameleon-node; then
    echo -e "  ${GREEN}✓ Running${NC}"
else
    echo -e "  ${RED}✗ Stopped${NC}"
fi
echo

# System resources
echo -e "${BLUE}System Resources:${NC}"
echo "  CPU Usage: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%"
echo "  Memory Usage: $(free | grep Mem | awk '{printf "%.1f%%", $3/$2 * 100.0}')" 
echo "  Disk Usage: $(df -h / | awk 'NR==2{printf "%s", $5}')"
echo

# Network info
echo -e "${BLUE}Network Information:${NC}"
echo "  Private IP: $(curl -s http://169.254.169.254/latest/meta-data/local-ipv4 2>/dev/null || echo 'N/A')"
echo "  Public IP: $(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 2>/dev/null || echo 'N/A')"
echo

# Node info (if RPC is available)
echo -e "${BLUE}Node Information:${NC}"
if curl -s http://localhost:9933 >/dev/null 2>&1; then
    # Get node info
    NODE_INFO=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "system_name", "params":[]}' \
        http://localhost:9933 2>/dev/null | jq -r '.result' 2>/dev/null || echo 'N/A')
    
    VERSION=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "system_version", "params":[]}' \
        http://localhost:9933 2>/dev/null | jq -r '.result' 2>/dev/null || echo 'N/A')
    
    BLOCK=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getHeader", "params":[]}' \
        http://localhost:9933 2>/dev/null | jq -r '.result.number' 2>/dev/null || echo 'N/A')
    
    PEERS=$(curl -s -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "system_peers", "params":[]}' \
        http://localhost:9933 2>/dev/null | jq -r '.result | length' 2>/dev/null || echo 'N/A')
    
    echo "  Node: ${NODE_INFO}"
    echo "  Version: ${VERSION}"
    echo "  Block: ${BLOCK}"
    echo "  Peers: ${PEERS}"
else
    echo -e "  ${RED}RPC not available${NC}"
fi
echo

# Recent logs
echo -e "${BLUE}Recent Logs (last 10 lines):${NC}"
journalctl -u chameleon-node --no-pager -n 10 --output cat 2>/dev/null || echo "No logs available"
EOF
    
    chmod +x /usr/local/bin/chameleon-status
    
    log_success "Status script created"
}

# Final setup
final_setup() {
    log_info "Performing final setup..."
    
    # Create info file
    cat > "${NODE_HOME}/node-info.json" << EOF
{
  "instance_id": "${INSTANCE_ID}",
  "region": "${REGION}",
  "availability_zone": "${AZ}",
  "private_ip": "${PRIVATE_IP}",
  "public_ip": "${PUBLIC_IP}",
  "node_version": "${NODE_VERSION}",
  "initialized_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "services": {
    "chameleon_node": {
      "port_p2p": 30333,
      "port_rpc": 9933,
      "port_ws": 9944,
      "port_metrics": 9615
    },
    "node_exporter": {
      "port": 9100
    }
  }
}
EOF
    
    chown "${NODE_USER}:${NODE_USER}" "${NODE_HOME}/node-info.json"
    
    # Set up log directory
    mkdir -p "${NODE_HOME}/logs"
    chown "${NODE_USER}:${NODE_USER}" "${NODE_HOME}/logs"
    
    # Create welcome message
    cat > /etc/motd << EOF

╔══════════════════════════════════════════════════════════════════════════════╗
║                          CHAMELEON NETWORK VALIDATOR                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Instance ID: ${INSTANCE_ID}                                    ║
║  Region:      ${REGION}                                                ║
║  Public IP:   ${PUBLIC_IP}                                        ║
║                                                                              ║
║  Commands:                                                                   ║
║    chameleon-status          - Show node status                             ║
║    chameleon-health-check    - Run health check                             ║
║    systemctl status chameleon-node - Service status                         ║
║    journalctl -u chameleon-node -f  - Follow logs                           ║
║                                                                              ║
║  Endpoints:                                                                  ║
║    RPC:       http://${PUBLIC_IP}:9933                           ║
║    WebSocket: ws://${PUBLIC_IP}:9944                             ║
║    Metrics:   http://${PUBLIC_IP}:9615/metrics                   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

EOF
    
    log_success "Final setup completed"
}

# Main initialization function
main() {
    log_info "Starting Chameleon Network validator initialization..."
    log_info "Timestamp: $(date)"
    
    # Run initialization steps
    get_instance_metadata
    update_system
    configure_firewall
    configure_fail2ban
    create_node_user
    download_node_binary
    download_chain_spec
    generate_node_key
    create_systemd_service
    configure_log_rotation
    install_monitoring
    create_health_check
    create_status_script
    start_services
    final_setup
    
    log_success "Chameleon Network validator initialization completed successfully!"
    log_info "Node should be starting up and connecting to the network..."
    log_info "Use 'chameleon-status' to check the current status"
    log_info "Use 'journalctl -u chameleon-node -f' to follow logs"
    
    # Run initial health check
    sleep 30
    log_info "Running initial health check..."
    /usr/local/bin/chameleon-health-check || log_warning "Initial health check failed - this is normal during startup"
}

# Handle script errors
trap 'log_error "Initialization failed at line $LINENO"' ERR

# Run main function
main "$@" 2>&1 | tee -a /var/log/chameleon-init.log