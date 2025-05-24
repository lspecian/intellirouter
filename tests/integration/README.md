# Integration Tests

This directory contains integration tests for IntelliRouter. Integration tests verify that different components work together correctly.

## Purpose

Integration tests focus on testing the interactions between different components of the system. They verify that:

1. Components can communicate with each other
2. Data flows correctly between components
3. API contracts are respected
4. Error handling works across component boundaries
5. The system behaves correctly as a whole

## Directory Structure

```
integration/
├── chain_tests.rs            # Tests for chain execution
├── router_integration_tests.rs # Tests for router integration
└── ...                       # Other integration tests
```

## Writing Integration Tests

Integration tests should focus on testing the interactions between components. They should use real implementations of components when possible, but may use mocks for external dependencies.

### Example

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

## Running Integration Tests

To run all integration tests:

```bash
cargo test --test 'integration_*'
```

To run a specific integration test:

```bash
cargo test --test integration_test_name
```

## Docker-based Integration Testing

For integration testing with all system components, use the Docker Compose configuration:

```bash
docker-compose -f docker-compose.integration.yml up -d
docker-compose -f docker-compose.integration.yml run test-runner
```

See [Integration Testing](../../INTEGRATION_TESTING.md) for more details.

## Best Practices

1. **Focus on Interactions**: Test how components work together, not individual component behavior.
2. **Use Real Implementations**: Use real implementations of components when possible.
3. **Mock External Dependencies**: Use mocks for external dependencies like databases or APIs.
4. **Test Error Handling**: Verify that errors are properly propagated between components.
5. **Test API Contracts**: Verify that components respect their API contracts.
6. **Test Data Flow**: Verify that data flows correctly between components.

## Test Utilities

Use the `intellirouter-test-utils` crate for common test utilities:

```rust
use intellirouter_test_utils::fixtures::create_test_request;
use intellirouter_test_utils::helpers::spawn_test_server;
```

For more details on integration testing, see the [Testing Guide](../../docs/testing_guide.md) and [Integration Testing](../../INTEGRATION_TESTING.md).