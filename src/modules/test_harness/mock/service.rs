//! Service Mocking
//!
//! This module provides functionality for mocking services.

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

/// Service request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Service name
    pub service: String,
    /// Method name
    pub method: String,
    /// Request parameters
    pub params: HashMap<String, serde_json::Value>,
}

impl ServiceRequest {
    /// Create a new service request
    pub fn new(service: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            method: method.into(),
            params: HashMap::new(),
        }
    }

    /// Add a parameter to the request
    pub fn with_param(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.params.insert(key.into(), value);
        Ok(self)
    }
}

/// Service response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Response status
    pub status: String,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Response error
    pub error: Option<String>,
}

impl ServiceResponse {
    /// Create a new successful service response
    pub fn success(data: impl Serialize) -> Result<Self, TestHarnessError> {
        let data = serde_json::to_value(data).map_err(TestHarnessError::SerializationError)?;
        Ok(Self {
            status: "success".to_string(),
            data: Some(data),
            error: None,
        })
    }

    /// Create a new error service response
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(error.into()),
        }
    }
}

/// Service mock
pub struct ServiceMock {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock behaviors
    behaviors: Mutex<Vec<MockBehavior<ServiceRequest, ServiceResponse>>>,
    /// Mock recorder
    recorder: MockRecorder,
}

impl ServiceMock {
    /// Create a new service mock
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
        behavior: MockBehavior<ServiceRequest, ServiceResponse>,
    ) -> Result<(), TestHarnessError> {
        let mut behaviors = self.behaviors.lock().await;
        behaviors.push(behavior);
        Ok(())
    }

    /// Handle a service request
    pub async fn handle_request(
        &self,
        request: &ServiceRequest,
    ) -> Result<ServiceResponse, TestHarnessError> {
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
impl Mock for ServiceMock {
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

/// Service mock builder
pub struct ServiceMockBuilder {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock manager
    manager: Arc<MockManager>,
}

impl ServiceMockBuilder {
    /// Create a new service mock builder
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

    /// Build the service mock
    pub async fn build(self) -> Result<Arc<ServiceMock>, TestHarnessError> {
        let mock = Arc::new(ServiceMock::new(
            self.name.clone(),
            self.description
                .unwrap_or_else(|| format!("Service mock: {}", self.name)),
        ));
        self.manager.register_mock(mock.clone()).await?;
        Ok(mock)
    }
}
