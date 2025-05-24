//! Mock Behavior Module
//!
//! This module provides functionality for defining mock behaviors and responses.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::recorder::{Interaction, MockRecorder};
use crate::modules::test_harness::types::TestHarnessError;

/// Response builder for creating mock responses
pub struct ResponseBuilder {
    /// Response data
    data: Option<serde_json::Value>,
    /// Response status
    status: Option<u16>,
    /// Response headers
    headers: HashMap<String, String>,
    /// Response delay
    delay: Option<Duration>,
    /// Response error
    error: Option<String>,
}

impl ResponseBuilder {
    /// Create a new response builder
    pub fn new() -> Self {
        Self {
            data: None,
            status: None,
            headers: HashMap::new(),
            delay: None,
            error: None,
        }
    }

    /// Set the response data
    pub fn with_data(mut self, data: impl Serialize) -> Result<Self, TestHarnessError> {
        self.data = Some(serde_json::to_value(data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize data: {}", e))
        })?);
        Ok(self)
    }

    /// Set the response status
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    /// Add a response header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add multiple response headers
    pub fn with_headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in headers {
            self.headers.insert(key.into(), value.into());
        }
        self
    }

    /// Set the response delay
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Set the response error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Build the response
    pub fn build(self) -> Response {
        Response {
            data: self.data,
            status: self.status.unwrap_or(200),
            headers: self.headers,
            delay: self.delay,
            error: self.error,
        }
    }

    /// Build a success response
    pub fn success(data: impl Serialize) -> Result<Response, TestHarnessError> {
        Self::new()
            .with_data(data)?
            .with_status(200)
            .build()
            .into_result()
    }

    /// Build an error response
    pub fn error(status: u16, message: impl Into<String>) -> Response {
        Self::new().with_status(status).with_error(message).build()
    }

    /// Build a not found response
    pub fn not_found(message: impl Into<String>) -> Response {
        Self::error(404, message)
    }

    /// Build a bad request response
    pub fn bad_request(message: impl Into<String>) -> Response {
        Self::error(400, message)
    }

    /// Build an unauthorized response
    pub fn unauthorized(message: impl Into<String>) -> Response {
        Self::error(401, message)
    }

    /// Build a forbidden response
    pub fn forbidden(message: impl Into<String>) -> Response {
        Self::error(403, message)
    }

    /// Build an internal server error response
    pub fn internal_server_error(message: impl Into<String>) -> Response {
        Self::error(500, message)
    }

    /// Build a service unavailable response
    pub fn service_unavailable(message: impl Into<String>) -> Response {
        Self::error(503, message)
    }

    /// Build a timeout response
    pub fn timeout(message: impl Into<String>) -> Response {
        Self::new()
            .with_status(408)
            .with_error(message)
            .with_delay(Duration::from_secs(30))
            .build()
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Response status
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response delay
    pub delay: Option<Duration>,
    /// Response error
    pub error: Option<String>,
}

impl Response {
    /// Create a new response
    pub fn new() -> Self {
        Self {
            data: None,
            status: 200,
            headers: HashMap::new(),
            delay: None,
            error: None,
        }
    }

    /// Create a new response builder
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300 && self.error.is_none()
    }

    /// Check if the response is an error
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Convert the response to a result
    pub fn into_result(self) -> Result<Response, TestHarnessError> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(TestHarnessError::MockError(
                self.error
                    .unwrap_or_else(|| format!("HTTP error: {}", self.status)),
            ))
        }
    }

    /// Get the response data
    pub fn data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, TestHarnessError> {
        if let Some(data) = &self.data {
            serde_json::from_value(data.clone()).map_err(|e| {
                TestHarnessError::SerializationError(format!("Failed to deserialize data: {}", e))
            })
        } else {
            Err(TestHarnessError::MockError(
                "No data in response".to_string(),
            ))
        }
    }

    /// Get a header value
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }

    /// Apply the response delay
    pub async fn apply_delay(&self) {
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

/// Behavior trait for implementing mock behaviors
#[async_trait]
pub trait Behavior: Send + Sync {
    /// Get the behavior name
    fn name(&self) -> &str;

    /// Get the behavior description
    fn description(&self) -> Option<&str>;

    /// Check if the behavior matches an interaction
    fn matches(&self, interaction: &Interaction) -> bool;

    /// Get the response for an interaction
    async fn respond(&self, interaction: &Interaction) -> Response;

    /// Get the number of times the behavior has been invoked
    fn invocation_count(&self) -> usize;

    /// Reset the behavior
    fn reset(&self);
}

/// Behavior builder for creating mock behaviors
pub struct BehaviorBuilder {
    /// Behavior name
    name: String,
    /// Behavior description
    description: Option<String>,
    /// Matcher function
    matcher: Option<Box<dyn Fn(&Interaction) -> bool + Send + Sync>>,
    /// Response function
    responder: Option<Box<dyn Fn(&Interaction) -> Response + Send + Sync>>,
    /// Invocation count
    invocation_count: Arc<RwLock<usize>>,
}

impl BehaviorBuilder {
    /// Create a new behavior builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            matcher: None,
            responder: None,
            invocation_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Set the behavior description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the matcher function
    pub fn with_matcher(
        mut self,
        matcher: impl Fn(&Interaction) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.matcher = Some(Box::new(matcher));
        self
    }

    /// Set the responder function
    pub fn with_responder(
        mut self,
        responder: impl Fn(&Interaction) -> Response + Send + Sync + 'static,
    ) -> Self {
        self.responder = Some(Box::new(responder));
        self
    }

    /// Build the behavior
    pub fn build(self) -> Result<MockBehavior, TestHarnessError> {
        let matcher = self.matcher.ok_or_else(|| {
            TestHarnessError::MockError("Matcher function is required".to_string())
        })?;

        let responder = self.responder.ok_or_else(|| {
            TestHarnessError::MockError("Responder function is required".to_string())
        })?;

        Ok(MockBehavior {
            name: self.name,
            description: self.description,
            matcher,
            responder,
            invocation_count: self.invocation_count,
        })
    }
}

/// Mock behavior implementation
pub struct MockBehavior {
    /// Behavior name
    name: String,
    /// Behavior description
    description: Option<String>,
    /// Matcher function
    matcher: Box<dyn Fn(&Interaction) -> bool + Send + Sync>,
    /// Response function
    responder: Box<dyn Fn(&Interaction) -> Response + Send + Sync>,
    /// Invocation count
    invocation_count: Arc<RwLock<usize>>,
}

#[async_trait]
impl Behavior for MockBehavior {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn matches(&self, interaction: &Interaction) -> bool {
        (self.matcher)(interaction)
    }

    async fn respond(&self, interaction: &Interaction) -> Response {
        // Increment the invocation count
        let mut count = self.invocation_count.write().await;
        *count += 1;

        // Generate the response
        (self.responder)(interaction)
    }

    fn invocation_count(&self) -> usize {
        // This is a bit of a hack, but it's the best we can do without making the trait async
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { *self.invocation_count.read().await })
        })
    }

    fn reset(&self) {
        // This is a bit of a hack, but it's the best we can do without making the trait async
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut count = self.invocation_count.write().await;
                *count = 0;
            })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::recorder::RecordedInteraction;
    use super::*;

    #[test]
    fn test_response_builder() {
        // Create a success response
        let response = ResponseBuilder::new()
            .with_data(serde_json::json!({"message": "success"}))
            .unwrap()
            .with_status(200)
            .build();

        assert!(response.is_success());
        assert_eq!(response.status, 200);
        assert_eq!(
            response.data.unwrap(),
            serde_json::json!({"message": "success"})
        );

        // Create an error response
        let response = ResponseBuilder::error(404, "Not found");

        assert!(response.is_error());
        assert_eq!(response.status, 404);
        assert_eq!(response.error.unwrap(), "Not found");
    }

    #[tokio::test]
    async fn test_mock_behavior() {
        // Create a behavior
        let behavior = BehaviorBuilder::new("test-behavior")
            .with_description("Test behavior")
            .with_matcher(|interaction| interaction.request.get("path").unwrap() == "/api/test")
            .with_responder(|_| {
                ResponseBuilder::new()
                    .with_data(serde_json::json!({"message": "success"}))
                    .unwrap()
                    .with_status(200)
                    .build()
            })
            .build()
            .unwrap();

        // Create an interaction
        let interaction = Interaction {
            id: "test-interaction".to_string(),
            timestamp: chrono::Utc::now(),
            request: serde_json::json!({
                "path": "/api/test",
                "method": "GET"
            }),
            response: None,
        };

        // Check if the behavior matches
        assert!(behavior.matches(&interaction));

        // Get the response
        let response = behavior.respond(&interaction).await;

        assert!(response.is_success());
        assert_eq!(response.status, 200);
        assert_eq!(
            response.data.unwrap(),
            serde_json::json!({"message": "success"})
        );

        // Check the invocation count
        assert_eq!(behavior.invocation_count(), 1);

        // Reset the behavior
        behavior.reset();

        // Check the invocation count again
        assert_eq!(behavior.invocation_count(), 0);
    }
}
