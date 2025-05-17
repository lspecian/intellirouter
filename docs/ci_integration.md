# CI Integration for IntelliRouter

This document explains how the CI pipeline is configured for IntelliRouter and how to run tests in CI environments.

## Overview

The IntelliRouter project uses GitHub Actions for continuous integration. The CI pipeline is configured to:

1. Run unit tests
2. Run integration tests
3. Run end-to-end tests
4. Generate test reports
5. Upload test logs as artifacts

## GitHub Actions Workflow

The CI pipeline is defined in `.github/workflows/e2e-tests.yml`. It runs on every push to the main branch and on pull requests.

The workflow:

1. Checks out the code
2. Sets up Rust
3. Caches dependencies
4. Starts the integration test environment using Docker Compose
5. Runs the end-to-end tests
6. Runs the ignored (longer) tests
7. Uploads test logs as artifacts
8. Cleans up the environment

## Running Tests in CI

### Regular Tests

Regular tests are run using the `scripts/run_e2e_tests.sh` script, which:

1. Sets up the environment variables
2. Runs the model routing tests
3. Runs the multi-step chain tests
4. Runs the RAG injection tests
5. Runs all non-ignored tests
6. Generates a test report

### Ignored Tests

Longer tests are marked with the `#[ignore]` attribute and are run separately using:

```bash
cargo test -- --ignored
```

This ensures that the CI pipeline doesn't time out due to long-running tests.

## Test Logging

Tests in CI environments are configured to output detailed logs for debugging. The logs include:

- File and line number information
- Thread IDs
- Target information
- Timestamps

Logs are captured and uploaded as artifacts in the GitHub Actions workflow.

## Running Tests Locally

To run the tests locally in the same way as the CI pipeline:

1. Start the integration test environment:

```bash
docker-compose -f docker-compose.integration.yml up -d
```

2. Run the tests:

```bash
./scripts/run_e2e_tests.sh
```

3. Run the ignored tests:

```bash
cargo test -- --ignored
```

4. Clean up the environment:

```bash
docker-compose -f docker-compose.integration.yml down -v
```

## Test Categories

The tests are categorized as follows:

1. **Unit Tests**: Located within each module in `src/modules/*/tests.rs`
2. **Integration Tests**: Located in the `tests/` directory
3. **End-to-End Tests**: Located in `tests/integration/` directory
4. **Property-Based Tests**: Located in `tests/property_tests.rs`

## Ignored Tests

The following tests are marked with the `#[ignore]` attribute because they take longer to run:

1. `test_end_to_end_request_flow`: Full end-to-end request flow through the system
2. `test_chat_completions_endpoint`: Chat completions endpoint with HTTP request
3. `test_multi_step_chain`: Multi-step chain execution with multiple models
4. `test_conditional_chain`: Conditional chain execution with multiple models
5. `test_error_handling_chain`: Error handling chain with failing model

## Test Artifacts

The CI pipeline uploads the following artifacts:

1. Test logs: Located in the `logs/` directory
2. Test reports: Located in the `logs/test_report.md` file
3. Debug logs: Located in the `target/debug/deps/*.log` files

## Troubleshooting

If tests fail in the CI pipeline, check the following:

1. Test logs for detailed error messages
2. Service health checks to ensure all services are running
3. Environment variables to ensure they are set correctly
4. Docker Compose logs to check for service startup issues

## Adding New Tests

When adding new tests:

1. Add unit tests for the new functionality
2. Add integration tests if the functionality interacts with other components
3. Consider adding property-based tests for invariants
4. Mark longer tests with the `#[ignore]` attribute
5. Update the `scripts/run_e2e_tests.sh` script if necessary
6. Run the tests locally to ensure they pass before pushing to the repository