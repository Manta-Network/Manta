# Chameleon Network Devnet

This directory contains the complete 5-validator devnet setup for Chameleon Network development and testing.

## Quick Start

```bash
# Start the devnet
./scripts/start-devnet.sh

# Check health
./scripts/health-check.sh

# Stop the devnet
./scripts/stop-devnet.sh

# Reset everything
./scripts/reset-devnet.sh
```

## Directory Structure

```
devnet/
├── README.md                    # This file
├── docker-compose.yml           # Docker Compose configuration
├── validator-keys.json          # Validator configuration and keys
├── chameleon-devnet-spec.json   # Generated chain specification
├── validator-*-data/            # Validator blockchain data (auto-created)
├── validator-*-keys/            # Validator cryptographic keys (auto-created)
├── monitoring/                  # Monitoring configuration
│   ├── prometheus.yml
│   ├── loki-config.yml
│   └── promtail-config.yml
└── scripts/                     # Utility scripts
    ├── start-devnet.sh
    ├── stop-devnet.sh
    ├── reset-devnet.sh
    └── health-check.sh
```

## Services

### Validators (5 nodes)
- **validator-0**: US-East (ports 30333, 9933, 9944, 9615)
- **validator-1**: US-West (ports 30334, 9934, 9945, 9616)
- **validator-2**: EU-Central (ports 30335, 9935, 9946, 9617)
- **validator-3**: Asia-East (ports 30336, 9936, 9947, 9618)
- **validator-4**: Asia-Southeast (ports 30337, 9937, 9948, 9619)

### Monitoring
- **Prometheus**: http://localhost:9090 (metrics collection)
- **Grafana**: http://localhost:3000 (dashboards, admin/chameleon)
- **Loki**: http://localhost:3100 (log aggregation)

### Tools
- **Polkadot.js Apps**: http://localhost:3001 (blockchain explorer)

## Network Configuration

- **Chain ID**: chameleon-devnet
- **Parachain ID**: 2105
- **Block Time**: 6 seconds
- **Total Supply**: 100,000,000 CHML
- **Validator Stake**: 1,000,000 CHML each (5M total)

## Token Allocation

| Account | Amount | Purpose |
|---------|--------|---------|
| Validators (5) | 5M CHML | Network security |
| Liquidity Pool | 2.5M CHML | Initial DEX liquidity |
| Treasury | 2.5M CHML | DAO reserve |
| Ecosystem | 5M CHML | Development funding |
| Presale | 15M CHML | Public sale |
| Airdrop | 5M CHML | Community distribution |
| Emission | 65M CHML | 20-year rewards |

## Development Workflow

1. **Build the node**:
   ```bash
   cd .. && cargo build --release
   ```

2. **Start devnet**:
   ```bash
   ./scripts/start-devnet.sh
   ```

3. **Connect with Polkadot.js**:
   - Open http://localhost:3001
   - Connect to ws://localhost:9944

4. **Monitor with Grafana**:
   - Open http://localhost:3000
   - Login: admin/chameleon

5. **Test your changes**:
   - Deploy contracts
   - Submit transactions
   - Monitor performance

6. **Reset for clean testing**:
   ```bash
   ./scripts/reset-devnet.sh
   ./scripts/start-devnet.sh
   ```

## Troubleshooting

See the [Devnet Setup Guide](../docs/devnet-setup.md) for comprehensive troubleshooting information.

### Common Issues

- **Containers won't start**: Check Docker daemon and available ports
- **No block production**: Verify validator keys and session configuration
- **High memory usage**: Adjust Docker resource limits
- **Network connectivity**: Check firewall and port availability

### Useful Commands

```bash
# View all container logs
docker-compose logs -f

# View specific validator logs
docker-compose logs -f validator-0

# Check container status
docker-compose ps

# Restart specific service
docker-compose restart validator-0

# View resource usage
docker stats
```

## Support

For issues and questions:
- Check the [documentation](../docs/devnet-setup.md)
- Review container logs
- Run health checks
- Contact the development team
