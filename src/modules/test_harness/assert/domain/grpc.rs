//! gRPC assertions for the assertion framework.
//!
//! This module provides assertions for gRPC requests and responses.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents a gRPC request for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcRequest {
    /// The request ID.
    pub request_id: String,
    /// The service name.
    pub service: String,
    /// The method name.
    pub method: String,
    /// The request message.
    pub message: Value,
    /// The request metadata.
    pub metadata: HashMap<String, String>,
    /// The request timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl GrpcRequest {
    /// Creates a new gRPC request.
    pub fn new(request_id: &str, service: &str, method: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            service: service.to_string(),
            method: method.to_string(),
            message: Value::Null,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Sets the request message.
    pub fn with_message(mut self, message: Value) -> Self {
        self.message = message;
        self
    }

    /// Adds a metadata entry to the request.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the request timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Represents a gRPC response for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcResponse {
    /// The response ID.
    pub response_id: String,
    /// The request ID.
    pub request_id: String,
    /// The service name.
    pub service: String,
    /// The method name.
    pub method: String,
    /// The response message.
    pub message: Value,
    /// The response metadata.
    pub metadata: HashMap<String, String>,
    /// The response timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The response time.
    pub response_time: Duration,
    /// The status code.
    pub status_code: i32,
    /// The status message.
    pub status_message: Option<String>,
    /// The error details.
    pub error_details: Option<Value>,
}

impl GrpcResponse {
    /// Creates a new gRPC response.
    pub fn new(response_id: &str, request_id: &str, service: &str, method: &str) -> Self {
        Self {
            response_id: response_id.to_string(),
            request_id: request_id.to_string(),
            service: service.to_string(),
            method: method.to_string(),
            message: Value::Null,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            response_time: Duration::default(),
            status_code: 0,
            status_message: None,
            error_details: None,
        }
    }

    /// Sets the response message.
    pub fn with_message(mut self, message: Value) -> Self {
        self.message = message;
        self
    }

    /// Adds a metadata entry to the response.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the response timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the response time.
    pub fn with_response_time(mut self, response_time: Duration) -> Self {
        self.response_time = response_time;
        self
    }

    /// Sets the status code.
    pub fn with_status_code(mut self, status_code: i32) -> Self {
        self.status_code = status_code;
        self
    }

    /// Sets the status message.
    pub fn with_status_message(mut self, status_message: &str) -> Self {
        self.status_message = Some(status_message.to_string());
        self
    }

    /// Sets the error details.
    pub fn with_error_details(mut self, error_details: Value) -> Self {
        self.error_details = Some(error_details);
        self
    }
}

/// Assertions for gRPC requests and responses.
#[derive(Debug, Clone)]
pub struct GrpcAssertions;

impl GrpcAssertions {
    /// Creates a new gRPC assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a response has a specific status code.
    pub fn assert_status_code(&self, response: &GrpcResponse, expected: i32) -> AssertionResult {
        assert_that(response.status_code)
            .with_name(&format!("Status code is {}", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response was successful (status code 0).
    pub fn assert_success(&self, response: &GrpcResponse) -> AssertionResult {
        self.assert_status_code(response, 0)
    }

    /// Asserts that a response has a specific status message.
    pub fn assert_status_message(
        &self,
        response: &GrpcResponse,
        expected: &str,
    ) -> AssertionResult {
        match &response.status_message {
            Some(message) => {
                if message == expected {
                    AssertionResult::new(
                        &format!("Status message is '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Status message is '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Status message does not match expected value",
                            expected,
                            message,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Status message is '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response does not have a status message",
                    expected,
                    "No status message",
                ),
            ),
        }
    }

    /// Asserts that a response has a specific metadata entry.
    pub fn assert_has_metadata(&self, response: &GrpcResponse, key: &str) -> AssertionResult {
        if response.metadata.contains_key(key) {
            AssertionResult::new(&format!("Has metadata '{}'", key), AssertionOutcome::Passed)
        } else {
            AssertionResult::new(&format!("Has metadata '{}'", key), AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        &format!("Response does not have metadata '{}'", key),
                        &format!("Metadata '{}'", key),
                        "No such metadata",
                    ),
                )
        }
    }

    /// Asserts that a response has a metadata entry with a specific value.
    pub fn assert_metadata_value(
        &self,
        response: &GrpcResponse,
        key: &str,
        expected: &str,
    ) -> AssertionResult {
        match response.metadata.get(key) {
            Some(value) => {
                if value == expected {
                    AssertionResult::new(
                        &format!("Metadata '{}' has value '{}'", key, expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Metadata '{}' has value '{}'", key, expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Metadata '{}' has unexpected value", key),
                            expected,
                            value,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Metadata '{}' has value '{}'", key, expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Response does not have metadata '{}'", key),
                    &format!("Metadata '{}' with value '{}'", key, expected),
                    "No such metadata",
                ),
            ),
        }
    }

    /// Asserts that a response has a specific message field.
    pub fn assert_has_message_field(
        &self,
        response: &GrpcResponse,
        field: &str,
    ) -> AssertionResult {
        let has_field = match &response.message {
            Value::Object(obj) => obj.contains_key(field),
            _ => false,
        };

        if has_field {
            AssertionResult::new(
                &format!("Message has field '{}'", field),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Message has field '{}'", field),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Message does not have field '{}'", field),
                    &format!("Field '{}'", field),
                    &format!("{:?}", response.message),
                ),
            )
        }
    }

    /// Asserts that a response has a message field with a specific value.
    pub fn assert_message_field_equals(
        &self,
        response: &GrpcResponse,
        field: &str,
        expected: Value,
    ) -> AssertionResult {
        let field_value = match &response.message {
            Value::Object(obj) => obj.get(field).cloned(),
            _ => None,
        };

        match field_value {
            Some(value) => {
                if value == expected {
                    AssertionResult::new(
                        &format!("Message field '{}' equals expected value", field),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Message field '{}' equals expected value", field),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Message field '{}' does not equal expected value", field),
                            &format!("{:?}", expected),
                            &format!("{:?}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Message field '{}' equals expected value", field),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Message does not have field '{}'", field),
                    &format!("Field '{}' with value '{:?}'", field, expected),
                    &format!("{:?}", response.message),
                ),
            ),
        }
    }

    /// Asserts that a response was received within a specific time.
    pub fn assert_response_time(&self, response: &GrpcResponse, max_ms: u64) -> AssertionResult {
        let response_time_ms = response.response_time.as_millis() as u64;

        if response_time_ms <= max_ms {
            AssertionResult::new(
                &format!("Response time <= {} ms", max_ms),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Response time <= {} ms", max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response time exceeds maximum",
                    &format!("<= {} ms", max_ms),
                    &format!("{} ms", response_time_ms),
                ),
            )
        }
    }
}

impl Default for GrpcAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new gRPC assertions instance.
pub fn create_grpc_assertions() -> GrpcAssertions {
    GrpcAssertions::new()
}

/// Creates a new gRPC request.
pub fn create_grpc_request(request_id: &str, service: &str, method: &str) -> GrpcRequest {
    GrpcRequest::new(request_id, service, method)
}

/// Creates a new gRPC response.
pub fn create_grpc_response(
    response_id: &str,
    request_id: &str,
    service: &str,
    method: &str,
) -> GrpcResponse {
    GrpcResponse::new(response_id, request_id, service, method)
}

use std::collections::HashMap;
