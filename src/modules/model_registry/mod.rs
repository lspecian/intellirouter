//! Model Registry Module
//!
//! This module handles tracking and metadata for various LLM models.
//! It provides information about model capabilities, versions, and requirements.

pub mod api;
pub mod connectors;
pub mod health;
pub mod persistence;
pub mod storage;
pub mod types;

#[cfg(test)]
mod tests;

use std::sync::Arc;

// Re-export types for easier access
pub use api::{create_model_registry_api, ModelRegistryApi};
pub use connectors::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ConnectorConfig,
    ConnectorError, ModelConnector, ModelConnectorFactory,
};
pub use health::{
    check_model_health, create_health_check_manager, HealthCheckConfig, HealthCheckManager,
    HealthCheckResult,
};
pub use persistence::{
    create_file_persistent_registry, ModelRegistryPersistence, PersistenceConfig,
    PersistentModelRegistry,
};
pub use storage::ModelRegistry;
pub use types::{
    capabilities::ModelCapabilities,
    errors::RegistryError,
    filters::ModelFilter,
    model::{ModelMetadata, ModelType},
    status::ModelStatus,
};

// Provide a global instance for convenience
use std::sync::OnceLock;

static GLOBAL_REGISTRY: OnceLock<ModelRegistryApi> = OnceLock::new();
static GLOBAL_HEALTH_MANAGER: OnceLock<std::sync::Mutex<HealthCheckManager>> = OnceLock::new();

/// Get the global Model Registry API instance
pub fn global_registry() -> &'static ModelRegistryApi {
    GLOBAL_REGISTRY.get_or_init(|| create_model_registry_api())
}

/// Get the global Health Check Manager instance
pub fn global_health_manager() -> &'static std::sync::Mutex<HealthCheckManager> {
    GLOBAL_HEALTH_MANAGER.get_or_init(|| {
        std::sync::Mutex::new(create_health_check_manager(Arc::new(
            global_registry().clone(),
        )))
    })
}

/// Start global health checks with default configuration
pub fn start_global_health_checks() -> Result<(), String> {
    match global_health_manager().lock() {
        Ok(mut manager) => {
            manager.start_health_checks();
            Ok(())
        }
        Err(e) => Err(format!("Failed to acquire lock on health manager: {}", e)),
    }
}

/// Stop global health checks
pub fn stop_global_health_checks() -> Result<(), String> {
    match global_health_manager().lock() {
        Ok(mut manager) => {
            manager.stop_health_checks();
            Ok(())
        }
        Err(e) => Err(format!("Failed to acquire lock on health manager: {}", e)),
    }
}

/// Register a model in the global registry
pub fn register_model(metadata: ModelMetadata) -> Result<(), RegistryError> {
    global_registry().register_model(metadata)
}

/// Get a model from the global registry
pub fn get_model(id: &str) -> Result<ModelMetadata, RegistryError> {
    global_registry().get_model(id)
}

/// Update a model in the global registry
pub fn update_model(metadata: ModelMetadata) -> Result<(), RegistryError> {
    global_registry().update_model(metadata)
}

/// Remove a model from the global registry
pub fn remove_model(id: &str) -> Result<ModelMetadata, RegistryError> {
    global_registry().remove_model(id)
}

/// List all models in the global registry
pub fn list_models() -> Vec<ModelMetadata> {
    global_registry().list_models()
}

/// Find models matching the given filter in the global registry
pub fn find_models(filter: &ModelFilter) -> Vec<ModelMetadata> {
    global_registry().find_models(filter)
}

/// Find models by provider in the global registry
pub fn find_by_provider(provider: &str) -> Vec<ModelMetadata> {
    global_registry().find_by_provider(provider)
}

/// Find models by type in the global registry
pub fn find_by_type(model_type: ModelType) -> Vec<ModelMetadata> {
    global_registry().find_by_type(model_type)
}

/// Find available models in the global registry
pub fn find_available_models() -> Vec<ModelMetadata> {
    global_registry().find_available_models()
}

/// Update model status in the global registry
pub fn update_model_status(id: &str, status: ModelStatus) -> Result<(), RegistryError> {
    global_registry().update_model_status(id, status)
}

/// Legacy model information structure
///
/// This is kept for backward compatibility and will be replaced by ModelMetadata
#[deprecated(
    since = "0.1.0",
    note = "Use ModelMetadata instead, this will be removed in a future version"
)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub version: String,
    pub context_window: usize,
    pub capabilities: Vec<String>,
}

// Global registry tests moved to tests/mod.rs

/// Create a persistent model registry with default configuration
pub fn create_persistent_registry() -> Result<PersistentModelRegistry, RegistryError> {
    create_file_persistent_registry(PersistenceConfig::default())
}
