#!/bin/bash

# Chameleon Network AWS Deployment Script
# This script deploys 5 validator instances across AWS regions
# Usage: ./aws-deployment.sh [environment]

set -euo pipefail

# Configuration
ENVIRONMENT=${1:-devnet}
PROJECT_NAME="chameleon-network"
KEY_NAME="chameleon-${ENVIRONMENT}"
INSTANCE_TYPE="t3.large"
AMI_ID="ami-0c02fb55956c7d316"  # Ubuntu 22.04 LTS (us-east-1)
SECURITY_GROUP="chameleon-validators"
SUBNET_PREFIX="chameleon-${ENVIRONMENT}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validator configuration
declare -A VALIDATORS=(
    ["validator-0"]="us-east-1:Validator-US-East-1"
    ["validator-1"]="us-west-2:Validator-US-West-1"
    ["validator-2"]="eu-west-1:Validator-EU-Central-1"
    ["validator-3"]="ap-southeast-1:Validator-Asia-Southeast-1"
    ["validator-4"]="ap-northeast-1:Validator-Asia-East-1"
)

# AMI IDs for different regions
declare -A REGION_AMIS=(
    ["us-east-1"]="ami-0c02fb55956c7d316"
    ["us-west-2"]="ami-017fecd1353bcc96e"
    ["eu-west-1"]="ami-0d71ea30463e0ff8d"
    ["ap-southeast-1"]="ami-0df7a207adb9748c7"
    ["ap-northeast-1"]="ami-03f4fa076d2981b45"
)

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check AWS CLI
    if ! command -v aws &> /dev/null; then
        log_error "AWS CLI not found. Please install it first."
        exit 1
    fi
    
    # Check AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        log_error "AWS credentials not configured. Run 'aws configure' first."
        exit 1
    fi
    
    # Check if user-data script exists
    if [[ ! -f "scripts/validator-init.sh" ]]; then
        log_error "Validator initialization script not found at scripts/validator-init.sh"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Create security group
create_security_group() {
    local region=$1
    log_info "Creating security group in ${region}..."
    
    # Get default VPC ID
    local vpc_id
    vpc_id=$(aws ec2 describe-vpcs \
        --region "${region}" \
        --filters "Name=is-default,Values=true" \
        --query 'Vpcs[0].VpcId' \
        --output text)
    
    if [[ "${vpc_id}" == "None" ]]; then
        log_error "No default VPC found in ${region}"
        return 1
    fi
    
    # Create security group
    local sg_id
    sg_id=$(aws ec2 create-security-group \
        --region "${region}" \
        --group-name "${SECURITY_GROUP}-${region}" \
        --description "Chameleon Network Validator Security Group" \
        --vpc-id "${vpc_id}" \
        --query 'GroupId' \
        --output text 2>/dev/null || echo "exists")
    
    if [[ "${sg_id}" == "exists" ]]; then
        # Get existing security group ID
        sg_id=$(aws ec2 describe-security-groups \
            --region "${region}" \
            --group-names "${SECURITY_GROUP}-${region}" \
            --query 'SecurityGroups[0].GroupId' \
            --output text)
        log_warning "Security group already exists: ${sg_id}"
    else
        log_success "Created security group: ${sg_id}"
    fi
    
    # Add security group rules
    add_security_rules "${region}" "${sg_id}"
    
    echo "${sg_id}"
}

# Add security group rules
add_security_rules() {
    local region=$1
    local sg_id=$2
    
    log_info "Adding security group rules..."
    
    # P2P port (30333)
    aws ec2 authorize-security-group-ingress \
        --region "${region}" \
        --group-id "${sg_id}" \
        --protocol tcp \
        --port 30333 \
        --cidr 0.0.0.0/0 \
        --output text &>/dev/null || true
    
    # RPC port (9933) - VPC only
    aws ec2 authorize-security-group-ingress \
        --region "${region}" \
        --group-id "${sg_id}" \
        --protocol tcp \
        --port 9933 \
        --cidr 10.0.0.0/8 \
        --output text &>/dev/null || true
    
    # WebSocket port (9944) - VPC only
    aws ec2 authorize-security-group-ingress \
        --region "${region}" \
        --group-id "${sg_id}" \
        --protocol tcp \
        --port 9944 \
        --cidr 10.0.0.0/8 \
        --output text &>/dev/null || true
    
    # Prometheus metrics (9615) - VPC only
    aws ec2 authorize-security-group-ingress \
        --region "${region}" \
        --group-id "${sg_id}" \
        --protocol tcp \
        --port 9615 \
        --cidr 10.0.0.0/8 \
        --output text &>/dev/null || true
    
    # SSH (22) - Restricted (replace with your IP)
    aws ec2 authorize-security-group-ingress \
        --region "${region}" \
        --group-id "${sg_id}" \
        --protocol tcp \
        --port 22 \
        --cidr 0.0.0.0/0 \
        --output text &>/dev/null || true
    
    log_success "Security group rules added"
}

# Create key pair
create_key_pair() {
    local region=$1
    log_info "Creating key pair in ${region}..."
    
    # Check if key pair already exists
    if aws ec2 describe-key-pairs \
        --region "${region}" \
        --key-names "${KEY_NAME}" &>/dev/null; then
        log_warning "Key pair ${KEY_NAME} already exists in ${region}"
        return 0
    fi
    
    # Create key pair and save private key
    aws ec2 create-key-pair \
        --region "${region}" \
        --key-name "${KEY_NAME}" \
        --query 'KeyMaterial' \
        --output text > "${KEY_NAME}-${region}.pem"
    
    chmod 600 "${KEY_NAME}-${region}.pem"
    log_success "Created key pair: ${KEY_NAME}"
}

# Deploy validator instance
deploy_validator() {
    local validator_name=$1
    local region_info=$2
    
    IFS=':' read -r region display_name <<< "${region_info}"
    
    log_info "Deploying ${validator_name} in ${region}..."
    
    # Create security group
    local sg_id
    sg_id=$(create_security_group "${region}")
    
    # Create key pair
    create_key_pair "${region}"
    
    # Get AMI ID for region
    local ami_id="${REGION_AMIS[${region}]}"
    
    # Encode user data script
    local user_data
    user_data=$(base64 -w 0 scripts/validator-init.sh)
    
    # Launch instance
    local instance_id
    instance_id=$(aws ec2 run-instances \
        --region "${region}" \
        --image-id "${ami_id}" \
        --count 1 \
        --instance-type "${INSTANCE_TYPE}" \
        --key-name "${KEY_NAME}" \
        --security-group-ids "${sg_id}" \
        --user-data "${user_data}" \
        --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=${display_name}},{Key=Project,Value=${PROJECT_NAME}},{Key=Environment,Value=${ENVIRONMENT}},{Key=Role,Value=validator},{Key=ValidatorIndex,Value=${validator_name##*-}}]" \
        --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":100,"VolumeType":"gp3","DeleteOnTermination":true}}]' \
        --query 'Instances[0].InstanceId' \
        --output text)
    
    if [[ -z "${instance_id}" ]]; then
        log_error "Failed to launch instance for ${validator_name}"
        return 1
    fi
    
    log_success "Launched ${validator_name}: ${instance_id}"
    
    # Wait for instance to be running
    log_info "Waiting for ${validator_name} to be running..."
    aws ec2 wait instance-running \
        --region "${region}" \
        --instance-ids "${instance_id}"
    
    # Get public IP
    local public_ip
    public_ip=$(aws ec2 describe-instances \
        --region "${region}" \
        --instance-ids "${instance_id}" \
        --query 'Reservations[0].Instances[0].PublicIpAddress' \
        --output text)
    
    log_success "${validator_name} is running at ${public_ip}"
    
    # Save instance information
    echo "${validator_name},${region},${instance_id},${public_ip}" >> "deployment-${ENVIRONMENT}.csv"
}

# Deploy all validators
deploy_all_validators() {
    log_info "Starting deployment of all validators..."
    
    # Create deployment log file
    echo "validator_name,region,instance_id,public_ip" > "deployment-${ENVIRONMENT}.csv"
    
    # Deploy each validator
    for validator_name in "${!VALIDATORS[@]}"; do
        deploy_validator "${validator_name}" "${VALIDATORS[${validator_name}]}"
        sleep 10  # Brief pause between deployments
    done
    
    log_success "All validators deployed successfully!"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    if [[ ! -f "deployment-${ENVIRONMENT}.csv" ]]; then
        log_error "Deployment log file not found"
        return 1
    fi
    
    local total_validators=0
    local running_validators=0
    
    while IFS=',' read -r validator_name region instance_id public_ip; do
        if [[ "${validator_name}" == "validator_name" ]]; then
            continue  # Skip header
        fi
        
        total_validators=$((total_validators + 1))
        
        # Check instance status
        local state
        state=$(aws ec2 describe-instances \
            --region "${region}" \
            --instance-ids "${instance_id}" \
            --query 'Reservations[0].Instances[0].State.Name' \
            --output text)
        
        if [[ "${state}" == "running" ]]; then
            running_validators=$((running_validators + 1))
            log_success "${validator_name}: ${state} (${public_ip})"
        else
            log_warning "${validator_name}: ${state}"
        fi
    done < "deployment-${ENVIRONMENT}.csv"
    
    log_info "Deployment summary: ${running_validators}/${total_validators} validators running"
    
    if [[ ${running_validators} -eq ${total_validators} ]]; then
        log_success "All validators are running successfully!"
        return 0
    else
        log_error "Some validators are not running properly"
        return 1
    fi
}

# Generate connection information
generate_connection_info() {
    log_info "Generating connection information..."
    
    cat > "connection-info-${ENVIRONMENT}.md" << EOF
# Chameleon Network ${ENVIRONMENT} Connection Information

Generated on: $(date)

## Validator Endpoints

| Validator | Region | Public IP | RPC Endpoint | WebSocket Endpoint |
|-----------|--------|-----------|--------------|--------------------|
EOF
    
    while IFS=',' read -r validator_name region instance_id public_ip; do
        if [[ "${validator_name}" == "validator_name" ]]; then
            continue  # Skip header
        fi
        
        echo "| ${validator_name} | ${region} | ${public_ip} | http://${public_ip}:9933 | ws://${public_ip}:9944 |" >> "connection-info-${ENVIRONMENT}.md"
    done < "deployment-${ENVIRONMENT}.csv"
    
    cat >> "connection-info-${ENVIRONMENT}.md" << EOF

## SSH Access

\`\`\`bash
# Connect to validators (replace with actual IPs)
EOF
    
    while IFS=',' read -r validator_name region instance_id public_ip; do
        if [[ "${validator_name}" == "validator_name" ]]; then
            continue  # Skip header
        fi
        
        echo "ssh -i ${KEY_NAME}-${region}.pem ubuntu@${public_ip}  # ${validator_name}" >> "connection-info-${ENVIRONMENT}.md"
    done < "deployment-${ENVIRONMENT}.csv"
    
    echo '```' >> "connection-info-${ENVIRONMENT}.md"
    
    log_success "Connection information saved to connection-info-${ENVIRONMENT}.md"
}

# Cleanup function
cleanup_deployment() {
    log_warning "Cleaning up deployment..."
    
    if [[ ! -f "deployment-${ENVIRONMENT}.csv" ]]; then
        log_error "No deployment log found to cleanup"
        return 1
    fi
    
    while IFS=',' read -r validator_name region instance_id public_ip; do
        if [[ "${validator_name}" == "validator_name" ]]; then
            continue  # Skip header
        fi
        
        log_info "Terminating ${validator_name} (${instance_id})..."
        aws ec2 terminate-instances \
            --region "${region}" \
            --instance-ids "${instance_id}" \
            --output text &>/dev/null
    done < "deployment-${ENVIRONMENT}.csv"
    
    log_success "Cleanup initiated for all instances"
}

# Main function
main() {
    log_info "Chameleon Network AWS Deployment Script"
    log_info "Environment: ${ENVIRONMENT}"
    log_info "Instance Type: ${INSTANCE_TYPE}"
    
    case "${2:-deploy}" in
        "deploy")
            check_prerequisites
            deploy_all_validators
            sleep 30  # Wait for instances to fully initialize
            verify_deployment
            generate_connection_info
            ;;
        "verify")
            verify_deployment
            ;;
        "cleanup")
            cleanup_deployment
            ;;
        "info")
            generate_connection_info
            ;;
        *)
            echo "Usage: $0 [environment] [deploy|verify|cleanup|info]"
            echo "  environment: devnet (default), testnet, mainnet"
            echo "  action:"
            echo "    deploy  - Deploy all validators (default)"
            echo "    verify  - Verify existing deployment"
            echo "    cleanup - Terminate all instances"
            echo "    info    - Generate connection information"
            exit 1
            ;;
    esac
}

# Handle script interruption
trap 'log_error "Script interrupted"; exit 1' INT TERM

# Run main function
main "$@"