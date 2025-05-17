//! Model Registry API
//!
//! This module provides a clean API interface for interacting with the Model Registry.
//! It includes functions for registering, retrieving, updating, and removing models,
//! as well as querying and filtering models.

use std::sync::Arc;
use tracing::debug;

use super::storage::ModelRegistry;
use super::types::{ModelFilter, ModelMetadata, ModelStatus, ModelType, RegistryError};

/// API for interacting with the Model Registry
#[derive(Debug, Clone)]
pub struct ModelRegistryApi {
    /// Internal storage
    registry: Arc<ModelRegistry>,
}

impl ModelRegistryApi {
    /// Create a new Model Registry API instance with a new registry
    pub fn new() -> Self {
        debug!("Creating new Model Registry API instance");
        Self {
            registry: Arc::new(ModelRegistry::new()),
        }
    }

    /// Create a new Model Registry API instance with an existing registry
    pub fn with_registry(registry: Arc<ModelRegistry>) -> Self {
        debug!("Creating Model Registry API with existing registry");
        Self { registry }
    }

    /// Get the internal registry
    pub fn registry(&self) -> Arc<ModelRegistry> {
        self.registry.clone()
    }

    /// Register a new model in the registry
    ///
    /// # Arguments
    /// * `metadata` - The model metadata to register
    ///
    /// # Returns
    /// * `Ok(())` if the model was registered successfully
    /// * `Err(RegistryError)` if the model could not be registered
    pub fn register_model(&self, metadata: ModelMetadata) -> Result<(), RegistryError> {
        debug!("API: Registering model: {}", metadata.id);

        // Validate the model metadata
        self.validate_model_metadata(&metadata)?;

        // Register the model
        self.registry.register_model(metadata)
    }

    /// Get a model by ID
    ///
    /// # Arguments
    /// * `id` - The ID of the model to retrieve
    ///
    /// # Returns
    /// * `Ok(ModelMetadata)` if the model was found
    /// * `Err(RegistryError)` if the model was not found
    pub fn get_model(&self, id: &str) -> Result<ModelMetadata, RegistryError> {
        debug!("API: Getting model: {}", id);
        self.registry.get_model(id)
    }

    /// Update an existing model
    ///
    /// # Arguments
    /// * `metadata` - The updated model metadata
    ///
    /// # Returns
    /// * `Ok(())` if the model was updated successfully
    /// * `Err(RegistryError)` if the model could not be updated
    pub fn update_model(&self, metadata: ModelMetadata) -> Result<(), RegistryError> {
        debug!("API: Updating model: {}", metadata.id);

        // Validate the model metadata
        self.validate_model_metadata(&metadata)?;

        // Update the model
        self.registry.update_model(metadata)
    }

    /// Remove a model from the registry
    ///
    /// # Arguments
    /// * `id` - The ID of the model to remove
    ///
    /// # Returns
    /// * `Ok(ModelMetadata)` if the model was removed successfully
    /// * `Err(RegistryError)` if the model could not be removed
    pub fn remove_model(&self, id: &str) -> Result<ModelMetadata, RegistryError> {
        debug!("API: Removing model: {}", id);
        self.registry.remove_model(id)
    }

    /// List all models in the registry
    ///
    /// # Returns
    /// * A vector of all model metadata in the registry
    pub fn list_models(&self) -> Vec<ModelMetadata> {
        debug!("API: Listing all models");
        self.registry.list_models()
    }

    /// Find models matching the given filter
    ///
    /// # Arguments
    /// * `filter` - The filter to apply
    ///
    /// # Returns
    /// * A vector of model metadata matching the filter
    pub fn find_models(&self, filter: &ModelFilter) -> Vec<ModelMetadata> {
        debug!("API: Finding models with filter");
        self.registry.find_models(filter)
    }

    /// Find models by provider
    ///
    /// # Arguments
    /// * `provider` - The provider to filter by
    ///
    /// # Returns
    /// * A vector of model metadata from the specified provider
    pub fn find_by_provider(&self, provider: &str) -> Vec<ModelMetadata> {
        debug!("API: Finding models by provider: {}", provider);
        self.registry.find_by_provider(provider)
    }

    /// Find models by type
    ///
    /// # Arguments
    /// * `model_type` - The model type to filter by
    ///
    /// # Returns
    /// * A vector of model metadata of the specified type
    pub fn find_by_type(&self, model_type: ModelType) -> Vec<ModelMetadata> {
        debug!("API: Finding models by type: {:?}", model_type);
        self.registry.find_by_type(model_type)
    }

    /// Find available models
    ///
    /// # Returns
    /// * A vector of model metadata for models with status Available
    pub fn find_available_models(&self) -> Vec<ModelMetadata> {
        debug!("API: Finding available models");
        self.registry.find_available_models()
    }

    /// Update model status
    ///
    /// # Arguments
    /// * `id` - The ID of the model to update
    /// * `status` - The new status
    ///
    /// # Returns
    /// * `Ok(())` if the status was updated successfully
    /// * `Err(RegistryError)` if the status could not be updated
    pub fn update_model_status(&self, id: &str, status: ModelStatus) -> Result<(), RegistryError> {
        debug!("API: Updating status for model {}: {:?}", id, status);
        self.registry.update_model_status(id, status)
    }

    /// Count the number of models in the registry
    ///
    /// # Returns
    /// * The number of models in the registry
    pub fn count(&self) -> usize {
        debug!("API: Getting model count");
        self.registry.count()
    }

    /// Check if the registry is empty
    ///
    /// # Returns
    /// * `true` if the registry is empty, `false` otherwise
    pub fn is_empty(&self) -> bool {
        debug!("API: Checking if registry is empty");
        self.registry.is_empty()
    }

    /// Clear all models from the registry
    pub fn clear(&self) {
        debug!("API: Clearing all models from registry");
        self.registry.clear()
    }

    /// Validate model metadata
    ///
    /// # Arguments
    /// * `metadata` - The model metadata to validate
    ///
    /// # Returns
    /// * `Ok(())` if the metadata is valid
    /// * `Err(RegistryError)` if the metadata is invalid
    fn validate_model_metadata(&self, metadata: &ModelMetadata) -> Result<(), RegistryError> {
        // Check required fields
        if metadata.id.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "Model ID cannot be empty".to_string(),
            ));
        }

        if metadata.name.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "Model name cannot be empty".to_string(),
            ));
        }

        if metadata.provider.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "Model provider cannot be empty".to_string(),
            ));
        }

        if metadata.version.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "Model version cannot be empty".to_string(),
            ));
        }

        if metadata.endpoint.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "Model endpoint cannot be empty".to_string(),
            ));
        }

        // Validate capabilities
        if metadata.capabilities.max_context_length == 0 {
            return Err(RegistryError::InvalidMetadata(
                "Max context length must be greater than 0".to_string(),
            ));
        }

        if metadata.capabilities.max_tokens_to_generate == 0 {
            return Err(RegistryError::InvalidMetadata(
                "Max tokens to generate must be greater than 0".to_string(),
            ));
        }

        // Validate input/output formats
        if metadata.capabilities.supported_input_formats.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "At least one input format must be supported".to_string(),
            ));
        }

        if metadata.capabilities.supported_output_formats.is_empty() {
            return Err(RegistryError::InvalidMetadata(
                "At least one output format must be supported".to_string(),
            ));
        }

        // Validate version info
        if metadata.capabilities.version_info.major == 0
            && metadata.capabilities.version_info.minor == 0
            && metadata.capabilities.version_info.patch == 0
        {
            return Err(RegistryError::InvalidMetadata(
                "Version information must be provided".to_string(),
            ));
        }

        // All validations passed
        Ok(())
    }
}

impl Default for ModelRegistryApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a new Model Registry API instance
pub fn create_model_registry_api() -> ModelRegistryApi {
    ModelRegistryApi::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::types::{ModelMetadata, ModelStatus};

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
    fn test_api_crud_operations() {
        let api = ModelRegistryApi::new();
        assert!(api.is_empty());

        // Register a model
        let model = create_test_model("gpt-4", "openai");
        api.register_model(model.clone()).unwrap();
        assert_eq!(api.count(), 1);

        // Get the model
        let retrieved = api.get_model("gpt-4").unwrap();
        assert_eq!(retrieved.id, "gpt-4");
        assert_eq!(retrieved.provider, "openai");

        // Update the model
        let mut updated = model.clone();
        updated.set_description("Updated model".to_string());
        api.update_model(updated.clone()).unwrap();

        let retrieved = api.get_model("gpt-4").unwrap();
        assert_eq!(retrieved.description, Some("Updated model".to_string()));

        // Remove the model
        let removed = api.remove_model("gpt-4").unwrap();
        assert_eq!(removed.id, "gpt-4");
        assert!(api.is_empty());
    }

    #[test]
    fn test_api_validation() {
        let api = ModelRegistryApi::new();

        // Test with invalid model (empty ID)
        let mut invalid_model = create_test_model("", "openai");
        let result = api.register_model(invalid_model.clone());
        assert!(result.is_err());
        match result {
            Err(RegistryError::InvalidMetadata(msg)) => {
                assert!(msg.contains("ID cannot be empty"));
            }
            _ => panic!("Expected InvalidMetadata error"),
        }

        // Test with invalid model (empty name)
        invalid_model = create_test_model("gpt-4", "openai");
        invalid_model.name = "".to_string();
        let result = api.register_model(invalid_model.clone());
        assert!(result.is_err());
        match result {
            Err(RegistryError::InvalidMetadata(msg)) => {
                assert!(msg.contains("name cannot be empty"));
            }
            _ => panic!("Expected InvalidMetadata error"),
        }

        // Test with invalid capabilities (max_context_length = 0)
        invalid_model = create_test_model("gpt-4", "openai");
        invalid_model.capabilities.max_context_length = 0;
        let result = api.register_model(invalid_model.clone());
        assert!(result.is_err());
        match result {
            Err(RegistryError::InvalidMetadata(msg)) => {
                assert!(msg.contains("Max context length"));
            }
            _ => panic!("Expected InvalidMetadata error"),
        }
    }

    #[test]
    fn test_api_filtering() {
        let api = ModelRegistryApi::new();

        // Register multiple models
        api.register_model(create_test_model("gpt-4", "openai"))
            .unwrap();
        api.register_model(create_test_model("gpt-3.5", "openai"))
            .unwrap();
        api.register_model(create_test_model("claude-2", "anthropic"))
            .unwrap();

        // Get all models
        let all_models = api.list_models();
        assert_eq!(all_models.len(), 3);

        // Filter by provider
        let openai_models = api.find_by_provider("openai");
        assert_eq!(openai_models.len(), 2);

        // Update model status
        api.update_model_status("gpt-4", ModelStatus::Available)
            .unwrap();
        api.update_model_status("claude-2", ModelStatus::Available)
            .unwrap();

        // Filter by status
        let available_models = api.find_available_models();
        assert_eq!(available_models.len(), 2);

        // Complex filter
        let mut gpt4 = api.get_model("gpt-4").unwrap();
        gpt4.capabilities.supports_function_calling = true;
        gpt4.capabilities.max_context_length = 8192;
        api.update_model(gpt4).unwrap();

        let filter = ModelFilter::new()
            .with_provider("openai".to_string())
            .with_status(ModelStatus::Available)
            .with_function_calling(true);

        let filtered = api.find_models(&filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "gpt-4");
    }

    #[test]
    fn test_with_registry() {
        // Create a registry and add a model
        let registry = Arc::new(ModelRegistry::new());
        registry
            .register_model(create_test_model("gpt-4", "openai"))
            .unwrap();

        // Create API with existing registry
        let api = ModelRegistryApi::with_registry(registry);

        // Verify the model exists
        let model = api.get_model("gpt-4").unwrap();
        assert_eq!(model.id, "gpt-4");

        // Add another model through the API
        api.register_model(create_test_model("claude-2", "anthropic"))
            .unwrap();

        // Verify both models exist
        assert_eq!(api.count(), 2);
        assert!(api.get_model("gpt-4").is_ok());
        assert!(api.get_model("claude-2").is_ok());
    }
}
