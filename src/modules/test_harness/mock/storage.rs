//! Storage Mocking
//!
//! This module provides functionality for mocking storage systems.

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

/// Storage operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageOperationType {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Delete operation
    Delete,
    /// List operation
    List,
    /// Exists operation
    Exists,
}

impl fmt::Display for StorageOperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageOperationType::Read => write!(f, "Read"),
            StorageOperationType::Write => write!(f, "Write"),
            StorageOperationType::Delete => write!(f, "Delete"),
            StorageOperationType::List => write!(f, "List"),
            StorageOperationType::Exists => write!(f, "Exists"),
        }
    }
}

/// Storage operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOperation {
    /// Operation type
    pub operation_type: StorageOperationType,
    /// Storage key
    pub key: String,
    /// Storage value (for write operations)
    pub value: Option<serde_json::Value>,
    /// Storage options
    pub options: HashMap<String, serde_json::Value>,
}

impl StorageOperation {
    /// Create a new read operation
    pub fn read(key: impl Into<String>) -> Self {
        Self {
            operation_type: StorageOperationType::Read,
            key: key.into(),
            value: None,
            options: HashMap::new(),
        }
    }

    /// Create a new write operation
    pub fn write(key: impl Into<String>, value: impl Serialize) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        Ok(Self {
            operation_type: StorageOperationType::Write,
            key: key.into(),
            value: Some(value),
            options: HashMap::new(),
        })
    }

    /// Create a new delete operation
    pub fn delete(key: impl Into<String>) -> Self {
        Self {
            operation_type: StorageOperationType::Delete,
            key: key.into(),
            value: None,
            options: HashMap::new(),
        }
    }

    /// Create a new list operation
    pub fn list(prefix: impl Into<String>) -> Self {
        Self {
            operation_type: StorageOperationType::List,
            key: prefix.into(),
            value: None,
            options: HashMap::new(),
        }
    }

    /// Create a new exists operation
    pub fn exists(key: impl Into<String>) -> Self {
        Self {
            operation_type: StorageOperationType::Exists,
            key: key.into(),
            value: None,
            options: HashMap::new(),
        }
    }

    /// Add an option to the operation
    pub fn with_option(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.options.insert(key.into(), value);
        Ok(self)
    }
}

/// Storage response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResponse {
    /// Response status
    pub status: String,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Response error
    pub error: Option<String>,
}

impl StorageResponse {
    /// Create a new successful storage response
    pub fn success(data: impl Serialize) -> Result<Self, TestHarnessError> {
        let data = serde_json::to_value(data).map_err(TestHarnessError::SerializationError)?;
        Ok(Self {
            status: "success".to_string(),
            data: Some(data),
            error: None,
        })
    }

    /// Create a new error storage response
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(error.into()),
        }
    }
}

/// Storage mock
pub struct StorageMock {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock behaviors
    behaviors: Mutex<Vec<MockBehavior<StorageOperation, StorageResponse>>>,
    /// Mock recorder
    recorder: MockRecorder,
}

impl StorageMock {
    /// Create a new storage mock
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
        behavior: MockBehavior<StorageOperation, StorageResponse>,
    ) -> Result<(), TestHarnessError> {
        let mut behaviors = self.behaviors.lock().await;
        behaviors.push(behavior);
        Ok(())
    }

    /// Handle a storage operation
    pub async fn handle_operation(
        &self,
        operation: &StorageOperation,
    ) -> Result<StorageResponse, TestHarnessError> {
        // Record the operation
        self.recorder
            .record_request(serde_json::to_value(operation).unwrap())
            .await;

        // Find a matching behavior
        let behaviors = self.behaviors.lock().await;
        for behavior in behaviors.iter() {
            if behavior.matches(operation).await? {
                let response = behavior.respond(operation).await?;
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
            "No matching behavior found for operation: {:?}",
            operation
        )))
    }
}

#[async_trait]
impl Mock for StorageMock {
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

/// Storage mock builder
pub struct StorageMockBuilder {
    /// Mock name
    name: String,
    /// Mock description
    description: Option<String>,
    /// Mock manager
    manager: Arc<MockManager>,
}

impl StorageMockBuilder {
    /// Create a new storage mock builder
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

    /// Build the storage mock
    pub async fn build(self) -> Result<Arc<StorageMock>, TestHarnessError> {
        let mock = Arc::new(StorageMock::new(
            self.name.clone(),
            self.description
                .unwrap_or_else(|| format!("Storage mock: {}", self.name)),
        ));
        self.manager.register_mock(mock.clone()).await?;
        Ok(mock)
    }
}
