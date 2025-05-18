//! Model filtering types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::model_registry::types::formats::{InputFormat, OutputFormat};
use crate::modules::model_registry::types::model::ModelType;
use crate::modules::model_registry::types::status::ModelStatus;
use crate::modules::model_registry::types::version::ModelVersionInfo;

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

    /// Set required feature filter
    pub fn with_required_feature(mut self, feature: String, enabled: bool) -> Self {
        self.required_features.insert(feature, enabled);
        self
    }

    /// Set additional filter
    pub fn with_additional_filter(mut self, key: String, value: String) -> Self {
        self.additional_filters.insert(key, value);
        self
    }
}
