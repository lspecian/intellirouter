//! Tests for error handling in model connectors

use super::*;
use mockito::{mock, server_url};
use std::time::Duration;

/// Test authentication errors for Ollama connector
#[tokio::test]
async fn test_ollama_authentication_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Unauthorized access"}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: Some("invalid-key".to_string()),
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::Authentication(msg)) => {
            assert!(msg.contains("Unauthorized"));
        }
        _ => panic!("Expected Authentication error"),
    }

    mock.assert();
}

/// Test rate limit errors for Ollama connector
#[tokio::test]
async fn test_ollama_rate_limit_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Rate limit exceeded"}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::RateLimit(msg)) => {
            assert!(msg.contains("Rate limit"));
        }
        _ => panic!("Expected RateLimit error"),
    }

    mock.assert();
}

/// Test model not found errors for Ollama connector
#[tokio::test]
async fn test_ollama_model_not_found_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Model not found"}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "nonexistent-model".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::ModelNotFound(msg)) => {
            assert!(msg.contains("Model not found"));
        }
        _ => panic!("Expected ModelNotFound error"),
    }

    mock.assert();
}

/// Test invalid request errors for Ollama connector
#[tokio::test]
async fn test_ollama_invalid_request_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Invalid request parameters"}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![], // Empty messages array is invalid
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: None,
        functions: None,
        tools: None,
        additional_params: None,
    };

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::InvalidRequest(msg)) => {
            assert!(msg.contains("Invalid request"));
        }
        _ => panic!("Expected InvalidRequest error"),
    }

    mock.assert();
}

/// Test server errors for Ollama connector
#[tokio::test]
async fn test_ollama_server_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal server error"}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::Server(msg)) => {
            assert!(msg.contains("Server error"));
        }
        _ => panic!("Expected Server error"),
    }

    mock.assert();
}

/// Test timeout errors for Ollama connector
#[tokio::test]
async fn test_ollama_timeout_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_delay(Duration::from_secs(2))  // Delay longer than timeout
        .with_body(r#"{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"Hello there"},"done":true}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1, // 1 second timeout
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::Network(msg)) => {
            assert!(msg.contains("timeout") || msg.contains("timed out"));
        }
        _ => panic!("Expected Network error with timeout"),
    }
}

/// Test parsing errors for Ollama connector
#[tokio::test]
async fn test_ollama_parsing_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"invalid_json: this is not valid JSON"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::Parsing(msg)) => {
            assert!(msg.contains("Failed to parse"));
        }
        _ => panic!("Expected Parsing error"),
    }

    mock.assert();
}

/// Test retry logic for network errors
#[tokio::test]
async fn test_ollama_retry_logic() {
    let mut server = mockito::Server::new();

    // First request fails with a 503 Service Unavailable
    let mock1 = server
        .mock("POST", "/api/chat")
        .with_status(503)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Service temporarily unavailable"}"#)
        .create();

    // Second request succeeds
    let mock2 = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"Hello there"},"done":true}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 1, // Allow 1 retry
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_ok());

    mock1.assert();
    mock2.assert();
}

/// Test streaming error handling
#[tokio::test]
async fn test_ollama_streaming_error_handling() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"Hello"},"done":false}
{"error": "Stream interrupted"}
{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":" there"},"done":true}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: None,
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);
    let request = ChatCompletionRequest {
        model: "llama2".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: Some(true),
        functions: None,
        tools: None,
        additional_params: None,
    };

    let result = connector.generate_streaming(request).await;
    assert!(result.is_ok());

    let stream = result.unwrap();
    let chunks: Vec<_> = stream.collect().await;

    // We should have at least one successful chunk and one error
    assert!(chunks.len() >= 2);
    assert!(chunks[0].is_ok());

    // At least one chunk should be an error
    let has_error = chunks.iter().any(|chunk| chunk.is_err());
    assert!(has_error);

    mock.assert();
}

/// Test authentication errors for OpenAI connector
#[tokio::test]
async fn test_openai_authentication_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"message":"Invalid API key","type":"authentication_error","param":null,"code":null}}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: Some("invalid-key".to_string()),
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OpenAIConnector::new(config);
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::Authentication(msg)) => {
            assert!(msg.contains("Invalid API key"));
        }
        _ => panic!("Expected Authentication error"),
    }

    mock.assert();
}

/// Test rate limit errors for OpenAI connector
#[tokio::test]
async fn test_openai_rate_limit_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"message":"Rate limit exceeded","type":"rate_limit_error","param":null,"code":null}}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: Some("test-key".to_string()),
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OpenAIConnector::new(config);
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::RateLimit(msg)) => {
            assert!(msg.contains("Rate limit"));
        }
        _ => panic!("Expected RateLimit error"),
    }

    mock.assert();
}

/// Test model not found errors for OpenAI connector
#[tokio::test]
async fn test_openai_model_not_found_error() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"message":"Model not found","type":"invalid_request_error","param":"model","code":"model_not_found"}}"#)
        .create();

    let config = ConnectorConfig {
        base_url: server.url(),
        api_key: Some("test-key".to_string()),
        org_id: None,
        timeout_secs: 1,
        max_retries: 0,
        additional_config: Default::default(),
    };

    let connector = OpenAIConnector::new(config);
    let request = ChatCompletionRequest {
        model: "nonexistent-model".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
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

    let result = connector.generate(request).await;
    assert!(result.is_err());
    match result {
        Err(ConnectorError::ModelNotFound(msg)) => {
            assert!(msg.contains("Model not found"));
        }
        _ => panic!("Expected ModelNotFound error"),
    }

    mock.assert();
}
