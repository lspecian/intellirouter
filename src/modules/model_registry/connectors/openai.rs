//! OpenAI connector for interacting with OpenAI API
//!
//! This module provides a connector for the OpenAI API, which allows
//! interaction with OpenAI's hosted LLM models.

use super::{
    ChatCompletionChoice, ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta,
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ConnectorConfig, ConnectorError,
    FunctionCall, FunctionCallDelta, MessageRole, ModelConnector, ModelConnectorFactory,
    StreamingResponse, TokenUsage, ToolCall, ToolCallDelta,
};
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// OpenAI connector for interacting with OpenAI API
pub struct OpenAIConnector {
    /// HTTP client
    client: Client,
    /// Configuration
    config: ConnectorConfig,
}

/// OpenAI chat request format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatRequest {
    /// Model name
    model: String,
    /// Messages in the conversation
    messages: Vec<OpenAIMessage>,
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    /// Temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// Top-p sampling (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    /// Functions that can be called by the model
    #[serde(skip_serializing_if = "Option::is_none")]
    functions: Option<Vec<OpenAIFunctionDefinition>>,
    /// Tools that can be used by the model
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAIToolDefinition>>,
}

/// OpenAI message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    /// Role of the message sender
    role: String,
    /// Content of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Name of the sender (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Function call information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<OpenAIFunctionCall>,
    /// Tool calls information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

/// OpenAI function call information
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    /// Name of the function
    name: String,
    /// Arguments to the function (as a JSON string)
    arguments: String,
}

/// OpenAI tool call information
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    /// ID of the tool call
    id: String,
    /// Type of the tool
    r#type: String,
    /// Function call information
    function: OpenAIFunctionCall,
}

/// OpenAI function definition
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionDefinition {
    /// Name of the function
    name: String,
    /// Description of the function
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Parameters schema (in JSON Schema format)
    parameters: serde_json::Value,
}

/// OpenAI tool definition
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolDefinition {
    /// Type of the tool
    r#type: String,
    /// Function definition
    function: OpenAIFunctionDefinition,
}

/// OpenAI chat response format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatResponse {
    /// ID of the completion
    id: String,
    /// Object type
    object: String,
    /// Created timestamp
    created: u64,
    /// Model name
    model: String,
    /// Choices returned by the model
    choices: Vec<OpenAIChoice>,
    /// Usage statistics
    usage: Option<OpenAIUsage>,
}

/// OpenAI choice in a chat completion response
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    /// Index of the choice
    index: usize,
    /// Message content
    message: OpenAIMessage,
    /// Reason for finishing
    finish_reason: Option<String>,
}

/// OpenAI token usage statistics
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    /// Number of tokens in the prompt
    prompt_tokens: u32,
    /// Number of tokens in the completion
    completion_tokens: u32,
    /// Total number of tokens used
    total_tokens: u32,
}

/// OpenAI streaming response format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamResponse {
    /// ID of the completion
    id: String,
    /// Object type
    object: String,
    /// Created timestamp
    created: u64,
    /// Model name
    model: String,
    /// Choices in this chunk
    choices: Vec<OpenAIStreamChoice>,
}

/// OpenAI choice in a streaming response
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamChoice {
    /// Index of the choice
    index: usize,
    /// Delta (incremental) content
    delta: OpenAIDelta,
    /// Reason for finishing
    finish_reason: Option<String>,
}

/// OpenAI delta content in a streaming response
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIDelta {
    /// Role of the message sender (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    /// Content of the message (incremental)
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Function call information (incremental)
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<OpenAIFunctionCallDelta>,
    /// Tool calls information (incremental)
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCallDelta>>,
}

/// OpenAI incremental function call information
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCallDelta {
    /// Name of the function (incremental)
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Arguments to the function (incremental)
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<String>,
}

/// OpenAI incremental tool call information
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCallDelta {
    /// ID of the tool call
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    /// Type of the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    /// Function call information (incremental)
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<OpenAIFunctionCallDelta>,
    /// Index of the tool call (for matching with previous chunks)
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
}

/// OpenAI error response
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIErrorResponse {
    /// Error information
    error: OpenAIError,
}

/// OpenAI error details
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIError {
    /// Error message
    message: String,
    /// Error type
    r#type: String,
    /// Parameter that caused the error (optional)
    param: Option<String>,
    /// Error code (optional)
    code: Option<String>,
}

/// OpenAI models list response
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIModelsResponse {
    /// Object type
    object: String,
    /// List of models
    data: Vec<OpenAIModel>,
}

/// OpenAI model information
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIModel {
    /// Model ID
    id: String,
    /// Object type
    object: String,
    /// Created timestamp
    created: u64,
    /// Owned by
    owned_by: String,
}

impl OpenAIConnector {
    /// Create a new OpenAI connector
    pub fn new(config: ConnectorConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();

        Self { client, config }
    }

    /// Convert our chat completion request to OpenAI format
    fn convert_request(&self, request: &ChatCompletionRequest) -> OpenAIChatRequest {
        // Convert messages
        let messages = request
            .messages
            .iter()
            .map(|msg| {
                let function_call = msg.function_call.as_ref().map(|fc| OpenAIFunctionCall {
                    name: fc.name.clone(),
                    arguments: fc.arguments.clone(),
                });

                let tool_calls = msg.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .map(|tc| OpenAIToolCall {
                            id: tc.id.clone(),
                            r#type: tc.r#type.clone(),
                            function: OpenAIFunctionCall {
                                name: tc.function.name.clone(),
                                arguments: tc.function.arguments.clone(),
                            },
                        })
                        .collect()
                });

                OpenAIMessage {
                    role: msg.role.to_string(),
                    content: Some(msg.content.clone()),
                    name: msg.name.clone(),
                    function_call,
                    tool_calls,
                }
            })
            .collect();

        // Convert function definitions
        let functions = request.functions.as_ref().map(|fns| {
            fns.iter()
                .map(|f| OpenAIFunctionDefinition {
                    name: f.name.clone(),
                    description: f.description.clone(),
                    parameters: f.parameters.clone(),
                })
                .collect()
        });

        // Convert tool definitions
        let tools = request.tools.as_ref().map(|ts| {
            ts.iter()
                .map(|t| OpenAIToolDefinition {
                    r#type: t.r#type.clone(),
                    function: OpenAIFunctionDefinition {
                        name: t.function.name.clone(),
                        description: t.function.description.clone(),
                        parameters: t.function.parameters.clone(),
                    },
                })
                .collect()
        });

        OpenAIChatRequest {
            model: request.model.clone(),
            messages,
            stream: request.stream,
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            functions,
            tools,
        }
    }

    /// Convert OpenAI response to our format
    fn convert_response(&self, response: OpenAIChatResponse) -> ChatCompletionResponse {
        // Convert choices
        let choices = response
            .choices
            .into_iter()
            .map(|choice| {
                let function_call = choice.message.function_call.map(|fc| FunctionCall {
                    name: fc.name,
                    arguments: fc.arguments,
                });

                let tool_calls = choice.message.tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| ToolCall {
                            id: tc.id,
                            r#type: tc.r#type,
                            function: FunctionCall {
                                name: tc.function.name,
                                arguments: tc.function.arguments,
                            },
                        })
                        .collect()
                });

                let role = match choice.message.role.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "function" => MessageRole::Function,
                    "tool" => MessageRole::Tool,
                    _ => MessageRole::User, // Default to user for unknown roles
                };

                ChatCompletionChoice {
                    index: choice.index,
                    message: ChatMessage {
                        role,
                        content: choice.message.content.unwrap_or_default(),
                        name: choice.message.name,
                        function_call,
                        tool_calls,
                    },
                    finish_reason: choice.finish_reason,
                }
            })
            .collect();

        // Convert usage
        let usage = response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        ChatCompletionResponse {
            id: response.id,
            model: response.model,
            created: response.created,
            choices,
            usage,
        }
    }

    /// Convert OpenAI streaming response to our chunk format
    fn convert_stream_chunk(&self, response: OpenAIStreamResponse) -> ChatCompletionChunk {
        // Convert choices
        let choices = response
            .choices
            .into_iter()
            .map(|choice| {
                let role = choice.delta.role.map(|r| match r.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "function" => MessageRole::Function,
                    "tool" => MessageRole::Tool,
                    _ => MessageRole::User, // Default to user for unknown roles
                });

                let function_call = choice.delta.function_call.map(|fc| FunctionCallDelta {
                    name: fc.name,
                    arguments: fc.arguments,
                });

                let tool_calls = choice.delta.tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| ToolCallDelta {
                            id: tc.id,
                            r#type: tc.r#type,
                            function: tc.function.map(|f| FunctionCallDelta {
                                name: f.name,
                                arguments: f.arguments,
                            }),
                            index: tc.index,
                        })
                        .collect()
                });

                ChatCompletionChunkChoice {
                    index: choice.index,
                    delta: ChatCompletionDelta {
                        role,
                        content: choice.delta.content,
                        function_call,
                        tool_calls,
                    },
                    finish_reason: choice.finish_reason,
                }
            })
            .collect();

        ChatCompletionChunk {
            id: response.id,
            model: response.model,
            created: response.created,
            choices,
        }
    }

    /// Build the API URL for a specific endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            endpoint
        )
    }

    /// Parse OpenAI error response
    async fn parse_error_response(
        &self,
        status: StatusCode,
        response: reqwest::Response,
    ) -> ConnectorError {
        let error_text = match response.text().await {
            Ok(text) => {
                // Try to parse as OpenAI error format
                match serde_json::from_str::<OpenAIErrorResponse>(&text) {
                    Ok(error_response) => error_response.error.message,
                    Err(_) => text,
                }
            }
            Err(_) => "Unknown error".to_string(),
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
            _ => ConnectorError::Server(format!("Server error ({}): {}", status, error_text)),
        }
    }
}

#[async_trait]
impl ModelConnector for OpenAIConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        // Convert the request to OpenAI format
        let openai_request = self.convert_request(&request);

        // Build the request
        let mut req_builder = self
            .client
            .post(self.build_url("v1/chat/completions"))
            .json(&openai_request);

        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add organization ID if available
        if let Some(org_id) = &self.config.org_id {
            req_builder = req_builder.header("OpenAI-Organization", org_id);
        }

        // Send the request to OpenAI
        let response = req_builder
            .send()
            .await
            .map_err(|e| ConnectorError::Network(format!("Failed to send request: {}", e)))?;

        // Check the response status
        let status = response.status();
        if !status.is_success() {
            return Err(self.parse_error_response(status, response).await);
        }

        // Parse the response
        let openai_response = response
            .json::<OpenAIChatResponse>()
            .await
            .map_err(|e| ConnectorError::Parsing(format!("Failed to parse response: {}", e)))?;

        // Convert the response to our format
        Ok(self.convert_response(openai_response))
    }

    async fn generate_streaming(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<StreamingResponse, ConnectorError> {
        // Convert the request to OpenAI format
        let mut openai_request = self.convert_request(&request);
        openai_request.stream = Some(true);

        // Build the request
        let mut req_builder = self
            .client
            .post(self.build_url("v1/chat/completions"))
            .json(&openai_request);

        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add organization ID if available
        if let Some(org_id) = &self.config.org_id {
            req_builder = req_builder.header("OpenAI-Organization", org_id);
        }

        // Send the request to OpenAI
        let response = req_builder
            .send()
            .await
            .map_err(|e| ConnectorError::Network(format!("Failed to send request: {}", e)))?;

        // Check the response status
        let status = response.status();
        if !status.is_success() {
            return Err(self.parse_error_response(status, response).await);
        }

        // Create a stream that processes each line from the response
        let self_clone = self.clone();

        // Process the stream using unfold to handle the response chunks
        let stream = Box::pin(stream::unfold(
            (response, self_clone),
            |(mut response, connector)| async move {
                // Read the next chunk from the response
                if let Ok(chunk) = response.chunk().await {
                    if let Some(bytes) = chunk {
                        // Convert the chunk to a string
                        let chunk_str = String::from_utf8_lossy(&bytes);

                        // OpenAI sends SSE format, each line starts with "data: "
                        // Split by lines and process each line
                        for line in chunk_str.lines() {
                            if line.starts_with("data: ") {
                                let data = line.trim_start_matches("data: ");

                                // Check for the end of the stream
                                if data == "[DONE]" {
                                    return None;
                                }

                                // Parse the JSON
                                match serde_json::from_str::<OpenAIStreamResponse>(data) {
                                    Ok(openai_response) => {
                                        // Convert to our format
                                        let result =
                                            Ok(connector.convert_stream_chunk(openai_response));
                                        return Some((result, (response, connector)));
                                    }
                                    Err(e) => {
                                        // Return parsing error
                                        let error = ConnectorError::Parsing(format!(
                                            "Failed to parse chunk: {}, data: {}",
                                            e, data
                                        ));
                                        return Some((Err(error), (response, connector)));
                                    }
                                }
                            }
                        }

                        // If we get here, we didn't find any data lines
                        // Continue to the next chunk
                        return Some((
                            Err(ConnectorError::Parsing(
                                "No data found in chunk".to_string(),
                            )),
                            (response, connector),
                        ));
                    } else {
                        // End of stream
                        return None;
                    }
                } else {
                    // Error reading from stream
                    let error = ConnectorError::Network("Error reading from stream".to_string());
                    return Some((Err(error), (response, connector)));
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
        "openai"
    }

    fn supports_model(&self, model_id: &str) -> bool {
        // Check if the model is supported by OpenAI
        // This is a basic check for common OpenAI models
        // A more comprehensive check would query the OpenAI API
        model_id.starts_with("gpt-")
            || model_id.starts_with("text-")
            || model_id.starts_with("davinci")
            || model_id.starts_with("curie")
            || model_id.starts_with("babbage")
            || model_id.starts_with("ada")
            || model_id.starts_with("claude")
            || model_id.starts_with("gemini")
            || model_id.starts_with("mistral")
            || model_id.starts_with("llama")
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        // Build the request
        let mut req_builder = self.client.get(self.build_url("v1/models"));

        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add organization ID if available
        if let Some(org_id) = &self.config.org_id {
            req_builder = req_builder.header("OpenAI-Organization", org_id);
        }

        // Send the request to OpenAI
        let response = req_builder
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
            .json::<OpenAIModelsResponse>()
            .await
            .map_err(|e| ConnectorError::Parsing(format!("Failed to parse models: {}", e)))?;

        // Extract model IDs
        let model_ids = models_response
            .data
            .into_iter()
            .map(|model| model.id)
            .collect();

        Ok(model_ids)
    }
}

// Implement Clone for OpenAIConnector
impl Clone for OpenAIConnector {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
        }
    }
}

/// Factory for creating OpenAI connectors
pub struct OpenAIConnectorFactory;

impl ModelConnectorFactory for OpenAIConnectorFactory {
    fn create_connector(&self, config: ConnectorConfig) -> Arc<dyn ModelConnector> {
        Arc::new(OpenAIConnector::new(config))
    }

    fn provider_name(&self) -> &'static str {
        "openai"
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
            base_url: "https://api.openai.com".to_string(),
            api_key: Some("test-api-key".to_string()),
            org_id: None,
            timeout_secs: 30,
            max_retries: 3,
            additional_config: Default::default(),
        };

        let connector = OpenAIConnector::new(config);

        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
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

        let openai_request = connector.convert_request(&request);

        assert_eq!(openai_request.model, "gpt-4");
        assert_eq!(openai_request.messages.len(), 2);
        assert_eq!(openai_request.messages[0].role, "system");
        assert_eq!(
            openai_request.messages[0].content,
            Some("You are a helpful assistant.".to_string())
        );
        assert_eq!(openai_request.messages[1].role, "user");
        assert_eq!(
            openai_request.messages[1].content,
            Some("Hello, how are you?".to_string())
        );
        assert_eq!(openai_request.stream, Some(false));
        assert_eq!(openai_request.temperature, Some(0.7));
        assert_eq!(openai_request.top_p, Some(0.9));
        assert_eq!(openai_request.max_tokens, Some(100));
    }

    #[test]
    fn test_convert_response() {
        let config = ConnectorConfig {
            base_url: "https://api.openai.com".to_string(),
            api_key: Some("test-api-key".to_string()),
            org_id: None,
            timeout_secs: 30,
            max_retries: 3,
            additional_config: Default::default(),
        };

        let connector = OpenAIConnector::new(config);

        let openai_response = OpenAIChatResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some("I'm doing well, thank you for asking!".to_string()),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(OpenAIUsage {
                prompt_tokens: 9,
                completion_tokens: 12,
                total_tokens: 21,
            }),
        };

        let response = connector.convert_response(openai_response);

        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.created, 1677652288);
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert_eq!(
            response.choices[0].message.content,
            "I'm doing well, thank you for asking!"
        );
        assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().prompt_tokens, 9);
        assert_eq!(response.usage.unwrap().completion_tokens, 12);
        assert_eq!(response.usage.unwrap().total_tokens, 21);
    }
}
