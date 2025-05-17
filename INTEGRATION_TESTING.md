# IntelliRouter Integration Testing

This document provides instructions for running integration tests using the Docker Compose configuration.

## Overview

The `docker-compose.integration.yml` file defines a complete IntelliRouter environment with all system roles running as separate services:

- **Router**: Routes requests to appropriate LLM backends
- **Orchestrator**: Manages the execution of chains and workflows (Chain Engine)
- **RAG Injector**: Manages retrieval-augmented generation
- **Summarizer**: Manages system prompts and personas (Persona Layer)
- **Supporting Services**: Redis for memory storage, ChromaDB for vector storage
- **Test Runner**: Executes the integration tests

## Prerequisites

- Docker and Docker Compose installed
- Git repository cloned
- Basic understanding of IntelliRouter architecture

## Running Integration Tests

### Starting the Environment

To start the complete integration testing environment:

```bash
docker-compose -f docker-compose.integration.yml up -d
```

This will start all services in detached mode. The services will be accessible on the following ports:

- Router: http://localhost:8080
- Orchestrator: http://localhost:8081
- RAG Injector: http://localhost:8082
- Summarizer: http://localhost:8083
- Redis: localhost:6379
- ChromaDB: http://localhost:8000

### Running Tests

To run the integration tests:

```bash
docker-compose -f docker-compose.integration.yml run test-runner
```

This will execute the integration tests defined in `tests/integration_test.rs` against the running services.

### Viewing Logs

To view logs from a specific service:

```bash
docker-compose -f docker-compose.integration.yml logs -f [service-name]
```

Replace `[service-name]` with one of: `router`, `orchestrator`, `rag-injector`, `summarizer`, `redis`, `chromadb`, or `test-runner`.

### Stopping the Environment

To stop all services:

```bash
docker-compose -f docker-compose.integration.yml down
```

To stop all services and remove volumes:

```bash
docker-compose -f docker-compose.integration.yml down -v
```

## Manual Testing

You can also interact with the services manually for testing:

### Health Checks

Verify that all services are running correctly:

```bash
curl http://localhost:8080/health  # Router
curl http://localhost:8081/health  # Orchestrator
curl http://localhost:8082/health  # RAG Injector
curl http://localhost:8083/health  # Summarizer
```

### Inter-Service Communication

Test that services can communicate with each other:

```bash
# Example: Test router to orchestrator communication
docker-compose -f docker-compose.integration.yml exec router curl -f http://orchestrator:8080/health
```

## Customizing the Environment

### Environment Variables

You can modify environment variables in the Docker Compose file to change service behavior:

- `INTELLIROUTER__TELEMETRY__LOG_LEVEL`: Set to `debug`, `info`, `warn`, or `error`
- `INTELLIROUTER__IPC__SECURITY__ENABLED`: Enable/disable IPC security
- `INTELLIROUTER__IPC__SECURITY__TOKEN`: Security token for inter-service communication

### Adding Test Data

Place test data files in the `test_data` directory. These will be mounted into each service container.

### Adding Custom Tests

1. Add new test files to the `tests` directory
2. Modify the test-runner command in `docker-compose.integration.yml` to run your specific tests

## Troubleshooting

### Service Not Starting

Check the logs for the specific service:

```bash
docker-compose -f docker-compose.integration.yml logs [service-name]
```

### Network Connectivity Issues

Ensure all services are on the same network:

```bash
docker network inspect intellirouter-integration-network
```

### Test Failures

If tests are failing, check:

1. Service health checks are passing
2. Environment variables are correctly set
3. Test data is properly mounted
4. Service discovery environment variables are correct

## CI/CD Integration

This Docker Compose configuration is designed to work in CI/CD environments. The GitHub Actions workflow in `.github/workflows/e2e-tests.yml` uses this configuration to run integration tests.

The workflow includes the following steps:

```yaml
- name: Start Integration Test Environment
  run: docker-compose -f docker-compose.integration.yml up -d

- name: Wait for Services to be Ready
  run: |
    echo "Waiting for services to start..."
    sleep 30
    docker-compose -f docker-compose.integration.yml ps

- name: Run E2E Tests
  run: |
    chmod +x ./scripts/run_e2e_tests.sh
    ./scripts/run_e2e_tests.sh

- name: Run Ignored Tests
  run: |
    echo "Running longer tests that are marked with #[ignore]..."
    cargo test -- --ignored

- name: Upload Test Logs
  uses: actions/upload-artifact@v3
  if: always()
  with:
    name: test-logs
    path: |
      logs/
      target/debug/deps/*.log

- name: Cleanup
  if: always()
  run: docker-compose -f docker-compose.integration.yml down -v
```

### Test Script

The `scripts/run_e2e_tests.sh` script is used to run the end-to-end tests in CI environments. It:

1. Sets up the environment variables
2. Runs the model routing tests
3. Runs the multi-step chain tests
4. Runs the RAG injection tests
5. Runs all non-ignored tests
6. Generates a test report

### Ignored Tests

Longer tests are marked with the `#[ignore]` attribute and are run separately to prevent CI timeouts. These tests include:

1. `test_end_to_end_request_flow`: Full end-to-end request flow through the system
2. `test_chat_completions_endpoint`: Chat completions endpoint with HTTP request
3. `test_multi_step_chain`: Multi-step chain execution with multiple models
4. `test_conditional_chain`: Conditional chain execution with multiple models
5. `test_error_handling_chain`: Error handling chain with failing model

For more details on the CI integration, see [CI Integration](docs/ci_integration.md).