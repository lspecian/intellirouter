//! Model Registry Storage
//!
//! This module implements a thread-safe in-memory storage for model metadata
//! using DashMap for concurrent access.

use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::types::{
    ModelFilter, ModelMetadata, ModelStatus, ModelType, RegistryError,
};

/// Thread-safe in-memory storage for model metadata
// Remove Debug derive since dyn ModelConnector doesn't implement Debug
#[derive(Clone)]
pub struct ModelRegistry {
    /// Internal storage using DashMap for thread-safe concurrent access
    models: Arc<DashMap<String, ModelMetadata>>,
    /// Model connectors
    connectors: Arc<DashMap<String, Arc<dyn super::connectors::ModelConnector>>>,
}

// Manual Debug implementation
impl std::fmt::Debug for ModelRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRegistry")
            .field("models", &self.models)
            .field(
                "connectors",
                &format!("<{} connectors>", self.connectors.len()),
            )
            .finish()
    }
}

impl ModelRegistry {
    /// Create a new empty model registry
    pub fn new() -> Self {
        debug!("Creating new model registry");
        Self {
            models: Arc::new(DashMap::new()),
            connectors: Arc::new(DashMap::new()),
        }
    }

    /// Register a connector for a model
    pub fn register_connector(
        &self,
        model_id: &str,
        connector: Arc<dyn super::connectors::ModelConnector>,
    ) {
        debug!("Registering connector for model: {}", model_id);
        self.connectors.insert(model_id.to_string(), connector);
    }

    /// Get a connector for a model
    pub fn get_connector(
        &self,
        model_id: &str,
    ) -> Option<Arc<dyn super::connectors::ModelConnector>> {
        debug!("Getting connector for model: {}", model_id);
        self.connectors.get(model_id).map(|c| c.clone())
    }

    /// Register a new model in the registry
    pub fn register_model(&self, metadata: ModelMetadata) -> Result<(), RegistryError> {
        let id = metadata.id.clone();

        // Check if model already exists
        if self.models.contains_key(&id) {
            error!("Model already exists: {}", id);
            return Err(RegistryError::AlreadyExists(id));
        }

        // Insert the model
        debug!("Registering model: {}", id);
        self.models.insert(id.clone(), metadata);
        info!("Model registered successfully: {}", id);
        Ok(())
    }

    /// Get a model by ID
    pub fn get_model(&self, id: &str) -> Result<ModelMetadata, RegistryError> {
        debug!("Getting model: {}", id);
        self.models
            .get(id)
            .map(|model| model.clone())
            .ok_or_else(|| {
                error!("Model not found: {}", id);
                RegistryError::NotFound(id.to_string())
            })
    }

    /// Update an existing model
    pub fn update_model(&self, metadata: ModelMetadata) -> Result<(), RegistryError> {
        let id = metadata.id.clone();

        // Check if model exists
        if !self.models.contains_key(&id) {
            error!("Cannot update non-existent model: {}", id);
            return Err(RegistryError::NotFound(id));
        }

        // Update the model
        debug!("Updating model: {}", id);
        self.models.insert(id.clone(), metadata);
        info!("Model updated successfully: {}", id);
        Ok(())
    }

    /// Remove a model from the registry
    pub fn remove_model(&self, id: &str) -> Result<ModelMetadata, RegistryError> {
        debug!("Removing model: {}", id);
        self.models
            .remove(id)
            .map(|(_, model)| {
                info!("Model removed successfully: {}", id);
                model
            })
            .ok_or_else(|| {
                error!("Cannot remove non-existent model: {}", id);
                RegistryError::NotFound(id.to_string())
            })
    }

    /// List all models in the registry
    pub fn list_models(&self) -> Vec<ModelMetadata> {
        debug!("Listing all models");
        self.models
            .iter()
            .map(|item| item.value().clone())
            .collect()
    }

    /// Find models matching the given filter
    pub fn find_models(&self, filter: &ModelFilter) -> Vec<ModelMetadata> {
        debug!("Finding models with filter");
        self.models
            .iter()
            .filter(|item| Self::matches_filter(item.value(), filter))
            .map(|item| item.value().clone())
            .collect()
    }

    /// Check if a model matches the given filter
    fn matches_filter(model: &ModelMetadata, filter: &ModelFilter) -> bool {
        // Provider filter
        if let Some(provider) = &filter.provider {
            if model.provider != *provider {
                return false;
            }
        }

        // Model type filter
        if let Some(model_type) = &filter.model_type {
            if model.model_type != *model_type {
                return false;
            }
        }

        // Status filter
        if let Some(status) = &filter.status {
            if model.status != *status {
                return false;
            }
        }

        // Min context length filter
        if let Some(min_context_length) = filter.min_context_length {
            if model.capabilities.max_context_length < min_context_length {
                return false;
            }
        }

        // Function calling support filter
        if let Some(supports_function_calling) = filter.supports_function_calling {
            if model.capabilities.supports_function_calling != supports_function_calling {
                return false;
            }
        }

        // Vision support filter
        if let Some(supports_vision) = filter.supports_vision {
            if model.capabilities.supports_vision != supports_vision {
                return false;
            }
        }

        // Streaming support filter
        if let Some(supports_streaming) = filter.supports_streaming {
            if model.capabilities.supports_streaming != supports_streaming {
                return false;
            }
        }

        // Embeddings support filter
        if let Some(supports_embeddings) = filter.supports_embeddings {
            if model.capabilities.supports_embeddings != supports_embeddings {
                return false;
            }
        }

        // Language filter
        if let Some(language) = &filter.language {
            if !model.capabilities.supported_languages.contains(language) {
                return false;
            }
        }

        // Input format filter
        if let Some(input_format) = &filter.input_format {
            if !model
                .capabilities
                .supported_input_formats
                .contains(input_format)
            {
                return false;
            }
        }

        // Output format filter
        if let Some(output_format) = &filter.output_format {
            if !model
                .capabilities
                .supported_output_formats
                .contains(output_format)
            {
                return false;
            }
        }

        // Minimum version filter
        if let Some(min_version) = &filter.min_version {
            if !model.capabilities.version_info.is_newer_than(min_version) {
                return false;
            }
        }

        // Maximum cost per 1K tokens (input) filter
        if let Some(max_cost) = filter.max_cost_per_1k_tokens_input {
            if model.capabilities.cost_per_1k_tokens_input > max_cost {
                return false;
            }
        }

        // Maximum cost per 1K tokens (output) filter
        if let Some(max_cost) = filter.max_cost_per_1k_tokens_output {
            if model.capabilities.cost_per_1k_tokens_output > max_cost {
                return false;
            }
        }

        // Maximum latency filter
        if let Some(max_latency) = filter.max_latency_ms {
            if let Some(avg_latency) = model.capabilities.performance.avg_latency_ms {
                if avg_latency > max_latency {
                    return false;
                }
            }
        }

        // Required features filter
        for (feature, enabled) in &filter.required_features {
            if model.capabilities.supports_feature(feature) != *enabled {
                return false;
            }
        }

        // Additional filters
        for (key, value) in &filter.additional_filters {
            if let Some(model_value) = model.additional_metadata.get(key) {
                if model_value != value {
                    return false;
                }
            } else {
                // Check if it's in capabilities
                if let Some(cap_value) = model.capabilities.additional_capabilities.get(key) {
                    if cap_value != value {
                        return false;
                    }
                } else {
                    // Neither in metadata nor capabilities
                    return false;
                }
            }
        }

        // All filters passed
        true
    }

    /// Count the number of models in the registry
    pub fn count(&self) -> usize {
        let count = self.models.len();
        debug!("Model count: {}", count);
        count
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        let empty = self.models.is_empty();
        debug!("Registry is empty: {}", empty);
        empty
    }

    /// Clear all models from the registry
    pub fn clear(&self) {
        debug!("Clearing all models from registry");
        self.models.clear();
        info!("Registry cleared");
    }

    /// Find models by provider
    pub fn find_by_provider(&self, provider: &str) -> Vec<ModelMetadata> {
        debug!("Finding models by provider: {}", provider);
        let filter = ModelFilter::new().with_provider(provider.to_string());
        self.find_models(&filter)
    }

    /// Find models by type
    pub fn find_by_type(&self, model_type: ModelType) -> Vec<ModelMetadata> {
        debug!("Finding models by type: {:?}", model_type);
        let filter = ModelFilter::new().with_model_type(model_type);
        self.find_models(&filter)
    }

    /// Find available models
    pub fn find_available_models(&self) -> Vec<ModelMetadata> {
        debug!("Finding available models");
        let filter = ModelFilter::new().with_status(ModelStatus::Available);
        self.find_models(&filter)
    }

    /// Update model status
    pub fn update_model_status(&self, id: &str, status: ModelStatus) -> Result<(), RegistryError> {
        debug!("Updating status for model {}: {:?}", id, status);
        let mut model = self.get_model(id)?;
        model.set_status(status);
        self.update_model(model)
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::types::{ModelMetadata, ModelStatus, ModelType};
    use std::collections::HashSet;

    fn create_test_model(id: &str, provider: &str) -> ModelMetadata {
        ModelMetadata::new(
            id.to_string(),
            format!("{} Model", id),
            provider.to_string(),
            "1.0".to_string(),
            format!("https://api.{}.com/v1", provider),
        )
    }

    #[test]
    fn test_registry_crud_operations() {
        let registry = ModelRegistry::new();
        assert!(registry.is_empty());

        // Register a model
        let model = create_test_model("gpt-4", "openai");
        registry.register_model(model.clone()).unwrap();
        assert_eq!(registry.count(), 1);

        // Get the model
        let retrieved = registry.get_model("gpt-4").unwrap();
        assert_eq!(retrieved.id, "gpt-4");
        assert_eq!(retrieved.provider, "openai");

        // Update the model
        let mut updated = model.clone();
        updated.set_description("Updated model".to_string());
        registry.update_model(updated.clone()).unwrap();

        let retrieved = registry.get_model("gpt-4").unwrap();
        assert_eq!(retrieved.description, Some("Updated model".to_string()));

        // Remove the model
        let removed = registry.remove_model("gpt-4").unwrap();
        assert_eq!(removed.id, "gpt-4");
        assert!(registry.is_empty());

        // Try to get a non-existent model
        let result = registry.get_model("gpt-4");
        assert!(result.is_err());
        match result {
            Err(RegistryError::NotFound(id)) => assert_eq!(id, "gpt-4"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_registry_duplicate_registration() {
        let registry = ModelRegistry::new();

        // Register a model
        let model = create_test_model("gpt-4", "openai");
        registry.register_model(model.clone()).unwrap();

        // Try to register the same model again
        let result = registry.register_model(model.clone());
        assert!(result.is_err());
        match result {
            Err(RegistryError::AlreadyExists(id)) => assert_eq!(id, "gpt-4"),
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[test]
    fn test_registry_filtering() {
        let registry = ModelRegistry::new();

        // Register multiple models
        registry
            .register_model(create_test_model("gpt-4", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("gpt-3.5", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("claude-2", "anthropic"))
            .unwrap();

        // Get all models
        let all_models = registry.list_models();
        assert_eq!(all_models.len(), 3);

        // Verify all models are present
        let model_ids: HashSet<String> = all_models.iter().map(|m| m.id.clone()).collect();
        assert!(model_ids.contains("gpt-4"));
        assert!(model_ids.contains("gpt-3.5"));
        assert!(model_ids.contains("claude-2"));

        // Filter by provider
        let openai_models = registry.find_by_provider("openai");
        assert_eq!(openai_models.len(), 2);

        let anthropic_models = registry.find_by_provider("anthropic");
        assert_eq!(anthropic_models.len(), 1);
        assert_eq!(anthropic_models[0].id, "claude-2");

        // Update model status
        registry
            .update_model_status("gpt-4", ModelStatus::Available)
            .unwrap();
        registry
            .update_model_status("claude-2", ModelStatus::Available)
            .unwrap();

        // Filter by status
        let available_models = registry.find_available_models();
        assert_eq!(available_models.len(), 2);

        // Complex filter
        let mut gpt4 = registry.get_model("gpt-4").unwrap();
        gpt4.capabilities.supports_function_calling = true;
        gpt4.capabilities.max_context_length = 8192;
        registry.update_model(gpt4).unwrap();

        let filter = ModelFilter::new()
            .with_provider("openai".to_string())
            .with_status(ModelStatus::Available)
            .with_function_calling(true);

        let filtered = registry.find_models(&filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "gpt-4");
    }

    #[test]
    fn test_registry_clear() {
        let registry = ModelRegistry::new();

        // Register multiple models
        registry
            .register_model(create_test_model("gpt-4", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("gpt-3.5", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("claude-2", "anthropic"))
            .unwrap();

        assert_eq!(registry.count(), 3);

        // Clear the registry
        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_model_type_filtering() {
        let registry = ModelRegistry::new();

        // Create models with different types
        let mut text_model = create_test_model("gpt-4", "openai");
        text_model.set_model_type(ModelType::TextGeneration);

        let mut embedding_model = create_test_model("text-embedding-ada", "openai");
        embedding_model.set_model_type(ModelType::Embedding);

        let mut multimodal_model = create_test_model("gpt-4-vision", "openai");
        multimodal_model.set_model_type(ModelType::MultiModal);

        // Register models
        registry.register_model(text_model).unwrap();
        registry.register_model(embedding_model).unwrap();
        registry.register_model(multimodal_model).unwrap();

        // Filter by model type
        let text_models = registry.find_by_type(ModelType::TextGeneration);
        assert_eq!(text_models.len(), 1);
        assert_eq!(text_models[0].id, "gpt-4");

        let embedding_models = registry.find_by_type(ModelType::Embedding);
        assert_eq!(embedding_models.len(), 1);
        assert_eq!(embedding_models[0].id, "text-embedding-ada");

        let multimodal_models = registry.find_by_type(ModelType::MultiModal);
        assert_eq!(multimodal_models.len(), 1);
        assert_eq!(multimodal_models[0].id, "gpt-4-vision");
    }

    #[test]
    fn test_capability_filtering() {
        let registry = ModelRegistry::new();

        // Create models with different capabilities
        let mut model1 = create_test_model("model1", "provider1");
        model1.capabilities.supports_vision = true;
        model1.capabilities.max_context_length = 16384;
        model1
            .capabilities
            .supported_input_formats
            .push(InputFormat::Image);
        model1.capabilities.version_info = ModelVersionInfo {
            major: 2,
            minor: 0,
            patch: 0,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };
        model1.capabilities.cost_per_1k_tokens_input = 0.01;
        model1.capabilities.cost_per_1k_tokens_output = 0.02;
        model1.capabilities.performance.avg_latency_ms = Some(50.0);

        let mut model2 = create_test_model("model2", "provider1");
        model2.capabilities.supports_function_calling = true;
        model2.capabilities.max_context_length = 8192;
        model2
            .capabilities
            .supported_output_formats
            .push(OutputFormat::Json);
        model2.capabilities.version_info = ModelVersionInfo {
            major: 1,
            minor: 5,
            patch: 0,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };
        model2.capabilities.cost_per_1k_tokens_input = 0.02;
        model2.capabilities.cost_per_1k_tokens_output = 0.04;
        model2.capabilities.performance.avg_latency_ms = Some(100.0);
        model2
            .capabilities
            .add_feature_flag("advanced_math".to_string(), true);

        let mut model3 = create_test_model("model3", "provider2");
        model3.capabilities.supports_embeddings = true;
        model3.capabilities.max_context_length = 4096;
        model3.capabilities.version_info = ModelVersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };
        model3.capabilities.cost_per_1k_tokens_input = 0.03;
        model3.capabilities.cost_per_1k_tokens_output = 0.06;
        model3.capabilities.performance.avg_latency_ms = Some(150.0);

        // Register models
        registry.register_model(model1).unwrap();
        registry.register_model(model2).unwrap();
        registry.register_model(model3).unwrap();

        // Filter by capabilities
        let vision_models = registry.find_models(&ModelFilter::new().with_vision(true));
        assert_eq!(vision_models.len(), 1);
        assert_eq!(vision_models[0].id, "model1");

        let function_calling_models =
            registry.find_models(&ModelFilter::new().with_function_calling(true));
        assert_eq!(function_calling_models.len(), 1);
        assert_eq!(function_calling_models[0].id, "model2");

        let embedding_models = registry.find_models(&ModelFilter::new().with_embeddings(true));
        assert_eq!(embedding_models.len(), 1);
        assert_eq!(embedding_models[0].id, "model3");

        // Filter by context length
        let large_context_models =
            registry.find_models(&ModelFilter::new().with_min_context_length(10000));
        assert_eq!(large_context_models.len(), 1);
        assert_eq!(large_context_models[0].id, "model1");

        // Filter by input format
        let image_models =
            registry.find_models(&ModelFilter::new().with_input_format(InputFormat::Image));
        assert_eq!(image_models.len(), 1);
        assert_eq!(image_models[0].id, "model1");

        // Filter by output format
        let json_models =
            registry.find_models(&ModelFilter::new().with_output_format(OutputFormat::Json));
        assert_eq!(json_models.len(), 1);
        assert_eq!(json_models[0].id, "model2");

        // Filter by version
        let min_version = ModelVersionInfo {
            major: 1,
            minor: 5,
            patch: 0,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };
        let newer_models = registry.find_models(&ModelFilter::new().with_min_version(min_version));
        assert_eq!(newer_models.len(), 1);

        // Filter by cost
        let cheap_models =
            registry.find_models(&ModelFilter::new().with_max_cost_per_1k_tokens_input(0.02));
        assert_eq!(cheap_models.len(), 2);

        // Filter by latency
        let fast_models = registry.find_models(&ModelFilter::new().with_max_latency_ms(100.0));
        assert_eq!(fast_models.len(), 2);

        // Filter by feature flag
        let math_models = registry.find_models(
            &ModelFilter::new().with_required_feature("advanced_math".to_string(), true),
        );
        assert_eq!(math_models.len(), 1);
        assert_eq!(math_models[0].id, "model2");

        // Combined filters
        let filter = ModelFilter::new()
            .with_provider("provider1".to_string())
            .with_min_context_length(8000);

        let filtered_models = registry.find_models(&filter);
        assert_eq!(filtered_models.len(), 2);

        // Complex combined filters
        let complex_filter = ModelFilter::new()
            .with_provider("provider1".to_string())
            .with_max_latency_ms(75.0)
            .with_max_cost_per_1k_tokens_input(0.015);

        let filtered_models = registry.find_models(&complex_filter);
        assert_eq!(filtered_models.len(), 1);
        assert_eq!(filtered_models[0].id, "model1");
    }

    #[test]
    fn test_language_filtering() {
        let registry = ModelRegistry::new();

        // Create models with different language support
        let mut model1 = create_test_model("model1", "provider1");
        model1.capabilities.supported_languages =
            vec!["en".to_string(), "fr".to_string(), "de".to_string()];

        let mut model2 = create_test_model("model2", "provider1");
        model2.capabilities.supported_languages = vec!["en".to_string(), "es".to_string()];

        let mut model3 = create_test_model("model3", "provider2");
        model3.capabilities.supported_languages = vec!["en".to_string(), "ja".to_string()];

        // Register models
        registry.register_model(model1).unwrap();
        registry.register_model(model2).unwrap();
        registry.register_model(model3).unwrap();

        // Filter by language
        let french_models =
            registry.find_models(&ModelFilter::new().with_language("fr".to_string()));
        assert_eq!(french_models.len(), 1);
        assert_eq!(french_models[0].id, "model1");

        let spanish_models =
            registry.find_models(&ModelFilter::new().with_language("es".to_string()));
        assert_eq!(spanish_models.len(), 1);
        assert_eq!(spanish_models[0].id, "model2");

        let japanese_models =
            registry.find_models(&ModelFilter::new().with_language("ja".to_string()));
        assert_eq!(japanese_models.len(), 1);
        assert_eq!(japanese_models[0].id, "model3");

        // All models should support English
        let english_models =
            registry.find_models(&ModelFilter::new().with_language("en".to_string()));
        assert_eq!(english_models.len(), 3);
    }

    #[test]
    fn test_additional_filter_criteria() {
        let registry = ModelRegistry::new();

        // Create models with additional metadata and capabilities
        let mut model1 = create_test_model("model1", "provider1");
        model1.add_metadata("release_date".to_string(), "2023-01-01".to_string());
        model1.add_capability("specialized_for".to_string(), "code".to_string());

        let mut model2 = create_test_model("model2", "provider1");
        model2.add_metadata("release_date".to_string(), "2023-06-15".to_string());
        model2.add_capability("specialized_for".to_string(), "creative".to_string());

        let mut model3 = create_test_model("model3", "provider2");
        model3.add_metadata("release_date".to_string(), "2022-11-30".to_string());
        model3.add_capability("specialized_for".to_string(), "analysis".to_string());

        // Register models
        registry.register_model(model1).unwrap();
        registry.register_model(model2).unwrap();
        registry.register_model(model3).unwrap();

        // Filter by additional metadata
        let filter1 = ModelFilter::new()
            .with_additional_filter("release_date".to_string(), "2023-01-01".to_string());

        let filtered1 = registry.find_models(&filter1);
        assert_eq!(filtered1.len(), 1);
        assert_eq!(filtered1[0].id, "model1");

        // Filter by additional capability
        let filter2 = ModelFilter::new()
            .with_additional_filter("specialized_for".to_string(), "creative".to_string());

        let filtered2 = registry.find_models(&filter2);
        assert_eq!(filtered2.len(), 1);
        assert_eq!(filtered2[0].id, "model2");

        // Combined filters
        let filter3 = ModelFilter::new()
            .with_provider("provider2".to_string())
            .with_additional_filter("specialized_for".to_string(), "analysis".to_string());

        let filtered3 = registry.find_models(&filter3);
        assert_eq!(filtered3.len(), 1);
        assert_eq!(filtered3[0].id, "model3");
    }
}
