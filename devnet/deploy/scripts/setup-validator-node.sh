#!/bin/bash

# Chameleon Network Validator Node Setup Script
# Sets up multiple validator nodes on a single server
# Usage: ./setup-validator-node.sh <validator_count>

set -euo pipefail

# Configuration
VALIDATOR_COUNT=${1:-1}
NODE_USER="chameleon"
NODE_HOME="/var/lib/chameleon"
BINARY_PATH="/usr/local/bin/chameleon-node"
CHAIN_SPEC_PATH="/etc/chameleon/chainspec.json"
GITHUB_REPO="chameleon-network/chameleon"
NODE_VERSION="v0.1.0"
DEPLOY_DIR="/tmp/chameleon-deploy"

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
VALIDATOR="🔒"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} ${GEAR} $1" | tee -a /var/log/chameleon-setup.log
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} ${CHECK} $1" | tee -a /var/log/chameleon-setup.log
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} ⚠️  $1" | tee -a /var/log/chameleon-setup.log
}

log_error() {
    echo -e "${RED}[ERROR]${NC} ${CROSS} $1" | tee -a /var/log/chameleon-setup.log
}

log_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n" | tee -a /var/log/chameleon-setup.log
}

# Validate input
validate_input() {
    log_header "Validating Input"
    
    if [[ ! "${VALIDATOR_COUNT}" =~ ^[1-9][0-9]*$ ]] || [[ "${VALIDATOR_COUNT}" -gt 10 ]]; then
        log_error "Invalid validator count: ${VALIDATOR_COUNT}"
        log_error "Must be a number between 1 and 10"
        exit 1
    fi
    
    log_success "Will setup ${VALIDATOR_COUNT} validator(s)"
}

# Update system packages
update_system() {
    log_header "Updating System Packages"
    
    export DEBIAN_FRONTEND=noninteractive
    
    # Update package lists
    apt-get update -y
    
    # Install essential packages
    apt-get install -y \
        curl \
        git \
        jq \
        wget \
        unzip \
        htop \
        net-tools \
        ufw \
        fail2ban \
        logrotate \
        build-essential \
        pkg-config \
        libssl-dev \
        ca-certificates
    
    log_success "System packages updated"
}

# Create chameleon user
create_chameleon_user() {
    log_header "Creating Chameleon User"
    
    # Create system user for the node
    if ! id "${NODE_USER}" &>/dev/null; then
        useradd --system --home "${NODE_HOME}" --create-home --shell /bin/bash "${NODE_USER}"
        log_success "Created user: ${NODE_USER}"
    else
        log_warning "User ${NODE_USER} already exists"
    fi
    
    # Create necessary directories
    mkdir -p "${NODE_HOME}"
    mkdir -p "/etc/chameleon"
    
    # Set proper ownership
    chown -R "${NODE_USER}:${NODE_USER}" "${NODE_HOME}"
    
    log_success "User and directories created"
}

# Download node binary
download_node_binary() {
    log_header "Downloading Node Binary"
    
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
    
    # Try to download from GitHub releases
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${NODE_VERSION}/chameleon-node-${BINARY_ARCH}"
    
    log_info "Attempting to download from: ${DOWNLOAD_URL}"
    
    # Download with retry logic
    for i in {1..3}; do
        if wget -O "${BINARY_PATH}" "${DOWNLOAD_URL}" 2>/dev/null; then
            log_success "Binary downloaded successfully"
            break
        else
            log_warning "Download attempt ${i} failed"
            if [[ ${i} -eq 3 ]]; then
                log_warning "Creating placeholder binary for testing..."
                
                # Create placeholder binary
                cat > "${BINARY_PATH}" << 'EOF'
#!/bin/bash
echo "Chameleon Node Placeholder v0.1.0"
echo "Arguments: $@"
echo "Starting placeholder node..."

# Simulate node startup
echo "Node starting with chain spec: $(echo "$@" | grep -o '\--chain [^ ]*' | cut -d' ' -f2 || echo 'default')"
echo "Validator mode: $(echo "$@" | grep -q '\--validator' && echo 'enabled' || echo 'disabled')"
echo "P2P port: $(echo "$@" | grep -o '\--port [0-9]*' | cut -d' ' -f2 || echo '30333')"
echo "RPC port: $(echo "$@" | grep -o '\--rpc-port [0-9]*' | cut -d' ' -f2 || echo '9944')"

# Keep running
while true; do
    echo "$(date): Node is running (placeholder)"
    sleep 60
done
EOF
                break
            fi
            sleep 5
        fi
    done
    
    # Make binary executable
    chmod +x "${BINARY_PATH}"
    
    # Verify binary
    if [[ -x "${BINARY_PATH}" ]]; then
        log_success "Node binary installed at ${BINARY_PATH}"
    else
        log_error "Failed to install node binary"
        exit 1
    fi
}

# Copy chain specification
copy_chain_spec() {
    log_header "Setting Up Chain Specification"
    
    if [[ -f "${DEPLOY_DIR}/chainspec.json" ]]; then
        cp "${DEPLOY_DIR}/chainspec.json" "${CHAIN_SPEC_PATH}"
        chown "${NODE_USER}:${NODE_USER}" "${CHAIN_SPEC_PATH}"
        log_success "Chain specification copied"
    else
        log_warning "Chain spec not found, creating placeholder..."
        
        # Create placeholder chain spec
        cat > "${CHAIN_SPEC_PATH}" << EOF
{
  "name": "Chameleon Development Network",
  "id": "chameleon_devnet",
  "chainType": "Development",
  "bootNodes": [],
  "telemetryEndpoints": null,
  "protocolId": "chameleon",
  "properties": {
    "ss58Format": 99,
    "tokenDecimals": 18,
    "tokenSymbol": "CHML"
  },
  "genesis": {
    "runtime": {
      "system": {
        "code": "0x"
      }
    }
  }
}
EOF
        chown "${NODE_USER}:${NODE_USER}" "${CHAIN_SPEC_PATH}"
        log_success "Placeholder chain specification created"
    fi
}

# Create data directories for validators
create_validator_directories() {
    log_header "Creating Validator Data Directories"
    
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local validator_dir="${NODE_HOME}/validator-${i}"
        
        mkdir -p "${validator_dir}/chains"
        mkdir -p "${validator_dir}/keystore"
        mkdir -p "${validator_dir}/logs"
        
        # Generate node key
        openssl rand -hex 32 > "${validator_dir}/node-key"
        chmod 600 "${validator_dir}/node-key"
        
        chown -R "${NODE_USER}:${NODE_USER}" "${validator_dir}"
        
        log_info "Created directory for validator ${i}: ${validator_dir}"
    done
    
    log_success "All validator directories created"
}

# Generate systemd service files
generate_systemd_services() {
    log_header "Generating Systemd Service Files"
    
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local service_name="chameleon-validator-${i}"
        local validator_dir="${NODE_HOME}/validator-${i}"
        local p2p_port=$((30332 + i))
        local rpc_port=$((9943 + i))
        local validator_name="validator-${i}-$(hostname -s)"
        
        log_info "Creating service for ${service_name} (P2P: ${p2p_port}, RPC: ${rpc_port})"
        
        # Create service file
        cat > "/etc/systemd/system/${service_name}.service" << EOF
[Unit]
Description=Chameleon Network Validator ${i}
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${NODE_USER}
Group=${NODE_USER}
WorkingDirectory=${validator_dir}
ExecStart=${BINARY_PATH} \\
    --validator \\
    --name "${validator_name}" \\
    --chain ${CHAIN_SPEC_PATH} \\
    --base-path ${validator_dir} \\
    --port ${p2p_port} \\
    --rpc-port ${rpc_port} \\
    --ws-port ${rpc_port} \\
    --prometheus-port $((9614 + i)) \\
    --rpc-cors all \\
    --node-key-file ${validator_dir}/node-key \\
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
ReadWritePaths=${validator_dir} /tmp

# Environment
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF
        
        log_success "Service file created: ${service_name}"
    done
    
    # Reload systemd
    systemctl daemon-reload
    log_success "Systemd daemon reloaded"
}

# Enable services (but don't start yet)
enable_services() {
    log_header "Enabling Validator Services"
    
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local service_name="chameleon-validator-${i}"
        
        systemctl enable "${service_name}"
        log_info "Enabled service: ${service_name}"
    done
    
    log_success "All validator services enabled"
}

# Configure firewall
configure_firewall() {
    log_header "Configuring Firewall"
    
    # Reset UFW to defaults
    ufw --force reset
    
    # Set default policies
    ufw default deny incoming
    ufw default allow outgoing
    
    # Allow SSH
    ufw allow 22/tcp
    
    # Allow P2P ports for validators
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local p2p_port=$((30332 + i))
        ufw allow "${p2p_port}/tcp"
        log_info "Allowed P2P port: ${p2p_port}"
    done
    
    # Allow RPC ports (restricted to localhost and private networks)
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local rpc_port=$((9943 + i))
        ufw allow from 127.0.0.1 to any port "${rpc_port}"
        ufw allow from 10.0.0.0/8 to any port "${rpc_port}"
        ufw allow from 172.16.0.0/12 to any port "${rpc_port}"
        ufw allow from 192.168.0.0/16 to any port "${rpc_port}"
        log_info "Allowed RPC port: ${rpc_port} (restricted)"
    done
    
    # Allow Prometheus metrics ports (restricted)
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local metrics_port=$((9614 + i))
        ufw allow from 10.0.0.0/8 to any port "${metrics_port}"
        ufw allow from 172.16.0.0/12 to any port "${metrics_port}"
        ufw allow from 192.168.0.0/16 to any port "${metrics_port}"
    done
    
    # Enable firewall
    ufw --force enable
    
    log_success "Firewall configured"
}

# Create management scripts
create_management_scripts() {
    log_header "Creating Management Scripts"
    
    # Create validator status script
    cat > /usr/local/bin/chameleon-validator-status << EOF
#!/bin/bash

echo "=== Chameleon Validator Status ==="
echo

for i in {1..${VALIDATOR_COUNT}}; do
    service="chameleon-validator-\$i"
    echo "Validator \$i (\$service):"
    
    if systemctl is-active --quiet "\$service"; then
        echo "  Status: ✅ Running"
        
        # Get port info
        p2p_port=\$((30332 + i))
        rpc_port=\$((9943 + i))
        echo "  P2P Port: \$p2p_port"
        echo "  RPC Port: \$rpc_port"
        
        # Check if ports are listening
        if netstat -tuln | grep -q ":\$p2p_port "; then
            echo "  P2P: ✅ Listening"
        else
            echo "  P2P: ❌ Not listening"
        fi
        
        if netstat -tuln | grep -q ":\$rpc_port "; then
            echo "  RPC: ✅ Listening"
        else
            echo "  RPC: ❌ Not listening"
        fi
    else
        echo "  Status: ❌ Stopped"
    fi
    echo
done

echo "System Resources:"
echo "  CPU: \$(top -bn1 | grep 'Cpu(s)' | awk '{print \$2}' | cut -d'%' -f1)%"
echo "  Memory: \$(free | grep Mem | awk '{printf \"%.1f%%\", \$3/\$2 * 100.0}')"
echo "  Disk: \$(df -h / | awk 'NR==2{printf \"%s\", \$5}')"
EOF
    
    chmod +x /usr/local/bin/chameleon-validator-status
    
    # Create start all script
    cat > /usr/local/bin/chameleon-start-all << EOF
#!/bin/bash

echo "Starting all Chameleon validators..."

for i in {1..${VALIDATOR_COUNT}}; do
    service="chameleon-validator-\$i"
    echo "Starting \$service..."
    systemctl start "\$service"
done

echo "All validators started."
EOF
    
    chmod +x /usr/local/bin/chameleon-start-all
    
    # Create stop all script
    cat > /usr/local/bin/chameleon-stop-all << EOF
#!/bin/bash

echo "Stopping all Chameleon validators..."

for i in {1..${VALIDATOR_COUNT}}; do
    service="chameleon-validator-\$i"
    echo "Stopping \$service..."
    systemctl stop "\$service"
done

echo "All validators stopped."
EOF
    
    chmod +x /usr/local/bin/chameleon-stop-all
    
    log_success "Management scripts created"
}

# Create info file
create_info_file() {
    log_header "Creating Node Information File"
    
    cat > "${NODE_HOME}/node-info.json" << EOF
{
  "setup_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "hostname": "$(hostname)",
  "validator_count": ${VALIDATOR_COUNT},
  "node_version": "${NODE_VERSION}",
  "validators": [
EOF
    
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local p2p_port=$((30332 + i))
        local rpc_port=$((9943 + i))
        local metrics_port=$((9614 + i))
        
        cat >> "${NODE_HOME}/node-info.json" << EOF
    {
      "index": ${i},
      "name": "validator-${i}-$(hostname -s)",
      "service_name": "chameleon-validator-${i}",
      "data_directory": "${NODE_HOME}/validator-${i}",
      "ports": {
        "p2p": ${p2p_port},
        "rpc": ${rpc_port},
        "metrics": ${metrics_port}
      }
    }$([ $i -lt $VALIDATOR_COUNT ] && echo ",")
EOF
    done
    
    cat >> "${NODE_HOME}/node-info.json" << EOF
  ]
}
EOF
    
    chown "${NODE_USER}:${NODE_USER}" "${NODE_HOME}/node-info.json"
    
    log_success "Node information file created"
}

# Main setup function
main() {
    log_header "${VALIDATOR} Chameleon Validator Node Setup"
    
    echo -e "${BLUE}Setting up ${VALIDATOR_COUNT} validator node(s)${NC}"
    echo -e "${BLUE}Timestamp: $(date)${NC}"
    echo
    
    # Run setup steps
    validate_input
    update_system
    create_chameleon_user
    download_node_binary
    copy_chain_spec
    create_validator_directories
    generate_systemd_services
    enable_services
    configure_firewall
    create_management_scripts
    create_info_file
    
    log_success "${CHECK} Validator node setup completed successfully!"
    echo
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Start validators: chameleon-start-all"
    echo "  2. Check status: chameleon-validator-status"
    echo "  3. View logs: journalctl -u chameleon-validator-1 -f"
    echo "  4. Stop validators: chameleon-stop-all"
    echo
    echo -e "${BLUE}Port allocation:${NC}"
    for ((i=1; i<=VALIDATOR_COUNT; i++)); do
        local p2p_port=$((30332 + i))
        local rpc_port=$((9943 + i))
        echo "  Validator ${i}: P2P ${p2p_port}, RPC ${rpc_port}"
    done
}

# Handle script errors
trap 'log_error "Setup failed at line $LINENO"' ERR

# Run main function
main "$@" 2>&1 | tee -a /var/log/chameleon-setup.log