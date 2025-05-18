//! Tests for WebSocket functionality in the LLM Proxy module

#[cfg(test)]
mod tests {
    use crate::modules::llm_proxy::websocket;
    use crate::modules::llm_proxy::{Provider, ServerConfig, SharedState};
    use axum::{
        body::Body,
        extract::ws::{Message, WebSocket},
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use futures::StreamExt;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    use crate::modules::llm_proxy::routes::ChatCompletionRequest;
    use crate::modules::llm_proxy::server::AppState;
    use crate::modules::model_registry::connectors::ChatMessage;

    // Helper function to create a test app state
    fn create_test_app_state() -> AppState {
        AppState {
            provider: Provider::OpenAI,
            config: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 100,
                request_timeout_secs: 30,
                cors_enabled: false,
                cors_allowed_origins: vec![],
                redis_url: None,
            },
            shared: Arc::new(Mutex::new(SharedState {
                active_connections: 0,
                shutting_down: false,
            })),
            telemetry: None,
            cost_calculator: None,
            shared: Arc::new(Mutex::new(SharedState::new())),
        }
    }

    #[tokio::test]
    async fn test_websocket_upgrade() {
        // Create a router with the WebSocket handler
        let app = Router::new()
            .route("/ws", get(websocket::ws_handler))
            .with_state(create_test_app_state());

        // Create a WebSocket upgrade request
        let request = Request::builder()
            .uri("/ws")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("sec-websocket-version", "13")
            .body(Body::empty())
            .unwrap();

        // Call the handler
        let response = app.oneshot(request).await.unwrap();

        // Verify that the response is a WebSocket upgrade
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(
            response.headers().get("upgrade").unwrap().to_str().unwrap(),
            "websocket"
        );
    }

    #[tokio::test]
    async fn test_websocket_chat_completion() {
        // This test would normally use a WebSocket client to connect to the server
        // and test the full WebSocket communication flow. However, that requires
        // setting up a full server and client, which is beyond the scope of this test.

        // Instead, we'll test the process_message function directly to ensure it
        // correctly handles chat completion requests.

        // Create a test request
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
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

        // Serialize the request to JSON
        let request_json = serde_json::to_string(&request).unwrap();

        // Create a WebSocket message with the request
        let message = Message::Text(request_json);

        // Create a channel for the response
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // Process the message
        let result = websocket::process_message(message, &create_test_app_state(), &tx).await;

        // Verify that the message was processed successfully
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should not break the connection

        // Verify that a response was sent
        let response = rx.recv().await.unwrap();
        assert!(response.is_ok());

        // Extract the response text
        let response_text = match response.unwrap() {
            Message::Text(text) => text,
            _ => panic!("Expected text response"),
        };

        // Parse the response
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify the response structure
        assert_eq!(response_json["object"], "chat.completion");
        assert!(response_json["id"].is_string());
        assert!(response_json["created"].is_number());
        assert_eq!(response_json["model"], "gpt-3.5-turbo");
        assert!(response_json["choices"].is_array());
        assert_eq!(response_json["choices"][0]["index"], 0);
        assert_eq!(response_json["choices"][0]["message"]["role"], "assistant");
        assert!(response_json["choices"][0]["message"]["content"].is_string());
        assert_eq!(response_json["choices"][0]["finish_reason"], "stop");
        assert!(response_json["usage"].is_object());
        assert!(response_json["usage"]["prompt_tokens"].is_number());
        assert!(response_json["usage"]["completion_tokens"].is_number());
        assert!(response_json["usage"]["total_tokens"].is_number());

        // Verify that the response contains the expected greeting
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap();
        assert!(content.contains("Hello"));
        assert!(content.contains("mock assistant"));
    }

    #[tokio::test]
    async fn test_websocket_streaming_request() {
        // Create a test streaming request
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
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

        // Serialize the request to JSON
        let request_json = serde_json::to_string(&request).unwrap();

        // Create a WebSocket message with the request
        let message = Message::Text(request_json);

        // Create a channel for the response
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // Process the message
        let result = websocket::process_message(message, &create_test_app_state(), &tx).await;

        // Verify that the message was processed successfully
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should not break the connection

        // Collect all streaming chunks
        let mut chunks = Vec::new();
        while let Some(response) = rx.recv().await {
            assert!(response.is_ok());

            // Extract the response text
            let response_text = match response.unwrap() {
                Message::Text(text) => text,
                _ => panic!("Expected text response"),
            };

            // Parse the response
            let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();
            chunks.push(response_json);
        }

        // Verify that we received multiple chunks
        assert!(!chunks.is_empty());

        // Verify the first chunk (should contain the role)
        assert_eq!(chunks[0]["object"], "chat.completion.chunk");
        assert!(chunks[0]["id"].is_string());
        assert!(chunks[0]["created"].is_number());
        assert_eq!(chunks[0]["model"], "gpt-3.5-turbo");
        assert!(chunks[0]["choices"].is_array());
        assert_eq!(chunks[0]["choices"][0]["index"], 0);
        assert_eq!(chunks[0]["choices"][0]["delta"]["role"], "assistant");
        assert!(chunks[0]["choices"][0]["finish_reason"].is_null());

        // Verify the last chunk (should contain the finish reason)
        let last_chunk = chunks.last().unwrap();
        assert_eq!(last_chunk["choices"][0]["finish_reason"], "stop");

        // Verify that at least one chunk contains content
        let has_content = chunks
            .iter()
            .any(|chunk| chunk["choices"][0]["delta"]["content"].is_string());
        assert!(has_content);
    }

    #[tokio::test]
    async fn test_websocket_error_handling() {
        // Create an invalid request (missing required fields)
        let invalid_request = json!({
            "model": "gpt-3.5-turbo",
            // Missing messages field
            "temperature": 0.7
        });

        // Serialize the request to JSON
        let request_json = serde_json::to_string(&invalid_request).unwrap();

        // Create a WebSocket message with the request
        let message = Message::Text(request_json);

        // Create a channel for the response
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // Process the message
        let result = websocket::process_message(message, &create_test_app_state(), &tx).await;

        // Verify that the message was processed successfully (even though the request was invalid)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should not break the connection

        // Verify that an error response was sent
        let response = rx.recv().await.unwrap();
        assert!(response.is_err());

        // Verify the error details
        let error = response.unwrap_err();
        assert_eq!(error.error.r#type, "invalid_request_error");
        assert!(error.error.message.contains("Invalid JSON"));
    }

    #[tokio::test]
    async fn test_websocket_binary_message() {
        // Create a binary message (not supported)
        let message = Message::Binary(vec![1, 2, 3, 4]);

        // Create a channel for the response
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // Process the message
        let result = websocket::process_message(message, &create_test_app_state(), &tx).await;

        // Verify that the message was processed successfully
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should not break the connection

        // Verify that an error response was sent
        let response = rx.recv().await.unwrap();
        assert!(response.is_err());

        // Verify the error details
        let error = response.unwrap_err();
        assert_eq!(error.error.r#type, "invalid_request_error");
        assert!(error
            .error
            .message
            .contains("Binary messages are not supported"));
    }

    #[tokio::test]
    async fn test_websocket_ping_pong() {
        // Create a ping message
        let ping_data = vec![1, 2, 3, 4];
        let message = Message::Ping(ping_data.clone());

        // Create a channel for the response
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // Process the message
        let result = websocket::process_message(message, &create_test_app_state(), &tx).await;

        // Verify that the message was processed successfully
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // Should not break the connection

        // Verify that a pong response was sent
        let response = rx.recv().await.unwrap();
        assert!(response.is_ok());

        // Verify the pong data
        match response.unwrap() {
            Message::Pong(data) => assert_eq!(data, ping_data),
            _ => panic!("Expected pong response"),
        }
    }

    #[tokio::test]
    async fn test_websocket_close() {
        // Create a close message
        let message = Message::Close(None);

        // Create a channel for the response
        let (tx, _rx) = tokio::sync::mpsc::channel(32);

        // Process the message
        let result = websocket::process_message(message, &create_test_app_state(), &tx).await;

        // Verify that the message was processed successfully
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true); // Should break the connection
    }
}
