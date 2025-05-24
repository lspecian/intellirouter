//! Ollama connector for interacting with Ollama API
//!
//! This module provides a connector for the Ollama API, which allows
//! interaction with locally hosted LLM models through the Ollama server.

use super::{
    ChatCompletionChoice, ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta,
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ConnectorConfig, ConnectorError,
    MessageRole, ModelConnector, ModelConnectorFactory, StreamingResponse, TokenUsage,
};
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Ollama connector for interacting with Ollama API
pub struct OllamaConnector {
    /// HTTP client
    client: Client,
    /// Configuration
    config: ConnectorConfig,
}

/// Ollama chat request format
#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatRequest {
    /// Model name
    model: String,
    /// Messages in the conversation
    messages: Vec<OllamaMessage>,
    /// Whether to stream the response
    stream: bool,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Ollama message format
#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    /// Role of the message sender
    role: String,
    /// Content of the message
    content: String,
}

/// Ollama options for request
#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    /// Temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// Top-p sampling (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

/// Ollama chat response format
#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatResponse {
    /// Model name
    model: String,
    /// Created timestamp
    created_at: String,
    /// Response message
    message: OllamaMessage,
    /// Done flag for streaming
    #[serde(default)]
    done: bool,
}

/// Ollama models list response
#[derive(Debug, Serialize, Deserialize)]
struct OllamaModelsResponse {
    /// List of models
    models: Vec<OllamaModel>,
}

/// Ollama model information
#[derive(Debug, Serialize, Deserialize)]
struct OllamaModel {
    /// Model name
    name: String,
    /// Model size
    #[serde(default)]
    size: u64,
    /// Model modified time
    #[serde(default)]
    modified_at: String,
}

impl OllamaConnector {
    /// Create a new Ollama connector
    pub fn new(config: ConnectorConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();

        Self { client, config }
    }

    /// Convert our chat completion request to Ollama format
    fn convert_request(&self, request: &ChatCompletionRequest) -> OllamaChatRequest {
        // Convert messages
        let messages = request
            .messages
            .iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    // Ollama doesn't support function or tool roles directly,
                    // so we'll convert them to user messages
                    MessageRole::Function => "user".to_string(),
                    MessageRole::Tool => "user".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        // Create options
        let options = Some(OllamaOptions {
            temperature: request.temperature,
            top_p: request.top_p,
            num_predict: request.max_tokens,
        });

        OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: request.stream.unwrap_or(false),
            options,
        }
    }

    /// Convert Ollama response to our format
    fn convert_response(
        &self,
        response: OllamaChatResponse,
        request_id: &str,
    ) -> ChatCompletionResponse {
        // Create a chat message from the Ollama response
        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: response.message.content,
            name: None,
            function_call: None,
            tool_calls: None,
        };

        // Create a choice
        let choice = ChatCompletionChoice {
            index: 0,
            message,
            finish_reason: Some("stop".to_string()),
        };

        // Create the response
        ChatCompletionResponse {
            id: request_id.to_string(),
            model: response.model,
            created: chrono::Utc::now().timestamp() as u64,
            choices: vec![choice],
            usage: Some(TokenUsage {
                prompt_tokens: 0, // Ollama doesn't provide token counts
                completion_tokens: 0,
                total_tokens: 0,
            }),
        }
    }

    /// Convert Ollama streaming response to our chunk format
    fn convert_stream_chunk(
        &self,
        response: OllamaChatResponse,
        request_id: &str,
    ) -> ChatCompletionChunk {
        // Create a delta
        let delta = ChatCompletionDelta {
            role: if response.done {
                None
            } else {
                Some(MessageRole::Assistant)
            },
            content: Some(response.message.content),
            function_call: None,
            tool_calls: None,
        };

        // Create a choice
        let choice = ChatCompletionChunkChoice {
            index: 0,
            delta,
            finish_reason: if response.done {
                Some("stop".to_string())
            } else {
                None
            },
        };

        // Create the chunk
        ChatCompletionChunk {
            id: request_id.to_string(),
            model: response.model,
            created: chrono::Utc::now().timestamp() as u64,
            choices: vec![choice],
        }
    }

    /// Build the API URL for a specific endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!(
            "{}/api/{}",
            self.config.base_url.trim_end_matches('/'),
            endpoint
        )
    }

    /// Parse Ollama error response
    async fn parse_error_response(
        &self,
        status: StatusCode,
        response: reqwest::Response,
    ) -> ConnectorError {
        // Try to extract error message from response
        let error_text = match response.text().await {
            Ok(text) => {
                // Try to parse as JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(error) = json.get("error") {
                        if let Some(error_str) = error.as_str() {
                            error_str.to_string()
                        } else {
                            error.to_string()
                        }
                    } else {
                        text
                    }
                } else {
                    text
                }
            }
            Err(e) => format!("Failed to read error response: {}", e),
        };

        match status {
            StatusCode::UNAUTHORIZED => {
                ConnectorError::Authentication(format!("Unauthorized: {}", error_text))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                ConnectorError::RateLimit(format!("Rate limited: {}", error_text))
            }
            StatusCode::NOT_FOUND => {
                ConnectorError::ModelNotFound(format!("Model not found: {}", error_text))
            }
            StatusCode::BAD_REQUEST => {
                ConnectorError::InvalidRequest(format!("Bad request: {}", error_text))
            }
            StatusCode::REQUEST_TIMEOUT => {
                ConnectorError::Timeout(format!("Request timed out: {}", error_text))
            }
            _ => ConnectorError::Server(format!("Server error ({}): {}", status, error_text)),
        }
    }
}

#[async_trait]
impl ModelConnector for OllamaConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        // Generate a request ID
        let request_id = Uuid::new_v4().to_string();

        // Convert the request to Ollama format
        let mut ollama_request = self.convert_request(&request);
        ollama_request.stream = false;

        // Send the request to Ollama with retry logic for transient errors
        let mut attempts = 0;
        let max_attempts = self.config.max_retries as usize + 1; // +1 for the initial attempt
        let mut last_error = None;

        let response = loop {
            attempts += 1;

            match self
                .client
                .post(self.build_url("chat"))
                .json(&ollama_request)
                .send()
                .await
            {
                Ok(resp) => break resp,
                Err(e) => {
                    // Check if we should retry
                    if attempts >= max_attempts {
                        return Err(ConnectorError::Network(format!(
                            "Failed to send request after {} attempts: {}",
                            attempts, e
                        )));
                    }

                    // Store the error and retry after a delay
                    last_error = Some(e);

                    // Exponential backoff: 100ms, 200ms, 400ms, etc.
                    let delay = std::time::Duration::from_millis(100 * (1 << (attempts - 1)));
                    tokio::time::sleep(delay).await;
                }
            }
        };

        // Check the response status
        let status = response.status();
        if !status.is_success() {
            return Err(self.parse_error_response(status, response).await);
        }

        // Parse the response
        let ollama_response = response
            .json::<OllamaChatResponse>()
            .await
            .map_err(|e| ConnectorError::Parsing(format!("Failed to parse response: {}", e)))?;

        // Convert the response to our format
        Ok(self.convert_response(ollama_response, &request_id))
    }

    async fn generate_streaming(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<StreamingResponse, ConnectorError> {
        // Generate a request ID
        let request_id = Uuid::new_v4().to_string();

        // Convert the request to Ollama format
        let mut ollama_request = self.convert_request(&request);
        ollama_request.stream = true;

        // Send the request to Ollama with retry logic for transient errors
        let mut attempts = 0;
        let max_attempts = self.config.max_retries as usize + 1; // +1 for the initial attempt
        let mut last_error = None;

        let response = loop {
            attempts += 1;

            match self
                .client
                .post(self.build_url("chat"))
                .json(&ollama_request)
                .send()
                .await
            {
                Ok(resp) => break resp,
                Err(e) => {
                    // Check if we should retry
                    if attempts >= max_attempts {
                        return Err(ConnectorError::Network(format!(
                            "Failed to send streaming request after {} attempts: {}",
                            attempts, e
                        )));
                    }

                    // Store the error and retry after a delay
                    last_error = Some(e);

                    // Exponential backoff: 100ms, 200ms, 400ms, etc.
                    let delay = std::time::Duration::from_millis(100 * (1 << (attempts - 1)));
                    tokio::time::sleep(delay).await;
                }
            }
        };

        // Check the response status
        let status = response.status();
        if !status.is_success() {
            return Err(self.parse_error_response(status, response).await);
        }

        // Create a stream that processes each line from the response
        let request_id_clone = request_id.clone();
        let self_clone = self.clone();

        // Process the stream using unfold to handle the response chunks
        let stream = Box::pin(stream::unfold(
            (response, request_id_clone, self_clone),
            |(mut response, request_id, connector)| async move {
                // Read the next chunk from the response
                if let Ok(chunk) = response.chunk().await {
                    if let Some(bytes) = chunk {
                        // Convert the chunk to a string
                        let chunk_str = String::from_utf8_lossy(&bytes);

                        // Parse the JSON
                        match serde_json::from_str::<OllamaChatResponse>(&chunk_str) {
                            Ok(ollama_response) => {
                                // Convert to our format
                                let result =
                                    Ok(connector
                                        .convert_stream_chunk(ollama_response, &request_id));
                                Some((result, (response, request_id, connector)))
                            }
                            Err(e) => {
                                // Try to determine if this is a valid JSON but not matching our expected format
                                if let Ok(value) =
                                    serde_json::from_str::<serde_json::Value>(&chunk_str)
                                {
                                    // Check if it's an error response
                                    if let Some(error) = value.get("error").and_then(|e| e.as_str())
                                    {
                                        let error = ConnectorError::Server(format!(
                                            "Server error in stream: {}",
                                            error
                                        ));
                                        return Some((
                                            Err(error),
                                            (response, request_id, connector),
                                        ));
                                    }
                                }

                                // Return parsing error
                                let error = ConnectorError::Parsing(format!(
                                    "Failed to parse chunk: {}, data: {}",
                                    e, chunk_str
                                ));
                                Some((Err(error), (response, request_id, connector)))
                            }
                        }
                    } else {
                        // End of stream
                        None
                    }
                } else {
                    // Error reading from stream
                    let error = ConnectorError::Network("Error reading from stream".to_string());
                    Some((Err(error), (response, request_id, connector)))
                }
            },
        ));

        // Return the stream
        Ok(stream as StreamingResponse)
    }

    fn get_config(&self) -> &ConnectorConfig {
        &self.config
    }

    fn update_config(&mut self, config: ConnectorConfig) {
        self.config = config;
    }

    fn provider_name(&self) -> &'static str {
        "ollama"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        // Ollama supports any model that's been pulled to the server
        // We could implement a more sophisticated check by querying the Ollama API
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        // Query Ollama's /api/tags endpoint to get available models
        let response = self
            .client
            .get(self.build_url("tags"))
            .send()
            .await
            .map_err(|e| ConnectorError::Network(format!("Failed to list models: {}", e)))?;

        // Check the response status
        let status = response.status();
        if !status.is_success() {
            return Err(self.parse_error_response(status, response).await);
        }

        // Parse the response
        let models_response = response
            .json::<OllamaModelsResponse>()
            .await
            .map_err(|e| ConnectorError::Parsing(format!("Failed to parse models: {}", e)))?;

        // Extract model names
        let model_names = models_response
            .models
            .into_iter()
            .map(|model| model.name)
            .collect();

        Ok(model_names)
    }
}

// Implement Clone for OllamaConnector
impl Clone for OllamaConnector {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
        }
    }
}

/// Factory for creating Ollama connectors
pub struct OllamaConnectorFactory;

impl ModelConnectorFactory for OllamaConnectorFactory {
    fn create_connector(&self, config: ConnectorConfig) -> Arc<dyn ModelConnector> {
        Arc::new(OllamaConnector::new(config))
    }

    fn provider_name(&self) -> &'static str {
        "ollama"
    }
}

#[cfg(all(test, not(feature = "production")))]
mod tests {
    use super::*;
    use crate::modules::model_registry::connectors::{ChatMessage, MessageRole};
    use serde_json::json;

    #[test]
    fn test_convert_request() {
        let config = ConnectorConfig {
            base_url: "http://localhost:11434".to_string(),
            api_key: None,
            org_id: None,
            timeout_secs: 30,
            max_retries: 3,
            additional_config: Default::default(),
        };

        let connector = OllamaConnector::new(config);

        let request = ChatCompletionRequest {
            model: "llama2".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: "Hello, how are you?".to_string(),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                },
            ],
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(100),
            stream: Some(false),
            functions: None,
            tools: None,
            additional_params: None,
        };

        let ollama_request = connector.convert_request(&request);

        assert_eq!(ollama_request.model, "llama2");
        assert_eq!(ollama_request.messages.len(), 2);
        assert_eq!(ollama_request.messages[0].role, "system");
        assert_eq!(
            ollama_request.messages[0].content,
            "You are a helpful assistant."
        );
        assert_eq!(ollama_request.messages[1].role, "user");
        assert_eq!(ollama_request.messages[1].content, "Hello, how are you?");
        assert_eq!(ollama_request.stream, false);
        assert_eq!(
            ollama_request.options.as_ref().unwrap().temperature,
            Some(0.7)
        );
        assert_eq!(ollama_request.options.as_ref().unwrap().top_p, Some(0.9));
        assert_eq!(
            ollama_request.options.as_ref().unwrap().num_predict,
            Some(100)
        );
    }

    #[test]
    fn test_convert_response() {
        let config = ConnectorConfig {
            base_url: "http://localhost:11434".to_string(),
            api_key: None,
            org_id: None,
            timeout_secs: 30,
            max_retries: 3,
            additional_config: Default::default(),
        };

        let connector = OllamaConnector::new(config);
        let request_id = "test-request-id";

        let ollama_response = OllamaChatResponse {
            model: "llama2".to_string(),
            created_at: "2023-11-06T09:00:00.000000Z".to_string(),
            message: OllamaMessage {
                role: "assistant".to_string(),
                content: "I'm doing well, thank you for asking!".to_string(),
            },
            done: true,
        };

        let response = connector.convert_response(ollama_response, request_id);

        assert_eq!(response.id, request_id);
        assert_eq!(response.model, "llama2");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert_eq!(
            response.choices[0].message.content,
            "I'm doing well, thank you for asking!"
        );
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
    }

    // Note: This test is commented out because it requires mockito, which we're not using in this implementation
    // We'll use integration tests instead to test the actual API calls
    /*
    #[tokio::test]
    async fn test_generate() {
        // This would normally use a mock HTTP server to test the API calls
        // For now, we'll skip this test and rely on unit tests for the conversion functions
        // and integration tests for the actual API calls
    }
    */

    // Note: This test is commented out because it requires mockito, which we're not using in this implementation
    // We'll use integration tests instead to test the actual API calls
    /*
    #[tokio::test]
    async fn test_list_models() {
        // This would normally use a mock HTTP server to test the API calls
        // For now, we'll skip this test and rely on unit tests for the conversion functions
        // and integration tests for the actual API calls
    }
    */
}
