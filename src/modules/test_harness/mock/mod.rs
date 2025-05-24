//! Mocking and Stubbing Framework
//!
//! This module provides functionality for creating mock objects and stubs for
//! external dependencies, making it easier to test components in isolation.
//!
//! The framework includes enhanced capabilities for HTTP interaction recording/replaying,
//! intelligent stub generation, configurable response behaviors, and network condition simulation.

mod behavior;
mod enhanced_mock;
mod http;
mod interaction_recorder;
mod network;
mod network_simulator;
mod recorder;
mod response_behavior;
mod service;
mod storage;
mod stub_generator;

pub use behavior::{Behavior, BehaviorBuilder, MockBehavior, Response, ResponseBuilder};
pub use enhanced_mock::{
    create_enhanced_http_mock, EnhancedHttpMock, EnhancedHttpMockBuilder, EnhancedMockConfig,
};
pub use http::{HttpMock, HttpMockBuilder, HttpRequest, HttpResponse, HttpStub};
pub use interaction_recorder::{
    HttpInteraction, HttpInteractionMatcher, HttpInteractionRecorder, InteractionRecorderConfig,
    RecordingMode,
};
pub use network::{NetworkMock, NetworkMockBuilder, NetworkRequest, NetworkResponse};
pub use network_simulator::{NetworkConditionType, NetworkSimulator, NetworkSimulatorConfig};
pub use recorder::{Interaction, InteractionMatcher, MockRecorder, RecordedInteraction};
pub use response_behavior::{
    ErrorConditionType, ErrorConfig, ErrorType, LatencyConfig, LatencyDistribution,
    ResponseBehaviorConfig, ResponseBehaviorHandler, TransformationConfig,
};
pub use service::{ServiceMock, ServiceMockBuilder, ServiceRequest, ServiceResponse};
pub use storage::{StorageMock, StorageMockBuilder, StorageOperation, StorageResponse};
pub use stub_generator::{StubGenerationConfig, StubGenerator};

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::types::TestHarnessError;

/// Mock trait for implementing mock objects
#[async_trait]
pub trait Mock: Send + Sync {
    /// Get the mock name
    fn name(&self) -> &str;

    /// Get the mock description
    fn description(&self) -> Option<&str>;

    /// Set up the mock
    async fn setup(&self) -> Result<(), TestHarnessError>;

    /// Tear down the mock
    async fn teardown(&self) -> Result<(), TestHarnessError>;

    /// Reset the mock
    async fn reset(&self) -> Result<(), TestHarnessError>;

    /// Verify the mock
    async fn verify(&self) -> Result<(), TestHarnessError>;

    /// Get the mock recorder
    fn recorder(&self) -> &MockRecorder;
}

/// Enhanced mock trait with additional capabilities
#[async_trait]
pub trait EnhancedMock: Mock {
    /// Get the interaction recorder
    fn interaction_recorder(&self) -> Arc<HttpInteractionRecorder>;

    /// Get the stub generator
    fn stub_generator(&self) -> Arc<StubGenerator>;

    /// Get the network simulator
    fn network_simulator(&self) -> Arc<NetworkSimulator>;

    /// Generate stubs from recorded interactions
    async fn generate_stubs(&self) -> Vec<HttpStub>;

    /// Save recorded interactions to a file
    async fn save_interactions(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<(), TestHarnessError>;

    /// Load recorded interactions from a file
    async fn load_interactions(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<(), TestHarnessError>;
}

/// Mock manager for managing mock objects
pub struct MockManager {
    /// Registered mocks
    mocks: RwLock<HashMap<String, Arc<dyn Mock>>>,
}

impl MockManager {
    /// Create a new mock manager
    pub fn new() -> Self {
        Self {
            mocks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a mock
    pub async fn register_mock(&self, mock: Arc<dyn Mock>) -> Result<(), TestHarnessError> {
        let name = mock.name().to_string();
        let mut mocks = self.mocks.write().await;

        if mocks.contains_key(&name) {
            return Err(TestHarnessError::MockError(format!(
                "Mock with name '{}' is already registered",
                name
            )));
        }

        mocks.insert(name.clone(), mock);
        info!("Registered mock: {}", name);

        Ok(())
    }

    /// Unregister a mock
    pub async fn unregister_mock(&self, name: &str) -> Result<(), TestHarnessError> {
        let mut mocks = self.mocks.write().await;

        if !mocks.contains_key(name) {
            return Err(TestHarnessError::MockError(format!(
                "Mock with name '{}' is not registered",
                name
            )));
        }

        mocks.remove(name);
        info!("Unregistered mock: {}", name);

        Ok(())
    }

    /// Get a mock by name
    pub async fn get_mock(&self, name: &str) -> Result<Arc<dyn Mock>, TestHarnessError> {
        let mocks = self.mocks.read().await;

        mocks.get(name).cloned().ok_or_else(|| {
            TestHarnessError::MockError(format!("Mock with name '{}' not found", name))
        })
    }

    /// Get all registered mocks
    pub async fn get_all_mocks(&self) -> Vec<Arc<dyn Mock>> {
        let mocks = self.mocks.read().await;
        mocks.values().cloned().collect()
    }

    /// Set up all mocks
    pub async fn setup_all_mocks(&self) -> Result<(), TestHarnessError> {
        let mocks = self.mocks.read().await;

        for (name, mock) in mocks.iter() {
            info!("Setting up mock: {}", name);

            if let Err(e) = mock.setup().await {
                error!("Failed to set up mock {}: {}", name, e);
                return Err(TestHarnessError::MockError(format!(
                    "Failed to set up mock {}: {}",
                    name, e
                )));
            }
        }

        Ok(())
    }

    /// Tear down all mocks
    pub async fn teardown_all_mocks(&self) -> Result<(), TestHarnessError> {
        let mocks = self.mocks.read().await;

        for (name, mock) in mocks.iter() {
            info!("Tearing down mock: {}", name);

            if let Err(e) = mock.teardown().await {
                warn!("Failed to tear down mock {}: {}", name, e);
                // Continue tearing down other mocks
            }
        }

        Ok(())
    }

    /// Reset all mocks
    pub async fn reset_all_mocks(&self) -> Result<(), TestHarnessError> {
        let mocks = self.mocks.read().await;

        for (name, mock) in mocks.iter() {
            info!("Resetting mock: {}", name);

            if let Err(e) = mock.reset().await {
                error!("Failed to reset mock {}: {}", name, e);
                return Err(TestHarnessError::MockError(format!(
                    "Failed to reset mock {}: {}",
                    name, e
                )));
            }
        }

        Ok(())
    }

    /// Verify all mocks
    pub async fn verify_all_mocks(&self) -> Result<(), TestHarnessError> {
        let mocks = self.mocks.read().await;
        let mut errors = Vec::new();

        for (name, mock) in mocks.iter() {
            info!("Verifying mock: {}", name);

            if let Err(e) = mock.verify().await {
                error!("Mock verification failed for {}: {}", name, e);
                errors.push(format!("Mock '{}': {}", name, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TestHarnessError::MockError(format!(
                "Mock verification failed: {}",
                errors.join(", ")
            )))
        }
    }
}

impl Default for MockManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock factory for creating mock objects
pub struct MockFactory {
    /// Mock manager
    manager: Arc<MockManager>,
}

impl MockFactory {
    /// Create a new mock factory
    pub fn new(manager: Arc<MockManager>) -> Self {
        Self { manager }
    }

    /// Create an HTTP mock
    pub fn create_http_mock(&self, name: impl Into<String>) -> HttpMockBuilder {
        HttpMockBuilder::new(name.into(), Arc::clone(&self.manager))
    }

    /// Create an enhanced HTTP mock
    pub fn create_enhanced_http_mock(&self, name: impl Into<String>) -> EnhancedHttpMockBuilder {
        enhanced_mock::create_enhanced_http_mock(name, Arc::clone(&self.manager))
    }

    /// Create a network mock
    pub fn create_network_mock(&self, name: impl Into<String>) -> NetworkMockBuilder {
        NetworkMockBuilder::new(name.into(), Arc::clone(&self.manager))
    }

    /// Create a service mock
    pub fn create_service_mock(&self, name: impl Into<String>) -> ServiceMockBuilder {
        ServiceMockBuilder::new(name.into(), Arc::clone(&self.manager))
    }

    /// Create a storage mock
    pub fn create_storage_mock(&self, name: impl Into<String>) -> StorageMockBuilder {
        StorageMockBuilder::new(name.into(), Arc::clone(&self.manager))
    }
}

/// Mock server for serving mock responses
pub struct MockServer {
    /// Server address
    address: String,
    /// Server port
    port: u16,
    /// Mock manager
    manager: Arc<MockManager>,
    /// Server handle
    server_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl MockServer {
    /// Create a new mock server
    pub fn new(address: impl Into<String>, port: u16, manager: Arc<MockManager>) -> Self {
        Self {
            address: address.into(),
            port,
            manager,
            server_handle: Mutex::new(None),
        }
    }

    /// Start the mock server
    pub async fn start(&self) -> Result<(), TestHarnessError> {
        let mut handle = self.server_handle.lock().await;

        if handle.is_some() {
            return Err(TestHarnessError::MockError(
                "Mock server is already running".to_string(),
            ));
        }

        let address = self.address.clone();
        let port = self.port;
        let manager = Arc::clone(&self.manager);

        // Start the server in a separate task
        let server_task = tokio::spawn(async move {
            info!("Starting mock server on {}:{}", address, port);

            // TODO: Implement the actual server

            info!("Mock server stopped");
        });

        *handle = Some(server_task);

        Ok(())
    }

    /// Stop the mock server
    pub async fn stop(&self) -> Result<(), TestHarnessError> {
        let mut handle = self.server_handle.lock().await;

        if let Some(task) = handle.take() {
            info!("Stopping mock server");
            task.abort();

            match task.await {
                Ok(_) => info!("Mock server stopped successfully"),
                Err(e) => warn!("Error stopping mock server: {}", e),
            }
        }

        Ok(())
    }

    /// Get the server address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get the server port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the server URL
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_manager() {
        // Create a mock manager
        let manager = MockManager::new();

        // Create a mock HTTP server
        let http_mock = Arc::new(HttpMock::new("http-mock", "HTTP mock"));

        // Register the mock
        manager.register_mock(http_mock.clone()).await.unwrap();

        // Get the mock
        let retrieved = manager.get_mock("http-mock").await.unwrap();
        assert_eq!(retrieved.name(), "http-mock");

        // Unregister the mock
        manager.unregister_mock("http-mock").await.unwrap();

        // Try to get the mock again
        let result = manager.get_mock("http-mock").await;
        assert!(result.is_err());
    }

    /// Helper functions for creating enhanced mock objects
    pub mod enhanced_mock_helpers {
        use super::*;

        /// Create a new enhanced HTTP mock
        pub fn create_enhanced_http_mock(name: impl Into<String>) -> EnhancedHttpMockBuilder {
            let manager = create_mock_manager();
            enhanced_mock::create_enhanced_http_mock(name, Arc::new(manager))
        }
    }
}
