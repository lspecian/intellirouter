# IntelliRouter Audit System

This document provides comprehensive documentation for the IntelliRouter Role Integration Audit System, which validates integration between all system roles.

## Overview

The audit system is designed to validate the integration between all IntelliRouter system roles:

- Router
- Chain Engine (Orchestrator)
- RAG Manager (RAG Injector)
- Persona Layer (Summarizer)
- Supporting services (Redis, ChromaDB)

The system performs comprehensive validation of:

- Service discovery
- Direct communication between services
- End-to-end flows
- Data integrity
- Error handling
- Security

## Running the Audit System

### Basic Usage

To run the audit system with default settings:

```bash
cargo run -- audit run
```

This will:
1. Start all services in the correct order
2. Validate service discovery
3. Test communication between services
4. Execute test flows
5. Collect and analyze metrics
6. Generate a report

### Command-Line Options

The audit system provides a rich set of command-line options:

```bash
cargo run -- audit run [OPTIONS]
```

#### General Options

- `-c, --config <PATH>`: Path to the audit configuration file
- `-o, --output <PATH>`: Path to save the audit report
- `-f, --format <FORMAT>`: Report format (json, markdown, html)
- `-v, --verbose`: Enable verbose output
- `--ci`: Run in CI mode (non-interactive, exit code reflects test status)

#### Dashboard Options

- `-d, --dashboard`: Start the dashboard server
- `--dashboard-host <HOST>`: Dashboard host (default: 127.0.0.1)
- `--dashboard-port <PORT>`: Dashboard port (default: 8090)

#### Test Selection and Configuration

- `--tests <TESTS>`: Specific tests to run (comma-separated)
- `--deployment <SCENARIO>`: Deployment scenario (single-node, distributed, cloud, local-dev)

#### Historical Comparison

- `--store-results`: Store test results for historical comparison
- `--compare`: Compare with previous test results

### Deployment Scenarios

The audit system supports different deployment scenarios:

#### Single-Node Deployment

All services run on a single machine:

```bash
cargo run -- audit run --deployment single-node
```

This configuration:
- Uses shorter timeouts
- Configures all services to use localhost
- Optimizes for local testing

#### Distributed Deployment

Services run on different machines:

```bash
cargo run -- audit run --deployment distributed
```

This configuration:
- Uses longer timeouts
- Configures services to use network hostnames
- Validates network connectivity

#### Cloud Deployment

Services run in Kubernetes:

```bash
cargo run -- audit run --deployment cloud
```

This configuration:
- Uses extended timeouts
- Configures services to use Kubernetes service names
- Validates cloud-specific connectivity

#### Local Development

Optimized for local development:

```bash
cargo run -- audit run --deployment local-dev
```

This configuration:
- Uses minimal timeouts
- Configures all services to use localhost with default ports
- Optimized for quick feedback during development

### Test Selection

You can select specific tests to run:

```bash
cargo run -- audit run --tests basic,rag,persona,e2e
```

Available tests:
- `basic`: Basic chain execution test
- `rag`: RAG integration test
- `persona`: Persona layer integration test
- `e2e`: End-to-end flow test

### Dashboard

The audit system includes a web dashboard for visualizing test results:

```bash
cargo run -- audit run --dashboard
```

This will start a dashboard server on http://127.0.0.1:8090 that provides:
- Real-time test results
- System topology visualization
- Performance metrics
- Detailed error information

## CI/CD Integration

The audit system can be integrated with various CI/CD platforms.

### Generating CI/CD Configurations

To generate CI/CD configuration files:

```bash
cargo run -- audit generate-ci --platform <PLATFORM> --output <DIR> --deployment <SCENARIO>
```

Supported platforms:
- `github`: GitHub Actions
- `jenkins`: Jenkins Pipeline
- `gitlab`: GitLab CI
- `circleci`: CircleCI

Example:
```bash
cargo run -- audit generate-ci --platform github --output ./ --deployment cloud
```

This will generate a GitHub Actions workflow configuration in `./.github/workflows/integration-tests.yml`.

### GitHub Actions Integration

The generated GitHub Actions workflow:
1. Starts the integration test environment using Docker Compose
2. Waits for services to be ready
3. Runs the integration tests
4. Collects and uploads test reports
5. Cleans up resources

Example workflow:
```yaml
name: Integration Tests

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]
  workflow_dispatch:

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Start Integration Test Environment
        run: docker-compose -f docker-compose.integration.yml up -d

      - name: Wait for Services to be Ready
        run: |
          echo "Waiting for services to be ready..."
          sleep 30
          docker-compose -f docker-compose.integration.yml ps

      - name: Run Integration Tests
        run: |
          docker-compose -f docker-compose.integration.yml run test-runner cargo run -- audit run --deployment cloud --ci --store-results

      - name: Collect Test Reports
        if: always()
        run: |
          mkdir -p test-reports
          docker-compose -f docker-compose.integration.yml cp test-runner:/app/audit_history test-reports/

      - name: Upload Test Reports
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-reports
          path: test-reports/

      - name: Cleanup
        if: always()
        run: docker-compose -f docker-compose.integration.yml down -v
```

### Jenkins Integration

The generated Jenkinsfile:
1. Uses Docker-in-Docker for container execution
2. Starts the integration test environment
3. Runs the integration tests
4. Collects and archives test reports
5. Cleans up resources

### GitLab CI Integration

The generated `.gitlab-ci.yml`:
1. Uses Docker-in-Docker for container execution
2. Starts the integration test environment
3. Runs the integration tests
4. Collects and stores test reports as artifacts
5. Cleans up resources

### CircleCI Integration

The generated CircleCI configuration:
1. Uses the CircleCI machine executor
2. Starts the integration test environment
3. Runs the integration tests
4. Collects and stores test reports as artifacts
5. Cleans up resources

## Historical Test Results

The audit system can store and compare test results over time.

### Storing Results

To store test results for historical comparison:

```bash
cargo run -- audit run --store-results
```

This will save the test results in the `audit_history` directory.

### Viewing Historical Results

To view historical test results:

```bash
cargo run -- audit history
```

Options:
- `-l, --limit <LIMIT>`: Number of historical results to show (default: 5)
- `-t, --test <TEST>`: Filter by test name
- `-f, --format <FORMAT>`: Output format (text, json)

### Comparing with Previous Results

To compare current test results with previous runs:

```bash
cargo run -- audit run --compare
```

This will:
1. Load previous test results
2. Compare current results with previous runs
3. Highlight regressions or improvements

## Troubleshooting

### Common Issues

#### Services Not Starting

If services fail to start:
1. Check Docker Compose logs: `docker-compose -f docker-compose.integration.yml logs <service>`
2. Verify resource availability (memory, disk space)
3. Check for port conflicts

#### Test Failures

If tests fail:
1. Check the test report for specific error messages
2. Verify service connectivity
3. Check service logs for errors
4. Ensure all services are healthy

#### Dashboard Not Accessible

If the dashboard is not accessible:
1. Verify the dashboard server is running
2. Check for port conflicts
3. Ensure the host is accessible from your browser

## Advanced Configuration

### Custom Test Flows

You can define custom test flows in the audit configuration file:

```toml
# config/audit.toml
[test_config]
test_flows = ["BasicChainExecution", "RagIntegration", "CustomFlow"]
```

### Custom Validation Rules

You can define custom validation rules in the audit configuration file:

```toml
# config/audit.toml
[validation_config]
validate_service_discovery = true
validate_direct_communication = true
validate_end_to_end_flows = true
validate_data_integrity = true
validate_error_handling = true
validate_security = true
validation_timeout_secs = 120
fail_fast = true
```

### Custom Metrics Collection

You can configure custom metrics collection in the audit configuration file:

```toml
# config/audit.toml
[metrics_config]
collect_metrics = true
collection_interval_ms = 1000
collection_duration_secs = 60
metric_types = ["Latency", "Throughput", "ErrorRate", "ResourceUsage"]
```

## Conclusion

The IntelliRouter Role Integration Audit System provides a comprehensive solution for validating the integration between all system roles. By using this system as part of your development workflow and CI/CD pipelines, you can ensure that all components work together correctly and catch integration issues early.