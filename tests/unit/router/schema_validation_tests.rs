//! Schema Validation Tests for Router
//!
//! These tests validate that the router correctly handles OpenAI-compatible
//! request and response schemas, ensuring all required fields exist and types match.

use intellirouter::modules::llm_proxy::{
    domain::content::{ContentPart, ImageUrl, MessageContent},
    domain::message::{Message, MessageRole},
    dto::{ApiError, ChatCompletionRequest, ChatCompletionResponse},
    validation,
};
use serde_json::{json, Value};

#[test]
fn test_validate_chat_completion_request_valid() {
    // Create a valid request
    let request = ChatCompletionRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello, world!".to_string()),
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

    // Validate the request
    let result = validation::validate_chat_completion_request(&request);
    assert!(result.is_ok(), "Valid request should pass validation");
}

#[test]
fn test_validate_chat_completion_request_empty_messages() {
    // Create a request with empty messages
    let request = ChatCompletionRequest {
        model: "gpt-4".to_string(),
        messages: vec![],
        temperature: Some(0.7),
        top_p: None,
        n: None,
        stream: false,
        max_tokens: Some(100),
        presence_penalty: None,
        frequency_penalty: None,
        user: None,
    };

    // Validate the request
    let result = validation::validate_chat_completion_request(&request);
    assert!(
        result.is_err(),
        "Request with empty messages should fail validation"
    );

    // Check error message
    if let Err(err) = result {
        assert!(
            err.error.message.contains("messages"),
            "Error should mention messages"
        );
    }
}

#[test]
fn test_validate_chat_completion_request_invalid_temperature() {
    // Create a request with invalid temperature
    let request = ChatCompletionRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello, world!".to_string()),
            name: None,
        }],
        temperature: Some(2.0), // Temperature should be between 0 and 1
        top_p: None,
        n: None,
        stream: false,
        max_tokens: Some(100),
        presence_penalty: None,
        frequency_penalty: None,
        user: None,
    };

    // Validate the request
    let result = validation::validate_chat_completion_request(&request);
    assert!(
        result.is_err(),
        "Request with invalid temperature should fail validation"
    );

    // Check error message
    if let Err(err) = result {
        assert!(
            err.error.message.contains("temperature"),
            "Error should mention temperature"
        );
    }
}

#[test]
fn test_validate_chat_completion_request_invalid_model() {
    // Create a request with empty model
    let request = ChatCompletionRequest {
        model: "".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello, world!".to_string()),
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

    // Validate the request
    let result = validation::validate_chat_completion_request(&request);
    assert!(
        result.is_err(),
        "Request with empty model should fail validation"
    );

    // Check error message
    if let Err(err) = result {
        assert!(
            err.error.message.contains("model"),
            "Error should mention model"
        );
    }
}

#[test]
fn test_validate_chat_completion_response_schema() {
    // Create a response
    let response = ChatCompletionResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1677858242,
        model: "gpt-4".to_string(),
        choices: vec![
            intellirouter::modules::llm_proxy::dto::ChatCompletionChoice {
                index: 0,
                message: Message {
                    role: MessageRole::Assistant,
                    content: MessageContent::String("Hello, how can I help you?".to_string()),
                    name: None,
                },
                finish_reason: "stop".to_string(),
            },
        ],
        usage: intellirouter::modules::llm_proxy::dto::TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    };

    // Serialize to JSON
    let json = serde_json::to_value(&response).unwrap();

    // Validate required fields
    assert!(json.get("id").is_some(), "Response should have id field");
    assert!(
        json.get("object").is_some(),
        "Response should have object field"
    );
    assert!(
        json.get("created").is_some(),
        "Response should have created field"
    );
    assert!(
        json.get("model").is_some(),
        "Response should have model field"
    );
    assert!(
        json.get("choices").is_some(),
        "Response should have choices field"
    );

    // Validate choices array
    let choices = json.get("choices").unwrap().as_array().unwrap();
    assert!(!choices.is_empty(), "Choices array should not be empty");

    // Validate first choice
    let choice = &choices[0];
    assert!(
        choice.get("index").is_some(),
        "Choice should have index field"
    );
    assert!(
        choice.get("message").is_some(),
        "Choice should have message field"
    );
    assert!(
        choice.get("finish_reason").is_some(),
        "Choice should have finish_reason field"
    );

    // Validate message
    let message = choice.get("message").unwrap();
    assert!(
        message.get("role").is_some(),
        "Message should have role field"
    );
    assert!(
        message.get("content").is_some(),
        "Message should have content field"
    );

    // Validate usage
    let usage = json.get("usage").unwrap();
    assert!(
        usage.get("prompt_tokens").is_some(),
        "Usage should have prompt_tokens field"
    );
    assert!(
        usage.get("completion_tokens").is_some(),
        "Usage should have completion_tokens field"
    );
    assert!(
        usage.get("total_tokens").is_some(),
        "Usage should have total_tokens field"
    );
}

#[test]
fn test_error_response_schema() {
    // Create an error response
    let error = ApiError {
        error: intellirouter::modules::llm_proxy::dto::ApiErrorDetail {
            message: "Invalid request".to_string(),
            r#type: "invalid_request_error".to_string(),
            param: Some("model".to_string()),
            code: None,
        },
    };

    // Serialize to JSON
    let json = serde_json::to_value(&error).unwrap();

    // Validate error structure
    assert!(
        json.get("error").is_some(),
        "Response should have error field"
    );

    // Validate error details
    let error_detail = json.get("error").unwrap();
    assert!(
        error_detail.get("message").is_some(),
        "Error should have message field"
    );
    assert!(
        error_detail.get("type").is_some(),
        "Error should have type field"
    );
    assert!(
        error_detail.get("param").is_some(),
        "Error should have param field"
    );
}
