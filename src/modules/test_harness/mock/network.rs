//! Network Mocking
//!
//! This module provides functionality for mocking network connections.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use super::behavior::{Behavior, MockBehavior, Response};
use super::recorder::{MockRecorder, RecordedInteraction};
use super::{Mock, MockManager};
use crate::modules::test_harness::types::TestHarnessError;

/// Network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Request protocol
    pub protocol: String,
    /// Request host
    pub host: String,
    /// Request port
    pub port: u16,
    /// Request path
    pub path: String,
    /// Request method
    pub method: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<Vec<u8>>,
}

impl NetworkRequest {
    /// Create a new network request
    pub fn new(
        protocol: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        path: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            protocol: protocol.into(),
            host: host.into(),
            port,
            path: path.into(),
            method: method.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

/// Network response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    /// Response status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<Vec<u8>>,
}

impl NetworkResponse {
    /// Create a new network response
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header to the response
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the response body
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

/// Network mock
pub struct NetworkMock {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock behaviors
    behaviors: Mutex<Vec<MockBehavior<NetworkRequest, NetworkResponse>>>,
    /// Mock recorder
    recorder: MockRecorder,
}

impl NetworkMock {
    /// Create a new network mock
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            behaviors: Mutex::new(Vec::new()),
            recorder: MockRecorder::new(),
        }
    }

    /// Add a behavior to the mock
    pub async fn add_behavior(
        &self,
        behavior: MockBehavior<NetworkRequest, NetworkResponse>,
    ) -> Result<(), TestHarnessError> {
        let mut behaviors = self.behaviors.lock().await;
        behaviors.push(behavior);
        Ok(())
    }

    /// Handle a network request
    pub async fn handle_request(
        &self,
        request: &NetworkRequest,
    ) -> Result<NetworkResponse, TestHarnessError> {
        // Record the request
        self.recorder
            .record_request(serde_json::to_value(request).unwrap())
            .await;

        // Find a matching behavior
        let behaviors = self.behaviors.lock().await;
        for behavior in behaviors.iter() {
            if behavior.matches(request).await? {
                let response = behavior.respond(request).await?;
                match response {
                    Response::Success(resp) => {
                        // Record the response
                        self.recorder
                            .record_response(serde_json::to_value(&resp).unwrap())
                            .await;
                        return Ok(resp);
                    }
                    Response::Error(err) => {
                        // Record the error
                        self.recorder
                            .record_error(serde_json::to_value(&err).unwrap())
                            .await;
                        return Err(TestHarnessError::MockError(err));
                    }
                }
            }
        }

        // No matching behavior found
        Err(TestHarnessError::MockError(format!(
            "No matching behavior found for request: {:?}",
            request
        )))
    }
}

#[async_trait]
impl Mock for NetworkMock {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn setup(&self) -> Result<(), TestHarnessError> {
        Ok(())
    }

    async fn teardown(&self) -> Result<(), TestHarnessError> {
        Ok(())
    }

    async fn reset(&self) -> Result<(), TestHarnessError> {
        let mut behaviors = self.behaviors.lock().await;
        behaviors.clear();
        self.recorder.reset().await;
        Ok(())
    }

    async fn verify(&self) -> Result<(), TestHarnessError> {
        let behaviors = self.behaviors.lock().await;
        for behavior in behaviors.iter() {
            if !behavior.verify().await? {
                return Err(TestHarnessError::MockError(format!(
                    "Behavior verification failed: {:?}",
                    behavior
                )));
            }
        }
        Ok(())
    }

    fn recorder(&self) -> &MockRecorder {
        &self.recorder
    }
}

/// Network mock builder
pub struct NetworkMockBuilder {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock manager
    manager: Arc<MockManager>,
}

impl NetworkMockBuilder {
    /// Create a new network mock builder
    pub fn new(name: impl Into<String>, manager: Arc<MockManager>) -> Self {
        Self {
            name: name.into(),
            description: None,
            manager,
        }
    }

    /// Set the mock description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Build the network mock
    pub async fn build(self) -> Result<Arc<NetworkMock>, TestHarnessError> {
        let mock = Arc::new(NetworkMock::new(
            self.name.clone(),
            self.description
                .unwrap_or_else(|| format!("Network mock: {}", self.name)),
        ));
        self.manager.register_mock(mock.clone()).await?;
        Ok(mock)
    }
}
