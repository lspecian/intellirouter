//! LLM Proxy Routes
//!
//! This module implements the route handlers for the LLM Proxy server,
//! providing OpenAI-compatible API endpoints.

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    Json,
};
use chrono::Utc;
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, fmt, time::Duration};
use tokio_stream::StreamExt;
use tracing::debug;
use uuid::Uuid;

use super::{
    formatting::{self},
    telemetry_integration::AppState,
};

/// OpenAI API chat completion request
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionRequest {
    /// The model to use for completion
    pub model: String,
    /// The messages to generate completions for
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 to 2.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Nucleus sampling parameter (0.0 to 1.0)
    #[serde(default)]
    pub top_p: Option<f32>,
    /// Number of completions to generate
    #[serde(default)]
    pub n: Option<u32>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
    /// Maximum number of tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Presence penalty (-2.0 to 2.0)
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    /// User identifier for tracking
    #[serde(default)]
    pub user: Option<String>,
}

/// Chat message in a completion request or response
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    /// The role of the message author (system, user, assistant)
    pub role: String,
    /// The content of the message
    pub content: String,
    /// Optional name of the author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// OpenAI API chat completion response
#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    /// Unique identifier for the completion
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used for completion
    pub model: String,
    /// Generated completions
    pub choices: Vec<ChatCompletionChoice>,
    /// Token usage statistics
    pub usage: TokenUsage,
}

/// A single completion choice in a response
#[derive(Debug, Serialize)]
pub struct ChatCompletionChoice {
    /// Index of the choice
    pub index: u32,
    /// The generated message
    pub message: ChatMessage,
    /// Reason why generation finished
    pub finish_reason: String,
}

/// OpenAI API chat completion chunk for streaming responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique identifier for the completion
    pub id: String,
    /// Object type (always "chat.completion.chunk")
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used for completion
    pub model: String,
    /// Generated completion chunks
    pub choices: Vec<ChatCompletionChunkChoice>,
}

/// A single completion chunk choice in a streaming response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Index of the choice
    pub index: u32,
    /// The delta content for this chunk
    pub delta: ChatMessageDelta,
    /// Reason why generation finished (only present in the final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Delta content for a streaming response chunk
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageDelta {
    /// Role of the message author (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Content delta for this chunk
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Token usage statistics
#[derive(Debug, Serialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error details
    pub error: ApiErrorDetail,
}

/// API error details
#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    /// Error message
    pub message: String,
    /// Error type
    pub r#type: String,
    /// Parameter that caused the error (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    /// Error code (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error.r#type, self.error.message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.error.r#type.as_str() {
            "invalid_request_error" => StatusCode::BAD_REQUEST,
            "authentication_error" => StatusCode::UNAUTHORIZED,
            "permission_error" => StatusCode::FORBIDDEN,
            "not_found_error" => StatusCode::NOT_FOUND,
            "rate_limit_error" => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

/// Route handler for /v1/chat/completions
pub async fn chat_completions(
    State(_state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, ApiError> {
    debug!(
        "Received chat completion request for model: {}",
        request.model
    );

    // Check if streaming is requested and redirect to streaming handler
    if request.stream {
        return Err(create_validation_error(
            "Streaming requests should be sent to /v1/chat/completions/stream endpoint",
            Some("stream"),
        ));
    }

    // Validate the request
    super::validation::validate_chat_completion_request(&request)?;

    // Extract the last user message to generate a contextual response
    let last_user_message = request
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or("Hello");

    // Generate a response based on the user's message
    let response_content = formatting::generate_contextual_response(last_user_message);

    // Apply temperature if specified
    let response_content =
        formatting::apply_temperature_effects(&response_content, request.temperature);

    // Apply max_tokens if specified
    let response_content =
        formatting::apply_max_tokens_truncation(&response_content, request.max_tokens);

    // Format the response using the formatting module
    let response = formatting::format_completion_response(
        &request.model,
        &request.messages,
        &response_content,
        "stop",
    );

    Ok(Json(response))
}

// The functions generate_mock_response, generate_contextual_response,
// calculate_prompt_tokens, and calculate_completion_tokens have been moved
// to the formatting module for better organization and reusability.

/// Route handler for /v1/chat/completions/stream
pub async fn chat_completions_stream(
    State(_state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    debug!(
        "Received streaming chat completion request for model: {}",
        request.model
    );

    // Validate the request
    super::validation::validate_chat_completion_request(&request)?;

    // Extract the last user message to generate a contextual response
    let last_user_message = request
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or("Hello");

    // Generate a response based on the user's message
    let response_content = formatting::generate_contextual_response(last_user_message);

    // Apply temperature if specified
    let response_content =
        formatting::apply_temperature_effects(&response_content, request.temperature);

    // Apply max_tokens if specified
    let response_content =
        formatting::apply_max_tokens_truncation(&response_content, request.max_tokens);

    // Create streaming chunks using the formatting module
    let chunks = formatting::create_streaming_chunks(&request.model, &response_content, 5);
    let model = request.model.clone();

    // Create a stream from the chunks
    let stream = stream::iter(chunks.into_iter().enumerate())
        .map(move |(_, chunk)| {
            let json = serde_json::to_string(&chunk).unwrap_or_default();
            Ok(Event::default().data(json))
        })
        .throttle(Duration::from_millis(300));

    Ok(Sse::new(stream))
}

// The create_mock_chunk function has been replaced by the formatting module's
// format_completion_chunk and create_streaming_chunks functions for better
// organization and reusability.

/// Error handler for invalid requests
pub fn handle_invalid_request(error_message: &str, param: Option<&str>) -> ApiError {
    ApiError {
        error: ApiErrorDetail {
            message: error_message.to_string(),
            r#type: "invalid_request_error".to_string(),
            param: param.map(|s| s.to_string()),
            code: None,
        },
    }
}

/// Create a validation error
fn create_validation_error(message: &str, param: Option<&str>) -> ApiError {
    handle_invalid_request(message, param)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::telemetry::{CostCalculator, TelemetryManager};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_chat_completions() {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            telemetry,
            cost_calculator,
        };

        // Create test request
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                name: None,
            }],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: false,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        // Call the handler
        let result = chat_completions(State(app_state), Json(request)).await;

        // Verify the result
        assert!(result.is_ok());

        if let Ok(Json(response)) = result {
            assert_eq!(response.object, "chat.completion");
            assert_eq!(response.choices.len(), 1);
            assert_eq!(response.choices[0].message.role, "assistant");
            // Verify that the response contains the expected greeting
            assert!(response.choices[0].message.content.contains("Hello"));
            assert!(response.choices[0]
                .message
                .content
                .contains("mock assistant"));
            // Verify that token usage is calculated
            assert!(response.usage.prompt_tokens > 0);
            assert!(response.usage.completion_tokens > 0);
            assert_eq!(
                response.usage.total_tokens,
                response.usage.prompt_tokens + response.usage.completion_tokens
            );
        }
    }

    #[tokio::test]
    async fn test_chat_completions_with_parameters() {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            telemetry,
            cost_calculator,
        };

        // Test with high temperature
        let high_temp_request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Tell me about the weather".to_string(),
                name: None,
            }],
            temperature: Some(1.5), // High temperature
            top_p: None,
            n: None,
            stream: false,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let result = chat_completions(State(app_state.clone()), Json(high_temp_request)).await;
        assert!(result.is_ok());
        if let Ok(Json(response)) = result {
            // High temperature should add creative variations text
            assert!(response.choices[0]
                .message
                .content
                .contains("high temperature"));
        }

        // Test with max_tokens limit
        let limited_tokens_request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Help me with something".to_string(),
                name: None,
            }],
            temperature: None,
            top_p: None,
            n: None,
            stream: false,
            max_tokens: Some(5), // Very limited tokens
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let result = chat_completions(State(app_state), Json(limited_tokens_request)).await;
        assert!(result.is_ok());
        if let Ok(Json(response)) = result {
            // Response should be truncated
            assert!(response.choices[0].message.content.contains("..."));
            // Count words to verify truncation
            let word_count = response.choices[0]
                .message
                .content
                .split_whitespace()
                .count();
            assert!(word_count <= 6); // 5 words + possible ellipsis
        }
    }

    #[tokio::test]
    async fn test_contextual_responses() {
        // Test different contextual responses
        let test_cases = vec![
            ("hello", "Hello"),
            ("Can you help me?", "I'm here to help"),
            ("What's the weather like?", "weather data"),
            ("Write some code", "programming"),
            ("Explain quantum physics", "explain"),
            ("Random query", "mock response"),
        ];

        for (input, expected_substring) in test_cases {
            let response = formatting::generate_contextual_response(input);
            assert!(
                response.contains(expected_substring),
                "Response for '{}' should contain '{}', but got: '{}'",
                input,
                expected_substring,
                response
            );
        }
    }

    #[tokio::test]
    async fn test_token_calculation() {
        // Test prompt token calculation
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "Hello, how are you?".to_string(),
                    name: None,
                },
            ],
            temperature: None,
            top_p: None,
            n: None,
            stream: false,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let prompt_tokens = formatting::calculate_prompt_tokens(&request.messages);
        assert!(prompt_tokens > 0);

        // Test completion token calculation
        let content = "This is a test response to calculate tokens.";
        let completion_tokens = formatting::calculate_completion_tokens(content);
        assert!(completion_tokens > 0);
        // Very rough approximation: 4 chars per token plus overhead
        assert!(completion_tokens >= (content.len() / 4) as u32);
    }

    #[tokio::test]
    async fn test_chat_completions_stream() {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            telemetry,
            cost_calculator,
        };

        // Create test request
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                name: None,
            }],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: true,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        // Call the handler
        let result = chat_completions_stream(State(app_state), Json(request.clone())).await;

        // Verify the result is ok (we can't easily test the stream contents in a unit test)
        assert!(result.is_ok());

        // Test the formatting module's chunk creation functions directly
        let role_chunk = formatting::format_completion_chunk(
            &request.model,
            0,
            None,
            Some("assistant".to_string()),
            None,
        );
        assert_eq!(role_chunk.object, "chat.completion.chunk");
        assert_eq!(role_chunk.choices.len(), 1);
        assert_eq!(
            role_chunk.choices[0].delta.role,
            Some("assistant".to_string())
        );
        assert_eq!(role_chunk.choices[0].delta.content, None);

        let content_chunk = formatting::format_completion_chunk(
            &request.model,
            1,
            Some("Hello".to_string()),
            None,
            None,
        );
        assert_eq!(content_chunk.choices[0].delta.role, None);
        assert_eq!(
            content_chunk.choices[0].delta.content,
            Some("Hello".to_string())
        );

        let final_chunk = formatting::format_completion_chunk(
            &request.model,
            2,
            Some("World".to_string()),
            None,
            Some("stop".to_string()),
        );
        assert_eq!(
            final_chunk.choices[0].finish_reason,
            Some("stop".to_string())
        );

        // Test the streaming chunks creation
        let chunks = formatting::create_streaming_chunks(&request.model, "Hello World", 2);
        assert!(chunks.len() >= 3); // At least role chunk + 2 content chunks
        assert_eq!(
            chunks[0].choices[0].delta.role,
            Some("assistant".to_string())
        );
        assert_eq!(
            chunks.last().unwrap().choices[0].finish_reason,
            Some("stop".to_string())
        );
    }

    #[tokio::test]
    async fn test_chat_completions_validation_failure() {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            telemetry,
            cost_calculator,
        };

        // Test cases for validation failures
        let test_cases = vec![
            // Invalid model
            (
                ChatCompletionRequest {
                    model: "invalid-model".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello!".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    top_p: None,
                    n: None,
                    stream: false,
                    max_tokens: None,
                    presence_penalty: None,
                    frequency_penalty: None,
                    user: None,
                },
                "model",
                "not supported",
            ),
            // Empty messages
            (
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![],
                    temperature: None,
                    top_p: None,
                    n: None,
                    stream: false,
                    max_tokens: None,
                    presence_penalty: None,
                    frequency_penalty: None,
                    user: None,
                },
                "messages",
                "cannot be empty",
            ),
            // Invalid temperature
            (
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello!".to_string(),
                        name: None,
                    }],
                    temperature: Some(3.0),
                    top_p: None,
                    n: None,
                    stream: false,
                    max_tokens: None,
                    presence_penalty: None,
                    frequency_penalty: None,
                    user: None,
                },
                "temperature",
                "must be between",
            ),
        ];

        for (request, expected_param, expected_message_part) in test_cases {
            // Call the handler
            let result = chat_completions(State(app_state.clone()), Json(request)).await;

            // Verify the result is an error
            assert!(result.is_err());

            if let Err(error) = result {
                assert_eq!(error.error.param, Some(expected_param.to_string()));
                assert!(error.error.message.contains(expected_message_part));
            }
        }
    }
}
