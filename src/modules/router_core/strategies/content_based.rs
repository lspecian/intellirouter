//! Content-Based Routing Strategy
//!
//! This module implements a content-based routing strategy that analyzes
//! the content of requests and routes them to the most appropriate model
//! based on the content characteristics.

use std::time::Instant;

use async_trait::async_trait;
use tracing::{debug, info};

use crate::modules::model_registry::{ModelMetadata, ModelRegistry};
use crate::modules::router_core::config::StrategyConfig;
use crate::modules::router_core::errors::RouterError;
use crate::modules::router_core::request::RoutingRequest;
use crate::modules::router_core::response::RoutingMetadata;
use crate::modules::router_core::strategies::BaseStrategy;
use crate::modules::router_core::{RoutingStrategy, RoutingStrategyTrait};

/// Configuration for content-based routing
#[derive(Debug, Clone)]
pub struct ContentBasedConfig {
    /// Threshold for technical content detection
    pub technical_threshold: f32,
    /// Threshold for creative content detection
    pub creative_threshold: f32,
    /// Threshold for code content detection
    pub code_threshold: f32,
}

impl Default for ContentBasedConfig {
    fn default() -> Self {
        Self {
            technical_threshold: 0.7,
            creative_threshold: 0.7,
            code_threshold: 0.7,
        }
    }
}

/// Content-based routing strategy
#[derive(Debug)]
pub struct ContentBasedStrategy {
    /// Base strategy implementation
    base: BaseStrategy,
    /// Content-based specific configuration
    config: ContentBasedConfig,
}

impl ContentBasedStrategy {
    /// Create a new content-based routing strategy
    pub fn new(config: StrategyConfig, content_config: ContentBasedConfig) -> Self {
        Self {
            base: BaseStrategy::new("content_based", RoutingStrategy::ContentBased, config),
            config: content_config,
        }
    }

    /// Analyze the content of a request to determine its characteristics
    fn analyze_content(&self, request: &RoutingRequest) -> ContentCharacteristics {
        let mut characteristics = ContentCharacteristics::default();

        // Get the content from the request messages
        let content = request
            .context
            .request
            .messages
            .iter()
            .map(|msg| msg.content.as_str())
            .collect::<Vec<&str>>()
            .join(" ")
            .to_lowercase();

        // Simple keyword-based analysis for demonstration purposes
        // In a real implementation, this would use more sophisticated NLP techniques

        // Technical content detection
        let technical_keywords = [
            "quantum",
            "algorithm",
            "physics",
            "mathematics",
            "theory",
            "engineering",
            "scientific",
            "analysis",
            "research",
            "technical",
        ];

        let technical_score = self.calculate_keyword_score(&content, &technical_keywords);
        characteristics.technical_score = technical_score;

        // Creative content detection
        let creative_keywords = [
            "poem",
            "story",
            "creative",
            "imagine",
            "art",
            "write",
            "novel",
            "fiction",
            "poetry",
            "narrative",
        ];

        let creative_score = self.calculate_keyword_score(&content, &creative_keywords);
        characteristics.creative_score = creative_score;

        // Code content detection
        let code_keywords = [
            "function",
            "code",
            "algorithm",
            "programming",
            "implementation",
            "class",
            "method",
            "variable",
            "rust",
            "python",
            "javascript",
        ];

        let code_score = self.calculate_keyword_score(&content, &code_keywords);
        characteristics.code_score = code_score;

        debug!(
            "Content analysis: technical={:.2}, creative={:.2}, code={:.2}",
            characteristics.technical_score,
            characteristics.creative_score,
            characteristics.code_score
        );

        characteristics
    }

    /// Calculate a score based on keyword presence
    fn calculate_keyword_score(&self, content: &str, keywords: &[&str]) -> f32 {
        let mut matches = 0;

        for keyword in keywords {
            if content.contains(keyword) {
                matches += 1;
            }
        }

        // Normalize score between 0 and 1
        matches as f32 / keywords.len() as f32
    }

    /// Select the most appropriate model based on content characteristics
    async fn select_model_by_content(
        &self,
        request: &RoutingRequest,
        models: &[ModelMetadata],
    ) -> Option<ModelMetadata> {
        // Analyze the content
        let characteristics = self.analyze_content(request);

        // Determine the primary content type
        let content_type = characteristics.primary_content_type();

        // Find models that are good at handling this content type
        let suitable_models = models
            .iter()
            .filter(|model| {
                match content_type {
                    ContentType::Technical => {
                        // Look for models with technical capabilities
                        model
                            .capabilities
                            .additional_capabilities
                            .contains_key("technical")
                            || model
                                .capabilities
                                .additional_capabilities
                                .contains_key("scientific")
                            || model.additional_metadata.contains_key("technical")
                            || model.additional_metadata.contains_key("scientific")
                    }
                    ContentType::Creative => {
                        // Look for models with creative capabilities
                        model
                            .capabilities
                            .additional_capabilities
                            .contains_key("creative")
                            || model
                                .capabilities
                                .additional_capabilities
                                .contains_key("generative")
                            || model.additional_metadata.contains_key("creative")
                            || model.additional_metadata.contains_key("generative")
                    }
                    ContentType::Code => {
                        // Look for models with coding capabilities
                        model
                            .capabilities
                            .additional_capabilities
                            .contains_key("code")
                            || model
                                .capabilities
                                .additional_capabilities
                                .contains_key("programming")
                            || model.additional_metadata.contains_key("code")
                            || model.additional_metadata.contains_key("programming")
                    }
                    ContentType::General => true, // Any model is suitable for general content
                }
            })
            .cloned()
            .collect::<Vec<ModelMetadata>>();

        // If we found suitable models, return the first one
        // In a real implementation, we might apply additional criteria
        if !suitable_models.is_empty() {
            info!(
                "Selected model {} based on content type {:?}",
                suitable_models[0].id, content_type
            );
            return Some(suitable_models[0].clone());
        }

        // If no suitable model was found, return None
        None
    }
}

#[async_trait]
impl RoutingStrategyTrait for ContentBasedStrategy {
    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn strategy_type(&self) -> RoutingStrategy {
        self.base.strategy_type()
    }

    async fn select_model(
        &self,
        request: &RoutingRequest,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError> {
        debug!("Selecting model using content-based strategy");

        // Filter models based on request criteria
        let models = self.base.filter_models(request, registry).await?;

        // Select model based on content analysis
        if let Some(model) = self.select_model_by_content(request, &models).await {
            info!("Selected model {} based on content analysis", model.id);
            return Ok(model);
        }

        // Fall back to base strategy if content analysis doesn't yield a result
        self.base.select_model(request, registry).await
    }

    async fn handle_failure(
        &self,
        request: &RoutingRequest,
        failed_model_id: &str,
        error: &RouterError,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError> {
        self.base
            .handle_failure(request, failed_model_id, error, registry)
            .await
    }

    fn get_routing_metadata(
        &self,
        model: &ModelMetadata,
        start_time: Instant,
        attempts: u32,
        is_fallback: bool,
    ) -> RoutingMetadata {
        let mut metadata = self
            .base
            .get_routing_metadata(model, start_time, attempts, is_fallback);

        // Add content-based specific metadata
        metadata
            .additional_metadata
            .insert("strategy_type".to_string(), "content_based".to_string());

        metadata
    }
}

/// Content characteristics
#[derive(Debug, Default, Clone)]
struct ContentCharacteristics {
    /// Score for technical content (0.0 - 1.0)
    technical_score: f32,
    /// Score for creative content (0.0 - 1.0)
    creative_score: f32,
    /// Score for code content (0.0 - 1.0)
    code_score: f32,
}

impl ContentCharacteristics {
    /// Determine the primary content type based on scores
    fn primary_content_type(&self) -> ContentType {
        if self.technical_score > self.creative_score && self.technical_score > self.code_score {
            ContentType::Technical
        } else if self.creative_score > self.technical_score
            && self.creative_score > self.code_score
        {
            ContentType::Creative
        } else if self.code_score > self.technical_score && self.code_score > self.creative_score {
            ContentType::Code
        } else {
            ContentType::General
        }
    }
}

/// Content types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentType {
    /// Technical content (science, math, etc.)
    Technical,
    /// Creative content (writing, art, etc.)
    Creative,
    /// Code content (programming, algorithms, etc.)
    Code,
    /// General content (no specific type)
    General,
}

#[cfg(all(test, not(feature = "production")))]
mod tests {
    use super::*;
    use crate::modules::model_registry::connectors::{
        ChatCompletionRequest, ChatMessage, MessageRole,
    };
    use crate::modules::model_registry::ModelCapabilities;
    use crate::test_utils::mocks::MockModelRegistry;
    use std::time::Duration;

    fn create_test_request(content: &str) -> RoutingRequest {
        let chat_request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: content.to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            }],
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        let mut request = RoutingRequest::new(chat_request);
        request.timeout = Duration::from_secs(10);
        request
    }

    fn create_test_model(id: &str, provider: &str, tags: Vec<&str>) -> ModelMetadata {
        let mut model = ModelMetadata::new(
            id.to_string(),
            format!("{} Model", id),
            provider.to_string(),
            "1.0".to_string(),
            "https://example.com".to_string(),
        );

        // Add capabilities for content specialization
        for tag in tags {
            model
                .capabilities
                .additional_capabilities
                .insert(tag.to_string(), "true".to_string());
        }

        model
    }

    #[test]
    fn test_content_analysis() {
        let config = StrategyConfig::default();
        let content_config = ContentBasedConfig::default();
        let strategy = ContentBasedStrategy::new(config, content_config);

        // Test technical content
        let technical_request = create_test_request(
            "Explain the principles of quantum computing and how qubits work in detail.",
        );
        let technical_characteristics = strategy.analyze_content(&technical_request);
        assert!(technical_characteristics.technical_score > 0.0);

        // Test creative content
        let creative_request =
            create_test_request("Write a short poem about a sunset over the ocean.");
        let creative_characteristics = strategy.analyze_content(&creative_request);
        assert!(creative_characteristics.creative_score > 0.0);

        // Test code content
        let code_request = create_test_request(
            "Write a function that implements a binary search algorithm in Rust.",
        );
        let code_characteristics = strategy.analyze_content(&code_request);
        assert!(code_characteristics.code_score > 0.0);
    }

    #[test]
    fn test_primary_content_type() {
        // Test technical content
        let technical = ContentCharacteristics {
            technical_score: 0.8,
            creative_score: 0.2,
            code_score: 0.3,
        };
        assert_eq!(technical.primary_content_type(), ContentType::Technical);

        // Test creative content
        let creative = ContentCharacteristics {
            technical_score: 0.3,
            creative_score: 0.9,
            code_score: 0.1,
        };
        assert_eq!(creative.primary_content_type(), ContentType::Creative);

        // Test code content
        let code = ContentCharacteristics {
            technical_score: 0.4,
            creative_score: 0.3,
            code_score: 0.7,
        };
        assert_eq!(code.primary_content_type(), ContentType::Code);

        // Test general content
        let general = ContentCharacteristics {
            technical_score: 0.5,
            creative_score: 0.5,
            code_score: 0.5,
        };
        assert_eq!(general.primary_content_type(), ContentType::General);
    }
}
