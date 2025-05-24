# Unit Tests

This directory contains unit tests for IntelliRouter components. The structure mirrors the `src/modules/` directory structure to make it easy to find tests for specific components.

## Directory Structure

```
unit/
└── modules/           # Tests for specific modules
    ├── audit/         # Tests for audit module
    ├── ipc/           # Tests for IPC module
    ├── router_core/   # Tests for router_core module
    └── ...            # Tests for other modules
```

## Writing Unit Tests

Unit tests should focus on testing individual components in isolation. Use mocks for dependencies to ensure that you're only testing the component itself.

### Example

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

## Running Unit Tests

To run all unit tests:

```bash
cargo test --test 'unit_*'
```

To run tests for a specific module:

```bash
cargo test unit::modules::router_core
```

## Best Practices

1. **Test in Isolation**: Use mocks for dependencies to ensure that you're only testing the component itself.
2. **Test Public API**: Focus on testing the public API of modules.
3. **Test Edge Cases**: Include tests for edge cases and error conditions.
4. **Descriptive Names**: Use descriptive test names that explain what is being tested.
5. **Arrange-Act-Assert**: Structure tests with clear setup, action, and assertion phases.
6. **Test-First Development**: Write tests before implementing functionality.

## Test Utilities

Use the `intellirouter-test-utils` crate for common test utilities:

```rust
use intellirouter_test_utils::fixtures::create_test_request;
use intellirouter_test_utils::mocks::create_mock_model_backend;
```

For more details on unit testing, see the [Testing Guide](../../docs/testing_guide.md).