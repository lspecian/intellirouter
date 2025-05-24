# Test Templates

This directory contains templates for writing tests following IntelliRouter's test-first approach. These templates provide a starting point for writing different types of tests and demonstrate best practices.

## Available Templates

- [Unit Test Template](unit_test_template.rs): Template for writing unit tests
- [Integration Test Template](integration_test_template.rs): Template for writing integration tests

## How to Use These Templates

1. **Choose the appropriate template** based on the type of test you need to write
2. **Copy the template** to your test file location
3. **Adapt the template** to your specific testing needs
4. **Remove any unnecessary sections** that don't apply to your test case
5. **Run the tests** to verify they fail (since you haven't implemented the functionality yet)
6. **Implement the functionality** to make the tests pass

## Unit Test Example

For a module `src/modules/router/strategy.rs`, you would create a test module within the same file:

```rust
// src/modules/router/strategy.rs

// Module implementation...

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_based_routing() {
        // Arrange
        let strategy = ContentBasedStrategy::new();
        let request = Request::new("test content");
        
        // Act
        let result = strategy.route(request);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().target(), "expected target");
    }
    
    // More tests...
}
```

## Integration Test Example

For testing the interaction between the router and model registry modules, you would create a test file in the `tests` directory:

```rust
// tests/router_model_registry_integration.rs

use intellirouter::router;
use intellirouter::model_registry;
use intellirouter::test_utils;

#[test]
fn test_router_uses_model_registry() {
    // Arrange
    let registry = model_registry::ModelRegistry::new();
    registry.register_model("test-model", test_utils::create_test_model())
        .expect("Failed to register model");
    
    let router = router::Router::new(registry);
    let request = test_utils::create_test_request("test-model");
    
    // Act
    let result = router.route(request);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().model_id(), "test-model");
}
```

## Best Practices

1. **Write tests first**: Always write tests before implementing functionality
2. **Verify test failure**: Run tests to verify they fail before implementation
3. **Use descriptive test names**: Test names should describe what is being tested
4. **Follow AAA pattern**: Arrange, Act, Assert
5. **Test edge cases**: Include tests for edge cases and error conditions
6. **Keep tests independent**: Tests should not depend on each other
7. **Use test utilities**: Use the provided test utilities for common testing tasks

## Additional Resources

- [Testing Policy](../../docs/testing_policy.md): Comprehensive guide to IntelliRouter's test-first approach
- [Test-First Development Rule](../../.roo/rules/test_first.md): Roo rule enforcing the test-first approach
- [TESTING.md](../../TESTING.md): General testing guidelines for IntelliRouter