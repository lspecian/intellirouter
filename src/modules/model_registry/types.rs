//! Model Registry Types
//!
//! This module defines the core data structures for the Model Registry.
//! These types are used to store and manage metadata about LLM models,
//! their capabilities, status, and other relevant information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Error types for the model registry
#[derive(Debug, Clone)]
pub enum RegistryError {
    /// Model already exists in the registry
    AlreadyExists(String),
    /// Model not found in the registry
    NotFound(String),
    /// Invalid model metadata
    InvalidMetadata(String),
    /// Error communicating with the model
    CommunicationError(String),
    /// Storage-related error
    StorageError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::AlreadyExists(msg) => write!(f, "Model already exists: {}", msg),
            RegistryError::NotFound(msg) => write!(f, "Model not found: {}", msg),
            RegistryError::InvalidMetadata(msg) => write!(f, "Invalid model metadata: {}", msg),
            RegistryError::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            RegistryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            RegistryError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Status of a model in the registry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model is available and ready to use
    Available,
    /// Model is unavailable (e.g., service down)
    Unavailable,
    /// Model is available but with limitations (e.g., rate limited)
    Limited,
    /// Model is in maintenance mode
    Maintenance,
    /// Model is deprecated and will be removed in the future
    Deprecated,
    /// Model is in an unknown state
    Unknown,
}

impl Default for ModelStatus {
    fn default() -> Self {
        ModelStatus::Unknown
    }
}

impl fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelStatus::Available => write!(f, "Available"),
            ModelStatus::Unavailable => write!(f, "Unavailable"),
            ModelStatus::Limited => write!(f, "Limited"),
            ModelStatus::Maintenance => write!(f, "Maintenance"),
            ModelStatus::Deprecated => write!(f, "Deprecated"),
            ModelStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelVersionInfo {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Release date
    pub release_date: Option<chrono::DateTime<chrono::Utc>>,
    /// End of life date
    pub end_of_life_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Is this a preview/beta version
    pub is_preview: bool,
}

impl ModelVersionInfo {
    /// Create a new model version info from a version string (e.g., "1.0.0")
    pub fn from_version_string(version: &str) -> Result<Self, RegistryError> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(RegistryError::InvalidMetadata(format!(
                "Invalid version string: {}",
                version
            )));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            RegistryError::InvalidMetadata(format!("Invalid major version: {}", parts[0]))
        })?;

        let minor = parts[1].parse::<u32>().map_err(|_| {
            RegistryError::InvalidMetadata(format!("Invalid minor version: {}", parts[1]))
        })?;

        let patch = parts[2].parse::<u32>().map_err(|_| {
            RegistryError::InvalidMetadata(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(Self {
            major,
            minor,
            patch,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        })
    }

    /// Convert to a version string (e.g., "1.0.0")
    pub fn to_version_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Check if this version is newer than another version
    pub fn is_newer_than(&self, other: &Self) -> bool {
        if self.major > other.major {
            return true;
        }
        if self.major < other.major {
            return false;
        }
        if self.minor > other.minor {
            return true;
        }
        if self.minor < other.minor {
            return false;
        }
        self.patch > other.patch
    }
}

/// Model performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPerformance {
    /// Average latency in milliseconds
    pub avg_latency_ms: Option<f64>,
    /// P95 latency in milliseconds
    pub p95_latency_ms: Option<f64>,
    /// P99 latency in milliseconds
    pub p99_latency_ms: Option<f64>,
    /// Tokens per second for generation
    pub tokens_per_second: Option<f64>,
    /// Maximum requests per minute
    pub max_requests_per_minute: Option<u32>,
    /// Maximum tokens per minute
    pub max_tokens_per_minute: Option<u32>,
}

/// Input format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputFormat {
    /// Plain text
    Text,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
    /// Image (base64 encoded)
    Image,
    /// Audio (base64 encoded)
    Audio,
    /// Other format
    Other(String),
}

/// Output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text
    Text,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
    /// Image (base64 encoded)
    Image,
    /// Audio (base64 encoded)
    Audio,
    /// Other format
    Other(String),
}

/// Fine-tuning capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FineTuningCapabilities {
    /// Whether fine-tuning is supported
    pub supported: bool,
    /// Minimum number of examples required
    pub min_examples: Option<u32>,
    /// Maximum number of examples allowed
    pub max_examples: Option<u32>,
    /// Supported training methods
    pub training_methods: Vec<String>,
}

/// Rate limits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimits {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
    /// Concurrent requests
    pub concurrent_requests: Option<u32>,
}

/// Capabilities of a model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelCapabilities {
    /// Maximum context window size in tokens
    pub max_context_length: usize,
    /// Maximum number of tokens the model can generate
    pub max_tokens_to_generate: usize,
    /// Whether the model supports function calling
    pub supports_function_calling: bool,
    /// Whether the model supports vision/image inputs
    pub supports_vision: bool,
    /// Whether the model supports streaming responses
    pub supports_streaming: bool,
    /// Whether the model supports embeddings generation
    pub supports_embeddings: bool,
    /// Cost per 1K tokens for input (prompt)
    pub cost_per_1k_tokens_input: f64,
    /// Cost per 1K tokens for output (completion)
    pub cost_per_1k_tokens_output: f64,
    /// Supported languages (ISO 639-1 codes)
    pub supported_languages: Vec<String>,
    /// Model version information
    pub version_info: ModelVersionInfo,
    /// Feature flags for specific capabilities
    pub feature_flags: HashMap<String, bool>,
    /// Performance characteristics
    pub performance: ModelPerformance,
    /// Supported input formats
    pub supported_input_formats: Vec<InputFormat>,
    /// Supported output formats
    pub supported_output_formats: Vec<OutputFormat>,
    /// Fine-tuning capabilities
    pub fine_tuning: Option<FineTuningCapabilities>,
    /// Rate limiting information
    pub rate_limits: Option<RateLimits>,
    /// Additional capabilities as key-value pairs
    pub additional_capabilities: HashMap<String, String>,
}

impl ModelCapabilities {
    /// Check if this model supports a specific feature
    pub fn supports_feature(&self, feature: &str) -> bool {
        if let Some(value) = self.feature_flags.get(feature) {
            return *value;
        }

        match feature {
            "function_calling" => self.supports_function_calling,
            "vision" => self.supports_vision,
            "streaming" => self.supports_streaming,
            "embeddings" => self.supports_embeddings,
            _ => false,
        }
    }

    /// Check if this model supports a specific input format
    pub fn supports_input_format(&self, format: &InputFormat) -> bool {
        self.supported_input_formats.contains(format)
    }

    /// Check if this model supports a specific output format
    pub fn supports_output_format(&self, format: &OutputFormat) -> bool {
        self.supported_output_formats.contains(format)
    }

    /// Check if this model supports a specific language
    pub fn supports_language(&self, language: &str) -> bool {
        self.supported_languages.contains(&language.to_string())
    }

    /// Check if this model has sufficient context length
    pub fn has_sufficient_context_length(&self, required_length: usize) -> bool {
        self.max_context_length >= required_length
    }

    /// Calculate the cost for a specific number of input and output tokens
    pub fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.cost_per_1k_tokens_input;
        let output_cost = (output_tokens as f64 / 1000.0) * self.cost_per_1k_tokens_output;
        input_cost + output_cost
    }

    /// Add a feature flag
    pub fn add_feature_flag(&mut self, feature: String, enabled: bool) {
        self.feature_flags.insert(feature, enabled);
    }

    /// Add an input format
    pub fn add_input_format(&mut self, format: InputFormat) {
        if !self.supported_input_formats.contains(&format) {
            self.supported_input_formats.push(format);
        }
    }

    /// Add an output format
    pub fn add_output_format(&mut self, format: OutputFormat) {
        if !self.supported_output_formats.contains(&format) {
            self.supported_output_formats.push(format);
        }
    }

    /// Add a supported language
    pub fn add_supported_language(&mut self, language: String) {
        if !self.supported_languages.contains(&language) {
            self.supported_languages.push(language);
        }
    }
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            max_context_length: 4096,
            max_tokens_to_generate: 2048,
            supports_function_calling: false,
            supports_vision: false,
            supports_streaming: true,
            supports_embeddings: false,
            cost_per_1k_tokens_input: 0.0,
            cost_per_1k_tokens_output: 0.0,
            supported_languages: vec!["en".to_string()],
            version_info: ModelVersionInfo {
                major: 1,
                minor: 0,
                patch: 0,
                release_date: None,
                end_of_life_date: None,
                is_preview: false,
            },
            feature_flags: HashMap::new(),
            performance: ModelPerformance {
                avg_latency_ms: None,
                p95_latency_ms: None,
                p99_latency_ms: None,
                tokens_per_second: None,
                max_requests_per_minute: None,
                max_tokens_per_minute: None,
            },
            supported_input_formats: vec![InputFormat::Text],
            supported_output_formats: vec![OutputFormat::Text],
            fine_tuning: None,
            rate_limits: None,
            additional_capabilities: HashMap::new(),
        }
    }
}

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

/// Filter criteria for querying models
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModelFilter {
    /// Filter by provider
    pub provider: Option<String>,
    /// Filter by model type
    pub model_type: Option<ModelType>,
    /// Filter by status
    pub status: Option<ModelStatus>,
    /// Filter by minimum context length
    pub min_context_length: Option<usize>,
    /// Filter by function calling support
    pub supports_function_calling: Option<bool>,
    /// Filter by vision support
    pub supports_vision: Option<bool>,
    /// Filter by streaming support
    pub supports_streaming: Option<bool>,
    /// Filter by embedding support
    pub supports_embeddings: Option<bool>,
    /// Filter by supported language
    pub language: Option<String>,
    /// Filter by input format
    pub input_format: Option<InputFormat>,
    /// Filter by output format
    pub output_format: Option<OutputFormat>,
    /// Filter by minimum version
    pub min_version: Option<ModelVersionInfo>,
    /// Filter by maximum cost per 1K tokens (input)
    pub max_cost_per_1k_tokens_input: Option<f64>,
    /// Filter by maximum cost per 1K tokens (output)
    pub max_cost_per_1k_tokens_output: Option<f64>,
    /// Filter by maximum latency (ms)
    pub max_latency_ms: Option<f64>,
    /// Filter by feature flags
    pub required_features: HashMap<String, bool>,
    /// Additional filter criteria as key-value pairs
    pub additional_filters: HashMap<String, String>,
}

impl ModelFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set provider filter
    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set model type filter
    pub fn with_model_type(mut self, model_type: ModelType) -> Self {
        self.model_type = Some(model_type);
        self
    }

    /// Set status filter
    pub fn with_status(mut self, status: ModelStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set minimum context length filter
    pub fn with_min_context_length(mut self, min_context_length: usize) -> Self {
        self.min_context_length = Some(min_context_length);
        self
    }

    /// Set function calling support filter
    pub fn with_function_calling(mut self, supports: bool) -> Self {
        self.supports_function_calling = Some(supports);
        self
    }

    /// Set vision support filter
    pub fn with_vision(mut self, supports: bool) -> Self {
        self.supports_vision = Some(supports);
        self
    }

    /// Set streaming support filter
    pub fn with_streaming(mut self, supports: bool) -> Self {
        self.supports_streaming = Some(supports);
        self
    }

    /// Set embeddings support filter
    pub fn with_embeddings(mut self, supports: bool) -> Self {
        self.supports_embeddings = Some(supports);
        self
    }

    /// Set language filter
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    /// Set input format filter
    pub fn with_input_format(mut self, format: InputFormat) -> Self {
        self.input_format = Some(format);
        self
    }

    /// Set output format filter
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = Some(format);
        self
    }

    /// Set minimum version filter
    pub fn with_min_version(mut self, version: ModelVersionInfo) -> Self {
        self.min_version = Some(version);
        self
    }

    /// Set maximum cost per 1K tokens (input) filter
    pub fn with_max_cost_per_1k_tokens_input(mut self, cost: f64) -> Self {
        self.max_cost_per_1k_tokens_input = Some(cost);
        self
    }

    /// Set maximum cost per 1K tokens (output) filter
    pub fn with_max_cost_per_1k_tokens_output(mut self, cost: f64) -> Self {
        self.max_cost_per_1k_tokens_output = Some(cost);
        self
    }

    /// Set maximum latency filter
    pub fn with_max_latency_ms(mut self, latency: f64) -> Self {
        self.max_latency_ms = Some(latency);
        self
    }

    /// Add required feature
    pub fn with_required_feature(mut self, feature: String, enabled: bool) -> Self {
        self.required_features.insert(feature, enabled);
        self
    }

    /// Add additional filter criterion
    pub fn with_additional_filter(mut self, key: String, value: String) -> Self {
        self.additional_filters.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_version_info() {
        let version = ModelVersionInfo {
            major: 1,
            minor: 2,
            patch: 3,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };

        assert_eq!(version.to_version_string(), "1.2.3");

        let version2 = ModelVersionInfo::from_version_string("1.2.3").unwrap();
        assert_eq!(version, version2);

        let version3 = ModelVersionInfo {
            major: 1,
            minor: 2,
            patch: 4,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };

        assert!(version3.is_newer_than(&version));
        assert!(!version.is_newer_than(&version3));

        // Test invalid version string
        let result = ModelVersionInfo::from_version_string("1.2");
        assert!(result.is_err());

        let result = ModelVersionInfo::from_version_string("1.x.3");
        assert!(result.is_err());
    }

    #[test]
    fn test_model_capabilities_features() {
        let mut capabilities = ModelCapabilities::default();

        // Test default values
        assert!(!capabilities.supports_feature("function_calling"));
        assert!(!capabilities.supports_feature("vision"));
        assert!(capabilities.supports_feature("streaming"));
        assert!(!capabilities.supports_feature("embeddings"));
        assert!(!capabilities.supports_feature("unknown_feature"));

        // Test setting features
        capabilities.supports_function_calling = true;
        assert!(capabilities.supports_feature("function_calling"));

        // Test feature flags
        capabilities.add_feature_flag("custom_feature".to_string(), true);
        assert!(capabilities.supports_feature("custom_feature"));

        capabilities.add_feature_flag("another_feature".to_string(), false);
        assert!(!capabilities.supports_feature("another_feature"));
    }

    #[test]
    fn test_model_capabilities_formats() {
        let mut capabilities = ModelCapabilities::default();

        // Test default values
        assert!(capabilities.supports_input_format(&InputFormat::Text));
        assert!(!capabilities.supports_input_format(&InputFormat::Image));

        // Test adding formats
        capabilities.add_input_format(InputFormat::Image);
        assert!(capabilities.supports_input_format(&InputFormat::Image));

        capabilities.add_output_format(OutputFormat::Json);
        assert!(capabilities.supports_output_format(&OutputFormat::Json));

        // Test adding duplicate formats (should not duplicate)
        let original_count = capabilities.supported_input_formats.len();
        capabilities.add_input_format(InputFormat::Image);
        assert_eq!(capabilities.supported_input_formats.len(), original_count);
    }

    #[test]
    fn test_model_capabilities_languages() {
        let mut capabilities = ModelCapabilities::default();

        // Test default values
        assert!(capabilities.supports_language("en"));
        assert!(!capabilities.supports_language("fr"));

        // Test adding languages
        capabilities.add_supported_language("fr".to_string());
        assert!(capabilities.supports_language("fr"));

        // Test adding duplicate languages (should not duplicate)
        let original_count = capabilities.supported_languages.len();
        capabilities.add_supported_language("fr".to_string());
        assert_eq!(capabilities.supported_languages.len(), original_count);
    }

    #[test]
    fn test_model_capabilities_cost() {
        let mut capabilities = ModelCapabilities::default();
        capabilities.cost_per_1k_tokens_input = 0.01;
        capabilities.cost_per_1k_tokens_output = 0.02;

        // Test cost calculation
        let cost = capabilities.calculate_cost(1000, 500);
        assert_eq!(cost, 0.01 + 0.01); // 1000 input tokens + 500 output tokens

        let cost = capabilities.calculate_cost(2000, 1000);
        assert_eq!(cost, 0.02 + 0.02); // 2000 input tokens + 1000 output tokens
    }

    #[test]
    fn test_model_filter_with_capabilities() {
        let filter = ModelFilter::new()
            .with_input_format(InputFormat::Image)
            .with_output_format(OutputFormat::Text)
            .with_max_cost_per_1k_tokens_input(0.02)
            .with_max_latency_ms(100.0)
            .with_required_feature("function_calling".to_string(), true);

        assert_eq!(filter.input_format, Some(InputFormat::Image));
        assert_eq!(filter.output_format, Some(OutputFormat::Text));
        assert_eq!(filter.max_cost_per_1k_tokens_input, Some(0.02));
        assert_eq!(filter.max_latency_ms, Some(100.0));
        assert_eq!(
            filter.required_features.get("function_calling"),
            Some(&true)
        );

        // Test with version
        let version = ModelVersionInfo {
            major: 1,
            minor: 2,
            patch: 3,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        };

        let filter = filter.with_min_version(version.clone());
        assert_eq!(filter.min_version, Some(version));
    }

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata::new(
            "gpt-4".to_string(),
            "GPT-4".to_string(),
            "openai".to_string(),
            "1.0".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
        );

        assert_eq!(metadata.id, "gpt-4");
        assert_eq!(metadata.name, "GPT-4");
        assert_eq!(metadata.provider, "openai");
        assert_eq!(metadata.version, "1.0");
        assert_eq!(
            metadata.endpoint,
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(metadata.status, ModelStatus::Unknown);
        assert_eq!(metadata.model_type, ModelType::TextGeneration);
        assert!(metadata.description.is_none());
        assert!(metadata.auth_key.is_none());
        assert!(metadata.last_checked.is_none());
    }

    #[test]
    fn test_model_status_updates() {
        let mut metadata = ModelMetadata::new(
            "gpt-4".to_string(),
            "GPT-4".to_string(),
            "openai".to_string(),
            "1.0".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
        );

        assert!(!metadata.is_available());

        metadata.set_status(ModelStatus::Available);
        assert!(metadata.is_available());
        assert!(metadata.last_checked.is_some());

        metadata.set_status(ModelStatus::Deprecated);
        assert!(metadata.is_deprecated());
        assert!(!metadata.is_available());
    }

    #[test]
    fn test_model_metadata_updates() {
        let mut metadata = ModelMetadata::new(
            "gpt-4".to_string(),
            "GPT-4".to_string(),
            "openai".to_string(),
            "1.0".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
        );

        let original_updated_at = metadata.updated_at;

        // Wait a moment to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(5));

        metadata.set_description("Advanced language model".to_string());
        assert_eq!(
            metadata.description,
            Some("Advanced language model".to_string())
        );
        assert!(metadata.updated_at > original_updated_at);

        let previous_updated_at = metadata.updated_at;

        // Wait a moment to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(5));

        metadata.add_metadata("release_date".to_string(), "2023-03-14".to_string());
        assert_eq!(
            metadata.additional_metadata.get("release_date"),
            Some(&"2023-03-14".to_string())
        );
        assert!(metadata.updated_at > previous_updated_at);
    }

    #[test]
    fn test_model_capabilities() {
        let mut metadata = ModelMetadata::new(
            "gpt-4".to_string(),
            "GPT-4".to_string(),
            "openai".to_string(),
            "1.0".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
        );

        assert_eq!(metadata.capabilities.max_context_length, 4096);
        assert_eq!(metadata.capabilities.max_tokens_to_generate, 2048);
        assert!(!metadata.capabilities.supports_function_calling);

        metadata.capabilities.max_context_length = 8192;
        metadata.capabilities.supports_function_calling = true;
        metadata.capabilities.cost_per_1k_tokens_input = 0.03;
        metadata.capabilities.cost_per_1k_tokens_output = 0.06;

        assert_eq!(metadata.capabilities.max_context_length, 8192);
        assert!(metadata.capabilities.supports_function_calling);
        assert_eq!(metadata.capabilities.cost_per_1k_tokens_input, 0.03);
        assert_eq!(metadata.capabilities.cost_per_1k_tokens_output, 0.06);
    }

    #[test]
    fn test_model_filter_builder() {
        let filter = ModelFilter::new()
            .with_provider("openai".to_string())
            .with_model_type(ModelType::TextGeneration)
            .with_status(ModelStatus::Available)
            .with_min_context_length(8192)
            .with_function_calling(true)
            .with_language("en".to_string());

        assert_eq!(filter.provider, Some("openai".to_string()));
        assert_eq!(filter.model_type, Some(ModelType::TextGeneration));
        assert_eq!(filter.status, Some(ModelStatus::Available));
        assert_eq!(filter.min_context_length, Some(8192));
        assert_eq!(filter.supports_function_calling, Some(true));
        assert_eq!(filter.language, Some("en".to_string()));
    }

    #[test]
    fn test_registry_error_display() {
        let error = RegistryError::NotFound("gpt-4".to_string());
        assert_eq!(error.to_string(), "Model not found: gpt-4");

        let error = RegistryError::AlreadyExists("gpt-4".to_string());
        assert_eq!(error.to_string(), "Model already exists: gpt-4");

        let error = RegistryError::InvalidMetadata("Missing required fields".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid model metadata: Missing required fields"
        );
    }

    #[test]
    fn test_model_status_display() {
        assert_eq!(ModelStatus::Available.to_string(), "Available");
        assert_eq!(ModelStatus::Unavailable.to_string(), "Unavailable");
        assert_eq!(ModelStatus::Limited.to_string(), "Limited");
        assert_eq!(ModelStatus::Maintenance.to_string(), "Maintenance");
        assert_eq!(ModelStatus::Deprecated.to_string(), "Deprecated");
        assert_eq!(ModelStatus::Unknown.to_string(), "Unknown");
    }
}
