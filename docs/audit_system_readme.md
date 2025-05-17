# IntelliRouter Audit System - Quick Reference

## Overview

The IntelliRouter Audit System validates integration between all system roles (Router, Chain Engine, RAG Manager, Persona Layer) and supporting services.

## Quick Start

Run the audit system with the provided script:

```bash
./scripts/run_audit.sh
```

Or with specific options:

```bash
./scripts/run_audit.sh --deployment cloud --dashboard --store
```

## Common Use Cases

### Local Development Testing

```bash
./scripts/run_audit.sh --deployment local-dev --verbose
```

### CI/CD Pipeline Integration

```bash
./scripts/run_audit.sh --deployment cloud --ci --store-results
```

### Visual Dashboard

```bash
./scripts/run_audit.sh --dashboard
```

Then open http://127.0.0.1:8090 in your browser.

### Historical Comparison

```bash
./scripts/run_audit.sh --compare --store-results
```

### Specific Test Selection

```bash
./scripts/run_audit.sh --tests basic,rag,e2e
```

## Deployment Scenarios

- `single-node`: All services on one machine
- `distributed`: Services on different machines
- `cloud`: Services in Kubernetes
- `local-dev`: Local development environment

## Available Tests

- `basic`: Basic chain execution test
- `rag`: RAG integration test
- `persona`: Persona layer integration test
- `e2e`: End-to-end flow test

## CI/CD Integration

Generate CI/CD configuration files:

```bash
cargo run -- audit generate-ci --platform github --output ./
```

Supported platforms:
- `github`: GitHub Actions
- `jenkins`: Jenkins Pipeline
- `gitlab`: GitLab CI
- `circleci`: CircleCI

## Documentation

For detailed documentation, see:
- [Audit System Documentation](audit_system.md)
- [Integration Testing Guide](../INTEGRATION_TESTING.md)

## Troubleshooting

If you encounter issues:

1. Run with verbose output: `./scripts/run_audit.sh --verbose`
2. Check service logs: `docker-compose -f docker-compose.integration.yml logs [service]`
3. Verify all services are healthy: `docker-compose -f docker-compose.integration.yml ps`