//! Model Registry IPC interface
//!
//! This module provides trait-based abstractions for the Model Registry service,
//! ensuring a clear separation between interface and transport logic.

use async_trait::async_trait;

use crate::modules::ipc::IpcResult;

/// Represents metadata for a model
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub version: String,
    pub model_type: ModelType,
    pub status: ModelStatus,
    pub context_window: u32,
    pub capabilities: ModelCapabilities,
    pub cost_per_1k_input: f32,
    pub cost_per_1k_output: f32,
    pub avg_latency_ms: f32,
    pub max_tokens_per_request: u32,
    pub max_requests_per_minute: u32,
    pub metadata: std::collections::HashMap<String, String>,
    pub tags: Vec<String>,
}

/// Represents the type of model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Chat,
    Text,
    Embedding,
    Image,
    Audio,
    Multimodal,
}

/// Represents the status of a model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelStatus {
    Available,
    Unavailable,
    Maintenance,
    Deprecated,
    Limited,
}

/// Represents the capabilities of a model
#[derive(Debug, Clone, Default)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub audio: bool,
    pub tools: bool,
    pub json_mode: bool,
    pub parallel_function_calling: bool,
    pub response_format: bool,
    pub seed: bool,
    pub additional_capabilities: Vec<String>,
}

/// Represents a filter for finding models
#[derive(Debug, Clone, Default)]
pub struct ModelFilter {
    pub providers: Option<Vec<String>>,
    pub types: Option<Vec<ModelType>>,
    pub statuses: Option<Vec<ModelStatus>>,
    pub min_context_window: Option<u32>,
    pub required_capabilities: Option<ModelCapabilities>,
    pub max_cost_per_1k_input: Option<f32>,
    pub max_cost_per_1k_output: Option<f32>,
    pub max_latency_ms: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Represents the result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub latency_ms: f32,
    pub error_message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: std::collections::HashMap<String, String>,
}

/// Client interface for the Model Registry service
#[async_trait]
pub trait ModelRegistryClient: Send + Sync {
    /// Register a model in the registry
    async fn register_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata>;

    /// Get a model by ID
    async fn get_model(&self, model_id: &str) -> IpcResult<ModelMetadata>;

    /// Update a model in the registry
    async fn update_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata>;

    /// Remove a model from the registry
    async fn remove_model(&self, model_id: &str) -> IpcResult<ModelMetadata>;

    /// List all models in the registry
    async fn list_models(&self) -> IpcResult<Vec<ModelMetadata>>;

    /// Find models matching a filter
    async fn find_models(&self, filter: ModelFilter) -> IpcResult<Vec<ModelMetadata>>;

    /// Update a model's status
    async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
        reason: &str,
    ) -> IpcResult<ModelMetadata>;

    /// Check a model's health
    async fn check_model_health(
        &self,
        model_id: &str,
        timeout_ms: Option<u32>,
    ) -> IpcResult<HealthCheckResult>;
}

/// Server interface for the Model Registry service
#[async_trait]
pub trait ModelRegistryService: Send + Sync {
    /// Register a model in the registry
    async fn register_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata>;

    /// Get a model by ID
    async fn get_model(&self, model_id: &str) -> IpcResult<ModelMetadata>;

    /// Update a model in the registry
    async fn update_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata>;

    /// Remove a model from the registry
    async fn remove_model(&self, model_id: &str) -> IpcResult<ModelMetadata>;

    /// List all models in the registry
    async fn list_models(&self) -> IpcResult<Vec<ModelMetadata>>;

    /// Find models matching a filter
    async fn find_models(&self, filter: ModelFilter) -> IpcResult<Vec<ModelMetadata>>;

    /// Update a model's status
    async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
        reason: &str,
    ) -> IpcResult<ModelMetadata>;

    /// Check a model's health
    async fn check_model_health(
        &self,
        model_id: &str,
        timeout_ms: Option<u32>,
    ) -> IpcResult<HealthCheckResult>;
}

/// gRPC implementation of the Model Registry client
pub struct GrpcModelRegistryClient {
    // This would contain the generated gRPC client
    // client: model_registry_client::ModelRegistryClient<tonic::transport::Channel>,
}

impl GrpcModelRegistryClient {
    /// Create a new gRPC Model Registry client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = model_registry_client::ModelRegistryClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl ModelRegistryClient for GrpcModelRegistryClient {
    async fn register_model(&self, _metadata: ModelMetadata) -> IpcResult<ModelMetadata> {
        // Stub implementation for now
        Ok(ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        })
    }

    async fn get_model(&self, _model_id: &str) -> IpcResult<ModelMetadata> {
        // Stub implementation for now
        Ok(ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        })
    }

    async fn update_model(&self, _metadata: ModelMetadata) -> IpcResult<ModelMetadata> {
        // Stub implementation for now
        Ok(ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        })
    }

    async fn remove_model(&self, _model_id: &str) -> IpcResult<ModelMetadata> {
        // Stub implementation for now
        Ok(ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        })
    }

    async fn list_models(&self) -> IpcResult<Vec<ModelMetadata>> {
        // Stub implementation for now
        Ok(vec![ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        }])
    }

    async fn find_models(&self, _filter: ModelFilter) -> IpcResult<Vec<ModelMetadata>> {
        // Stub implementation for now
        Ok(vec![ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        }])
    }

    async fn update_model_status(
        &self,
        _model_id: &str,
        _status: ModelStatus,
        _reason: &str,
    ) -> IpcResult<ModelMetadata> {
        // Stub implementation for now
        Ok(ModelMetadata {
            id: "stub-model".to_string(),
            name: "Stub Model".to_string(),
            provider: "stub-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                function_calling: false,
                vision: false,
                audio: false,
                tools: false,
                json_mode: false,
                parallel_function_calling: false,
                response_format: false,
                seed: false,
                additional_capabilities: vec![],
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["stub".to_string()],
        })
    }

    async fn check_model_health(
        &self,
        _model_id: &str,
        _timeout_ms: Option<u32>,
    ) -> IpcResult<HealthCheckResult> {
        // Stub implementation for now
        Ok(HealthCheckResult {
            healthy: true,
            latency_ms: 10.0,
            error_message: None,
            timestamp: chrono::Utc::now(),
            details: std::collections::HashMap::new(),
        })
    }
}

/// Mock implementation of the Model Registry client for testing
#[cfg(test)]
pub struct MockModelRegistryClient {
    models: std::collections::HashMap<String, ModelMetadata>,
}

#[cfg(test)]
impl MockModelRegistryClient {
    /// Create a new mock Model Registry client
    pub fn new() -> Self {
        Self {
            models: std::collections::HashMap::new(),
        }
    }

    /// Add a model to the mock registry
    pub fn add_model(&mut self, metadata: ModelMetadata) {
        self.models.insert(metadata.id.clone(), metadata);
    }
}

#[cfg(test)]
#[async_trait]
impl ModelRegistryClient for MockModelRegistryClient {
    async fn register_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata> {
        Ok(metadata)
    }

    async fn get_model(&self, model_id: &str) -> IpcResult<ModelMetadata> {
        self.models
            .get(model_id)
            .cloned()
            .ok_or_else(|| IpcError::NotFound(format!("Model not found: {}", model_id)))
    }

    async fn update_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata> {
        if self.models.contains_key(&metadata.id) {
            Ok(metadata)
        } else {
            Err(IpcError::NotFound(format!(
                "Model not found: {}",
                metadata.id
            )))
        }
    }

    async fn remove_model(&self, model_id: &str) -> IpcResult<ModelMetadata> {
        self.models
            .get(model_id)
            .cloned()
            .ok_or_else(|| IpcError::NotFound(format!("Model not found: {}", model_id)))
    }

    async fn list_models(&self) -> IpcResult<Vec<ModelMetadata>> {
        Ok(self.models.values().cloned().collect())
    }

    async fn find_models(&self, _filter: ModelFilter) -> IpcResult<Vec<ModelMetadata>> {
        Ok(self.models.values().cloned().collect())
    }

    async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
        _reason: &str,
    ) -> IpcResult<ModelMetadata> {
        self.models
            .get(model_id)
            .cloned()
            .map(|mut metadata| {
                metadata.status = status;
                metadata
            })
            .ok_or_else(|| IpcError::NotFound(format!("Model not found: {}", model_id)))
    }

    async fn check_model_health(
        &self,
        model_id: &str,
        _timeout_ms: Option<u32>,
    ) -> IpcResult<HealthCheckResult> {
        if self.models.contains_key(model_id) {
            Ok(HealthCheckResult {
                healthy: true,
                latency_ms: 10.0,
                error_message: None,
                timestamp: chrono::Utc::now(),
                details: std::collections::HashMap::new(),
            })
        } else {
            Err(IpcError::NotFound(format!("Model not found: {}", model_id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_model_registry_client() {
        let mut client = MockModelRegistryClient::new();

        // Create a test model
        let model = ModelMetadata {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            provider: "test-provider".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Chat,
            status: ModelStatus::Available,
            context_window: 4096,
            capabilities: ModelCapabilities {
                streaming: true,
                ..Default::default()
            },
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.02,
            avg_latency_ms: 100.0,
            max_tokens_per_request: 4096,
            max_requests_per_minute: 100,
            metadata: std::collections::HashMap::new(),
            tags: vec!["test".to_string()],
        };

        // Add the model to the mock registry
        client.add_model(model.clone());

        // Test get_model
        let result = client.get_model(&model.id).await.unwrap();
        assert_eq!(result.id, model.id);

        // Test list_models
        let models = client.list_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, model.id);

        // Test update_model_status
        let updated = client
            .update_model_status(&model.id, ModelStatus::Maintenance, "Testing")
            .await
            .unwrap();
        assert_eq!(updated.status, ModelStatus::Maintenance);

        // Test get_model with non-existent ID
        let result = client.get_model("non-existent").await;
        assert!(result.is_err());
    }
}
