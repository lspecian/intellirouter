# IntelliRouter Testing Guide

This document outlines the testing approach for the IntelliRouter project, including unit tests, integration tests, property-based tests, and test coverage reporting.

## Testing Philosophy

IntelliRouter follows a comprehensive testing approach to ensure the reliability and correctness of the codebase:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test how components work together
3. **Property-Based Tests**: Test invariants and properties across a wide range of inputs
4. **Test Coverage**: Ensure code is adequately tested

## Test Structure

The test structure is organized as follows:

- **Unit Tests**: Located within each module in `src/modules/*/tests.rs`
- **Integration Tests**: Located in the `tests/` directory
- **Property-Based Tests**: Located in `tests/property_tests.rs`
- **Test Utilities**: Located in `src/test_utils.rs`

## Running Tests

### Running All Tests

```bash
cargo test
```

### Running Unit Tests Only

```bash
cargo test --lib
```

### Running Integration Tests Only

```bash
cargo test --test integration_test
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

Unit tests should be placed in a `tests.rs` file within each module. For example:

```rust
// src/modules/router_core/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_initialization() {
        let config = RouterConfig {
            strategy: RoutingStrategy::ContentBased,
        };
        
        let result = init(config);
        assert!(result.is_ok());
    }
}
```

Then, include the tests module in the module's `mod.rs` file:

```rust
// src/modules/router_core/mod.rs

#[cfg(test)]
mod tests;
```

### Integration Tests

Integration tests should be placed in the `tests/` directory. These tests verify that different components work together correctly.

```rust
// tests/integration_test.rs

#[test]
fn test_end_to_end_request_flow() {
    // Test the full request flow through the system
    let request = test_utils::create_test_request("Test request content");
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

### Test Utilities

Common test utilities are provided in the `src/test_utils.rs` module. These include:

- Test fixtures
- Mock implementations
- Helper functions

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

1. **Test Isolation**: Tests should be independent and not rely on the state from other tests
2. **Descriptive Names**: Use descriptive test names that explain what is being tested
3. **Arrange-Act-Assert**: Structure tests with clear setup, action, and assertion phases
4. **Test Edge Cases**: Include tests for edge cases and error conditions
5. **Keep Tests Fast**: Tests should run quickly to provide fast feedback
6. **Test Public API**: Focus on testing the public API of modules
7. **Use Test Utilities**: Use the provided test utilities for common testing tasks
8. **Document Test Strategy**: Include comments explaining the testing strategy for complex tests

## Adding New Tests

When adding new functionality, follow these steps:

1. Add unit tests for the new functionality
2. Add integration tests if the functionality interacts with other components
3. Consider adding property-based tests for invariants
4. Run the tests and coverage to ensure adequate test coverage
5. Update this documentation if necessary

## Test Environment

Tests can use the configuration in `config/testing.toml` for test-specific configuration.

For tests that require external services, use the provided mock implementations or set up test fixtures.

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
use intellirouter::test_utils::init_test_logging_with_file;

#[test]
fn my_test() {
    init_test_logging_with_file("my_test").unwrap();
    // Test implementation...
}
```

This will create a log file in the `logs/` directory with the name `my_test.log`.