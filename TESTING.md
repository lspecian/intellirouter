# IntelliRouter Testing Guide

This document outlines the testing approach for the IntelliRouter project, including unit tests, integration tests, property-based tests, and test coverage reporting. For our test-first development policy and process, see [Testing Policy](docs/testing_policy.md).

## Testing Philosophy

IntelliRouter follows a comprehensive testing approach to ensure the reliability and correctness of the codebase:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test how components work together
3. **Property-Based Tests**: Test invariants and properties across a wide range of inputs
4. **Test Coverage**: Ensure code is adequately tested

## Test Structure

The test structure has been reorganized as follows:

```
tests/
├── unit/                  # Unit tests mirroring src/ structure
│   └── modules/           # Tests for specific modules
│       ├── audit/         # Tests for audit module
│       ├── ipc/           # Tests for IPC module
│       └── ...            # Tests for other modules
├── integration/           # Integration tests between components
├── e2e/                   # End-to-end tests for complete workflows
│   ├── api/               # API-focused end-to-end tests
│   ├── performance/       # Performance and load tests
│   └── scenarios/         # Scenario-based end-to-end tests
├── bin/                   # Test binaries
│   └── run_tests.rs       # Test runner (moved from src/bin/)
├── templates/             # Test templates for new tests
│   ├── unit_test_template.rs
│   ├── integration_test_template.rs
│   └── e2e_test_template.rs
└── framework/             # Test framework components
    └── test_harness/      # Test harness components
```

- **Unit Tests**: Located in `tests/unit/modules/`, mirroring the src/ structure
- **Integration Tests**: Located in `tests/integration/` directory
- **End-to-End Tests**: Located in `tests/e2e/` directory
- **Property-Based Tests**: Located in `tests/property_tests.rs`
- **Test Utilities**: Located in the separate `intellirouter-test-utils` crate

For a comprehensive guide on the new test structure, see [Testing Guide](docs/testing_guide.md).

## Running Tests

### Running All Tests

```bash
cargo test
```

### Running Unit Tests Only

```bash
cargo test --test 'unit_*'
```

### Running Integration Tests Only

```bash
cargo test --test 'integration_*'
```

### Running End-to-End Tests Only

```bash
cargo test --test 'e2e_*'
```

### Running Property-Based Tests Only

```bash
cargo test --test property_tests
```

### Running Tests with Specific Features

```bash
cargo test --features redis-backend
```

## Test Coverage

We use [tarpaulin](https://github.com/xd009642/tarpaulin) for test coverage reporting.

### Running Coverage Locally

```bash
cargo tarpaulin --verbose --workspace --out Html --output-dir coverage
```

### Coverage Requirements

- Minimum coverage threshold: 80%
- All new code should include tests
- Critical components should aim for 90%+ coverage

## Writing Tests

### Unit Tests

Unit tests should be placed in the appropriate directory under `tests/unit/modules/`, mirroring the structure of the `src/modules/` directory. For example:

```rust
// tests/unit/modules/router_core/unit_tests.rs

use intellirouter::modules::router_core::{RouterConfig, RoutingStrategy, init};

#[test]
fn test_router_initialization() {
    let config = RouterConfig {
        strategy: RoutingStrategy::ContentBased,
    };
    
    let result = init(config);
    assert!(result.is_ok());
}
```

Then, include the test file in the module's `mod.rs` file:

```rust
// tests/unit/modules/router_core/mod.rs

mod unit_tests;
```

### Integration Tests

Integration tests should be placed in the `tests/integration/` directory. These tests verify that different components work together correctly.

```rust
// tests/integration/router_integration_tests.rs

use intellirouter::modules::router_core;
use intellirouter_test_utils::fixtures::create_test_request;

#[test]
fn test_end_to_end_request_flow() {
    // Test the full request flow through the system
    let request = create_test_request("Test request content");
    let routing_result = router_core::route_request(&request);
    assert!(routing_result.is_ok());
}
```

### Property-Based Tests

Property-based tests use frameworks like `proptest` and `quickcheck` to test properties and invariants across a wide range of inputs.

```rust
// tests/property_tests.rs

proptest! {
    #[test]
    fn router_handles_any_string(s in "\\PC*") {
        let router_config = RouterConfig {
            strategy: RoutingStrategy::ContentBased,
        };
        let init_result = router_core::init(router_config);
        assert!(init_result.is_ok());
        
        let routing_result = router_core::route_request(&s);
        assert!(routing_result.is_ok());
    }
}
```

### End-to-End Tests

End-to-end tests should be placed in the `tests/e2e/` directory. These tests validate complete workflows through the entire system.

```rust
// tests/e2e/api/model_routing_tests.rs

use intellirouter_test_utils::helpers::spawn_test_server;
use intellirouter_test_utils::fixtures::create_test_client;

#[test]
fn test_model_routing_api() {
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
}
```

### Test Utilities

Common test utilities are provided in the separate `intellirouter-test-utils` crate. These include:

- Test fixtures
- Mock implementations
- Helper functions

```rust
// Example of using test utilities
use intellirouter_test_utils::fixtures::create_test_request;
use intellirouter_test_utils::mocks::create_mock_model_backend;
use intellirouter_test_utils::helpers::spawn_test_server;

#[test]
fn test_with_utilities() {
    let request = create_test_request("Test content");
    let mock_backend = create_mock_model_backend();
    let server = spawn_test_server().expect("Failed to spawn server");
    
    // Test implementation
}
```

## Mocking

We use [mockall](https://docs.rs/mockall/latest/mockall/) for creating mock implementations for testing.

Example of using a mock:

```rust
#[test]
fn test_with_mock_router() {
    let mut mock_router = MockRouter::new();
    
    mock_router
        .expect_route()
        .with(eq("test request"))
        .times(1)
        .returning(|_| Ok("mocked response".to_string()));
    
    let result = mock_router.route("test request");
    assert_eq!(result, Ok("mocked response".to_string()));
}
```

## Continuous Integration

Tests are automatically run on GitHub Actions for every pull request and push to the main branch. The workflow includes:

1. Running all tests (unit, integration, and end-to-end)
2. Running ignored (longer) tests separately
3. Checking code coverage
4. Running clippy for linting
5. Checking formatting with rustfmt
6. Generating test reports
7. Uploading test logs as artifacts

For more details on the CI integration, see [CI Integration](docs/ci_integration.md).

### Ignored Tests

Longer tests are marked with the `#[ignore]` attribute to prevent CI timeouts. These tests can be run separately:

```bash
cargo test -- --ignored
```

The following tests are marked with the `#[ignore]` attribute:

1. `test_end_to_end_request_flow`: Full end-to-end request flow through the system
2. `test_chat_completions_endpoint`: Chat completions endpoint with HTTP request
3. `test_multi_step_chain`: Multi-step chain execution with multiple models
4. `test_conditional_chain`: Conditional chain execution with multiple models
5. `test_error_handling_chain`: Error handling chain with failing model

## Best Practices

1. **Test-First Development**: Write tests before implementing functionality
   - Create test modules before implementing features
   - Define expected behavior through test assertions
   - Verify tests fail appropriately before implementation
   - Implement only what's needed to make tests pass
   - Refactor after tests are passing

2. **Test Isolation**: Tests should be independent and not rely on the state from other tests
3. **Descriptive Names**: Use descriptive test names that explain what is being tested
4. **Arrange-Act-Assert**: Structure tests with clear setup, action, and assertion phases
5. **Test Edge Cases**: Include tests for edge cases and error conditions
6. **Keep Tests Fast**: Tests should run quickly to provide fast feedback
7. **Test Public API**: Focus on testing the public API of modules
8. **Use Test Utilities**: Use the provided test utilities for common testing tasks
9. **Document Test Strategy**: Include comments explaining the testing strategy for complex tests

## Adding New Tests

When adding new functionality, follow these test-first steps:

1. Write unit tests for the new functionality **before** implementing it
2. Verify that the tests fail appropriately (since the functionality doesn't exist yet)
3. Implement the minimum code needed to make the tests pass
4. Add integration tests if the functionality interacts with other components
5. Consider adding property-based tests for invariants
6. Run the tests and coverage to ensure adequate test coverage
7. Refactor the code while maintaining passing tests
8. Update this documentation if necessary

## Test Environment

Tests can use the configuration in `config/testing.toml` for test-specific configuration.

For tests that require external services, use the provided mock implementations or set up test fixtures.

### Test Templates

The `tests/templates/` directory contains templates for different types of tests:

- `unit_test_template.rs`: Template for unit tests
- `integration_test_template.rs`: Template for integration tests
- `e2e_test_template.rs`: Template for end-to-end tests

Copy these templates and adapt them to your specific testing needs.

### Docker-based Integration Testing

For integration testing with all system components, use the Docker Compose configuration:

```bash
docker-compose -f docker-compose.integration.yml up -d
```

This will start all required services in containers. See [Integration Testing](INTEGRATION_TESTING.md) for more details.

### Test Logging

Tests are configured to output detailed logs for debugging. The logs include:

- File and line number information
- Thread IDs
- Target information
- Timestamps

To capture test output to a file, use the `init_test_logging_with_file` function:

```rust
use intellirouter_test_utils::helpers::init_test_logging_with_file;

#[test]
fn my_test() {
    init_test_logging_with_file("my_test").unwrap();
    // Test implementation...
}
```

This will create a log file in the `logs/` directory with the name `my_test.log`.

## Additional Resources

- [Testing Guide](docs/testing_guide.md): Comprehensive guide to the new test structure
- [Test Harness Documentation](docs/test_harness.md): Documentation for the test harness
- [Integration Testing](INTEGRATION_TESTING.md): Guide for integration testing with Docker Compose
- [Testing Policy](docs/testing_policy.md): Test-first development policy