//! Tests for request/response conversion in model connectors

use super::*;
use serde_json::json;

/// Test request conversion for Ollama connector
#[test]
fn test_ollama_request_conversion() {
    let config = ConnectorConfig {
        base_url: "http://localhost:11434".to_string(),
        api_key: None,
        org_id: None,
        timeout_secs: 30,
        max_retries: 3,
        additional_config: Default::default(),
    };

    let connector = OllamaConnector::new(config);

    // Create a complex request with various message types and parameters
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
                name: Some("user1".to_string()),
                function_call: None,
                tool_calls: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "I'm doing well, thank you!".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            },
            ChatMessage {
                role: MessageRole::Function,
                content: "Function result".to_string(),
                name: Some("get_time".to_string()),
                function_call: None,
                tool_calls: None,
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: "Tool result".to_string(),
                name: Some("calculator".to_string()),
                function_call: None,
                tool_calls: None,
            },
        ],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: Some(true),
        functions: None,
        tools: None,
        additional_params: None,
    };

    // Convert the request to Ollama format
    let ollama_request = connector.convert_request(&request);

    // Verify the conversion
    assert_eq!(ollama_request.model, "llama2");
    assert_eq!(ollama_request.messages.len(), 5);
    assert_eq!(ollama_request.stream, true);

    // Check message roles
    assert_eq!(ollama_request.messages[0].role, "system");
    assert_eq!(ollama_request.messages[1].role, "user");
    assert_eq!(ollama_request.messages[2].role, "assistant");
    assert_eq!(ollama_request.messages[3].role, "user"); // Function converted to user
    assert_eq!(ollama_request.messages[4].role, "user"); // Tool converted to user

    // Check message content
    assert_eq!(
        ollama_request.messages[0].content,
        "You are a helpful assistant."
    );
    assert_eq!(ollama_request.messages[1].content, "Hello, how are you?");
    assert_eq!(
        ollama_request.messages[2].content,
        "I'm doing well, thank you!"
    );
    assert_eq!(ollama_request.messages[3].content, "Function result");
    assert_eq!(ollama_request.messages[4].content, "Tool result");

    // Check options
    assert_eq!(ollama_request.options.unwrap().temperature, Some(0.7));
    assert_eq!(ollama_request.options.unwrap().top_p, Some(0.9));
    assert_eq!(ollama_request.options.unwrap().num_predict, Some(100));
}

/// Test response conversion for Ollama connector
#[test]
fn test_ollama_response_conversion() {
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

    // Create an Ollama response
    let ollama_response = OllamaChatResponse {
        model: "llama2".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        message: OllamaMessage {
            role: "assistant".to_string(),
            content: "I'm an AI assistant. How can I help you today?".to_string(),
        },
        done: true,
    };

    // Convert the response
    let response = connector.convert_response(ollama_response, request_id);

    // Verify the conversion
    assert_eq!(response.id, request_id);
    assert_eq!(response.model, "llama2");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].index, 0);
    assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
    assert_eq!(
        response.choices[0].message.content,
        "I'm an AI assistant. How can I help you today?"
    );
    assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));

    // Check usage (Ollama doesn't provide token counts)
    assert!(response.usage.is_some());
    assert_eq!(response.usage.unwrap().prompt_tokens, 0);
    assert_eq!(response.usage.unwrap().completion_tokens, 0);
    assert_eq!(response.usage.unwrap().total_tokens, 0);
}

/// Test streaming chunk conversion for Ollama connector
#[test]
fn test_ollama_streaming_chunk_conversion() {
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

    // Test first chunk (not done)
    let first_chunk = OllamaChatResponse {
        model: "llama2".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        message: OllamaMessage {
            role: "assistant".to_string(),
            content: "Hello".to_string(),
        },
        done: false,
    };

    let converted_first = connector.convert_stream_chunk(first_chunk, request_id);
    assert_eq!(converted_first.id, request_id);
    assert_eq!(converted_first.model, "llama2");
    assert_eq!(converted_first.choices.len(), 1);
    assert_eq!(converted_first.choices[0].index, 0);
    assert_eq!(
        converted_first.choices[0].delta.role,
        Some(MessageRole::Assistant)
    );
    assert_eq!(
        converted_first.choices[0].delta.content,
        Some("Hello".to_string())
    );
    assert_eq!(converted_first.choices[0].finish_reason, None);

    // Test last chunk (done)
    let last_chunk = OllamaChatResponse {
        model: "llama2".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        message: OllamaMessage {
            role: "assistant".to_string(),
            content: " world!".to_string(),
        },
        done: true,
    };

    let converted_last = connector.convert_stream_chunk(last_chunk, request_id);
    assert_eq!(converted_last.id, request_id);
    assert_eq!(converted_last.model, "llama2");
    assert_eq!(converted_last.choices.len(), 1);
    assert_eq!(converted_last.choices[0].index, 0);
    assert_eq!(converted_last.choices[0].delta.role, None);
    assert_eq!(
        converted_last.choices[0].delta.content,
        Some(" world!".to_string())
    );
    assert_eq!(
        converted_last.choices[0].finish_reason,
        Some("stop".to_string())
    );
}

/// Test request conversion for OpenAI connector
#[test]
fn test_openai_request_conversion() {
    let config = ConnectorConfig {
        base_url: "https://api.openai.com".to_string(),
        api_key: Some("test-api-key".to_string()),
        org_id: None,
        timeout_secs: 30,
        max_retries: 3,
        additional_config: Default::default(),
    };

    let connector = OpenAIConnector::new(config);

    // Create a complex request with various message types and parameters
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
                name: Some("user1".to_string()),
                function_call: None,
                tool_calls: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "I'm doing well, thank you!".to_string(),
                name: None,
                function_call: Some(FunctionCall {
                    name: "get_time".to_string(),
                    arguments: "{\"timezone\": \"UTC\"}".to_string(),
                }),
                tool_calls: None,
            },
            ChatMessage {
                role: MessageRole::Function,
                content: "The current time is 12:00 PM".to_string(),
                name: Some("get_time".to_string()),
                function_call: None,
                tool_calls: None,
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: "The result is 42".to_string(),
                name: Some("calculator".to_string()),
                function_call: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_abc123".to_string(),
                    r#type: "function".to_string(),
                    function: FunctionCall {
                        name: "calculate".to_string(),
                        arguments: "{\"expression\": \"6 * 7\"}".to_string(),
                    },
                }]),
            },
        ],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: Some(false),
        functions: Some(vec![FunctionDefinition {
            name: "get_time".to_string(),
            description: Some("Get the current time in a specific timezone".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "timezone": {
                        "type": "string",
                        "description": "The timezone to get the time for"
                    }
                },
                "required": ["timezone"]
            }),
        }]),
        tools: Some(vec![ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "calculate".to_string(),
                description: Some("Calculate a mathematical expression".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "The expression to calculate"
                        }
                    },
                    "required": ["expression"]
                }),
            },
        }]),
        additional_params: None,
    };

    // Convert the request to OpenAI format
    let openai_request = connector.convert_request(&request);

    // Verify the conversion
    assert_eq!(openai_request.model, "gpt-4");
    assert_eq!(openai_request.messages.len(), 5);
    assert_eq!(openai_request.stream, Some(false));
    assert_eq!(openai_request.temperature, Some(0.7));
    assert_eq!(openai_request.top_p, Some(0.9));
    assert_eq!(openai_request.max_tokens, Some(100));

    // Check message roles
    assert_eq!(openai_request.messages[0].role, "system");
    assert_eq!(openai_request.messages[1].role, "user");
    assert_eq!(openai_request.messages[2].role, "assistant");
    assert_eq!(openai_request.messages[3].role, "function");
    assert_eq!(openai_request.messages[4].role, "tool");

    // Check message content
    assert_eq!(
        openai_request.messages[0].content,
        Some("You are a helpful assistant.".to_string())
    );
    assert_eq!(
        openai_request.messages[1].content,
        Some("Hello, how are you?".to_string())
    );
    assert_eq!(
        openai_request.messages[2].content,
        Some("I'm doing well, thank you!".to_string())
    );
    assert_eq!(
        openai_request.messages[3].content,
        Some("The current time is 12:00 PM".to_string())
    );
    assert_eq!(
        openai_request.messages[4].content,
        Some("The result is 42".to_string())
    );

    // Check names
    assert_eq!(openai_request.messages[1].name, Some("user1".to_string()));
    assert_eq!(
        openai_request.messages[3].name,
        Some("get_time".to_string())
    );
    assert_eq!(
        openai_request.messages[4].name,
        Some("calculator".to_string())
    );

    // Check function call
    assert!(openai_request.messages[2].function_call.is_some());
    assert_eq!(
        openai_request.messages[2]
            .function_call
            .as_ref()
            .unwrap()
            .name,
        "get_time"
    );
    assert_eq!(
        openai_request.messages[2]
            .function_call
            .as_ref()
            .unwrap()
            .arguments,
        "{\"timezone\": \"UTC\"}"
    );

    // Check tool calls
    assert!(openai_request.messages[4].tool_calls.is_some());
    assert_eq!(
        openai_request.messages[4]
            .tool_calls
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        openai_request.messages[4].tool_calls.as_ref().unwrap()[0].id,
        "call_abc123"
    );
    assert_eq!(
        openai_request.messages[4].tool_calls.as_ref().unwrap()[0].r#type,
        "function"
    );
    assert_eq!(
        openai_request.messages[4].tool_calls.as_ref().unwrap()[0]
            .function
            .name,
        "calculate"
    );
    assert_eq!(
        openai_request.messages[4].tool_calls.as_ref().unwrap()[0]
            .function
            .arguments,
        "{\"expression\": \"6 * 7\"}"
    );

    // Check functions
    assert!(openai_request.functions.is_some());
    assert_eq!(openai_request.functions.as_ref().unwrap().len(), 1);
    assert_eq!(
        openai_request.functions.as_ref().unwrap()[0].name,
        "get_time"
    );
    assert_eq!(
        openai_request.functions.as_ref().unwrap()[0].description,
        Some("Get the current time in a specific timezone".to_string())
    );

    // Check tools
    assert!(openai_request.tools.is_some());
    assert_eq!(openai_request.tools.as_ref().unwrap().len(), 1);
    assert_eq!(openai_request.tools.as_ref().unwrap()[0].r#type, "function");
    assert_eq!(
        openai_request.tools.as_ref().unwrap()[0].function.name,
        "calculate"
    );
    assert_eq!(
        openai_request.tools.as_ref().unwrap()[0]
            .function
            .description,
        Some("Calculate a mathematical expression".to_string())
    );
}

/// Test response conversion for OpenAI connector
#[test]
fn test_openai_response_conversion() {
    let config = ConnectorConfig {
        base_url: "https://api.openai.com".to_string(),
        api_key: Some("test-api-key".to_string()),
        org_id: None,
        timeout_secs: 30,
        max_retries: 3,
        additional_config: Default::default(),
    };

    let connector = OpenAIConnector::new(config);

    // Create an OpenAI response
    let openai_response = OpenAIChatResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1677652288,
        model: "gpt-4".to_string(),
        choices: vec![OpenAIChoice {
            index: 0,
            message: OpenAIMessage {
                role: "assistant".to_string(),
                content: Some("I'm an AI assistant. How can I help you today?".to_string()),
                name: None,
                function_call: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Some(OpenAIUsage {
            prompt_tokens: 10,
            completion_tokens: 15,
            total_tokens: 25,
        }),
    };

    // Convert the response
    let response = connector.convert_response(openai_response);

    // Verify the conversion
    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "gpt-4");
    assert_eq!(response.created, 1677652288);
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].index, 0);
    assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
    assert_eq!(
        response.choices[0].message.content,
        "I'm an AI assistant. How can I help you today?"
    );
    assert_eq!(response.choices[0].finish_reason, Some("stop".to_string()));

    // Check usage
    assert!(response.usage.is_some());
    assert_eq!(response.usage.as_ref().unwrap().prompt_tokens, 10);
    assert_eq!(response.usage.as_ref().unwrap().completion_tokens, 15);
    assert_eq!(response.usage.as_ref().unwrap().total_tokens, 25);
}

/// Test streaming chunk conversion for OpenAI connector
#[test]
fn test_openai_streaming_chunk_conversion() {
    let config = ConnectorConfig {
        base_url: "https://api.openai.com".to_string(),
        api_key: Some("test-api-key".to_string()),
        org_id: None,
        timeout_secs: 30,
        max_retries: 3,
        additional_config: Default::default(),
    };

    let connector = OpenAIConnector::new(config);

    // Test first chunk with role
    let first_chunk = OpenAIStreamResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1677652288,
        model: "gpt-4".to_string(),
        choices: vec![OpenAIStreamChoice {
            index: 0,
            delta: OpenAIDelta {
                role: Some("assistant".to_string()),
                content: None,
                function_call: None,
                tool_calls: None,
            },
            finish_reason: None,
        }],
    };

    let converted_first = connector.convert_stream_chunk(first_chunk);
    assert_eq!(converted_first.id, "chatcmpl-123");
    assert_eq!(converted_first.model, "gpt-4");
    assert_eq!(converted_first.created, 1677652288);
    assert_eq!(converted_first.choices.len(), 1);
    assert_eq!(converted_first.choices[0].index, 0);
    assert_eq!(
        converted_first.choices[0].delta.role,
        Some(MessageRole::Assistant)
    );
    assert_eq!(converted_first.choices[0].delta.content, None);
    assert_eq!(converted_first.choices[0].finish_reason, None);

    // Test content chunk
    let content_chunk = OpenAIStreamResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1677652288,
        model: "gpt-4".to_string(),
        choices: vec![OpenAIStreamChoice {
            index: 0,
            delta: OpenAIDelta {
                role: None,
                content: Some("Hello world".to_string()),
                function_call: None,
                tool_calls: None,
            },
            finish_reason: None,
        }],
    };

    let converted_content = connector.convert_stream_chunk(content_chunk);
    assert_eq!(converted_content.id, "chatcmpl-123");
    assert_eq!(converted_content.model, "gpt-4");
    assert_eq!(converted_content.choices.len(), 1);
    assert_eq!(converted_content.choices[0].index, 0);
    assert_eq!(converted_content.choices[0].delta.role, None);
    assert_eq!(
        converted_content.choices[0].delta.content,
        Some("Hello world".to_string())
    );
    assert_eq!(converted_content.choices[0].finish_reason, None);

    // Test function call chunk
    let function_chunk = OpenAIStreamResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1677652288,
        model: "gpt-4".to_string(),
        choices: vec![OpenAIStreamChoice {
            index: 0,
            delta: OpenAIDelta {
                role: None,
                content: None,
                function_call: Some(OpenAIFunctionCallDelta {
                    name: Some("get_weather".to_string()),
                    arguments: Some("{\"location\":".to_string()),
                }),
                tool_calls: None,
            },
            finish_reason: None,
        }],
    };

    let converted_function = connector.convert_stream_chunk(function_chunk);
    assert_eq!(converted_function.id, "chatcmpl-123");
    assert_eq!(converted_function.model, "gpt-4");
    assert_eq!(converted_function.choices.len(), 1);
    assert_eq!(converted_function.choices[0].index, 0);
    assert_eq!(converted_function.choices[0].delta.role, None);
    assert_eq!(converted_function.choices[0].delta.content, None);
    assert!(converted_function.choices[0].delta.function_call.is_some());
    assert_eq!(
        converted_function.choices[0]
            .delta
            .function_call
            .as_ref()
            .unwrap()
            .name,
        Some("get_weather".to_string())
    );
    assert_eq!(
        converted_function.choices[0]
            .delta
            .function_call
            .as_ref()
            .unwrap()
            .arguments,
        Some("{\"location\":".to_string())
    );
    assert_eq!(converted_function.choices[0].finish_reason, None);

    // Test tool call chunk
    let tool_chunk = OpenAIStreamResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1677652288,
        model: "gpt-4".to_string(),
        choices: vec![OpenAIStreamChoice {
            index: 0,
            delta: OpenAIDelta {
                role: None,
                content: None,
                function_call: None,
                tool_calls: Some(vec![OpenAIToolCallDelta {
                    id: Some("call_abc123".to_string()),
                    r#type: Some("function".to_string()),
                    function: Some(OpenAIFunctionCallDelta {
                        name: Some("get_weather".to_string()),
                        arguments: Some("{\"location\":".to_string()),
                    }),
                    index: Some(0),
                }]),
            },
            finish_reason: None,
        }],
    };

    let converted_tool = connector.convert_stream_chunk(tool_chunk);
    assert_eq!(converted_tool.id, "chatcmpl-123");
    assert_eq!(converted_tool.model, "gpt-4");
    assert_eq!(converted_tool.choices.len(), 1);
    assert_eq!(converted_tool.choices[0].index, 0);
    assert_eq!(converted_tool.choices[0].delta.role, None);
    assert_eq!(converted_tool.choices[0].delta.content, None);
    assert!(converted_tool.choices[0].delta.tool_calls.is_some());
    assert_eq!(
        converted_tool.choices[0]
            .delta
            .tool_calls
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        converted_tool.choices[0].delta.tool_calls.as_ref().unwrap()[0].id,
        Some("call_abc123".to_string())
    );
    assert_eq!(
        converted_tool.choices[0].delta.tool_calls.as_ref().unwrap()[0].r#type,
        Some("function".to_string())
    );
    assert!(
        converted_tool.choices[0].delta.tool_calls.as_ref().unwrap()[0]
            .function
            .is_some()
    );
    assert_eq!(
        converted_tool.choices[0].delta.tool_calls.as_ref().unwrap()[0]
            .function
            .as_ref()
            .unwrap()
            .name,
        Some("get_weather".to_string())
    );
    assert_eq!(
        converted_tool.choices[0].delta.tool_calls.as_ref().unwrap()[0]
            .function
            .as_ref()
            .unwrap()
            .arguments,
        Some("{\"location\":".to_string())
    );
    assert_eq!(converted_tool.choices[0].finish_reason, None);

    // Test final chunk with finish reason
    let final_chunk = OpenAIStreamResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1677652288,
        model: "gpt-4".to_string(),
        choices: vec![OpenAIStreamChoice {
            index: 0,
            delta: OpenAIDelta {
                role: None,
                content: Some("!".to_string()),
                function_call: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
    };

    let converted_final = connector.convert_stream_chunk(final_chunk);
    assert_eq!(converted_final.id, "chatcmpl-123");
    assert_eq!(converted_final.model, "gpt-4");
    assert_eq!(converted_final.choices.len(), 1);
    assert_eq!(converted_final.choices[0].index, 0);
    assert_eq!(converted_final.choices[0].delta.role, None);
    assert_eq!(
        converted_final.choices[0].delta.content,
        Some("!".to_string())
    );
    assert_eq!(
        converted_final.choices[0].finish_reason,
        Some("stop".to_string())
    );
}
