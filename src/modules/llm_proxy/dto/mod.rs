//! Data Transfer Objects for the LLM Proxy API
//!
//! This module contains the DTOs used for the API interface,
//! following clean architecture principles to separate the API
//! layer from the domain layer.

use crate::modules::llm_proxy::domain::message::Message;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OpenAI API chat completion request
#[derive(Debug, Deserialize, Clone)]
pub struct ChatCompletionRequest {
    /// The model to use for completion
    pub model: String,
    /// The messages to generate completions for
    pub messages: Vec<Message>,
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
    pub message: Message,
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

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        let json = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"error":{"message":"Failed to serialize error","type":"internal_error"}}"#
                .to_string()
        });

        (status, json).into_response()
    }
}

/// API error detail
#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    /// Error message
    pub message: String,
    /// Error type
    pub r#type: String,
    /// Parameter that caused the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    /// Error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ChatCompletionResponse {
    /// Create a new chat completion response
    pub fn new(model: String, message: Message) -> Self {
        let id = format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", ""));
        let created = Utc::now().timestamp() as u64;
        let content_length = message.extract_text_content().len() as u32;

        Self {
            id,
            object: "chat.completion".to_string(),
            created,
            model,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message,
                finish_reason: "stop".to_string(),
            }],
            usage: TokenUsage {
                prompt_tokens: 10,                     // Mock values
                completion_tokens: content_length / 4, // Rough approximation
                total_tokens: 10 + (content_length / 4),
            },
        }
    }
}

impl ChatCompletionChunk {
    /// Create a new chat completion chunk
    pub fn new(model: String, delta: ChatMessageDelta, finish_reason: Option<String>) -> Self {
        let id = format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", ""));
        let created = Utc::now().timestamp() as u64;

        Self {
            id,
            object: "chat.completion.chunk".to_string(),
            created,
            model,
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta,
                finish_reason,
            }],
        }
    }

    /// Create a new chat completion chunk with role
    pub fn new_with_role(model: String, role: String) -> Self {
        Self::new(
            model,
            ChatMessageDelta {
                role: Some(role),
                content: None,
            },
            None,
        )
    }

    /// Create a new chat completion chunk with content
    pub fn new_with_content(model: String, content: String) -> Self {
        Self::new(
            model,
            ChatMessageDelta {
                role: None,
                content: Some(content),
            },
            None,
        )
    }

    /// Create a new chat completion chunk with finish reason
    pub fn new_with_finish(model: String, content: Option<String>, finish_reason: String) -> Self {
        Self::new(
            model,
            ChatMessageDelta {
                role: None,
                content,
            },
            Some(finish_reason),
        )
    }
}

// Tests moved to tests/unit/modules/llm_proxy/dto/
