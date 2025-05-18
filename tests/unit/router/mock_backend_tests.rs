//! Mock Backend Tests for Router
//!
//! These tests validate the router's behavior using mock backends,
//! ensuring that routing logic works correctly without requiring
//! actual API calls to external services.

use intellirouter::modules::llm_proxy::{
    domain::content::MessageContent,
    domain::message::{Message, MessageRole},
    dto::{ChatCompletionRequest, ChatCompletionResponse},
    router_integration::create_mock_router_service,
    service::ChatCompletionService,
};
use intellirouter::modules::router_core::{RouterConfig, RouterError, RoutingStrategy};
use std::sync::Arc;

#[tokio::test]
async fn test_mock_backend_known_model() {
    // Create a chat completion service with a mock router
    let service = ChatCompletionService::new(create_mock_router_service());

    // Create a test request with a known model
    let request = ChatCompletionRequest {
        model: "mock-llama".to_string(), // This model should be registered in the mock router
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello from the test!".to_string()),
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

    // Process the request
    let response = service.process_completion_request(&request).await;

    // Verify the response
    assert!(response.is_ok(), "Request with known model should succeed");

    let response = response.unwrap();
    assert_eq!(
        response.model, "mock-llama",
        "Response model should match request model"
    );
    assert!(!response.choices.is_empty(), "Response should have choices");
    assert_eq!(
        response.choices[0].message.role,
        MessageRole::Assistant,
        "Response role should be assistant"
    );

    // The mock backend should include the original message in the response
    if let MessageContent::String(content) = &response.choices[0].message.content {
        assert!(
            content.contains("Hello from the test!"),
            "Response should contain the original message"
        );
    } else {
        panic!("Response content should be a string");
    }

    // Verify usage information
    assert!(
        response.usage.is_some(),
        "Response should include usage information"
    );
}

#[tokio::test]
async fn test_mock_backend_unknown_model() {
    // Create a chat completion service with a mock router
    let service = ChatCompletionService::new(create_mock_router_service());

    // Create a test request with an unknown model
    let request = ChatCompletionRequest {
        model: "unknown-model".to_string(), // This model should not be registered in the mock router
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello from the test!".to_string()),
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

    // Process the request
    let response = service.process_completion_request(&request).await;

    // Verify the response is an error
    assert!(response.is_err(), "Request with unknown model should fail");

    // Check the error type
    match response.unwrap_err() {
        RouterError::NoSuitableModel(_) => {
            // This is the expected error type
        }
        err => panic!("Unexpected error type: {:?}", err),
    }
}

#[tokio::test]
async fn test_mock_backend_streaming() {
    // Create a chat completion service with a mock router
    let service = ChatCompletionService::new(create_mock_router_service());

    // Create a test request for streaming
    let request = ChatCompletionRequest {
        model: "mock-llama".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello from the streaming test!".to_string()),
            name: None,
        }],
        temperature: Some(0.7),
        top_p: None,
        n: None,
        stream: true, // Enable streaming
        max_tokens: Some(100),
        presence_penalty: None,
        frequency_penalty: None,
        user: None,
    };

    // Process the streaming request
    let stream_result = service.process_streaming_request(&request).await;

    // Verify the stream is created successfully
    assert!(stream_result.is_ok(), "Streaming request should succeed");

    // Collect all chunks from the stream
    let stream = stream_result.unwrap();
    let chunks = tokio_stream::StreamExt::collect::<Vec<_>>(stream).await;

    // Verify we got some chunks
    assert!(!chunks.is_empty(), "Stream should produce chunks");

    // Verify all chunks are successful
    for chunk_result in chunks {
        assert!(chunk_result.is_ok(), "All chunks should be successful");

        // Parse the chunk as JSON
        let chunk_json: serde_json::Value = serde_json::from_str(&chunk_result.unwrap()).unwrap();

        // Verify chunk structure
        assert!(chunk_json.get("id").is_some(), "Chunk should have id");
        assert!(chunk_json.get("model").is_some(), "Chunk should have model");
        assert!(
            chunk_json.get("choices").is_some(),
            "Chunk should have choices"
        );

        // Verify choices structure
        let choices = chunk_json.get("choices").unwrap().as_array().unwrap();
        assert!(!choices.is_empty(), "Choices should not be empty");

        // Verify first choice
        let choice = &choices[0];
        assert!(choice.get("index").is_some(), "Choice should have index");
        assert!(choice.get("delta").is_some(), "Choice should have delta");
    }
}

#[tokio::test]
async fn test_routing_strategies() {
    // Test different routing strategies with the mock backend

    // Create a round-robin router config
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::RoundRobin;

    // Create a mock router service with this config
    let router_service = intellirouter::modules::llm_proxy::router_integration::create_mock_router_service_with_config(router_config);
    let service = ChatCompletionService::new(router_service);

    // Create a test request
    let request = ChatCompletionRequest {
        model: "mock-llama".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("Test round-robin routing".to_string()),
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

    // Process the request
    let response = service.process_completion_request(&request).await;

    // Verify the response
    assert!(response.is_ok(), "Round-robin routing should succeed");

    // Now test cost-optimized strategy
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::CostOptimized;

    // Create a mock router service with this config
    let router_service = intellirouter::modules::llm_proxy::router_integration::create_mock_router_service_with_config(router_config);
    let service = ChatCompletionService::new(router_service);

    // Process the request
    let response = service.process_completion_request(&request).await;

    // Verify the response
    assert!(response.is_ok(), "Cost-optimized routing should succeed");

    // Now test performance-optimized strategy
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::PerformanceOptimized;

    // Create a mock router service with this config
    let router_service = intellirouter::modules::llm_proxy::router_integration::create_mock_router_service_with_config(router_config);
    let service = ChatCompletionService::new(router_service);

    // Process the request
    let response = service.process_completion_request(&request).await;

    // Verify the response
    assert!(
        response.is_ok(),
        "Performance-optimized routing should succeed"
    );
}

#[tokio::test]
async fn test_mock_backend_error_simulation() {
    // Create a mock router service with error simulation
    let router_service = intellirouter::modules::llm_proxy::router_integration::create_mock_router_service_with_errors();
    let service = ChatCompletionService::new(router_service);

    // Create a test request
    let request = ChatCompletionRequest {
        model: "mock-llama".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::String("This should trigger an error".to_string()),
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

    // Process the request
    let response = service.process_completion_request(&request).await;

    // Verify the response is an error
    assert!(
        response.is_err(),
        "Request with error simulation should fail"
    );

    // Check the error type
    match response.unwrap_err() {
        RouterError::ConnectorError(_) => {
            // This is the expected error type
        }
        err => panic!("Unexpected error type: {:?}", err),
    }
}
