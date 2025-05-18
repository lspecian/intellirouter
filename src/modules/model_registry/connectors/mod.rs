//! Model Connector Interface
//!
//! This module defines the interface for connecting to different LLM backends.
//! It provides a common abstraction layer for interacting with various LLM providers
//! such as OpenAI, Ollama, and others.

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

/// Error types for model connectors
#[derive(Error, Debug, Clone)]
pub enum ConnectorError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Server error
    #[error("Server error: {0}")]
    Server(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Parsing error
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Role of a message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    /// System message (instructions to the model)
    System,
    /// User message (input from the user)
    User,
    /// Assistant message (response from the model)
    Assistant,
    /// Function message (result of a function call)
    Function,
    /// Tool message (result of a tool call)
    Tool,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Function => write!(f, "function"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// A message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
    /// Name of the sender (optional)
    pub name: Option<String>,
    /// Function call information (optional)
    pub function_call: Option<FunctionCall>,
    /// Tool calls information (optional)
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Function call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function
    pub name: String,
    /// Arguments to the function (as a JSON string)
    pub arguments: String,
}

/// Tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// ID of the tool call
    pub id: String,
    /// Type of the tool
    pub r#type: String,
    /// Function call information
    pub function: FunctionCall,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Name of the function
    pub name: String,
    /// Description of the function
    pub description: Option<String>,
    /// Parameters schema (in JSON Schema format)
    pub parameters: serde_json::Value,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Type of the tool
    pub r#type: String,
    /// Function definition
    pub function: FunctionDefinition,
}

/// Request for a chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// ID of the model to use
    pub model: String,
    /// Messages in the conversation
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling (0.0 to 1.0)
    pub top_p: Option<f32>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Whether to stream the response
    pub stream: Option<bool>,
    /// Function definitions
    pub functions: Option<Vec<FunctionDefinition>>,
    /// Tool definitions
    pub tools: Option<Vec<ToolDefinition>>,
    /// Additional model-specific parameters
    pub additional_params: Option<HashMap<String, serde_json::Value>>,
}

/// Response from a chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// ID of the completion
    pub id: String,
    /// Name of the model used
    pub model: String,
    /// Created timestamp
    pub created: u64,
    /// Choices returned by the model
    pub choices: Vec<ChatCompletionChoice>,
    /// Usage statistics
    pub usage: Option<TokenUsage>,
}

/// A choice in a chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// Index of the choice
    pub index: usize,
    /// Message content
    pub message: ChatMessage,
    /// Reason for finishing
    pub finish_reason: Option<String>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

/// A chunk in a streaming chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// ID of the completion
    pub id: String,
    /// Name of the model used
    pub model: String,
    /// Created timestamp
    pub created: u64,
    /// Choices in this chunk
    pub choices: Vec<ChatCompletionChunkChoice>,
}

/// A choice in a streaming chat completion chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Index of the choice
    pub index: usize,
    /// Delta (incremental) content
    pub delta: ChatCompletionDelta,
    /// Reason for finishing
    pub finish_reason: Option<String>,
}

/// Delta content in a streaming chat completion chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionDelta {
    /// Role of the message sender (only in first chunk)
    pub role: Option<MessageRole>,
    /// Content of the message (incremental)
    pub content: Option<String>,
    /// Function call information (incremental)
    pub function_call: Option<FunctionCallDelta>,
    /// Tool calls information (incremental)
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// Incremental function call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// Name of the function (incremental)
    pub name: Option<String>,
    /// Arguments to the function (incremental)
    pub arguments: Option<String>,
}

/// Incremental tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// ID of the tool call
    pub id: Option<String>,
    /// Type of the tool
    pub r#type: Option<String>,
    /// Function call information (incremental)
    pub function: Option<FunctionCallDelta>,
    /// Index of the tool call (for matching with previous chunks)
    pub index: Option<usize>,
}

/// Configuration for a model connector
#[derive(Debug, Clone)]
pub struct ConnectorConfig {
    /// Base URL for the API
    pub base_url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Organization ID (if applicable)
    pub org_id: Option<String>,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retries
    pub max_retries: u32,
    /// Additional configuration parameters
    pub additional_config: HashMap<String, String>,
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.example.com".to_string(),
            api_key: None,
            org_id: None,
            timeout_secs: 30,
            max_retries: 3,
            additional_config: HashMap::new(),
        }
    }
}

/// Type alias for a streaming response
pub type StreamingResponse =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, ConnectorError>> + Send>>;

/// Interface for model connectors
#[async_trait]
pub trait ModelConnector: Send + Sync {
    /// Generate a completion (non-streaming)
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError>;

    /// Generate a streaming completion
    async fn generate_streaming(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<StreamingResponse, ConnectorError>;

    /// Get the configuration for this connector
    fn get_config(&self) -> &ConnectorConfig;

    /// Update the configuration for this connector
    fn update_config(&mut self, config: ConnectorConfig);

    /// Get the provider name for this connector
    fn provider_name(&self) -> &'static str;

    /// Check if a model is supported by this connector
    fn supports_model(&self, model_id: &str) -> bool;

    /// List available models for this connector
    async fn list_models(&self) -> Result<Vec<String>, ConnectorError>;
}

/// Factory for creating model connectors
pub trait ModelConnectorFactory: Send + Sync {
    /// Create a new model connector
    fn create_connector(&self, config: ConnectorConfig) -> Arc<dyn ModelConnector>;

    /// Get the provider name for this factory
    fn provider_name(&self) -> &'static str;
}

/// Helper function to convert a connector error to a registry error
pub fn connector_error_to_registry_error(
    error: ConnectorError,
) -> crate::modules::model_registry::types::RegistryError {
    use crate::modules::model_registry::types::RegistryError;

    match error {
        ConnectorError::Authentication(msg) => {
            RegistryError::CommunicationError(format!("Authentication error: {}", msg))
        }
        ConnectorError::RateLimit(msg) => {
            RegistryError::CommunicationError(format!("Rate limit exceeded: {}", msg))
        }
        ConnectorError::ModelNotFound(msg) => RegistryError::NotFound(msg),
        ConnectorError::InvalidRequest(msg) => RegistryError::InvalidMetadata(msg),
        ConnectorError::Network(msg) => {
            RegistryError::CommunicationError(format!("Network error: {}", msg))
        }
        ConnectorError::Server(msg) => {
            RegistryError::CommunicationError(format!("Server error: {}", msg))
        }
        ConnectorError::Timeout(msg) => {
            RegistryError::CommunicationError(format!("Timeout: {}", msg))
        }
        ConnectorError::Parsing(msg) => {
            RegistryError::CommunicationError(format!("Parsing error: {}", msg))
        }
        ConnectorError::UnsupportedOperation(msg) => {
            RegistryError::Other(format!("Unsupported operation: {}", msg))
        }
        ConnectorError::Other(msg) => RegistryError::Other(msg),
    }
}

// Ollama connector
pub mod ollama;
pub use ollama::{OllamaConnector, OllamaConnectorFactory};

// OpenAI connector
pub mod openai;
pub use openai::{OpenAIConnector, OpenAIConnectorFactory};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::Function.to_string(), "function");
        assert_eq!(MessageRole::Tool.to_string(), "tool");
    }

    #[test]
    fn test_connector_error_to_registry_error() {
        use crate::modules::model_registry::types::RegistryError;

        let connector_error = ConnectorError::ModelNotFound("Model not found".to_string());
        let registry_error = connector_error_to_registry_error(connector_error);

        match registry_error {
            RegistryError::NotFound(msg) => assert_eq!(msg, "Model not found"),
            _ => panic!("Expected NotFound error"),
        }

        let connector_error = ConnectorError::Authentication("Invalid API key".to_string());
        let registry_error = connector_error_to_registry_error(connector_error);

        match registry_error {
            RegistryError::CommunicationError(msg) => {
                assert_eq!(msg, "Authentication error: Invalid API key")
            }
            _ => panic!("Expected CommunicationError error"),
        }
    }
}
