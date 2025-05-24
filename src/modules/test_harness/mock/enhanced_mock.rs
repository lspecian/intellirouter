//! Enhanced Mock Module
//!
//! This module provides enhanced mocking capabilities with support for
//! HTTP interaction recording/replaying, intelligent stub generation,
//! configurable response behaviors, and network condition simulation.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::http::{HttpMock, HttpMockBuilder, HttpRequest, HttpResponse, HttpStub};
use super::interaction_recorder::{
    HttpInteraction, HttpInteractionRecorder, InteractionRecorderConfig, RecordingMode,
};
use super::network_simulator::{NetworkSimulator, NetworkSimulatorConfig};
use super::recorder::{Interaction, MockRecorder, RecordedInteraction};
use super::stub_generator::{StubGenerationConfig, StubGenerator};
use crate::modules::test_harness::types::TestHarnessError;

/// Enhanced mock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMockConfig {
    /// Interaction recorder configuration
    pub recorder: InteractionRecorderConfig,
    /// Stub generation configuration
    pub stub_generator: StubGenerationConfig,
    /// Network simulator configuration
    pub network_simulator: NetworkSimulatorConfig,
}

impl Default for EnhancedMockConfig {
    fn default() -> Self {
        Self {
            recorder: InteractionRecorderConfig::default(),
            stub_generator: StubGenerationConfig::default(),
            network_simulator: NetworkSimulatorConfig::default(),
        }
    }
}

/// Enhanced HTTP mock with advanced capabilities
pub struct EnhancedHttpMock {
    /// Base HTTP mock
    mock: Arc<HttpMock>,
    /// Interaction recorder
    recorder: Arc<HttpInteractionRecorder>,
    /// Stub generator
    stub_generator: Arc<StubGenerator>,
    /// Network simulator
    network_simulator: Arc<NetworkSimulator>,
    /// Configuration
    config: RwLock<EnhancedMockConfig>,
}

impl EnhancedHttpMock {
    /// Create a new enhanced HTTP mock
    pub fn new(mock: Arc<HttpMock>, config: EnhancedMockConfig) -> Self {
        let recorder = Arc::new(HttpInteractionRecorder::new(
            format!("{}-recorder", mock.name()),
            config.recorder.clone(),
        ));

        let stub_generator = Arc::new(StubGenerator::new(config.stub_generator.clone()));

        let network_simulator = Arc::new(NetworkSimulator::new(config.network_simulator.clone()));

        Self {
            mock,
            recorder,
            stub_generator,
            network_simulator,
            config: RwLock::new(config),
        }
    }

    /// Get the base HTTP mock
    pub fn mock(&self) -> Arc<HttpMock> {
        self.mock.clone()
    }

    /// Get the interaction recorder
    pub fn recorder(&self) -> Arc<HttpInteractionRecorder> {
        self.recorder.clone()
    }

    /// Get the stub generator
    pub fn stub_generator(&self) -> Arc<StubGenerator> {
        self.stub_generator.clone()
    }

    /// Get the network simulator
    pub fn network_simulator(&self) -> Arc<NetworkSimulator> {
        self.network_simulator.clone()
    }

    /// Get the configuration
    pub async fn config(&self) -> EnhancedMockConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    pub async fn update_config(&self, config: EnhancedMockConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config.clone();

        // Update component configurations
        self.recorder.update_config(config.recorder).await;
        self.stub_generator
            .update_config(config.stub_generator)
            .await;
        self.network_simulator
            .update_config(config.network_simulator)
            .await;
    }

    /// Handle a request with enhanced capabilities
    pub async fn handle_request(
        &self,
        request: &HttpRequest,
    ) -> Result<HttpResponse, TestHarnessError> {
        // Get the current configuration
        let config = self.config.read().await.clone();

        // Calculate the request size for network simulation
        let request_size = self.calculate_request_size(request);

        // Simulate network conditions
        self.network_simulator.simulate(request_size).await?;

        // Handle the request based on the recording mode
        let response = match self.recorder.mode().await {
            RecordingMode::Record => {
                // Forward to the real handler and record
                let response = self.mock.handle_request(request).await;

                // Create and record the interaction
                let interaction = HttpInteraction::new(request.clone(), response.clone());
                self.recorder.record(interaction).await;

                response
            }
            RecordingMode::Replay => {
                // Try to find a matching interaction
                if let Some(interaction) = self.recorder.find_matching(request).await {
                    interaction.response.clone()
                } else {
                    // No matching interaction found
                    HttpResponse::new(404)
                        .with_body(serde_json::json!({
                            "error": "Not found",
                            "message": "No matching interaction found"
                        }))
                        .unwrap_or_else(|_| HttpResponse::new(404))
                }
            }
            RecordingMode::Auto => {
                // Try to find a matching interaction
                if let Some(interaction) = self.recorder.find_matching(request).await {
                    interaction.response.clone()
                } else {
                    // No matching interaction found, forward to the real handler and record
                    let response = self.mock.handle_request(request).await;

                    // Create and record the interaction
                    let interaction = HttpInteraction::new(request.clone(), response.clone());
                    self.recorder.record(interaction).await;

                    response
                }
            }
            RecordingMode::Passthrough => {
                // Forward to the real handler without recording
                self.mock.handle_request(request).await
            }
        };

        // Calculate the response size for network simulation
        let response_size = self.calculate_response_size(&response);

        // Simulate network conditions for the response
        self.network_simulator.simulate(response_size).await?;

        Ok(response)
    }

    /// Calculate the size of a request in bytes
    fn calculate_request_size(&self, request: &HttpRequest) -> usize {
        let mut size = 0;

        // Method and path
        size += request.method.len() + request.path.len();

        // Headers
        for (key, value) in &request.headers {
            size += key.len() + value.len() + 2; // +2 for ": "
        }

        // Query parameters
        for (key, value) in &request.query {
            size += key.len() + value.len() + 2; // +2 for "=&"
        }

        // Body
        if let Some(body) = &request.body {
            size += serde_json::to_string(body).unwrap_or_default().len();
        }

        size
    }

    /// Calculate the size of a response in bytes
    fn calculate_response_size(&self, response: &HttpResponse) -> usize {
        let mut size = 0;

        // Status code
        size += 3; // Assuming 3 digits

        // Headers
        for (key, value) in &response.headers {
            size += key.len() + value.len() + 2; // +2 for ": "
        }

        // Body
        if let Some(body) = &response.body {
            size += serde_json::to_string(body).unwrap_or_default().len();
        }

        size
    }

    /// Generate stubs from recorded interactions
    pub async fn generate_stubs(&self) -> Vec<HttpStub> {
        // Get all recorded interactions
        let interactions = self.recorder.get_interactions().await;

        // Convert to RecordedInteraction format
        let recorded_interactions: Vec<RecordedInteraction> = interactions
            .iter()
            .map(|i| i.to_recorded_interaction())
            .collect();

        // Generate stubs
        self.stub_generator
            .generate_stubs(&recorded_interactions)
            .await
    }

    /// Save recorded interactions to a file
    pub async fn save_interactions(&self, path: impl AsRef<Path>) -> Result<(), TestHarnessError> {
        self.recorder.save(path).await
    }

    /// Load recorded interactions from a file
    pub async fn load_interactions(&self, path: impl AsRef<Path>) -> Result<(), TestHarnessError> {
        self.recorder.load(path).await
    }
}

/// Enhanced HTTP mock builder
pub struct EnhancedHttpMockBuilder {
    /// Base HTTP mock builder
    mock_builder: HttpMockBuilder,
    /// Enhanced mock configuration
    config: EnhancedMockConfig,
}

impl EnhancedHttpMockBuilder {
    /// Create a new enhanced HTTP mock builder
    pub fn new(name: impl Into<String>, manager: Arc<super::MockManager>) -> Self {
        Self {
            mock_builder: HttpMockBuilder::new(name, manager),
            config: EnhancedMockConfig::default(),
        }
    }

    /// Set the mock description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.mock_builder = self.mock_builder.with_description(description);
        self
    }

    /// Set the recorder configuration
    pub fn with_recorder_config(mut self, config: InteractionRecorderConfig) -> Self {
        self.config.recorder = config;
        self
    }

    /// Set the stub generator configuration
    pub fn with_stub_generator_config(mut self, config: StubGenerationConfig) -> Self {
        self.config.stub_generator = config;
        self
    }

    /// Set the network simulator configuration
    pub fn with_network_simulator_config(mut self, config: NetworkSimulatorConfig) -> Self {
        self.config.network_simulator = config;
        self
    }

    /// Add a stub
    pub fn with_stub(mut self, stub: HttpStub) -> Self {
        self.mock_builder = self.mock_builder.with_stub(stub);
        self
    }

    /// Set the server and port
    pub fn with_server(mut self, server: impl Into<String>, port: u16) -> Self {
        self.mock_builder = self.mock_builder.with_server(server, port);
        self
    }

    /// Build the enhanced HTTP mock
    pub async fn build(self) -> Result<Arc<EnhancedHttpMock>, TestHarnessError> {
        // Build the base HTTP mock
        let mock = self.mock_builder.build().await?;

        // Create the enhanced HTTP mock
        let enhanced_mock = EnhancedHttpMock::new(mock, self.config);

        Ok(Arc::new(enhanced_mock))
    }
}

/// Create a new enhanced HTTP mock
pub fn create_enhanced_http_mock(
    name: impl Into<String>,
    manager: Arc<super::MockManager>,
) -> EnhancedHttpMockBuilder {
    EnhancedHttpMockBuilder::new(name, manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_http_mock() {
        // Create a mock manager
        let manager = super::super::MockManager::new();

        // Create an enhanced HTTP mock
        let mock = create_enhanced_http_mock("test-mock", Arc::new(manager))
            .with_description("Test enhanced HTTP mock")
            .with_recorder_config(InteractionRecorderConfig {
                mode: RecordingMode::Record,
                ..Default::default()
            })
            .build()
            .await
            .unwrap();

        // Create a request
        let request = HttpRequest::new("GET", "/api/test");

        // Handle the request
        let response = mock.handle_request(&request).await.unwrap();

        // Check that the response was generated
        assert_eq!(response.status, 404); // Default response when no stubs are defined

        // Check that the interaction was recorded
        let interactions = mock.recorder().get_interactions().await;
        assert_eq!(interactions.len(), 1);
        assert_eq!(interactions[0].request.method, "GET");
        assert_eq!(interactions[0].request.path, "/api/test");
    }
}
