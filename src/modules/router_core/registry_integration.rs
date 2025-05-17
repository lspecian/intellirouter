//! Model Registry Integration for Router Core
//!
//! This module provides integration between the Router Core and Model Registry,
//! enabling dynamic model discovery and status updates.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::modules::model_registry::{
    ModelFilter, ModelMetadata, ModelRegistry, ModelStatus, RegistryError,
};

use super::{RouterError, RoutingRequest};

/// Integration between Router Core and Model Registry
pub struct RegistryIntegration {
    /// Reference to the model registry
    registry: Arc<ModelRegistry>,
}

impl RegistryIntegration {
    /// Create a new registry integration
    pub fn new(registry: Arc<ModelRegistry>) -> Self {
        Self { registry }
    }

    /// Update metrics with model registry information
    pub async fn update_metrics(
        &self,
        metrics: &mut HashMap<String, serde_json::Value>,
    ) -> Result<(), RouterError> {
        // Get all available models
        let models = self.registry.list_models();

        // Update available models count
        metrics.insert(
            "available_models_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(models.len())),
        );

        // Update available models list
        let models_list = models
            .iter()
            .map(|m| serde_json::Value::String(m.id.clone()))
            .collect::<Vec<_>>();
        metrics.insert(
            "available_models".to_string(),
            serde_json::Value::Array(models_list),
        );

        // Update provider information
        let mut providers = HashMap::new();
        for model in &models {
            let provider = model.provider.clone();
            let count = providers.entry(provider.clone()).or_insert(0);
            *count += 1;
        }

        let provider_info = providers
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::Value::Number(serde_json::Number::from(*v)),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        metrics.insert(
            "providers".to_string(),
            serde_json::Value::Object(provider_info),
        );

        // Update model types information
        let mut model_types = HashMap::new();
        for model in &models {
            let model_type = format!("{:?}", model.model_type);
            let count = model_types.entry(model_type.clone()).or_insert(0);
            *count += 1;
        }

        let model_types_info = model_types
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::Value::Number(serde_json::Number::from(*v)),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        metrics.insert(
            "model_types".to_string(),
            serde_json::Value::Object(model_types_info),
        );

        // Update model status information
        let mut statuses = HashMap::new();
        for model in &models {
            let status = format!("{:?}", model.status);
            let count = statuses.entry(status.clone()).or_insert(0);
            *count += 1;
        }

        let status_info = statuses
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::Value::Number(serde_json::Number::from(*v)),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        metrics.insert(
            "model_statuses".to_string(),
            serde_json::Value::Object(status_info),
        );

        Ok(())
    }

    /// Get a filtered list of models based on request criteria
    pub async fn get_filtered_models(
        &self,
        request: &RoutingRequest,
    ) -> Result<Vec<ModelMetadata>, RouterError> {
        // Get all available models
        let models = self.registry.list_models();

        // Apply model filter if provided
        let filtered_models = if let Some(filter) = &request.model_filter {
            self.registry.find_models(filter)
        } else {
            // Default filter: only available models
            let filter = ModelFilter::new().with_status(ModelStatus::Available);
            self.registry.find_models(&filter)
        };

        // Filter out excluded models
        let filtered_models = filtered_models
            .into_iter()
            .filter(|model| !request.excluded_model_ids.contains(&model.id))
            .collect::<Vec<_>>();

        // Check if we have any models left
        if filtered_models.is_empty() {
            return Err(RouterError::NoSuitableModel(
                "No suitable models found after filtering".to_string(),
            ));
        }

        // Apply preferred model if specified
        if let Some(preferred_id) = &request.preferred_model_id {
            if let Some(preferred_model) = filtered_models.iter().find(|m| &m.id == preferred_id) {
                return Ok(vec![preferred_model.clone()]);
            }
            // If preferred model not found in filtered list, continue with filtered models
            warn!(
                "Preferred model {} not found in filtered models",
                preferred_id
            );
        }

        Ok(filtered_models)
    }

    /// Subscribe to model registry updates
    pub async fn subscribe_to_registry_updates(&self) -> Result<(), RouterError> {
        // This would typically involve setting up a subscription to registry events
        // For now, we'll just log that we're subscribing
        info!("Subscribing to model registry updates");
        Ok(())
    }

    /// Check model health and update status
    pub async fn check_model_health(&self, model_id: &str) -> Result<ModelStatus, RouterError> {
        // Get the model
        let model = self
            .registry
            .get_model(model_id)
            .map_err(|e| RouterError::RegistryError(e))?;

        // In a real implementation, we would check the model's health
        // For now, we'll just return the current status
        Ok(model.status)
    }

    /// Update model status in the registry
    pub async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
    ) -> Result<(), RouterError> {
        self.registry
            .update_model_status(model_id, status)
            .map_err(|e| RouterError::RegistryError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::ModelType;

    // Helper function to create a test model
    fn create_test_model(id: &str, provider: &str) -> ModelMetadata {
        let mut model = ModelMetadata::new(
            id.to_string(),
            format!("Test Model {}", id),
            provider.to_string(),
            "1.0".to_string(),
            "https://example.com".to_string(),
        );

        model.set_status(ModelStatus::Available);
        model.set_model_type(ModelType::TextGeneration);
        model.capabilities.max_context_length = 4096;
        model.capabilities.supports_streaming = true;
        model.capabilities.supports_function_calling = true;

        model
    }

    #[tokio::test]
    async fn test_update_metrics() {
        // Create a registry with test models
        let registry = Arc::new(ModelRegistry::new());

        // Add test models
        registry
            .register_model(create_test_model("model1", "provider1"))
            .unwrap();
        registry
            .register_model(create_test_model("model2", "provider1"))
            .unwrap();
        registry
            .register_model(create_test_model("model3", "provider2"))
            .unwrap();

        // Create registry integration
        let integration = RegistryIntegration::new(registry);

        // Create metrics
        let mut metrics = HashMap::new();

        // Update metrics
        let result = integration.update_metrics(&mut metrics).await;
        assert!(result.is_ok());

        // Check metrics
        assert_eq!(
            metrics.get("available_models_count"),
            Some(&serde_json::Value::Number(serde_json::Number::from(3)))
        );

        // Check providers
        if let Some(serde_json::Value::Object(providers)) = metrics.get("providers") {
            assert_eq!(
                providers.get("provider1"),
                Some(&serde_json::Value::Number(serde_json::Number::from(2)))
            );
            assert_eq!(
                providers.get("provider2"),
                Some(&serde_json::Value::Number(serde_json::Number::from(1)))
            );
        } else {
            panic!("Expected providers to be an object");
        }
    }

    #[tokio::test]
    async fn test_get_filtered_models() {
        // Create a registry with test models
        let registry = Arc::new(ModelRegistry::new());

        // Add test models
        registry
            .register_model(create_test_model("model1", "provider1"))
            .unwrap();
        registry
            .register_model(create_test_model("model2", "provider1"))
            .unwrap();
        registry
            .register_model(create_test_model("model3", "provider2"))
            .unwrap();

        // Create registry integration
        let integration = RegistryIntegration::new(registry);

        // Create a request with a filter
        let mut request = RoutingRequest::new(
            crate::modules::model_registry::connectors::ChatCompletionRequest {
                model: "test-model".to_string(),
                messages: vec![],
                temperature: None,
                top_p: None,
                max_tokens: None,
                stream: None,
                functions: None,
                tools: None,
                additional_params: None,
            },
        );

        request.model_filter = Some(ModelFilter::new().with_provider("provider1".to_string()));

        // Get filtered models
        let result = integration.get_filtered_models(&request).await;
        assert!(result.is_ok());

        let models = result.unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "model1");
        assert_eq!(models[1].id, "model2");

        // Test with excluded models
        let mut request = RoutingRequest::new(
            crate::modules::model_registry::connectors::ChatCompletionRequest {
                model: "test-model".to_string(),
                messages: vec![],
                temperature: None,
                top_p: None,
                max_tokens: None,
                stream: None,
                functions: None,
                tools: None,
                additional_params: None,
            },
        );

        request.excluded_model_ids = vec!["model1".to_string(), "model2".to_string()];

        let result = integration.get_filtered_models(&request).await;
        assert!(result.is_ok());

        let models = result.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "model3");
    }
}
