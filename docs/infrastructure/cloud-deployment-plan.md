# Chameleon Network Cloud Deployment Plan

## Overview

This document outlines the comprehensive cloud infrastructure plan for deploying Chameleon Network validators and supporting infrastructure across multiple regions. The deployment targets a production-ready devnet with 5 validators, monitoring, and public infrastructure.

## Cloud Provider Selection

### Primary: Amazon Web Services (AWS)

**Rationale:**
- Global presence with 31 regions
- Mature blockchain infrastructure support
- Comprehensive monitoring and security services
- Proven reliability for validator networks
- Advanced networking capabilities (VPC, Security Groups)
- Cost-effective for sustained workloads

**Services Used:**
- **EC2**: Validator instances
- **VPC**: Network isolation
- **Security Groups**: Firewall rules
- **CloudWatch**: Monitoring and alerting
- **Route 53**: DNS management
- **S3**: Backup storage
- **IAM**: Access management
- **Systems Manager**: Remote management

### Alternative: DigitalOcean

**Rationale:**
- Simpler pricing model
- Developer-friendly interface
- Good performance for blockchain nodes
- Lower complexity for smaller deployments

**Services Used:**
- **Droplets**: Validator instances
- **VPC**: Private networking
- **Firewall**: Security rules
- **Monitoring**: Basic metrics
- **Spaces**: Object storage

## Resource Requirements

### Validator Node Specifications

**Instance Type: AWS t3.large / DO s-2vcpu-4gb**

```
CPU:     2 vCPUs (Intel Xeon Platinum 8000 series)
RAM:     4 GB DDR4
Storage: 100 GB SSD (gp3 on AWS, NVMe on DO)
Network: Up to 5 Gbps
OS:      Ubuntu 22.04 LTS
```

**Rationale:**
- 2 vCPUs sufficient for Substrate consensus (Aura + Grandpa)
- 4 GB RAM handles blockchain state and networking
- 100 GB storage accommodates 6+ months of blockchain data
- High-performance SSD ensures fast block import/export

### RPC Node Specifications

**Instance Type: AWS t3.medium / DO s-2vcpu-2gb**

```
CPU:     2 vCPUs
RAM:     2 GB DDR4
Storage: 50 GB SSD
Network: Up to 5 Gbps
OS:      Ubuntu 22.04 LTS
```

### Monitoring Infrastructure

**Instance Type: AWS t3.small / DO s-1vcpu-2gb**

```
CPU:     1 vCPU
RAM:     2 GB DDR4
Storage: 20 GB SSD
Services: Prometheus, Grafana, AlertManager
```

## Cost Estimates

### AWS Pricing (Monthly)

| Component | Instance Type | Quantity | Unit Cost | Total Cost |
|-----------|---------------|----------|-----------|------------|
| Validators | t3.large | 5 | $67.07 | $335.35 |
| RPC Nodes | t3.medium | 2 | $33.41 | $66.82 |
| Monitoring | t3.small | 1 | $16.79 | $16.79 |
| Storage (EBS) | gp3 100GB | 5 | $8.00 | $40.00 |
| Storage (EBS) | gp3 50GB | 2 | $4.00 | $8.00 |
| Storage (EBS) | gp3 20GB | 1 | $1.60 | $1.60 |
| Data Transfer | - | - | $20.00 | $20.00 |
| **Total** | | | | **$488.56** |

### DigitalOcean Pricing (Monthly)

| Component | Droplet Size | Quantity | Unit Cost | Total Cost |
|-----------|--------------|----------|-----------|------------|
| Validators | s-2vcpu-4gb | 5 | $24.00 | $120.00 |
| RPC Nodes | s-2vcpu-2gb | 2 | $18.00 | $36.00 |
| Monitoring | s-1vcpu-2gb | 1 | $12.00 | $12.00 |
| Storage | 100GB Volume | 5 | $10.00 | $50.00 |
| Storage | 50GB Volume | 2 | $5.00 | $10.00 |
| Storage | 20GB Volume | 1 | $2.00 | $2.00 |
| Bandwidth | 1TB | - | $10.00 | $10.00 |
| **Total** | | | | **$240.00** |

**Recommendation**: Start with DigitalOcean for cost efficiency (~$240/month vs $488/month)

## Network Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CHAMELEON NETWORK INFRASTRUCTURE                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   US-EAST-1     │    │   US-WEST-2     │    │   EU-WEST-1     │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Validator-0 │ │    │ │ Validator-1 │ │    │ │ Validator-2 │ │
│ │ t3.large    │ │    │ │ t3.large    │ │    │ │ t3.large    │ │
│ │ 2vCPU/4GB   │ │    │ │ 2vCPU/4GB   │ │    │ │ 2vCPU/4GB   │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ RPC Node    │ │    │ │ Monitoring  │ │    │ │ Explorer    │ │
│ │ t3.medium   │ │    │ │ t3.small    │ │    │ │ t3.small    │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘

┌─────────────────┐    ┌─────────────────┐
│ AP-SOUTHEAST-1  │    │ AP-NORTHEAST-1  │
│                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Validator-3 │ │    │ │ Validator-4 │ │
│ │ t3.large    │ │    │ │ t3.large    │ │
│ │ 2vCPU/4GB   │ │    │ │ 2vCPU/4GB   │ │
│ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ RPC Node    │ │    │ │ Backup      │ │
│ │ t3.medium   │ │    │ │ t3.small    │ │
│ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘

                    ┌─────────────────┐
                    │   GLOBAL DNS    │
                    │                 │
                    │ rpc.chameleon   │
                    │ explorer.cham   │
                    │ monitor.cham    │
                    └─────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              NETWORK FLOW                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Internet ──► Load Balancer ──► RPC Nodes ──► Validators                   │
│      │                                            │                         │
│      └──► Explorer ──► RPC Nodes ──► Validators ──┘                         │
│                                                                             │
│  Validators ←──► P2P Network (30333) ←──► Validators                       │
│                                                                             │
│  Monitoring ←── Metrics (9615) ←── All Nodes                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Security Configuration

### Firewall Rules (Security Groups)

#### Validator Nodes
```bash
# Inbound Rules
Port 30333 (P2P)     - Source: 0.0.0.0/0        # Blockchain P2P
Port 9933 (RPC)      - Source: VPC CIDR         # Internal RPC only
Port 9944 (WebSocket)- Source: VPC CIDR         # Internal WS only
Port 9615 (Metrics)  - Source: Monitoring IP    # Prometheus scraping
Port 22 (SSH)        - Source: Admin IPs        # Management access

# Outbound Rules
All traffic          - Destination: 0.0.0.0/0   # Full internet access
```

#### RPC Nodes
```bash
# Inbound Rules
Port 9933 (RPC)      - Source: 0.0.0.0/0        # Public RPC access
Port 9944 (WebSocket)- Source: 0.0.0.0/0        # Public WS access
Port 9615 (Metrics)  - Source: Monitoring IP    # Prometheus scraping
Port 22 (SSH)        - Source: Admin IPs        # Management access

# Outbound Rules
All traffic          - Destination: 0.0.0.0/0   # Full internet access
```

#### Monitoring Node
```bash
# Inbound Rules
Port 3000 (Grafana)  - Source: Admin IPs        # Dashboard access
Port 9090 (Prometheus)- Source: Admin IPs       # Metrics access
Port 22 (SSH)        - Source: Admin IPs        # Management access

# Outbound Rules
All traffic          - Destination: 0.0.0.0/0   # Scraping and alerts
```

### SSH Configuration

```bash
# /etc/ssh/sshd_config
Port 22
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
ClientAliveInterval 300
ClientAliveCountMax 2
MaxAuthTries 3
MaxSessions 10
```

### Authentication

**SSH Key Management:**
- Generate unique SSH key pairs for each environment
- Use ed25519 keys for better security
- Rotate keys every 90 days
- Store private keys in secure key management system

**Service Authentication:**
- Use systemd service accounts for node processes
- Implement proper file permissions (600 for keys)
- Use sudo for administrative tasks only

## Deployment Regions

### Primary Regions

1. **us-east-1 (N. Virginia)**
   - Validator-0 + RPC Node
   - Primary monitoring
   - Lowest latency to US users

2. **us-west-2 (Oregon)**
   - Validator-1 + Monitoring Stack
   - Secondary RPC endpoint
   - West Coast coverage

3. **eu-west-1 (Ireland)**
   - Validator-2 + Explorer
   - European user coverage
   - GDPR compliance region

4. **ap-southeast-1 (Singapore)**
   - Validator-3 + RPC Node
   - Asian market coverage
   - Low latency to major Asian cities

5. **ap-northeast-1 (Tokyo)**
   - Validator-4 + Backup Services
   - Japanese market access
   - Disaster recovery location

### Region Selection Criteria

- **Latency**: <100ms between validators
- **Reliability**: 99.9%+ uptime SLA
- **Compliance**: Regional data protection laws
- **Cost**: Balanced performance/cost ratio
- **Connectivity**: High-quality internet infrastructure

## Deployment Strategy

### Phase 1: Bootstrap (Day 1)

**Objective**: Deploy core validator infrastructure

**Tasks**:
1. Deploy 5 validator instances across regions
2. Configure networking and security groups
3. Install and configure Chameleon node software
4. Generate and distribute validator keys
5. Start genesis block production

**Success Criteria**:
- All 5 validators online and producing blocks
- Block time stable at 6 seconds
- No network partitions or connectivity issues

### Phase 2: Add Validators (Day 2)

**Objective**: Scale validator set and add redundancy

**Tasks**:
1. Deploy additional validator instances (optional)
2. Add RPC nodes for public access
3. Configure load balancing for RPC endpoints
4. Implement health checks and auto-recovery

**Success Criteria**:
- RPC endpoints responding to public queries
- Load balancing working correctly
- Health checks passing consistently

### Phase 3: Public Infrastructure (Day 3)

**Objective**: Deploy public-facing services

**Tasks**:
1. Deploy blockchain explorer
2. Set up monitoring dashboards
3. Configure DNS and SSL certificates
4. Implement backup and recovery systems

**Success Criteria**:
- Explorer accessible and synced
- Monitoring dashboards functional
- SSL certificates valid and auto-renewing
- Backup systems operational

### Phase 4: Validation (Day 4-5)

**Objective**: Comprehensive testing and optimization

**Tasks**:
1. Performance testing and optimization
2. Security scanning and hardening
3. Disaster recovery testing
4. Documentation and handover

**Success Criteria**:
- Performance targets met (TPS, latency)
- Security scan results acceptable
- Disaster recovery procedures validated
- Operations documentation complete

## Monitoring & Alerting

### Prometheus Configuration

**Metrics Collection**:
- Node metrics (CPU, RAM, disk, network)
- Blockchain metrics (block height, finality, peers)
- Custom Chameleon metrics (staking, emissions)
- System metrics (systemd services, logs)

**Scraping Targets**:
```yaml
scrape_configs:
  - job_name: 'chameleon-validators'
    static_configs:
      - targets: 
        - 'validator-0:9615'
        - 'validator-1:9615'
        - 'validator-2:9615'
        - 'validator-3:9615'
        - 'validator-4:9615'
    scrape_interval: 15s
    metrics_path: /metrics

  - job_name: 'node-exporter'
    static_configs:
      - targets:
        - 'validator-0:9100'
        - 'validator-1:9100'
        # ... all nodes
    scrape_interval: 30s
```

### Grafana Dashboards

**Core Dashboards**:
1. **Network Overview**: Block production, finality, validator status
2. **Node Health**: CPU, RAM, disk usage, network I/O
3. **Blockchain Metrics**: Transaction throughput, block times, peer count
4. **Staking Dashboard**: Validator performance, rewards, slashing events
5. **Security Dashboard**: Failed login attempts, unusual network activity

### AlertManager Rules

**Critical Alerts**:
```yaml
groups:
  - name: chameleon-critical
    rules:
      - alert: ValidatorDown
        expr: up{job="chameleon-validators"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Validator {{ $labels.instance }} is down"

      - alert: BlockProductionStopped
        expr: increase(substrate_block_height[5m]) == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Block production stopped on {{ $labels.instance }}"

      - alert: HighMemoryUsage
        expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
```

**Notification Channels**:
- Slack integration for team notifications
- Email alerts for critical issues
- PagerDuty for 24/7 on-call support
- SMS for validator down events

## Backup & Recovery

### Recovery Time Objective (RTO): <1 hour

**Target**: Restore service within 1 hour of failure

**Strategies**:
- Hot standby validators in each region
- Automated failover for RPC endpoints
- Pre-configured replacement instances
- Automated deployment scripts

### Recovery Point Objective (RPO): <24 hours

**Target**: Maximum 24 hours of data loss acceptable

**Strategies**:
- Daily blockchain state snapshots
- Continuous key backup to secure storage
- Configuration backup every 6 hours
- Log retention for 30 days

### Backup Procedures

**Blockchain Data**:
```bash
# Daily snapshot creation
#!/bin/bash
DATE=$(date +%Y%m%d)
NODE_DATA="/var/lib/chameleon"
BACKUP_DEST="s3://chameleon-backups/snapshots/"

# Stop node gracefully
sudo systemctl stop chameleon-node

# Create compressed snapshot
tar -czf "/tmp/chameleon-snapshot-${DATE}.tar.gz" "${NODE_DATA}"

# Upload to S3
aws s3 cp "/tmp/chameleon-snapshot-${DATE}.tar.gz" "${BACKUP_DEST}"

# Restart node
sudo systemctl start chameleon-node

# Cleanup local backup
rm "/tmp/chameleon-snapshot-${DATE}.tar.gz"
```

**Key Management**:
```bash
# Encrypted key backup
#!/bin/bash
KEY_DIR="/var/lib/chameleon/chains/chameleon-devnet/keystore"
BACKUP_DEST="s3://chameleon-backups/keys/"
DATE=$(date +%Y%m%d)

# Encrypt and backup keys
tar -czf - "${KEY_DIR}" | \
  gpg --cipher-algo AES256 --compress-algo 1 --symmetric \
      --output "/tmp/keys-${DATE}.tar.gz.gpg"

# Upload encrypted backup
aws s3 cp "/tmp/keys-${DATE}.tar.gz.gpg" "${BACKUP_DEST}"

# Cleanup
rm "/tmp/keys-${DATE}.tar.gz.gpg"
```

### Disaster Recovery Procedures

**Validator Failure**:
1. Detect failure via monitoring alerts
2. Launch replacement instance from AMI/snapshot
3. Restore keys from encrypted backup
4. Start node and verify sync
5. Update DNS/load balancer if needed

**Region Failure**:
1. Activate standby validators in other regions
2. Redirect traffic via DNS failover
3. Scale up remaining validators if needed
4. Monitor network stability
5. Plan region restoration

**Complete Network Failure**:
1. Restore from latest blockchain snapshot
2. Coordinate validator restart sequence
3. Verify genesis state consistency
4. Resume block production
5. Validate network integrity

## Prerequisites for Deployment

### Cloud Account Setup

**AWS Requirements**:
- AWS account with billing configured
- IAM user with EC2, VPC, S3, CloudWatch permissions
- AWS CLI configured with access keys
- Default VPC or custom VPC configured
- Key pairs generated for each region

**DigitalOcean Requirements**:
- DigitalOcean account with payment method
- API token with read/write permissions
- SSH keys uploaded to account
- doctl CLI tool installed and configured

### Domain and DNS

**Domain Registration**:
- Register chameleon.network domain (or similar)
- Configure DNS hosting (Route 53 or DigitalOcean DNS)
- Set up subdomains:
  - rpc.chameleon.network
  - explorer.chameleon.network
  - monitor.chameleon.network

### SSL Certificates

**Let's Encrypt Setup**:
- Install certbot on relevant servers
- Configure automatic renewal
- Set up wildcard certificates for subdomains

### GitHub Integration

**Repository Setup**:
- GitHub repository with deployment scripts
- GitHub Actions secrets configured:
  - AWS_ACCESS_KEY_ID / DO_API_TOKEN
  - AWS_SECRET_ACCESS_KEY
  - SSH_PRIVATE_KEY
  - DEPLOYMENT_HOSTS

### Local Development Environment

**Required Tools**:
- AWS CLI or doctl
- Terraform (optional, for infrastructure as code)
- Ansible (optional, for configuration management)
- Docker and Docker Compose
- Git and GitHub CLI

### Security Preparations

**Key Management**:
- Generate SSH key pairs for deployment
- Set up GPG keys for backup encryption
- Configure secure storage for sensitive data
- Implement access control policies

**Network Security**:
- Plan IP whitelisting for administrative access
- Configure VPN if required
- Set up monitoring for security events
- Prepare incident response procedures

## Cost Optimization

### Reserved Instances

**AWS Savings**:
- 1-year reserved instances: ~30% savings
- 3-year reserved instances: ~50% savings
- Spot instances for non-critical workloads

**DigitalOcean Savings**:
- Annual billing: 8% discount
- Volume discounts for multiple droplets

### Resource Right-Sizing

**Monitoring-Based Optimization**:
- Start with recommended sizes
- Monitor actual usage for 30 days
- Downsize underutilized instances
- Upsize if performance issues occur

### Storage Optimization

**Lifecycle Policies**:
- Move old backups to cheaper storage tiers
- Implement automatic cleanup of old snapshots
- Use compression for backup data
- Regular cleanup of log files

## Compliance and Governance

### Data Protection

**GDPR Compliance** (EU regions):
- Data processing agreements with cloud providers
- User data encryption at rest and in transit
- Right to erasure implementation
- Data breach notification procedures

### Audit and Compliance

**Security Audits**:
- Quarterly security assessments
- Penetration testing annually
- Compliance with blockchain security standards
- Regular review of access controls

### Change Management

**Deployment Procedures**:
- All changes via version control
- Peer review for infrastructure changes
- Staged deployment (dev → staging → prod)
- Rollback procedures documented and tested

---

**Document Version**: 1.0  
**Last Updated**: December 19, 2024  
**Next Review**: January 19, 2025  
**Owner**: Chameleon Network Infrastructure Team