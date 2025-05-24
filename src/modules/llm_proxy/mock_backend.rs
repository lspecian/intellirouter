//! Mock Model Backend
//!
//! This module provides a mock implementation of a model backend
//! for testing and development purposes.
//!
//! This module is only available when the `test-utils` feature is enabled.
#![cfg(feature = "test-utils")]

use async_trait::async_trait;
use chrono::Utc;
use futures::stream;
use std::sync::atomic::AtomicU64;
use tracing::debug;
use uuid::Uuid;

use crate::modules::model_registry::{
    connectors::{
        ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ModelConnector,
        StreamingResponse,
    },
    ConnectorConfig, ConnectorError,
};

/// Static counter for request IDs
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Mock model backend for testing and development
#[derive(Debug, Clone)]
pub struct MockModelBackend {
    /// Model ID
    model_id: String,
    /// Model name
    model_name: String,
    /// Provider name
    provider: String,
    /// Whether to simulate errors
    simulate_errors: bool,
    /// Simulated latency in milliseconds
    simulated_latency_ms: u64,
    /// Connector configuration
    config: ConnectorConfig,
}

impl MockModelBackend {
    /// Create a new mock model backend
    pub fn new(model_id: String, model_name: String, provider: String) -> Self {
        Self {
            model_id,
            model_name,
            provider,
            simulate_errors: false,
            simulated_latency_ms: 500,
            config: ConnectorConfig::default(),
        }
    }

    /// Set whether to simulate errors
    pub fn with_simulated_errors(mut self, simulate_errors: bool) -> Self {
        self.simulate_errors = simulate_errors;
        self
    }

    /// Set simulated latency
    pub fn with_simulated_latency(mut self, latency_ms: u64) -> Self {
        self.simulated_latency_ms = latency_ms;
        self
    }

    /// Generate a mock response based on the request
    fn generate_mock_response(&self, request: &ChatCompletionRequest) -> ChatCompletionResponse {
        // Extract the last user message to generate a contextual response
        let last_user_message = request
            .messages
            .iter()
            .filter(|m| m.role == crate::modules::model_registry::connectors::MessageRole::User)
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "Hello".to_string());

        // Generate a response based on the user's message
        let response_content = format!(
            "Hello! I'm a mock model ({}) from the IntelliRouter. You said: {}",
            self.model_id, last_user_message
        );

        // Create the response
        let id = format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", ""));
        let created = Utc::now().timestamp() as u64;
        let content_length = response_content.len() as u32;

        ChatCompletionResponse {
            id,
            model: self.model_id.clone(),
            created,
            choices: vec![
                crate::modules::model_registry::connectors::ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage {
                        role: crate::modules::model_registry::connectors::MessageRole::Assistant,
                        content: response_content,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                },
            ],
            usage: Some(crate::modules::model_registry::connectors::TokenUsage {
                prompt_tokens: 10,                     // Mock values
                completion_tokens: content_length / 4, // Rough approximation
                total_tokens: 10 + (content_length / 4),
            }),
        }
    }
    /// Generate mock streaming chunks for a request
    fn generate_mock_streaming_chunks(
        &self,
        request: &ChatCompletionRequest,
    ) -> Vec<crate::modules::model_registry::connectors::ChatCompletionChunk> {
        use crate::modules::model_registry::connectors::{
            ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta,
        };

        // Extract the last user message to generate a contextual response
        let last_user_message = request
            .messages
            .iter()
            .filter(|m| m.role == crate::modules::model_registry::connectors::MessageRole::User)
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "Hello".to_string());

        // Generate a response based on the user's message
        let response_content = format!(
            "Hello! I'm a mock model ({}) from the IntelliRouter. You said: {}",
            self.model_id, last_user_message
        );

        // Split the response into chunks
        let words: Vec<&str> = response_content.split_whitespace().collect();
        let chunk_size = 3; // Number of words per chunk
        let mut chunks = Vec::new();

        // Create chunks of words
        for i in (0..words.len()).step_by(chunk_size) {
            let end = std::cmp::min(i + chunk_size, words.len());
            let chunk = words[i..end].join(" ");

            // Create a proper ChatCompletionChunk
            let completion_chunk = ChatCompletionChunk {
                id: format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", "")),
                model: self.model_id.clone(),
                created: Utc::now().timestamp() as u64,
                choices: vec![ChatCompletionChunkChoice {
                    index: 0,
                    delta: ChatCompletionDelta {
                        role: None,
                        content: Some(chunk),
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: None,
                }],
            };

            chunks.push(completion_chunk);
        }

        // Add a final chunk with finish_reason
        let final_chunk = ChatCompletionChunk {
            id: format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", "")),
            model: self.model_id.clone(),
            created: Utc::now().timestamp() as u64,
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionDelta {
                    role: None,
                    content: Some("".to_string()),
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };
        chunks.push(final_chunk);

        chunks
    }
}

#[async_trait]
impl ModelConnector for MockModelBackend {
    /// Generate a completion
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        debug!(
            "Generating completion with mock backend for model: {}",
            self.model_id
        );

        // Simulate latency
        if self.simulated_latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.simulated_latency_ms,
            ))
            .await;
        }

        // Simulate errors if configured
        if self.simulate_errors {
            return Err(ConnectorError::Other(
                "Simulated error from mock backend".to_string(),
            ));
        }

        // Generate mock response
        let response = self.generate_mock_response(&request);

        Ok(response)
    }

    /// Generate a streaming completion
    async fn generate_streaming(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<StreamingResponse, ConnectorError> {
        debug!(
            "Generating streaming completion with mock backend for model: {}",
            self.model_id
        );

        // Simulate latency
        if self.simulated_latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.simulated_latency_ms,
            ))
            .await;
        }

        // Simulate errors if configured
        if self.simulate_errors {
            return Err(ConnectorError::Other(
                "Simulated error from mock backend".to_string(),
            ));
        }

        // Generate mock streaming chunks
        let chunks = self.generate_mock_streaming_chunks(&request);

        // Create a stream from the chunks
        let stream = stream::iter(chunks.into_iter().map(Ok));
        Ok(Box::pin(stream) as StreamingResponse)
    }

    /// Get the configuration for this connector
    fn get_config(&self) -> &ConnectorConfig {
        &self.config
    }

    /// Update the configuration for this connector
    fn update_config(&mut self, config: ConnectorConfig) {
        self.config = config;
    }

    /// Get the provider name for this connector
    fn provider_name(&self) -> &'static str {
        "mock"
    }

    /// Check if a model is supported by this connector
    fn supports_model(&self, model_id: &str) -> bool {
        model_id == self.model_id
    }

    /// List available models for this connector
    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        Ok(vec![self.model_id.clone()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::connectors::{ChatMessage, MessageRole};

    #[tokio::test]
    async fn test_mock_backend_generate() {
        let backend = MockModelBackend::new(
            "mock-model".to_string(),
            "Mock Model".to_string(),
            "mock-provider".to_string(),
        )
        .with_simulated_latency(0); // No latency for tests

        let request = ChatCompletionRequest {
            model: "mock-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello, world!".to_string(),
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

        let response = backend.generate(request).await.unwrap();

        assert_eq!(response.model, "mock-model");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert!(response.choices[0]
            .message
            .content
            .contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_mock_backend_error_simulation() {
        let backend = MockModelBackend::new(
            "mock-model".to_string(),
            "Mock Model".to_string(),
            "mock-provider".to_string(),
        )
        .with_simulated_errors(true)
        .with_simulated_latency(0); // No latency for tests

        let request = ChatCompletionRequest {
            model: "mock-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello, world!".to_string(),
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

        let result = backend.generate(request).await;
        assert!(result.is_err());
    }
}
