//! Router assertions for the assertion framework.
//!
//! This module provides assertions specific to the IntelliRouter router component.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents a router request for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterRequest {
    /// The request ID.
    pub request_id: String,
    /// The model ID.
    pub model_id: String,
    /// The request parameters.
    pub parameters: Value,
    /// The request timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The request metadata.
    pub metadata: Value,
}

impl RouterRequest {
    /// Creates a new router request.
    pub fn new(request_id: &str, model_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            model_id: model_id.to_string(),
            parameters: Value::Null,
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
        }
    }

    /// Sets the request parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the request timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the request metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a router response for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterResponse {
    /// The response ID.
    pub response_id: String,
    /// The request ID.
    pub request_id: String,
    /// The model ID.
    pub model_id: String,
    /// The response data.
    pub data: Value,
    /// The response timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The response metadata.
    pub metadata: Value,
    /// The response time.
    pub response_time: Duration,
    /// Whether the response was cached.
    pub cached: bool,
    /// The error, if any.
    pub error: Option<String>,
}

impl RouterResponse {
    /// Creates a new router response.
    pub fn new(response_id: &str, request_id: &str, model_id: &str) -> Self {
        Self {
            response_id: response_id.to_string(),
            request_id: request_id.to_string(),
            model_id: model_id.to_string(),
            data: Value::Null,
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
            response_time: Duration::default(),
            cached: false,
            error: None,
        }
    }

    /// Sets the response data.
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Sets the response timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the response metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the response time.
    pub fn with_response_time(mut self, response_time: Duration) -> Self {
        self.response_time = response_time;
        self
    }

    /// Sets whether the response was cached.
    pub fn with_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Sets the error.
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }
}

/// Represents a router model for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterModel {
    /// The model ID.
    pub model_id: String,
    /// The model provider.
    pub provider: String,
    /// The model version.
    pub version: String,
    /// The model capabilities.
    pub capabilities: Vec<String>,
    /// The model parameters.
    pub parameters: Value,
    /// The model metadata.
    pub metadata: Value,
}

impl RouterModel {
    /// Creates a new router model.
    pub fn new(model_id: &str, provider: &str) -> Self {
        Self {
            model_id: model_id.to_string(),
            provider: provider.to_string(),
            version: "".to_string(),
            capabilities: Vec::new(),
            parameters: Value::Null,
            metadata: Value::Null,
        }
    }

    /// Sets the model version.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Adds a capability to the model.
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }

    /// Sets the model parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the model metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a router route for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterRoute {
    /// The route ID.
    pub route_id: String,
    /// The route pattern.
    pub pattern: String,
    /// The target model ID.
    pub target_model_id: String,
    /// The route priority.
    pub priority: i32,
    /// The route conditions.
    pub conditions: Value,
    /// The route transformations.
    pub transformations: Value,
    /// The route metadata.
    pub metadata: Value,
}

impl RouterRoute {
    /// Creates a new router route.
    pub fn new(route_id: &str, pattern: &str, target_model_id: &str) -> Self {
        Self {
            route_id: route_id.to_string(),
            pattern: pattern.to_string(),
            target_model_id: target_model_id.to_string(),
            priority: 0,
            conditions: Value::Null,
            transformations: Value::Null,
            metadata: Value::Null,
        }
    }

    /// Sets the route priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the route conditions.
    pub fn with_conditions(mut self, conditions: Value) -> Self {
        self.conditions = conditions;
        self
    }

    /// Sets the route transformations.
    pub fn with_transformations(mut self, transformations: Value) -> Self {
        self.transformations = transformations;
        self
    }

    /// Sets the route metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Assertions for router components.
#[derive(Debug, Clone)]
pub struct RouterAssertions;

impl RouterAssertions {
    /// Creates a new router assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a response has a specific model ID.
    pub fn assert_model_id(&self, response: &RouterResponse, expected: &str) -> AssertionResult {
        assert_that(response.model_id.as_str())
            .with_name(&format!("Model ID is '{}'", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response was successful (no error).
    pub fn assert_success(&self, response: &RouterResponse) -> AssertionResult {
        match &response.error {
            None => AssertionResult::new("Response is successful", AssertionOutcome::Passed),
            Some(error) => AssertionResult::new("Response is successful", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Response has an error",
                        "No error",
                        error,
                    ),
                ),
        }
    }

    /// Asserts that a response has a specific error.
    pub fn assert_error(&self, response: &RouterResponse, expected: &str) -> AssertionResult {
        match &response.error {
            Some(error) => {
                if error == expected {
                    AssertionResult::new(
                        &format!("Response has error '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Response has error '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Response has a different error",
                            expected,
                            error,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Response has error '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response does not have an error",
                    expected,
                    "No error",
                ),
            ),
        }
    }

    /// Asserts that a response was cached.
    pub fn assert_cached(&self, response: &RouterResponse) -> AssertionResult {
        if response.cached {
            AssertionResult::new("Response is cached", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("Response is cached", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response is not cached",
                    "Cached",
                    "Not cached",
                ),
            )
        }
    }

    /// Asserts that a response was not cached.
    pub fn assert_not_cached(&self, response: &RouterResponse) -> AssertionResult {
        if !response.cached {
            AssertionResult::new("Response is not cached", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("Response is not cached", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response is cached",
                    "Not cached",
                    "Cached",
                ),
            )
        }
    }

    /// Asserts that a response was received within a specific time.
    pub fn assert_response_time(&self, response: &RouterResponse, max_ms: u64) -> AssertionResult {
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

    /// Asserts that a model has a specific capability.
    pub fn assert_model_capability(
        &self,
        model: &RouterModel,
        capability: &str,
    ) -> AssertionResult {
        if model.capabilities.contains(&capability.to_string()) {
            AssertionResult::new(
                &format!("Model has capability '{}'", capability),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Model has capability '{}'", capability),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Model does not have capability '{}'", capability),
                    capability,
                    &format!("{:?}", model.capabilities),
                ),
            )
        }
    }

    /// Asserts that a route has a specific priority.
    pub fn assert_route_priority(&self, route: &RouterRoute, expected: i32) -> AssertionResult {
        assert_that(route.priority)
            .with_name(&format!("Route priority is {}", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a route has a specific target model ID.
    pub fn assert_route_target_model_id(
        &self,
        route: &RouterRoute,
        expected: &str,
    ) -> AssertionResult {
        assert_that(route.target_model_id.as_str())
            .with_name(&format!("Route target model ID is '{}'", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a route matches a specific pattern.
    pub fn assert_route_matches_pattern(
        &self,
        route: &RouterRoute,
        pattern: &str,
    ) -> AssertionResult {
        assert_that(route.pattern.as_str())
            .with_name(&format!("Route pattern is '{}'", pattern))
            .is_equal_to(pattern)
    }
}

impl Default for RouterAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new router assertions instance.
pub fn create_router_assertions() -> RouterAssertions {
    RouterAssertions::new()
}

/// Creates a new router request.
pub fn create_router_request(request_id: &str, model_id: &str) -> RouterRequest {
    RouterRequest::new(request_id, model_id)
}

/// Creates a new router response.
pub fn create_router_response(
    response_id: &str,
    request_id: &str,
    model_id: &str,
) -> RouterResponse {
    RouterResponse::new(response_id, request_id, model_id)
}

/// Creates a new router model.
pub fn create_router_model(model_id: &str, provider: &str) -> RouterModel {
    RouterModel::new(model_id, provider)
}

/// Creates a new router route.
pub fn create_router_route(route_id: &str, pattern: &str, target_model_id: &str) -> RouterRoute {
    RouterRoute::new(route_id, pattern, target_model_id)
}
