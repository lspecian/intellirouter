//! Tests for the model connector interface

use super::*;
use futures::StreamExt;
use mockito::{mock, server_url};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Mock connector for testing
struct MockConnector {
    config: Mutex<ConnectorConfig>,
    provider: &'static str,
    supported_models: Vec<String>,
}

impl MockConnector {
    fn new(provider: &'static str, supported_models: Vec<String>) -> Self {
        Self {
            config: Mutex::new(ConnectorConfig {
                base_url: "https://api.example.com".to_string(),
                api_key: Some("test-api-key".to_string()),
                org_id: None,
                timeout_secs: 30,
                max_retries: 3,
                additional_config: HashMap::new(),
            }),
            provider,
            supported_models,
        }
    }
}

#[async_trait]
impl ModelConnector for MockConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        // Simple mock implementation that echoes back the last message
        let last_message = request
            .messages
            .last()
            .ok_or_else(|| ConnectorError::InvalidRequest("No messages provided".to_string()))?;

        let response = ChatCompletionResponse {
            id: "mock-completion-id".to_string(),
            model: request.model,
            created: chrono::Utc::now().timestamp() as u64,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: format!("Echo: {}", last_message.content),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 10,
                total_tokens: 20,
            }),
        };

        Ok(response)
    }

    async fn generate_streaming(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<StreamingResponse, ConnectorError> {
        // Simple mock implementation that returns a stream with a single chunk
        let last_message = request
            .messages
            .last()
            .ok_or_else(|| ConnectorError::InvalidRequest("No messages provided".to_string()))?;

        let chunk = ChatCompletionChunk {
            id: "mock-completion-id".to_string(),
            model: request.model.clone(),
            created: chrono::Utc::now().timestamp() as u64,
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionDelta {
                    role: Some(MessageRole::Assistant),
                    content: Some(format!("Echo: {}", last_message.content)),
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        // Create a stream with a single chunk
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    fn get_config(&self) -> &ConnectorConfig {
        // This is a bit of a hack for testing purposes
        // In a real implementation, we would use a proper synchronization mechanism
        // or store the config in an Arc
        static CONFIG: std::sync::OnceLock<ConnectorConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ConnectorConfig {
            base_url: "https://api.example.com".to_string(),
            api_key: Some("test-api-key".to_string()),
            org_id: None,
            timeout_secs: 30,
            max_retries: 3,
            additional_config: HashMap::new(),
        })
    }

    fn update_config(&mut self, config: ConnectorConfig) {
        // For testing purposes, we'll just create a new Mutex with the new config
        self.config = Mutex::new(config);
    }

    fn provider_name(&self) -> &'static str {
        self.provider
    }

    fn supports_model(&self, model_id: &str) -> bool {
        self.supported_models.contains(&model_id.to_string())
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        Ok(self.supported_models.clone())
    }
}

// Mock connector factory for testing
struct MockConnectorFactory {
    provider: &'static str,
    supported_models: Vec<String>,
}

impl MockConnectorFactory {
    fn new(provider: &'static str, supported_models: Vec<String>) -> Self {
        Self {
            provider,
            supported_models,
        }
    }
}

impl ModelConnectorFactory for MockConnectorFactory {
    fn create_connector(&self, config: ConnectorConfig) -> Arc<dyn ModelConnector> {
        let mut connector = MockConnector::new(self.provider, self.supported_models.clone());
        connector.update_config(config);
        Arc::new(connector)
    }

    fn provider_name(&self) -> &'static str {
        self.provider
    }
}

#[tokio::test]
async fn test_mock_connector() {
    // Create a mock connector
    let connector = MockConnector::new(
        "mock-provider",
        vec!["mock-model-1".to_string(), "mock-model-2".to_string()],
    );

    // Test generate
    let request = ChatCompletionRequest {
        model: "mock-model-1".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello, world!".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: None,
        functions: None,
        tools: None,
        additional_params: None,
    };

    let response = connector.generate(request.clone()).await.unwrap();
    assert_eq!(response.choices[0].message.content, "Echo: Hello, world!");

    // Test generate_streaming
    let stream = connector.generate_streaming(request).await.unwrap();
    let chunks: Vec<_> = futures::StreamExt::collect(stream).await;
    assert_eq!(chunks.len(), 1);
    let chunk = chunks[0].as_ref().unwrap();
    assert_eq!(
        chunk.choices[0].delta.content.as_ref().unwrap(),
        "Echo: Hello, world!"
    );

    // Test provider_name
    assert_eq!(connector.provider_name(), "mock-provider");

    // Test supports_model
    assert!(connector.supports_model("mock-model-1"));
    assert!(connector.supports_model("mock-model-2"));
    assert!(!connector.supports_model("unknown-model"));

    // Test list_models
    let models = connector.list_models().await.unwrap();
    assert_eq!(
        models,
        vec!["mock-model-1".to_string(), "mock-model-2".to_string()]
    );
}

#[tokio::test]
async fn test_mock_connector_factory() {
    // Create a mock connector factory
    let factory = MockConnectorFactory::new(
        "mock-provider",
        vec!["mock-model-1".to_string(), "mock-model-2".to_string()],
    );

    // Create a connector
    let config = ConnectorConfig {
        base_url: "https://api.example.com".to_string(),
        api_key: Some("test-api-key".to_string()),
        org_id: None,
        timeout_secs: 30,
        max_retries: 3,
        additional_config: HashMap::new(),
    };

    let connector = factory.create_connector(config);

    // Test provider_name
    assert_eq!(factory.provider_name(), "mock-provider");
    assert_eq!(connector.provider_name(), "mock-provider");

    // Test supports_model
    assert!(connector.supports_model("mock-model-1"));
    assert!(!connector.supports_model("unknown-model"));
}
