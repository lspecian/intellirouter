//! HTTP Mock Module
//!
//! This module provides functionality for mocking HTTP services.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::behavior::{Behavior, BehaviorBuilder, MockBehavior, Response, ResponseBuilder};
use super::recorder::{Interaction, MockRecorder};
use super::Mock;
use crate::modules::test_harness::types::TestHarnessError;

/// HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Request method
    pub method: String,
    /// Request path
    pub path: String,
    /// Request query parameters
    pub query: HashMap<String, String>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<serde_json::Value>,
}

impl HttpRequest {
    /// Create a new HTTP request
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a query parameter
    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    /// Add multiple query parameters
    pub fn with_queries(
        mut self,
        queries: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in queries {
            self.query.insert(key.into(), value.into());
        }
        self
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add multiple headers
    pub fn with_headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in headers {
            self.headers.insert(key.into(), value.into());
        }
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: impl Serialize) -> Result<Self, TestHarnessError> {
        self.body = Some(serde_json::to_value(body).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize body: {}", e))
        })?);
        Ok(self)
    }

    /// Convert to an interaction
    pub fn to_interaction(&self, id: impl Into<String>) -> Interaction {
        let mut request = serde_json::Map::new();
        request.insert(
            "method".to_string(),
            serde_json::Value::String(self.method.clone()),
        );
        request.insert(
            "path".to_string(),
            serde_json::Value::String(self.path.clone()),
        );
        request.insert(
            "query".to_string(),
            serde_json::to_value(&self.query).unwrap(),
        );
        request.insert(
            "headers".to_string(),
            serde_json::to_value(&self.headers).unwrap(),
        );

        if let Some(body) = &self.body {
            request.insert("body".to_string(), body.clone());
        }

        Interaction::new(id, serde_json::Value::Object(request))
    }
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// Response status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<serde_json::Value>,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add multiple headers
    pub fn with_headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in headers {
            self.headers.insert(key.into(), value.into());
        }
        self
    }

    /// Set the response body
    pub fn with_body(mut self, body: impl Serialize) -> Result<Self, TestHarnessError> {
        self.body = Some(serde_json::to_value(body).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize body: {}", e))
        })?);
        Ok(self)
    }

    /// Convert to a response
    pub fn to_response(&self) -> Response {
        let mut response = Response::new();
        response.status = self.status;
        response.headers = self.headers.clone();
        response.data = self.body.clone();

        response
    }

    /// Convert from a response
    pub fn from_response(response: &Response) -> Self {
        Self {
            status: response.status,
            headers: response.headers.clone(),
            body: response.data.clone(),
        }
    }
}

/// HTTP stub for defining HTTP mock behavior
pub struct HttpStub {
    /// Request matcher
    request_matcher: Box<dyn Fn(&HttpRequest) -> bool + Send + Sync>,
    /// Response generator
    response_generator: Box<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync>,
}

impl HttpStub {
    /// Create a new HTTP stub
    pub fn new(
        request_matcher: impl Fn(&HttpRequest) -> bool + Send + Sync + 'static,
        response_generator: impl Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    ) -> Self {
        Self {
            request_matcher: Box::new(request_matcher),
            response_generator: Box::new(response_generator),
        }
    }

    /// Create a new HTTP stub for a specific path
    pub fn for_path(
        path: impl Into<String>,
        response_generator: impl Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    ) -> Self {
        let path = path.into();
        Self::new(move |request| request.path == path, response_generator)
    }

    /// Create a new HTTP stub for a specific path and method
    pub fn for_path_and_method(
        path: impl Into<String>,
        method: impl Into<String>,
        response_generator: impl Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    ) -> Self {
        let path = path.into();
        let method = method.into();
        Self::new(
            move |request| request.path == path && request.method.eq_ignore_ascii_case(&method),
            response_generator,
        )
    }

    /// Check if the stub matches a request
    pub fn matches(&self, request: &HttpRequest) -> bool {
        (self.request_matcher)(request)
    }

    /// Generate a response for a request
    pub fn generate_response(&self, request: &HttpRequest) -> HttpResponse {
        (self.response_generator)(request)
    }
}

/// HTTP mock for mocking HTTP services
pub struct HttpMock {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock behaviors
    behaviors: RwLock<Vec<Arc<dyn Behavior>>>,
    /// Mock stubs
    stubs: RwLock<Vec<HttpStub>>,
    /// Mock recorder
    recorder: Arc<MockRecorder>,
    /// Mock server
    server: Option<String>,
    /// Mock port
    port: Option<u16>,
}

impl HttpMock {
    /// Create a new HTTP mock
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            behaviors: RwLock::new(Vec::new()),
            stubs: RwLock::new(Vec::new()),
            recorder: Arc::new(MockRecorder::new()),
            server: None,
            port: None,
        }
    }

    /// Add a behavior
    pub async fn add_behavior(&self, behavior: Arc<dyn Behavior>) {
        let mut behaviors = self.behaviors.write().await;
        behaviors.push(behavior);
    }

    /// Add a stub
    pub async fn add_stub(&self, stub: HttpStub) {
        let mut stubs = self.stubs.write().await;
        stubs.push(stub);
    }

    /// Set the server and port
    pub fn set_server(&mut self, server: impl Into<String>, port: u16) {
        self.server = Some(server.into());
        self.port = Some(port);
    }

    /// Get the server URL
    pub fn url(&self) -> Option<String> {
        if let (Some(server), Some(port)) = (&self.server, self.port) {
            Some(format!("http://{}:{}", server, port))
        } else {
            None
        }
    }

    /// Handle a request
    pub async fn handle_request(&self, request: &HttpRequest) -> HttpResponse {
        // Create an interaction
        let interaction = request.to_interaction(format!("http-{}", uuid::Uuid::new_v4()));

        // Check if any behavior matches
        let behaviors = self.behaviors.read().await;
        for behavior in behaviors.iter() {
            if behavior.matches(&interaction) {
                // Record the interaction with the behavior
                self.recorder
                    .record_with_behavior(interaction.clone(), behavior.name())
                    .await;

                // Generate the response
                let response = behavior.respond(&interaction).await;

                // Convert to an HTTP response
                let http_response = HttpResponse {
                    status: response.status,
                    headers: response.headers.clone(),
                    body: response.data.clone(),
                };

                return http_response;
            }
        }

        // Check if any stub matches
        let stubs = self.stubs.read().await;
        for stub in stubs.iter() {
            if stub.matches(request) {
                // Record the interaction
                self.recorder.record(interaction).await;

                // Generate the response
                return stub.generate_response(request);
            }
        }

        // No behavior or stub matched
        warn!("No behavior or stub matched request: {:?}", request);

        // Record the interaction
        self.recorder.record(interaction).await;

        // Return a 404 response
        HttpResponse::new(404)
            .with_body(serde_json::json!({
                "error": "Not found",
                "message": "No matching behavior or stub found"
            }))
            .unwrap()
    }
}

#[async_trait]
impl Mock for HttpMock {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn setup(&self) -> Result<(), TestHarnessError> {
        info!("Setting up HTTP mock: {}", self.name);
        Ok(())
    }

    async fn teardown(&self) -> Result<(), TestHarnessError> {
        info!("Tearing down HTTP mock: {}", self.name);
        Ok(())
    }

    async fn reset(&self) -> Result<(), TestHarnessError> {
        info!("Resetting HTTP mock: {}", self.name);

        // Clear the recorder
        self.recorder.clear().await;

        // Reset all behaviors
        let behaviors = self.behaviors.read().await;
        for behavior in behaviors.iter() {
            behavior.reset();
        }

        Ok(())
    }

    async fn verify(&self) -> Result<(), TestHarnessError> {
        info!("Verifying HTTP mock: {}", self.name);

        // Verify the recorder
        self.recorder.verify().await?;

        Ok(())
    }

    fn recorder(&self) -> &MockRecorder {
        &self.recorder
    }
}

/// HTTP mock builder for creating HTTP mocks
pub struct HttpMockBuilder {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock behaviors
    behaviors: Vec<Arc<dyn Behavior>>,
    /// Mock stubs
    stubs: Vec<HttpStub>,
    /// Mock server
    server: Option<String>,
    /// Mock port
    port: Option<u16>,
    /// Mock manager
    manager: Arc<super::MockManager>,
}

impl HttpMockBuilder {
    /// Create a new HTTP mock builder
    pub fn new(name: impl Into<String>, manager: Arc<super::MockManager>) -> Self {
        Self {
            name: name.into(),
            description: None,
            behaviors: Vec::new(),
            stubs: Vec::new(),
            server: None,
            port: None,
            manager,
        }
    }

    /// Set the mock description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a behavior
    pub fn with_behavior(mut self, behavior: Arc<dyn Behavior>) -> Self {
        self.behaviors.push(behavior);
        self
    }

    /// Add a stub
    pub fn with_stub(mut self, stub: HttpStub) -> Self {
        self.stubs.push(stub);
        self
    }

    /// Set the server and port
    pub fn with_server(mut self, server: impl Into<String>, port: u16) -> Self {
        self.server = Some(server.into());
        self.port = Some(port);
        self
    }

    /// Build the mock
    pub async fn build(self) -> Result<Arc<HttpMock>, TestHarnessError> {
        // Create the mock
        let mut mock = HttpMock::new(self.name.clone(), self.description.unwrap_or_default());

        // Set the server and port
        if let (Some(server), Some(port)) = (self.server, self.port) {
            mock.set_server(server, port);
        }

        // Add behaviors
        for behavior in self.behaviors {
            mock.add_behavior(behavior).await;
        }

        // Add stubs
        for stub in self.stubs {
            mock.add_stub(stub).await;
        }

        // Create an Arc
        let mock = Arc::new(mock);

        // Register the mock
        self.manager.register_mock(Arc::clone(&mock)).await?;

        Ok(mock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_mock() {
        // Create a mock
        let mock = HttpMock::new("test-mock", "Test HTTP mock");

        // Create a behavior
        let behavior = BehaviorBuilder::new("test-behavior")
            .with_description("Test behavior")
            .with_matcher(|interaction| {
                let path = interaction
                    .request_field::<String>("path")
                    .unwrap_or_default();
                let method = interaction
                    .request_field::<String>("method")
                    .unwrap_or_default();

                path == "/api/test" && method.eq_ignore_ascii_case("GET")
            })
            .with_responder(|_| {
                ResponseBuilder::new()
                    .with_data(serde_json::json!({"message": "success"}))
                    .unwrap()
                    .with_status(200)
                    .build()
            })
            .build()
            .unwrap();

        // Add the behavior
        mock.add_behavior(Arc::new(behavior)).await;

        // Create a stub
        let stub = HttpStub::for_path_and_method("/api/stub", "GET", |_| {
            HttpResponse::new(200)
                .with_body(serde_json::json!({"message": "stub"}))
                .unwrap()
        });

        // Add the stub
        mock.add_stub(stub).await;

        // Create a request that matches the behavior
        let request1 = HttpRequest::new("GET", "/api/test");

        // Handle the request
        let response1 = mock.handle_request(&request1).await;

        // Check the response
        assert_eq!(response1.status, 200);
        assert_eq!(
            response1.body.unwrap(),
            serde_json::json!({"message": "success"})
        );

        // Create a request that matches the stub
        let request2 = HttpRequest::new("GET", "/api/stub");

        // Handle the request
        let response2 = mock.handle_request(&request2).await;

        // Check the response
        assert_eq!(response2.status, 200);
        assert_eq!(
            response2.body.unwrap(),
            serde_json::json!({"message": "stub"})
        );

        // Create a request that doesn't match anything
        let request3 = HttpRequest::new("GET", "/api/unknown");

        // Handle the request
        let response3 = mock.handle_request(&request3).await;

        // Check the response
        assert_eq!(response3.status, 404);

        // Check the recorder
        assert_eq!(mock.recorder.count().await, 3);
    }
}
