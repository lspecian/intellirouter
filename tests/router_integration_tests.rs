use intellirouter::modules::llm_proxy::domain::message::{Message, MessageRole};
use intellirouter::modules::llm_proxy::dto::{ChatCompletionRequest, ChatCompletionResponse};
use intellirouter::modules::llm_proxy::router_integration::create_mock_router_service;
use intellirouter::modules::llm_proxy::service::ChatCompletionService;

#[tokio::test]
async fn test_router_chat_completions() {
    // Create a chat completion service with a mock router
    let service = ChatCompletionService::new(create_mock_router_service());

    // Create a test request
    let request = ChatCompletionRequest {
        model: "mock-llama".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: intellirouter::modules::llm_proxy::domain::content::MessageContent::String(
                "Hello from the test script!".to_string(),
            ),
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
    let response = service.process_completion_request(&request).await.unwrap();

    // Verify the response
    assert_eq!(response.model, "mock-llama");
    assert!(!response.choices.is_empty());
    assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
    assert!(response.usage.is_some());

    // Print the response for debugging
    println!("Response: {:?}", response);
}
