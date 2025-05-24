# IntelliRouter Testing Guide

This guide provides comprehensive documentation on the IntelliRouter testing structure, including how to run and write tests with the new organization.

## Table of Contents

- [Overview](#overview)
- [Test Structure](#test-structure)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Test Utilities](#test-utilities)
- [Test Harness](#test-harness)
- [Continuous Integration](#continuous-integration)
- [Best Practices](#best-practices)
- [FAQ](#faq)

## Overview

IntelliRouter follows a test-first development approach, where tests are written before implementing functionality. This ensures that:

1. All code is testable by design
2. All code has tests
3. Implementation meets requirements
4. Edge cases are considered from the start
5. Refactoring can be done safely

The testing system is organized into three main categories:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test how components work together
3. **End-to-End Tests**: Test complete workflows through the entire system

## Test Structure

The test structure has been reorganized to improve maintainability and clarity:

```
intellirouter/
├── src/                    # Source code
├── tests/                  # All tests are now in this directory
│   ├── unit/               # Unit tests mirroring src/ structure
│   │   └── modules/        # Tests for specific modules
│   │       ├── audit/      # Tests for audit module
│   │       ├── ipc/        # Tests for IPC module
│   │       └── ...         # Tests for other modules
│   ├── integration/        # Integration tests between components
│   ├── e2e/                # End-to-end tests for complete workflows
│   │   ├── api/            # API-focused end-to-end tests
│   │   ├── performance/    # Performance and load tests
│   │   └── scenarios/      # Scenario-based end-to-end tests
│   ├── bin/                # Test binaries
│   │   └── run_tests.rs    # Test runner (moved from src/bin/)
│   ├── templates/          # Test templates for new tests
│   │   ├── unit_test_template.rs
│   │   ├── integration_test_template.rs
│   │   └── e2e_test_template.rs
│   ├── framework/          # Test framework components
│   │   └── test_harness/   # Test harness components
│   └── README.md           # Test structure documentation
└── intellirouter-test-utils/ # Separate crate for test utilities
    ├── src/                # Test utility source code
    └── README.md           # Test utilities documentation
```

### Key Changes from Previous Structure

1. **Moved Module-Specific Tests**: All module-specific tests have been moved from `src/modules/*/tests.rs` to `tests/unit/modules/*/mod.rs` and related files.
2. **Moved Test Runner**: The custom test runner has been moved from `src/bin/run_tests.rs` to `tests/bin/run_tests.rs`.
3. **Removed Test Code from Production Modules**: Test code has been removed from production modules to keep them focused on implementation.
4. **Created Integration Tests Structure**: A dedicated structure for integration tests has been created in `tests/integration/`.
5. **Created End-to-End Tests Structure**: A dedicated structure for end-to-end tests has been created in `tests/e2e/`.
6. **Separated Test Utilities**: Test utilities have been moved to a separate crate `intellirouter-test-utils`.

## Running Tests

### Running All Tests

```bash
cargo test
```

### Running Specific Test Categories

```bash
# Run only unit tests
cargo test --test 'unit_*'

# Run only integration tests
cargo test --test 'integration_*'

# Run only e2e tests
cargo test --test 'e2e_*'

# Run ignored tests (longer running tests)
cargo test -- --ignored
```

### Running Tests for Specific Modules

```bash
# Run tests for the audit module
cargo test unit::modules::audit

# Run tests for the IPC module
cargo test unit::modules::ipc
```

### Running Tests with Docker Compose

For integration testing with all system components, use the Docker Compose configuration:

```bash
docker-compose -f docker-compose.integration.yml up -d
docker-compose -f docker-compose.integration.yml run test-runner
```

See [Integration Testing](../INTEGRATION_TESTING.md) for more details.

## Writing Tests

### Test-First Development Process

1. **Understand Requirements**: Ensure you understand what functionality is needed
2. **Write Tests**: Write tests that verify the expected behavior
3. **Verify Test Failure**: Run the tests to verify they fail (since the functionality doesn't exist yet)
4. **Implement Functionality**: Implement the minimum code needed to make the tests pass
5. **Refactor**: Refactor the code while ensuring tests continue to pass

### Creating Unit Tests

Unit tests should be placed in the appropriate directory under `tests/unit/modules/`:

```rust
// tests/unit/modules/router_core/strategies/content_based_tests.rs

use intellirouter::modules::router_core::strategies::ContentBasedStrategy;
use intellirouter_test_utils::fixtures::create_test_request;

#[test]
fn test_content_based_routing() {
    // Arrange
    let strategy = ContentBasedStrategy::new();
    let request = create_test_request("test content");
    
    // Act
    let result = strategy.route(request);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().target(), "expected target");
}
```

### Creating Integration Tests

Integration tests should be placed in the `tests/integration/` directory:

```rust
// tests/integration/router_model_registry_tests.rs

use intellirouter::modules::router_core::Router;
use intellirouter::modules::model_registry::ModelRegistry;
use intellirouter_test_utils::fixtures::create_test_request;
use intellirouter_test_utils::mocks::create_test_model;

#[test]
fn test_router_uses_model_registry() {
    // Arrange
    let registry = ModelRegistry::new();
    registry.register_model("test-model", create_test_model())
        .expect("Failed to register model");
    
    let router = Router::new(registry);
    let request = create_test_request("test-model");
    
    // Act
    let result = router.route(request);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().model_id(), "test-model");
}
```

### Creating End-to-End Tests

End-to-end tests should be placed in the `tests/e2e/` directory:

```rust
// tests/e2e/scenarios/complete_workflow_tests.rs

use intellirouter_test_utils::helpers::spawn_test_server;
use intellirouter_test_utils::fixtures::create_test_client;

#[test]
fn test_end_to_end_workflow() {
    // Arrange
    let server = spawn_test_server().expect("Failed to spawn test server");
    let client = create_test_client();
    
    // Act
    let response = client.post(&format!("{}/v1/chat/completions", server.url()))
        .json(&serde_json::json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}]
        }))
        .send()
        .expect("Failed to send request");
    
    // Assert
    assert_eq!(response.status(), 200);
    let json = response.json::<serde_json::Value>().expect("Failed to parse JSON");
    assert!(json.get("choices").is_some());
}
```

### Using Test Templates

The `tests/templates/` directory contains templates for different types of tests:

- `unit_test_template.rs`: Template for unit tests
- `integration_test_template.rs`: Template for integration tests
- `e2e_test_template.rs`: Template for end-to-end tests

Copy these templates and adapt them to your specific testing needs.

## Test Utilities

The `intellirouter-test-utils` crate provides common utilities for testing:

### Fixtures

```rust
use intellirouter_test_utils::fixtures;

// Create a test request
let request = fixtures::create_test_request("test content");

// Create a test model
let model = fixtures::create_test_model();

// Create a temporary directory
let temp_dir = fixtures::create_temp_dir().expect("Failed to create temp dir");
```

### Mocks

```rust
use intellirouter_test_utils::mocks;

// Create a mock HTTP server
let mock_server = mocks::create_mock_http_server();

// Create a mock model backend
let mock_backend = mocks::create_mock_model_backend();
```

### Helpers

```rust
use intellirouter_test_utils::helpers;

// Spawn a test server
let server = helpers::spawn_test_server().expect("Failed to spawn test server");

// Wait for a condition
helpers::wait_for_condition(|| server.is_ready(), Duration::from_secs(5))
    .expect("Server did not become ready");
```

## Test Harness

The test harness provides a comprehensive framework for testing different aspects of the system:

### Basic Usage

```rust
use intellirouter::modules::test_harness::{
    engine::TestEngine,
    types::{TestCase, TestCategory, TestContext, TestOutcome, TestResult, TestSuite},
};

// Create a test engine
let engine = TestEngine::new();

// Create a test case
let test_case = TestCase::new(
    TestContext::new(TestCategory::Unit, "test_example".to_string()),
    |_ctx| {
        // Test implementation
        Ok(TestResult::new(
            "test_example",
            TestCategory::Unit,
            TestOutcome::Passed,
        ))
    },
);

// Create a test suite
let test_suite = TestSuite::new("Example Test Suite")
    .with_description("A simple example test suite")
    .with_test_case(test_case);

// Run the test suite
let result = engine.run_suite(test_suite).await?;

// Print the results
println!("Total Tests: {}", result.total_tests());
println!("Passed: {}", result.passed);
println!("Failed: {}", result.failed);
```

For more details on the test harness, see [Test Harness Documentation](test_harness.md).

## Continuous Integration

Tests are automatically run on GitHub Actions for every pull request and push to the main branch. The workflow includes:

1. Running all tests (unit, integration, and end-to-end)
2. Running ignored (longer) tests separately
3. Checking code coverage
4. Running clippy for linting
5. Checking formatting with rustfmt
6. Generating test reports
7. Uploading test logs as artifacts

For more details on the CI integration, see [CI Integration](ci_integration.md).

## Best Practices

1. **Test-First Development**: Write tests before implementing functionality
2. **Test Isolation**: Tests should be independent and not rely on the state from other tests
3. **Descriptive Names**: Use descriptive test names that explain what is being tested
4. **Arrange-Act-Assert**: Structure tests with clear setup, action, and assertion phases
5. **Test Edge Cases**: Include tests for edge cases and error conditions
6. **Keep Tests Fast**: Tests should run quickly to provide fast feedback
7. **Test Public API**: Focus on testing the public API of modules
8. **Use Test Utilities**: Use the provided test utilities for common testing tasks
9. **Document Test Strategy**: Include comments explaining the testing strategy for complex tests

## FAQ

### Q: Where should I put my unit tests?

A: Unit tests should be placed in the `tests/unit/modules/` directory, mirroring the structure of the `src/modules/` directory. For example, tests for `src/modules/router_core/strategies.rs` should be placed in `tests/unit/modules/router_core/strategies/mod.rs` or a specific file like `tests/unit/modules/router_core/strategies/content_based_tests.rs`.

### Q: How do I run only my specific test?

A: You can run a specific test by name:

```bash
cargo test test_function_name
```

Or by module path:

```bash
cargo test unit::modules::router_core::strategies
```

### Q: How do I create mocks for my tests?

A: You can use the `mockall` crate to create mocks for your tests. The `intellirouter-test-utils` crate also provides common mocks that you can use.

### Q: How do I test async functions?

A: Use the `#[tokio::test]` attribute for async tests:

```rust
#[tokio::test]
async fn test_async_function() {
    // Test implementation
}
```

### Q: How do I test error conditions?

A: Use the `Result` type and the `assert!(result.is_err())` assertion:

```rust
#[test]
fn test_error_condition() {
    // Arrange
    let invalid_input = "invalid input";
    
    // Act
    let result = function_that_should_fail(invalid_input);
    
    // Assert
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "expected error message");
}
```

### Q: How do I test with environment variables?

A: Use the `with_env_vars` helper from `intellirouter-test-utils`:

```rust
use intellirouter_test_utils::helpers::with_env_vars;

#[test]
fn test_with_env_vars() {
    with_env_vars(vec![("TEST_VAR", Some("test value"))], || {
        // Test implementation that uses TEST_VAR
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "test value");
    });
}
```

### Q: How do I test with a database?

A: Use the `TestDatabase` helper from `intellirouter-test-utils`:

```rust
use intellirouter_test_utils::helpers::TestDatabase;

#[tokio::test]
async fn test_with_database() {
    let db = TestDatabase::new().await.expect("Failed to create test database");
    
    // Test implementation that uses the database
    
    db.cleanup().await.expect("Failed to clean up test database");
}