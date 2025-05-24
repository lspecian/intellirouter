# Feature Flags in IntelliRouter

This document explains the feature flags used in the IntelliRouter project, particularly focusing on the `test-utils` feature flag that controls the inclusion of test helpers in production builds.

## Available Feature Flags

### `test-utils`

The `test-utils` feature flag controls the inclusion of test utilities and mock implementations in the codebase. When enabled, it makes test helpers available to production code. When disabled (in production builds), these test helpers are excluded.

#### Usage

To enable the `test-utils` feature flag, add it to your Cargo.toml dependencies:

```toml
[dependencies]
intellirouter = { version = "0.1.0", features = ["test-utils"] }
```

Or when running Cargo commands:

```bash
cargo build --features test-utils
```

#### Components Controlled by `test-utils`

The following components are only available when the `test-utils` feature flag is enabled:

1. **Mock Model Backend**
   - `src/modules/llm_proxy/mock_backend.rs`: A mock implementation of a model backend for testing
   - Re-exported in `src/modules/llm_proxy/mod.rs`

2. **Mock Router Service Functions**
   - `create_mock_router_service()`: Creates a router service with a mock backend
   - `create_mock_router_service_with_config()`: Creates a router service with a custom configuration
   - `create_mock_router_service_with_errors()`: Creates a router service with error simulation

3. **Service Layer Mock Functions**
   - `ChatCompletionService::new_with_mock_router()`: Creates a chat completion service with a mock router

4. **Test Client Binary**
   - `src/bin/test_client.rs`: A simple client for testing the chat completion service

5. **Audit Module Communication Helpers**
   - Re-exports from `intellirouter-test-utils` in `src/modules/audit/communication_tests.rs`

## Using Test Helpers in Non-Production Environments

The `test-utils` feature flag allows you to include test helpers in development and testing environments while excluding them from production builds. This is useful for:

1. **Development**: Use mock implementations during development to avoid dependencies on external services
2. **Testing**: Include test helpers in test builds to facilitate testing
3. **Benchmarking**: Use mock implementations for benchmarking to isolate performance issues

## Best Practices

1. **Always use feature flags for test code**: Any code that is primarily for testing should be behind the `test-utils` feature flag
2. **Keep production code clean**: Avoid dependencies on test code in production code
3. **Test both with and without feature flags**: Ensure your code works correctly both with and without the `test-utils` feature flag enabled
4. **Document feature flag usage**: Always document when a component requires a feature flag to be enabled

## Example: Using Mock Model Backend

```rust
// This code will only compile when the test-utils feature is enabled
#[cfg(feature = "test-utils")]
fn test_with_mock_backend() {
    use intellirouter::modules::llm_proxy::MockModelBackend;
    
    let backend = MockModelBackend::new(
        "mock-model".to_string(),
        "Mock Model".to_string(),
        "mock-provider".to_string(),
    );
    
    // Use the mock backend for testing
}