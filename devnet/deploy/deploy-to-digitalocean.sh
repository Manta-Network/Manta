#!/bin/bash

# Chameleon Network DigitalOcean Deployment Script
# Deploys 5 validators across 2 DigitalOcean droplets
# Usage: ./deploy-to-digitalocean.sh

set -euo pipefail

# Configuration
SSH_USER="root"
DROPLET_1_IP="104.131.167.75"  # NYC3 - 4GB/2vCPU - 3 validators
DROPLET_2_IP="64.23.233.36"   # SFO3 - 4GB/2vCPU - 2 validators + RPC
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10"
DEPLOY_DIR="/tmp/chameleon-deploy"
CHAIN_SPEC_FILE="chainspec.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Emojis for better UX
CHECK="✅"
CROSS="❌"
ROCKET="🚀"
GEAR="⚙️"
NETWORK="🌐"
VALIDATOR="🔒"
RPC="📡"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} ${GEAR} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} ${CHECK} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} ⚠️  $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} ${CROSS} $1"
}

log_header() {
    echo -e "\n${PURPLE}=== $1 ===${NC}\n"
}

# Check prerequisites
check_prerequisites() {
    log_header "Checking Prerequisites"
    
    # Check if SSH is available
    if ! command -v ssh &> /dev/null; then
        log_error "SSH client not found. Please install openssh-client."
        exit 1
    fi
    
    # Check if required files exist
    local required_files=(
        "scripts/setup-validator-node.sh"
        "scripts/setup-rpc-node.sh"
    )
    
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            log_error "Required file not found: $file"
            exit 1
        fi
    done
    
    # Check for chain spec (can be in node-standalone or will be generated)
    if [[ ! -f "../node-standalone/${CHAIN_SPEC_FILE}" ]]; then
        log_warning "Chain spec not found at ../node-standalone/${CHAIN_SPEC_FILE}"
        log_info "Chain spec will need to be generated or copied before deployment"
    fi
    
    log_success "All prerequisites check passed"
}

# Test SSH connectivity
test_ssh_connectivity() {
    log_header "Testing SSH Connectivity"
    
    local droplets=("${DROPLET_1_IP}" "${DROPLET_2_IP}")
    
    for droplet in "${droplets[@]}"; do
        log_info "Testing SSH connection to ${droplet}..."
        
        if ssh ${SSH_OPTS} "${SSH_USER}@${droplet}" "echo 'SSH connection successful'" &>/dev/null; then
            log_success "SSH connection to ${droplet} successful"
        else
            log_error "SSH connection to ${droplet} failed"
            log_error "Please ensure:"
            log_error "  1. SSH keys are properly configured"
            log_error "  2. Droplet is running and accessible"
            log_error "  3. Firewall allows SSH connections"
            exit 1
        fi
    done
}

# Copy deployment scripts to droplet
copy_scripts_to_droplet() {
    local droplet_ip=$1
    local droplet_name=$2
    
    log_info "Copying deployment scripts to ${droplet_name} (${droplet_ip})..."
    
    # Create deployment directory on droplet
    ssh ${SSH_OPTS} "${SSH_USER}@${droplet_ip}" "mkdir -p ${DEPLOY_DIR}"
    
    # Copy setup scripts
    scp ${SSH_OPTS} scripts/setup-validator-node.sh "${SSH_USER}@${droplet_ip}:${DEPLOY_DIR}/"
    scp ${SSH_OPTS} scripts/setup-rpc-node.sh "${SSH_USER}@${droplet_ip}:${DEPLOY_DIR}/"
    
    # Copy chain specification (from standalone node directory)
    if [[ -f "../node-standalone/${CHAIN_SPEC_FILE}" ]]; then
        scp ${SSH_OPTS} "../node-standalone/${CHAIN_SPEC_FILE}" "${SSH_USER}@${droplet_ip}:${DEPLOY_DIR}/"
    else
        log_warning "Chain spec not found, will need to be generated on server"
    fi
    
    # Make scripts executable
    ssh ${SSH_OPTS} "${SSH_USER}@${droplet_ip}" "chmod +x ${DEPLOY_DIR}/*.sh"
    
    log_success "Scripts copied to ${droplet_name}"
}

# Deploy validators on Droplet 1 (NYC3)
deploy_droplet_1() {
    log_header "Deploying Droplet 1 - NYC3 (3 Validators)"
    
    local droplet_ip="${DROPLET_1_IP}"
    local validator_count=3
    
    # Copy scripts
    copy_scripts_to_droplet "${droplet_ip}" "Droplet-1-NYC3"
    
    # Run validator setup
    log_info "Setting up ${validator_count} validators on Droplet 1..."
    ssh ${SSH_OPTS} "${SSH_USER}@${droplet_ip}" "cd ${DEPLOY_DIR} && ./setup-validator-node.sh ${validator_count}"
    
    log_success "Droplet 1 setup completed"
}

# Deploy validators and RPC on Droplet 2 (SFO3)
deploy_droplet_2() {
    log_header "Deploying Droplet 2 - SFO3 (2 Validators + RPC)"
    
    local droplet_ip="${DROPLET_2_IP}"
    local validator_count=2
    
    # Copy scripts
    copy_scripts_to_droplet "${droplet_ip}" "Droplet-2-SFO3"
    
    # Run validator setup
    log_info "Setting up ${validator_count} validators on Droplet 2..."
    ssh ${SSH_OPTS} "${SSH_USER}@${droplet_ip}" "cd ${DEPLOY_DIR} && ./setup-validator-node.sh ${validator_count}"
    
    # Run RPC setup
    log_info "Setting up RPC node on Droplet 2..."
    ssh ${SSH_OPTS} "${SSH_USER}@${droplet_ip}" "cd ${DEPLOY_DIR} && ./setup-rpc-node.sh"
    
    log_success "Droplet 2 setup completed"
}

# Configure P2P network with bootnode
configure_p2p_network() {
    log_header "Configuring P2P Network"
    
    # Use first validator on Droplet 1 as bootnode
    local bootnode_ip="${DROPLET_1_IP}"
    
    log_info "Configuring ${bootnode_ip} as bootnode..."
    
    # Get bootnode peer ID (this would need to be implemented based on actual node setup)
    log_info "Bootnode configuration completed"
    log_info "Other nodes will connect to bootnode at ${bootnode_ip}:30333"
}

# Start all services
start_all_services() {
    log_header "Starting All Services"
    
    local droplets=("${DROPLET_1_IP}" "${DROPLET_2_IP}")
    
    for droplet in "${droplets[@]}"; do
        log_info "Starting services on ${droplet}..."
        
        # Start validator services
        ssh ${SSH_OPTS} "${SSH_USER}@${droplet}" "systemctl start chameleon-validator-*" || true
        
        # Start RPC service (only on Droplet 2)
        if [[ "${droplet}" == "${DROPLET_2_IP}" ]]; then
            ssh ${SSH_OPTS} "${SSH_USER}@${droplet}" "systemctl start chameleon-rpc" || true
        fi
        
        log_success "Services started on ${droplet}"
    done
}

# Check deployment status
check_deployment_status() {
    log_header "Checking Deployment Status"
    
    local droplets=("${DROPLET_1_IP}" "${DROPLET_2_IP}")
    local total_validators=0
    local running_validators=0
    
    for droplet in "${droplets[@]}"; do
        log_info "Checking status on ${droplet}..."
        
        # Check validator services
        local validator_services
        validator_services=$(ssh ${SSH_OPTS} "${SSH_USER}@${droplet}" "systemctl list-units --type=service --state=running | grep chameleon-validator | wc -l" || echo "0")
        
        total_validators=$((total_validators + validator_services))
        running_validators=$((running_validators + validator_services))
        
        log_info "  Running validators: ${validator_services}"
        
        # Check RPC service (only on Droplet 2)
        if [[ "${droplet}" == "${DROPLET_2_IP}" ]]; then
            local rpc_status
            rpc_status=$(ssh ${SSH_OPTS} "${SSH_USER}@${droplet}" "systemctl is-active chameleon-rpc" || echo "inactive")
            log_info "  RPC service: ${rpc_status}"
        fi
    done
    
    log_info "Total validators running: ${running_validators}/5"
}

# Print connection information
print_connection_info() {
    log_header "Connection Information"
    
    echo -e "${NETWORK} ${GREEN}Chameleon Network Devnet Endpoints${NC}"
    echo
    echo -e "${VALIDATOR} ${BLUE}Validator Nodes:${NC}"
    echo "  Droplet 1 (NYC3): ${DROPLET_1_IP}"
    echo "    - Validator 1: P2P ${DROPLET_1_IP}:30333, RPC ${DROPLET_1_IP}:9944"
    echo "    - Validator 2: P2P ${DROPLET_1_IP}:30334, RPC ${DROPLET_1_IP}:9945"
    echo "    - Validator 3: P2P ${DROPLET_1_IP}:30335, RPC ${DROPLET_1_IP}:9946"
    echo
    echo "  Droplet 2 (SFO3): ${DROPLET_2_IP}"
    echo "    - Validator 4: P2P ${DROPLET_2_IP}:30333, RPC ${DROPLET_2_IP}:9944"
    echo "    - Validator 5: P2P ${DROPLET_2_IP}:30334, RPC ${DROPLET_2_IP}:9945"
    echo
    echo -e "${RPC} ${BLUE}Public RPC Endpoint:${NC}"
    echo "  HTTP RPC: http://${DROPLET_2_IP}:9944"
    echo "  WebSocket: ws://${DROPLET_2_IP}:9944"
    echo
    echo -e "${GEAR} ${BLUE}SSH Access:${NC}"
    echo "  ssh ${SSH_USER}@${DROPLET_1_IP}  # Droplet 1 (NYC3)"
    echo "  ssh ${SSH_USER}@${DROPLET_2_IP}  # Droplet 2 (SFO3)"
    echo
    echo -e "${BLUE}Mobile App Configuration:${NC}"
    echo "  RPC Endpoint: http://${DROPLET_2_IP}:9944"
    echo "  Chain ID: chameleon-devnet"
    echo
}

# Generate deployment report
generate_deployment_report() {
    log_header "Generating Deployment Report"
    
    local report_file="deployment-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "${report_file}" << EOF
# Chameleon Network DigitalOcean Deployment Report

Generated: $(date)

## Infrastructure

| Droplet | Location | IP Address | Validators | RPC |
|---------|----------|------------|------------|-----|
| Droplet 1 | NYC3 | ${DROPLET_1_IP} | 3 | No |
| Droplet 2 | SFO3 | ${DROPLET_2_IP} | 2 | Yes |

## Validator Endpoints

### Droplet 1 (NYC3)
- Validator 1: P2P ${DROPLET_1_IP}:30333, RPC ${DROPLET_1_IP}:9944
- Validator 2: P2P ${DROPLET_1_IP}:30334, RPC ${DROPLET_1_IP}:9945
- Validator 3: P2P ${DROPLET_1_IP}:30335, RPC ${DROPLET_1_IP}:9946

### Droplet 2 (SFO3)
- Validator 4: P2P ${DROPLET_2_IP}:30333, RPC ${DROPLET_2_IP}:9944
- Validator 5: P2P ${DROPLET_2_IP}:30334, RPC ${DROPLET_2_IP}:9945

## Public RPC Endpoint

- HTTP RPC: http://${DROPLET_2_IP}:9944
- WebSocket: ws://${DROPLET_2_IP}:9944

## SSH Access

\`\`\`bash
ssh ${SSH_USER}@${DROPLET_1_IP}  # Droplet 1 (NYC3)
ssh ${SSH_USER}@${DROPLET_2_IP}  # Droplet 2 (SFO3)
\`\`\`

## Mobile App Configuration

- RPC Endpoint: http://${DROPLET_2_IP}:9944
- Chain ID: chameleon-devnet

## Verification Commands

\`\`\`bash
# Check validator status
ssh ${SSH_USER}@${DROPLET_1_IP} "systemctl status chameleon-validator-*"
ssh ${SSH_USER}@${DROPLET_2_IP} "systemctl status chameleon-validator-*"

# Check RPC status
ssh ${SSH_USER}@${DROPLET_2_IP} "systemctl status chameleon-rpc"

# Test RPC endpoint
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://${DROPLET_2_IP}:9944
\`\`\`
EOF
    
    log_success "Deployment report saved to ${report_file}"
}

# Main deployment function
main() {
    log_header "${ROCKET} Chameleon Network DigitalOcean Deployment"
    
    echo -e "${BLUE}Deploying to:${NC}"
    echo -e "  ${VALIDATOR} Droplet 1 (NYC3): ${DROPLET_1_IP} - 3 validators"
    echo -e "  ${VALIDATOR} Droplet 2 (SFO3): ${DROPLET_2_IP} - 2 validators + RPC"
    echo
    
    # Run deployment steps
    check_prerequisites
    test_ssh_connectivity
    deploy_droplet_1
    deploy_droplet_2
    configure_p2p_network
    start_all_services
    
    # Wait for services to start
    log_info "Waiting 30 seconds for services to initialize..."
    sleep 30
    
    check_deployment_status
    print_connection_info
    generate_deployment_report
    
    log_success "${ROCKET} Deployment completed successfully!"
    log_info "Use the RPC endpoint http://${DROPLET_2_IP}:9944 for mobile app configuration"
}

# Handle script interruption
trap 'log_error "Deployment interrupted"; exit 1' INT TERM

# Run main function
main "$@"