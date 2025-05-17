//! Tests for streaming functionality in model connectors

use super::*;
use futures::StreamExt;
use mockito::{mock, server_url};
use std::time::Duration;

/// Test successful streaming for Ollama connector
#[tokio::test]
async fn test_ollama_streaming_success() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"Hello"},"done":false}
{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":" world"},"done":false}
{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"!"},"done":true}"#,
        )
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
            content: "Say hello".to_string(),
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

    // We should have 3 chunks
    assert_eq!(chunks.len(), 3);
    assert!(chunks[0].is_ok());
    assert!(chunks[1].is_ok());
    assert!(chunks[2].is_ok());

    // Check content of chunks
    let chunk1 = chunks[0].as_ref().unwrap();
    let chunk2 = chunks[1].as_ref().unwrap();
    let chunk3 = chunks[2].as_ref().unwrap();

    assert_eq!(
        chunk1.choices[0].delta.content.as_ref().unwrap(),
        "Hello"
    );
    assert_eq!(
        chunk2.choices[0].delta.content.as_ref().unwrap(),
        " world"
    );
    assert_eq!(
        chunk3.choices[0].delta.content.as_ref().unwrap(),
        "!"
    );

    // First chunk should have role, last should have finish_reason
    assert_eq!(
        chunk1.choices[0].delta.role.as_ref().unwrap(),
        &MessageRole::Assistant
    );
    assert!(chunk3.choices[0].finish_reason.is_some());
    assert_eq!(chunk3.choices[0].finish_reason.as_ref().unwrap(), "stop");

    mock.assert();
}

/// Test streaming with partial JSON for Ollama connector
#[tokio::test]
async fn test_ollama_streaming_partial_json() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"Hello"},"done":false}
{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":" world"},"done":false}
{"model":"llama2","created_at":"2023-01-01T00:00:00Z","message":{"role":"assistant","content":"!"},"done":true}"#,
        )
        .with_chunked_body(|body| {
            // Split the body into smaller chunks to simulate partial JSON
            let bytes = body.as_bytes();
            let chunk_size = bytes.len() / 5;
            let mut chunks = Vec::new();
            
            for i in 0..5 {
                let start = i * chunk_size;
                let end = if i == 4 { bytes.len() } else { (i + 1) * chunk_size };
                chunks.push(bytes[start..end].to_vec());
            }
            
            chunks
        })
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
            content: "Say hello".to_string(),
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

    // We should have at least one chunk
    assert!(!chunks.is_empty());
    
    // Some chunks might be errors due to partial JSON, but we should have at least one success
    let success_count = chunks.iter().filter(|chunk| chunk.is_ok()).count();
    assert!(success_count > 0);

    mock.assert();
}

/// Test successful streaming for OpenAI connector
#[tokio::test]
async fn test_openai_streaming_success() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":" world"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":"stop"}]}

data: [DONE]
"#,
        )
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
            content: "Say hello".to_string(),
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

    // We should have 4 chunks (role, "Hello", " world", "!" with finish_reason)
    assert_eq!(chunks.len(), 4);
    assert!(chunks[0].is_ok());
    assert!(chunks[1].is_ok());
    assert!(chunks[2].is_ok());
    assert!(chunks[3].is_ok());

    // Check content of chunks
    let chunk1 = chunks[0].as_ref().unwrap();
    let chunk2 = chunks[1].as_ref().unwrap();
    let chunk3 = chunks[2].as_ref().unwrap();
    let chunk4 = chunks[3].as_ref().unwrap();

    // First chunk should have role
    assert_eq!(
        chunk1.choices[0].delta.role.as_ref().unwrap(),
        &MessageRole::Assistant
    );
    
    // Content chunks
    assert_eq!(
        chunk2.choices[0].delta.content.as_ref().unwrap(),
        "Hello"
    );
    assert_eq!(
        chunk3.choices[0].delta.content.as_ref().unwrap(),
        " world"
    );
    assert_eq!(
        chunk4.choices[0].delta.content.as_ref().unwrap(),
        "!"
    );

    // Last chunk should have finish_reason
    assert!(chunk4.choices[0].finish_reason.is_some());
    assert_eq!(chunk4.choices[0].finish_reason.as_ref().unwrap(), "stop");

    mock.assert();
}

/// Test streaming with function calls for OpenAI connector
#[tokio::test]
async fn test_openai_streaming_function_calls() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"function_call":{"name":"get_weather"}},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"function_call":{"arguments":"{\n"}},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"function_call":{"arguments":"  \"location\": \"San Francisco\""}},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"function_call":{"arguments":"}"}},"finish_reason":"function_call"}]}

data: [DONE]
"#,
        )
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
            content: "What's the weather in San Francisco?".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: Some(true),
        functions: Some(vec![FunctionDefinition {
            name: "get_weather".to_string(),
            description: Some("Get the weather for a location".to_string()),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The location to get weather for"
                    }
                },
                "required": ["location"]
            }),
        }]),
        tools: None,
        additional_params: None,
    };

    let result = connector.generate_streaming(request).await;
    assert!(result.is_ok());

    let stream = result.unwrap();
    let chunks: Vec<_> = stream.collect().await;

    // We should have 5 chunks
    assert_eq!(chunks.len(), 5);
    
    // Check function call in chunks
    let chunk2 = chunks[1].as_ref().unwrap();
    let chunk3 = chunks[2].as_ref().unwrap();
    let chunk4 = chunks[3].as_ref().unwrap();
    let chunk5 = chunks[4].as_ref().unwrap();
    
    // Function name in first function call chunk
    assert!(chunk2.choices[0].delta.function_call.is_some());
    assert_eq!(
        chunk2.choices[0].delta.function_call.as_ref().unwrap().name.as_ref().unwrap(),
        "get_weather"
    );
    
    // Arguments built up across chunks
    assert!(chunk3.choices[0].delta.function_call.is_some());
    assert_eq!(
        chunk3.choices[0].delta.function_call.as_ref().unwrap().arguments.as_ref().unwrap(),
        "{\n"
    );
    
    assert!(chunk4.choices[0].delta.function_call.is_some());
    assert_eq!(
        chunk4.choices[0].delta.function_call.as_ref().unwrap().arguments.as_ref().unwrap(),
        "  \"location\": \"San Francisco\""
    );
    
    assert!(chunk5.choices[0].delta.function_call.is_some());
    assert_eq!(
        chunk5.choices[0].delta.function_call.as_ref().unwrap().arguments.as_ref().unwrap(),
        "}"
    );
    
    // Last chunk should have finish_reason
    assert!(chunk5.choices[0].finish_reason.is_some());
    assert_eq!(chunk5.choices[0].finish_reason.as_ref().unwrap(), "function_call");

    mock.assert();
}

/// Test streaming with tool calls for OpenAI connector
#[tokio::test]
async fn test_openai_streaming_tool_calls() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_abc123","type":"function","function":{"name":"get_weather"}}]},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\n"}}]},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"  \"location\": \"San Francisco\""}}]},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"}"}}]},"finish_reason":"tool_calls"}]}

data: [DONE]
"#,
        )
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
            content: "What's the weather in San Francisco?".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: Some(true),
        functions: None,
        tools: Some(vec![ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get the weather for a location".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The location to get weather for"
                        }
                    },
                    "required": ["location"]
                }),
            },
        }]),
        additional_params: None,
    };

    let result = connector.generate_streaming(request).await;
    assert!(result.is_ok());

    let stream = result.unwrap();
    let chunks: Vec<_> = stream.collect().await;

    // We should have 5 chunks
    assert_eq!(chunks.len(), 5);
    
    // Check tool calls in chunks
    let chunk2 = chunks[1].as_ref().unwrap();
    let chunk3 = chunks[2].as_ref().unwrap();
    let chunk4 = chunks[3].as_ref().unwrap();
    let chunk5 = chunks[4].as_ref().unwrap();
    
    // Tool call details in first tool call chunk
    assert!(chunk2.choices[0].delta.tool_calls.is_some());
    let tool_calls2 = chunk2.choices[0].delta.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls2.len(), 1);
    assert_eq!(tool_calls2[0].id.as_ref().unwrap(), "call_abc123");
    assert_eq!(tool_calls2[0].r#type.as_ref().unwrap(), "function");
    assert!(tool_calls2[0].function.is_some());
    assert_eq!(
        tool_calls2[0].function.as_ref().unwrap().name.as_ref().unwrap(),
        "get_weather"
    );
    
    // Arguments built up across chunks
    assert!(chunk3.choices[0].delta.tool_calls.is_some());
    let tool_calls3 = chunk3.choices[0].delta.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls3.len(), 1);
    assert!(tool_calls3[0].function.is_some());
    assert_eq!(
        tool_calls3[0].function.as_ref().unwrap().arguments.as_ref().unwrap(),
        "{\n"
    );
    
    // Last chunk should have finish_reason
    assert!(chunk5.choices[0].finish_reason.is_some());
    assert_eq!(chunk5.choices[0].finish_reason.as_ref().unwrap(), "tool_calls");

    mock.assert();
}