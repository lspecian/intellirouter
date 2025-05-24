//! Test Client
//!
//! This binary provides a simple client for testing the IntelliRouter chat completion service.
//!
//! This binary is only available when the `test-utils` feature is enabled.
#![cfg(feature = "test-utils")]

use intellirouter::modules::llm_proxy::domain::message::{Message, MessageRole};
use intellirouter::modules::llm_proxy::dto::ChatCompletionRequest;
use intellirouter::modules::llm_proxy::service::ChatCompletionService;

#[tokio::main]
async fn main() {
    println!("Testing IntelliRouter Chat Completion Service");

    // Create a test request
    let request = ChatCompletionRequest {
        model: "mock-llama".to_string(),
        messages: vec![Message::new_user("Hello from the test client!".to_string())],
        temperature: Some(0.7),
        top_p: None,
        n: None,
        stream: false,
        max_tokens: Some(100),
        presence_penalty: None,
        frequency_penalty: None,
        user: None,
    };

    // Use the legacy method for simplicity
    let response = ChatCompletionService::legacy_process_completion_request(&request);

    // Print the response
    println!("Response:");
    println!("ID: {}", response.id);
    println!("Model: {}", response.model);
    println!("Choices: {}", response.choices.len());
    println!(
        "Message: {}",
        response.choices[0].message.extract_text_content()
    );
    println!("Finish reason: {}", response.choices[0].finish_reason);
    println!(
        "Usage: {} prompt tokens, {} completion tokens, {} total tokens",
        response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens
    );
}
