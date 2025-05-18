//! Model metadata and type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::model_registry::types::capabilities::ModelCapabilities;
use crate::modules::model_registry::types::status::ModelStatus;

/// Model type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelType {
    /// Large language model for text generation
    TextGeneration,
    /// Model for embedding generation
    Embedding,
    /// Model for image generation
    ImageGeneration,
    /// Model for audio processing
    AudioProcessing,
    /// Multi-modal model supporting multiple input/output types
    MultiModal,
    /// Other specialized model types
    Other(String),
}

impl Default for ModelType {
    fn default() -> Self {
        ModelType::TextGeneration
    }
}

/// Metadata for a model in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Unique identifier for the model
    pub id: String,
    /// Display name for the model
    pub name: String,
    /// Provider of the model (e.g., "openai", "anthropic", "ollama")
    pub provider: String,
    /// Version of the model
    pub version: String,
    /// Type of the model
    pub model_type: ModelType,
    /// Description of the model
    pub description: Option<String>,
    /// Capabilities of the model
    pub capabilities: ModelCapabilities,
    /// Current status of the model
    pub status: ModelStatus,
    /// Endpoint URL for the model
    pub endpoint: String,
    /// Authentication key for the model (if applicable)
    #[serde(skip_serializing)]
    pub auth_key: Option<String>,
    /// Last time the model was checked for availability
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
    /// Creation date of this metadata entry
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update date of this metadata entry
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Additional metadata as key-value pairs
    pub additional_metadata: HashMap<String, String>,
}

impl ModelMetadata {
    /// Create a new model metadata instance with minimal required fields
    pub fn new(
        id: String,
        name: String,
        provider: String,
        version: String,
        endpoint: String,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            provider,
            version,
            model_type: ModelType::default(),
            description: None,
            capabilities: ModelCapabilities::default(),
            status: ModelStatus::Unknown,
            endpoint,
            auth_key: None,
            last_checked: None,
            created_at: now,
            updated_at: now,
            additional_metadata: HashMap::new(),
        }
    }

    /// Check if the model is available
    pub fn is_available(&self) -> bool {
        matches!(self.status, ModelStatus::Available)
    }

    /// Check if the model is deprecated
    pub fn is_deprecated(&self) -> bool {
        matches!(self.status, ModelStatus::Deprecated)
    }

    /// Set the model status and update the last_checked timestamp
    pub fn set_status(&mut self, status: ModelStatus) {
        self.status = status;
        self.last_checked = Some(chrono::Utc::now());
        self.updated_at = chrono::Utc::now();
    }

    /// Add or update additional metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.additional_metadata.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }

    /// Add or update additional capability
    pub fn add_capability(&mut self, key: String, value: String) {
        self.capabilities.additional_capabilities.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }

    /// Update the model description
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
        self.updated_at = chrono::Utc::now();
    }

    /// Update the model type
    pub fn set_model_type(&mut self, model_type: ModelType) {
        self.model_type = model_type;
        self.updated_at = chrono::Utc::now();
    }

    /// Update the authentication key
    pub fn set_auth_key(&mut self, auth_key: Option<String>) {
        self.auth_key = auth_key;
        self.updated_at = chrono::Utc::now();
    }
}
