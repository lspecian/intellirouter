# IntelliRouter Test Utilities

This crate provides test utilities, fixtures, and mocks for testing the IntelliRouter project.
It is designed to be used as a dev-dependency in the main IntelliRouter crate and other related crates.

## Features

- **Fixtures**: Common test data and fixtures for testing
- **Mocks**: Mock implementations of IntelliRouter components and services
- **Helpers**: Helper functions and utilities for testing

## Module Structure

The crate is organized into the following modules:

### Fixtures (`fixtures.rs`)

- **Common Fixtures**: Basic test data like temporary directories and sample payloads
- **Audit Fixtures**: Test fixtures for audit functionality
  - `ServiceType`: Enum for different service types
  - `ServiceStatus`: Enum for service status
  - `ServiceInfo`: Service information for testing
  - `CommunicationTestResult`: Test result for service communication
  - `sample_services()`: Creates a sample set of services for testing

### Mocks (`mocks.rs`)

- **Common Mocks**: Generic mock implementations like `MockHttpServer`
- **Audit Mocks**: Mocks for audit functionality
  - `MockAuditController`: Mock for the audit controller
  - `MockServiceHealthCheck`: Mock for service health checks
  - `MockServiceClient`: Mock HTTP client for testing service communication

### Helpers (`helpers.rs`)

- **Common Helpers**: Utility functions like `wait_for_condition` and `retry`
- **Communication Helpers**: Utilities for testing service communication
  - `test_service_connection()`: Test if a service can reach another service
  - `test_redis_connection()`: Test if a service can connect to Redis
  - `test_grpc_communication()`: Test gRPC communication between services
  - `test_redis_pubsub()`: Test Redis pub/sub communication

## Usage

Add this crate as a dev-dependency in your Cargo.toml:

```toml
[dev-dependencies]
intellirouter-test-utils = { path = "../intellirouter-test-utils" }
```

Then import and use the utilities in your tests:

```rust
// Import common utilities
use intellirouter_test_utils::fixtures;
use intellirouter_test_utils::mocks;
use intellirouter_test_utils::helpers;

// Import specific modules
use intellirouter_test_utils::fixtures::audit::{ServiceType, ServiceInfo};
use intellirouter_test_utils::mocks::audit::MockServiceClient;
use intellirouter_test_utils::helpers::communication::test_service_connection;

#[tokio::test]
async fn test_service_communication() {
    // Create a mock service client
    let mock_client = MockServiceClient::new();
    
    // Mock a health check endpoint
    mock_client.mock_health_check(ServiceType::Router, true);
    
    // Test service communication
    let result = test_service_connection(
        &reqwest::Client::new(),
        "http://router:8080",
        "http://orchestrator:8080"
    ).await;
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}
```

## Feature Flags

- `with-intellirouter`: Enables integration with the main IntelliRouter crate