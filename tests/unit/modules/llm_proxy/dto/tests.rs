use crate::modules::llm_proxy::domain::message::{Message, MessageRole};
use crate::modules::llm_proxy::dto::{ChatCompletionRequest, ChatCompletionResponse};

#[test]
fn test_chat_completion_request_deserialize() {
    let json = r#"{
        "model": "claude-3-sonnet",
        "messages": [
            {"role": "user", "content": "Hello world"}
        ],
        "temperature": 0.7,
        "max_tokens": 100
    }"#;

    let request: ChatCompletionRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.model, "claude-3-sonnet");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.messages[0].role, MessageRole::User);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(100));
}

#[test]
fn test_chat_completion_response_serialize() {
    let response = ChatCompletionResponse::new(
        "claude-3-sonnet".to_string(),
        Message::new_assistant("Hello, I'm an AI assistant.".to_string()),
    );

    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"object\":\"chat.completion\""));
    assert!(json.contains("\"model\":\"claude-3-sonnet\""));
    assert!(json.contains("\"role\":\"assistant\""));
    assert!(json.contains("\"content\":\"Hello, I'm an AI assistant.\""));
}
