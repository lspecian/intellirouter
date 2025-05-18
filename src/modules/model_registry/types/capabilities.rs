//! Model capabilities types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::model_registry::types::formats::{InputFormat, OutputFormat};
use crate::modules::model_registry::types::performance::ModelPerformance;
use crate::modules::model_registry::types::version::ModelVersionInfo;

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
