//! Service layer for the LLM Proxy module
//!
//! This module contains the business logic for processing chat completion
//! requests and generating responses, following clean architecture principles.

use futures::stream::{self, Stream};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

use crate::modules::llm_proxy::domain::message::{Message, MessageRole};
use crate::modules::llm_proxy::dto::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatMessageDelta,
    TokenUsage,
};
use crate::modules::llm_proxy::router_integration::{create_mock_router_service, RouterService};
use crate::modules::model_registry::connectors;
use crate::modules::router_core::RouterError;

/// Service for handling chat completion requests

/// Convert a DTO ChatCompletionRequest to a connector ChatCompletionRequest
fn convert_to_connector_request(
    request: &ChatCompletionRequest,
) -> connectors::ChatCompletionRequest {
    // Create a simplified connector request with just the essential fields
    connectors::ChatCompletionRequest {
        model: request.model.clone(),
        messages: vec![connectors::ChatMessage {
            role: connectors::MessageRole::User,
            content: match &request.messages.first() {
                Some(msg) => match &msg.content {
                    crate::modules::llm_proxy::domain::content::MessageContent::String(s) => {
                        s.clone()
                    }
                    _ => "Hello from the test script!".to_string(),
                },
                None => "Hello from the test script!".to_string(),
            },
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: request.temperature,
        top_p: request.top_p,
        max_tokens: request.max_tokens,
        stream: Some(request.stream),
        functions: None,
        tools: None,
        additional_params: None,
    }
}

/// Convert a connector ChatCompletionResponse to a DTO ChatCompletionResponse
fn convert_from_connector_response(
    response: connectors::ChatCompletionResponse,
) -> ChatCompletionResponse {
    // Create a simplified DTO response with just the essential fields
    ChatCompletionResponse {
        id: response.id,
        object: "chat.completion".to_string(),
        created: response.created,
        model: response.model,
        choices: vec![{
            crate::modules::llm_proxy::dto::ChatCompletionChoice {
                index: 0,
                message: Message {
                    role: MessageRole::Assistant,
                    content: crate::modules::llm_proxy::domain::content::MessageContent::String(
                        response
                            .choices
                            .first()
                            .map(|c| c.message.content.clone())
                            .unwrap_or_else(|| {
                                "This is a mock response from the router".to_string()
                            }),
                    ),
                    name: None,
                },
                finish_reason: "stop".to_string(),
            }
        }],
        usage: TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 10,
            total_tokens: 20,
        },
    }
}
pub struct ChatCompletionService {
    /// Router service for routing requests to the appropriate model
    router_service: RouterService,
}

impl ChatCompletionService {
    /// Create a new chat completion service
    pub fn new(router_service: RouterService) -> Self {
        Self { router_service }
    }

    /// Create a new chat completion service with a mock router
    pub fn new_with_mock_router() -> Self {
        let router_service = create_mock_router_service();
        Self { router_service }
    }

    /// Process a chat completion request and generate a response
    pub async fn process_completion_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, RouterError> {
        debug!(
            "Processing chat completion request for model: {}",
            request.model
        );

        // Convert the DTO request to a connector request
        let connector_request = convert_to_connector_request(request);

        // Route the request to the appropriate model
        let connector_response = self
            .router_service
            .route_request(&connector_request)
            .await?;

        // Convert the connector response to a DTO response
        Ok(convert_from_connector_response(connector_response))
    }

    /// Generate streaming chunks for a chat completion request
    pub async fn generate_streaming_chunks<'a>(
        &'a self,
        request: &'a ChatCompletionRequest,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk, RouterError>> + Send + 'a, RouterError>
    {
        debug!("Generating streaming chunks for model: {}", request.model);

        // Convert DTO request to connector request
        let connector_request =
            crate::modules::model_registry::connectors::ChatCompletionRequest {
                model: request.model.clone(),
                messages: request
                    .messages
                    .iter()
                    .map(|m| {
                        let content = match &m.content {
                            crate::modules::llm_proxy::domain::content::MessageContent::String(
                                text,
                            ) => text.clone(),
                            crate::modules::llm_proxy::domain::content::MessageContent::Array(
                                _,
                            ) =>
                            // Simplify multimodal content to text for now
                            {
                                "Content contains multiple parts".to_string()
                            }
                        };

                        crate::modules::model_registry::connectors::ChatMessage {
                        role: match m.role {
                            crate::modules::llm_proxy::domain::message::MessageRole::System =>
                                crate::modules::model_registry::connectors::MessageRole::System,
                            crate::modules::llm_proxy::domain::message::MessageRole::User =>
                                crate::modules::model_registry::connectors::MessageRole::User,
                            crate::modules::llm_proxy::domain::message::MessageRole::Assistant =>
                                crate::modules::model_registry::connectors::MessageRole::Assistant,
                            crate::modules::llm_proxy::domain::message::MessageRole::Tool =>
                                crate::modules::model_registry::connectors::MessageRole::Tool,
                            crate::modules::llm_proxy::domain::message::MessageRole::Function =>
                                crate::modules::model_registry::connectors::MessageRole::Function,
                            crate::modules::llm_proxy::domain::message::MessageRole::Developer =>
                                crate::modules::model_registry::connectors::MessageRole::System,
                            crate::modules::llm_proxy::domain::message::MessageRole::Unknown =>
                                crate::modules::model_registry::connectors::MessageRole::User,
                        },
                        content,
                        name: m.name.clone(),
                        function_call: None,
                        tool_calls: None,
                    }
                    })
                    .collect(),
                temperature: request.temperature,
                top_p: request.top_p,
                max_tokens: request.max_tokens,
                stream: Some(request.stream),
                functions: None,
                tools: None,
                additional_params: None,
            };

        // Route the streaming request
        let stream = self
            .router_service
            .route_streaming_request(&connector_request)
            .await?;

        // Convert the stream of strings to a stream of chunks
        let chunk_stream = stream.map(|result| {
            result.map(|chunk_str| {
                // Parse the chunk string into a ChatCompletionChunk
                serde_json::from_str::<ChatCompletionChunk>(&chunk_str).unwrap_or_else(|e| {
                    error!("Failed to parse chunk: {}", e);
                    // Create a fallback chunk in case of parsing error
                    ChatCompletionChunk::new_with_content(
                        request.model.clone(),
                        format!("Error parsing chunk: {}", e),
                    )
                })
            })
        });

        Ok(chunk_stream)
    }

    /// Legacy method for backward compatibility
    pub fn legacy_process_completion_request(
        request: &ChatCompletionRequest,
    ) -> ChatCompletionResponse {
        debug!(
            "Processing chat completion request (legacy) for model: {}",
            request.model
        );

        // Extract the last user message to generate a contextual response
        let last_user_message = request
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::User)
            .last()
            .map(|m| m.extract_text_content())
            .unwrap_or_else(|| "Hello".to_string());

        // Generate a response based on the user's message
        let response_content = Self::generate_contextual_response(&last_user_message);

        // Apply temperature if specified
        let response_content =
            Self::apply_temperature_effects(&response_content, request.temperature);

        // Apply max_tokens if specified
        let response_content =
            Self::apply_max_tokens_truncation(&response_content, request.max_tokens);

        // Create the assistant message
        let assistant_message = Message::new_assistant(response_content);

        // Create the response
        ChatCompletionResponse::new(request.model.clone(), assistant_message)
    }

    /// Legacy method for backward compatibility
    pub fn legacy_generate_streaming_chunks(
        request: &ChatCompletionRequest,
        chunk_size: usize,
    ) -> Vec<ChatCompletionChunk> {
        debug!(
            "Generating streaming chunks (legacy) for model: {}",
            request.model
        );

        // Extract the last user message to generate a contextual response
        let last_user_message = request
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::User)
            .last()
            .map(|m| m.extract_text_content())
            .unwrap_or_else(|| "Hello".to_string());

        // Generate a response based on the user's message
        let response_content = Self::generate_contextual_response(&last_user_message);

        // Apply temperature if specified
        let response_content =
            Self::apply_temperature_effects(&response_content, request.temperature);

        // Apply max_tokens if specified
        let response_content =
            Self::apply_max_tokens_truncation(&response_content, request.max_tokens);

        // Create chunks
        Self::create_streaming_chunks(&request.model, &response_content, chunk_size)
    }

    /// Generate a contextual response based on the user's message
    fn generate_contextual_response(message: &str) -> String {
        // This is a mock implementation that would be replaced with actual LLM calls
        format!(
            "Hello! I'm a mock assistant from the IntelliRouter LLM proxy. You said: {}",
            message
        )
    }

    /// Apply temperature effects to the response content
    fn apply_temperature_effects(content: &str, temperature: Option<f32>) -> String {
        // This is a mock implementation that would apply temperature effects in a real system
        if let Some(temp) = temperature {
            if temp > 1.0 {
                // Higher temperature, more random/creative response
                format!(
                    "{} [Generated with higher creativity at temperature {}]",
                    content, temp
                )
            } else {
                // Lower temperature, more deterministic response
                format!(
                    "{} [Generated with higher precision at temperature {}]",
                    content, temp
                )
            }
        } else {
            content.to_string()
        }
    }

    /// Apply max_tokens truncation to the response content
    fn apply_max_tokens_truncation(content: &str, max_tokens: Option<u32>) -> String {
        // This is a mock implementation that would truncate based on token count in a real system
        if let Some(max) = max_tokens {
            // Rough approximation: 1 token ≈ 4 characters
            let max_chars = max as usize * 4;
            if content.len() > max_chars {
                format!("{}...", &content[0..max_chars])
            } else {
                content.to_string()
            }
        } else {
            content.to_string()
        }
    }

    /// Create streaming chunks from a response content
    fn create_streaming_chunks(
        model: &str,
        content: &str,
        chunk_size: usize,
    ) -> Vec<ChatCompletionChunk> {
        let mut chunks = Vec::new();
        let model = model.to_string();

        // First chunk with role
        chunks.push(ChatCompletionChunk::new_with_role(
            model.clone(),
            "assistant".to_string(),
        ));

        // Content chunks
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut start = 0;

        while start < words.len() {
            let end = (start + chunk_size).min(words.len());
            let chunk_content = words[start..end].join(" ");

            chunks.push(ChatCompletionChunk::new_with_content(
                model.clone(),
                chunk_content,
            ));

            start = end;
        }

        // Final chunk with finish reason
        chunks.push(ChatCompletionChunk::new_with_finish(
            model.clone(),
            None,
            "stop".to_string(),
        ));

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_completion_request() {
        let service = ChatCompletionService::new_with_mock_router();

        let request = ChatCompletionRequest {
            model: "claude-3-sonnet".to_string(),
            messages: vec![Message::new_user("Hello, how are you?".to_string())],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: false,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let response = service.process_completion_request(&request).await.unwrap();

        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert!(response.choices[0].message.content.contains("Hello"));
    }

    #[test]
    fn test_legacy_process_completion_request() {
        let request = ChatCompletionRequest {
            model: "claude-3-sonnet".to_string(),
            messages: vec![Message::new_user("Hello, how are you?".to_string())],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: false,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let response = ChatCompletionService::legacy_process_completion_request(&request);

        assert_eq!(response.model, "claude-3-sonnet");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert!(response.choices[0]
            .message
            .extract_text_content()
            .contains("Hello"));
    }

    #[test]
    fn test_legacy_generate_streaming_chunks() {
        let request = ChatCompletionRequest {
            model: "claude-3-sonnet".to_string(),
            messages: vec![Message::new_user("Hello, how are you?".to_string())],
            temperature: None,
            top_p: None,
            n: None,
            stream: true,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let chunks = ChatCompletionService::legacy_generate_streaming_chunks(&request, 2);

        assert!(chunks.len() >= 3); // At least first chunk, one content chunk, and final chunk

        // First chunk should have role
        assert_eq!(
            chunks[0].choices[0].delta.role,
            Some("assistant".to_string())
        );
        assert_eq!(chunks[0].choices[0].delta.content, None);

        // Middle chunks should have content
        assert_eq!(chunks[1].choices[0].delta.role, None);
        assert!(chunks[1].choices[0].delta.content.is_some());

        // Last chunk should have finish reason
        assert_eq!(
            chunks.last().unwrap().choices[0].finish_reason,
            Some("stop".to_string())
        );
    }
}
