# End-to-End Tests

This directory contains end-to-end (E2E) tests for IntelliRouter. E2E tests verify that the entire system works correctly from end to end.

## Purpose

End-to-end tests focus on testing the system as a whole, from the user's perspective. They verify that:

1. Complete workflows function correctly
2. The system behaves correctly in real-world scenarios
3. All components work together as expected
4. Performance meets requirements
5. The system is resilient to failures

## Directory Structure

```
e2e/
├── api/                # API-focused end-to-end tests
│   └── model_routing_tests.rs # Tests for model routing API
├── performance/        # Performance and load tests
│   └── latency_tests.rs # Tests for system latency
└── scenarios/          # Scenario-based end-to-end tests
    ├── rag_tests.rs    # Tests for RAG scenarios
    └── workflow_tests.rs # Tests for workflow scenarios
```

## Writing End-to-End Tests

End-to-end tests should focus on testing complete workflows from the user's perspective. They should use real implementations of all components and interact with the system through its public APIs.

### Example

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
    let json = response.json::<serde_json::Value>().expect("Failed to parse JSON");
    assert!(json.get("choices").is_some());
}
```

## Running End-to-End Tests

To run all end-to-end tests:

```bash
cargo test --test 'e2e_*'
```

To run a specific end-to-end test:

```bash
cargo test --test e2e_test_name
```

Some end-to-end tests may be marked with `#[ignore]` because they take a long time to run. To run these tests:

```bash
cargo test --test 'e2e_*' -- --ignored
```

## Docker-based End-to-End Testing

For end-to-end testing with all system components, use the Docker Compose configuration:

```bash
docker-compose -f docker-compose.integration.yml up -d
docker-compose -f docker-compose.integration.yml run test-runner
```

See [Integration Testing](../../INTEGRATION_TESTING.md) for more details.

## Test Categories

### API Tests

API tests focus on testing the system's public APIs. They verify that:

1. API endpoints return the expected responses
2. Error handling works correctly
3. API contracts are respected

### Performance Tests

Performance tests focus on testing the system's performance characteristics. They verify that:

1. The system meets latency requirements
2. The system can handle the expected load
3. The system degrades gracefully under high load

### Scenario Tests

Scenario tests focus on testing complete workflows that represent real-world usage scenarios. They verify that:

1. The system behaves correctly in realistic scenarios
2. All components work together as expected
3. The system is resilient to failures

## Best Practices

1. **Test Complete Workflows**: Focus on testing complete workflows from the user's perspective.
2. **Use Real Implementations**: Use real implementations of all components.
3. **Test Through Public APIs**: Interact with the system through its public APIs.
4. **Test Realistic Scenarios**: Test scenarios that represent real-world usage.
5. **Test Performance**: Verify that the system meets performance requirements.
6. **Test Resilience**: Verify that the system is resilient to failures.

## Test Utilities

Use the `intellirouter-test-utils` crate for common test utilities:

```rust
use intellirouter_test_utils::helpers::spawn_test_server;
use intellirouter_test_utils::fixtures::create_test_client;
```

For more details on end-to-end testing, see the [Testing Guide](../../docs/testing_guide.md) and [Integration Testing](../../INTEGRATION_TESTING.md).