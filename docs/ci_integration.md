# CI Integration for IntelliRouter

This document explains how the CI pipeline is configured for IntelliRouter and how to run tests in CI environments.

## Overview

The IntelliRouter project uses GitHub Actions for continuous integration. The project has multiple CI workflows:

1. **General CI Workflow** (`.github/workflows/ci.yml`): Handles compilation checks, builds, unit tests, linting, security scanning, and code coverage
2. **End-to-End Tests Workflow** (`.github/workflows/e2e-tests.yml`): Handles integration and end-to-end tests

Together, these workflows ensure:

1. Code compiles without errors
2. Unit tests pass
3. Integration tests pass
4. End-to-end tests pass
5. Code quality standards are maintained
6. Security vulnerabilities are detected
7. Test reports and logs are generated and uploaded as artifacts

## General CI Workflow

The general CI workflow is defined in `.github/workflows/ci.yml` and includes the following jobs:

1. **Compilation Check**: Runs `cargo check` on library code, binary code, and all targets to ensure everything compiles without errors
2. **Build**: Builds the project on multiple operating systems and Rust versions
3. **Test**: Runs unit tests
4. **Lint**: Checks code formatting and runs clippy for static analysis
5. **Security Scan**: Runs security audits on dependencies
6. **Coverage**: Generates code coverage reports
7. **Warning Analysis**: Analyzes compilation warnings and provides suggestions for fixing them

This workflow runs on every push to the main branch and on pull requests.

## End-to-End Tests Workflow

The end-to-end tests workflow is defined in `.github/workflows/e2e-tests.yml`. It runs on every push to the main branch and on pull requests.

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
4. Warning report: Located in the `warning_report.md` file

## Troubleshooting

If tests fail in the CI pipeline, check the following:

1. Test logs for detailed error messages
2. Compilation errors in the "Compilation Check" job
3. Service health checks to ensure all services are running
4. Environment variables to ensure they are set correctly
5. Docker Compose logs to check for service startup issues

## Adding New Tests

When adding new tests:

1. Add unit tests for the new functionality
2. Add integration tests if the functionality interacts with other components
3. Consider adding property-based tests for invariants
4. Mark longer tests with the `#[ignore]` attribute
5. Update the `scripts/run_e2e_tests.sh` script if necessary
6. Run the tests locally to ensure they pass before pushing to the repository
7. Ensure your code passes the compilation check with `cargo check --all-targets`

## Compilation Status Check

The compilation status check job runs `cargo check` on:

1. Library code (`--lib`)
2. Binary code (`--bins`)
3. All targets (`--all-targets`)

This check runs before the build job and helps catch compilation errors early in the development process. It's faster than a full build and provides quick feedback on code correctness.

If the compilation check fails:

1. Check the job logs for specific compilation errors
2. Fix the errors locally using `cargo check`
3. Verify all targets compile before pushing again
4. Refer to [Compilation Best Practices](compilation_best_practices.md) for guidelines on avoiding common compilation errors

## Compilation Warning Analysis

The CI pipeline includes a job that analyzes compilation warnings to help developers identify and fix potential issues. This job:

1. Runs `cargo check --message-format=json` to get warnings in JSON format
2. Parses the JSON output to extract warnings
3. Categorizes warnings by type (e.g., unused variables, unused imports, dead code)
4. Counts the number of warnings in each category
5. Identifies the files with the most warnings
6. Provides suggestions for fixing the most common warnings
7. Generates a markdown report that is uploaded as an artifact

The warning analyzer job is configured to be non-blocking, meaning it won't fail the build if warnings are found. This allows developers to address warnings at their own pace while still being aware of them.

### Running the Warning Analyzer Locally

To run the warning analyzer locally:

```bash
./scripts/analyze_warnings.sh
```

If you want to analyze warnings even when there are compilation errors:

```bash
./scripts/analyze_warnings.sh --allow-errors
```

This will generate a `warning_report.md` file with detailed information about the warnings in the codebase.

### Common Warning Types and Fixes

The warning analyzer provides suggestions for fixing common warning types:

1. **Unused Variables**: Prefix with underscore (`_variable_name`) or remove
2. **Unused Imports**: Remove or use `cargo fix --allow-dirty`
3. **Dead Code**: Remove or add `#[allow(dead_code)]` attribute
4. **Unused Functions**: Remove or add `#[allow(dead_code)]` attribute
5. **Unused Fields**: Remove or add `#[allow(dead_code)]` attribute
6. **Naming Convention Issues**: Follow Rust naming conventions (snake_case for variables/functions, CamelCase for types)
7. **Deprecated Items**: Update to use non-deprecated alternatives

### Interpreting the Warning Report

The warning report includes:

1. Total number of warnings
2. Breakdown of warnings by type
3. List of files with the most warnings
4. Suggestions for fixing common warning types

This information can help prioritize which warnings to address first and how to fix them.