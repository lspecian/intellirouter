//! LLM Proxy API Conformance Tests
//!
//! This module contains comprehensive tests to verify that the LLM Proxy API
//! implementation conforms to the OpenAI API specifications.

#[cfg(test)]
mod tests {
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
        response::Response,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::modules::llm_proxy::{
        routes::ChatCompletionRequest,
        server::{create_router, AppState, ServerConfig, SharedState},
        Provider,
    };
    use crate::modules::model_registry::connectors::ChatMessage;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Helper function to create a test app
    async fn create_test_app() -> axum::Router {
        let app_state = AppState {
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
            shared: Arc::new(Mutex::new(SharedState::new())),
            telemetry: None,
            cost_calculator: None,
        };

        create_router(app_state)
    }

    /// Helper function to create a test request
    fn create_test_request(json_body: Value) -> Request<Body> {
        Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(json_body.to_string()))
            .unwrap()
    }

    /// Helper function to create a streaming test request
    fn create_streaming_test_request(json_body: Value) -> Request<Body> {
        Request::builder()
            .uri("/v1/chat/completions/stream")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(json_body.to_string()))
            .unwrap()
    }

    /// Helper function to parse response body
    async fn parse_response_body(response: Response) -> Value {
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        match serde_json::from_slice(&body_bytes) {
            Ok(json) => json,
            Err(_) => {
                // If we can't parse as JSON, return the raw body as a string
                let body_str = String::from_utf8_lossy(&body_bytes);
                json!({ "raw_body": body_str })
            }
        }
    }

    /// Helper function to extract SSE events from a streaming response
    async fn extract_sse_events(response: Response) -> Vec<Value> {
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Parse SSE events
        let mut events = Vec::new();
        for line in body_str.lines() {
            if line.starts_with("data: ") {
                let data = line.trim_start_matches("data: ");
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(json) = serde_json::from_str::<Value>(data) {
                    events.push(json);
                }
            }
        }

        events
    }

    #[tokio::test]
    async fn test_chat_completions_basic_conformance() {
        // Create test app
        let app = create_test_app().await;

        // Create test request
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ]
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify response format conforms to OpenAI API specifications
        assert!(body.get("id").is_some(), "Response should have an id");
        assert!(
            body["id"].as_str().unwrap().starts_with("chatcmpl-"),
            "ID should start with 'chatcmpl-'"
        );

        assert_eq!(
            body["object"], "chat.completion",
            "Object type should be chat.completion"
        );
        assert!(
            body.get("created").is_some(),
            "Response should have a created timestamp"
        );
        assert!(
            body["created"].is_u64(),
            "Created timestamp should be a u64"
        );
        assert_eq!(body["model"], "gpt-3.5-turbo", "Model should match request");

        // Verify choices
        assert!(body["choices"].is_array(), "Choices should be an array");
        assert!(
            !body["choices"].as_array().unwrap().is_empty(),
            "Choices should not be empty"
        );

        let choice = &body["choices"][0];
        assert_eq!(choice["index"], 0, "First choice should have index 0");
        assert!(
            choice["message"].is_object(),
            "Choice should have a message object"
        );
        assert_eq!(
            choice["message"]["role"], "assistant",
            "Message role should be assistant"
        );
        assert!(
            choice["message"]["content"].is_string(),
            "Message should have content"
        );
        assert!(
            choice.get("finish_reason").is_some(),
            "Choice should have a finish_reason"
        );

        // Verify usage
        assert!(
            body["usage"].is_object(),
            "Response should have usage object"
        );
        assert!(
            body["usage"]["prompt_tokens"].is_number(),
            "Usage should have prompt_tokens"
        );
        assert!(
            body["usage"]["completion_tokens"].is_number(),
            "Usage should have completion_tokens"
        );
        assert!(
            body["usage"]["total_tokens"].is_number(),
            "Usage should have total_tokens"
        );
    }

    #[tokio::test]
    async fn test_chat_completions_with_parameters() {
        // Create test app
        let app = create_test_app().await;

        // Create test request with parameters
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "temperature": 0.7,
            "top_p": 0.9,
            "n": 1,
            "max_tokens": 100,
            "presence_penalty": 0.0,
            "frequency_penalty": 0.0
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify response format conforms to OpenAI API specifications
        assert!(body.get("id").is_some());
        assert_eq!(body["object"], "chat.completion");
        assert!(body.get("created").is_some());
        assert_eq!(body["model"], "gpt-3.5-turbo");

        // Verify choices
        assert!(body["choices"].is_array());
        assert!(!body["choices"].as_array().unwrap().is_empty());

        let choice = &body["choices"][0];
        assert_eq!(choice["index"], 0);
        assert!(choice["message"].is_object());
        assert_eq!(choice["message"]["role"], "assistant");
        assert!(choice["message"]["content"].is_string());
        assert!(choice.get("finish_reason").is_some());

        // Verify usage
        assert!(body["usage"].is_object());
        assert!(body["usage"]["prompt_tokens"].is_number());
        assert!(body["usage"]["completion_tokens"].is_number());
        assert!(body["usage"]["total_tokens"].is_number());
    }

    #[tokio::test]
    async fn test_chat_completions_invalid_request() {
        // Create test app
        let app = create_test_app().await;

        // Create invalid test request (missing messages)
        let request_body = json!({
            "model": "gpt-3.5-turbo"
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code (should be 400 Bad Request or 422 Unprocessable Entity)
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected status code 400 or 422, got {}",
            response.status()
        );

        // Parse response body
        let body = parse_response_body(response).await;

        // Check if we have a raw body (non-JSON response)
        if body.get("raw_body").is_some() {
            // If we got a raw body, just verify it's not empty
            assert!(
                !body["raw_body"].as_str().unwrap().is_empty(),
                "Response body should not be empty"
            );
            return;
        }

        // Verify error format conforms to OpenAI API specifications
        if body.get("error").is_some() {
            assert!(
                body["error"].is_object(),
                "Response should have an error object"
            );
            assert!(
                body["error"]["message"].is_string(),
                "Error should have a message"
            );

            if body["error"].get("type").is_some() {
                assert!(
                    body["error"]["type"].is_string(),
                    "Error should have a type"
                );
                // Check if the type is what we expect
                if body["error"]["type"].is_string() {
                    let error_type = body["error"]["type"].as_str().unwrap();
                    assert!(
                        error_type == "invalid_request_error" || error_type == "validation_error",
                        "Error type should be invalid_request_error or validation_error, got {}",
                        error_type
                    );
                }
            }
        } else {
            // If there's no error object, the response should at least contain some error indication
            assert!(
                !body.as_object().unwrap().is_empty(),
                "Response should contain error information"
            );
        }
    }

    #[tokio::test]
    async fn test_chat_completions_invalid_model() {
        // Create test app
        let app = create_test_app().await;

        // Create invalid test request (unsupported model)
        let request_body = json!({
            "model": "unsupported-model",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ]
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code (should be 400 Bad Request)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify error format
        assert!(body["error"].is_object());
        assert!(body["error"]["message"].is_string());
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not supported"));
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["param"], "model");
    }

    #[tokio::test]
    async fn test_chat_completions_invalid_temperature() {
        // Create test app
        let app = create_test_app().await;

        // Create invalid test request (temperature out of range)
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "temperature": 3.0
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code (should be 400 Bad Request)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify error format
        assert!(body["error"].is_object());
        assert!(body["error"]["message"].is_string());
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("temperature"));
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["param"], "temperature");
    }

    #[tokio::test]
    async fn test_chat_completions_invalid_messages() {
        // Create test app
        let app = create_test_app().await;

        // Create invalid test request (empty messages array)
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": []
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code (should be 400 Bad Request)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify error format
        assert!(body["error"].is_object());
        assert!(body["error"]["message"].is_string());
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("messages"));
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["param"], "messages");
    }

    #[tokio::test]
    async fn test_chat_completions_invalid_role() {
        // Create test app
        let app = create_test_app().await;

        // Create invalid test request (invalid role)
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "invalid_role",
                    "content": "Hello!"
                }
            ]
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code (should be 400 Bad Request)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify error format
        assert!(body["error"].is_object());
        assert!(body["error"]["message"].is_string());
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("invalid role"));
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["param"], "messages");
    }

    #[tokio::test]
    async fn test_chat_completions_streaming() {
        // Create test app
        let app = create_test_app().await;

        // Create test request with stream=true
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "stream": true
        });

        let request = create_streaming_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Check content type (should be text/event-stream for SSE)
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "text/event-stream");

        // Extract and verify SSE events
        let events = extract_sse_events(response).await;
        assert!(!events.is_empty(), "Should have at least one SSE event");

        // Verify first event format (should have role)
        let first_event = &events[0];
        assert!(first_event.get("id").is_some(), "Event should have an id");
        assert_eq!(
            first_event["object"], "chat.completion.chunk",
            "Object type should be chat.completion.chunk"
        );
        assert!(
            first_event.get("created").is_some(),
            "Event should have a created timestamp"
        );
        assert_eq!(
            first_event["model"], "gpt-3.5-turbo",
            "Model should match request"
        );

        assert!(
            first_event["choices"].is_array(),
            "Choices should be an array"
        );
        assert!(
            !first_event["choices"].as_array().unwrap().is_empty(),
            "Choices should not be empty"
        );

        let first_choice = &first_event["choices"][0];
        assert_eq!(first_choice["index"], 0, "First choice should have index 0");
        assert!(
            first_choice["delta"].is_object(),
            "Choice should have a delta object"
        );

        // First event should have role
        assert_eq!(
            first_choice["delta"]["role"], "assistant",
            "First delta should have assistant role"
        );

        // Last event should have finish_reason
        if events.len() > 1 {
            let last_event = &events[events.len() - 1];
            let last_choice = &last_event["choices"][0];
            assert!(
                last_choice.get("finish_reason").is_some(),
                "Last choice should have a finish_reason"
            );
        }
    }

    #[tokio::test]
    async fn test_chat_completions_with_system_message() {
        // Create test app
        let app = create_test_app().await;

        // Create test request with system message
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ]
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify response format
        assert!(body.get("id").is_some());
        assert_eq!(body["object"], "chat.completion");
        assert!(body.get("created").is_some());
        assert_eq!(body["model"], "gpt-3.5-turbo");

        // Verify choices
        assert!(body["choices"].is_array());
        assert!(!body["choices"].as_array().unwrap().is_empty());

        let choice = &body["choices"][0];
        assert_eq!(choice["index"], 0);
        assert!(choice["message"].is_object());
        assert_eq!(choice["message"]["role"], "assistant");
        assert!(choice["message"]["content"].is_string());
        assert!(choice.get("finish_reason").is_some());
    }

    #[tokio::test]
    async fn test_chat_completions_with_conversation_history() {
        // Create test app
        let app = create_test_app().await;

        // Create test request with conversation history
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                },
                {
                    "role": "assistant",
                    "content": "Hi there! How can I help you today?"
                },
                {
                    "role": "user",
                    "content": "Tell me about the weather."
                }
            ]
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify response format
        assert!(body.get("id").is_some());
        assert_eq!(body["object"], "chat.completion");
        assert!(body.get("created").is_some());
        assert_eq!(body["model"], "gpt-3.5-turbo");

        // Verify choices
        assert!(body["choices"].is_array());
        assert!(!body["choices"].as_array().unwrap().is_empty());

        let choice = &body["choices"][0];
        assert_eq!(choice["index"], 0);
        assert!(choice["message"].is_object());
        assert_eq!(choice["message"]["role"], "assistant");
        assert!(choice["message"]["content"].is_string());
        // The response should contain "weather" since that was the last user message
        assert!(choice["message"]["content"]
            .as_str()
            .unwrap()
            .contains("weather"));
        assert!(choice.get("finish_reason").is_some());
    }

    #[tokio::test]
    async fn test_chat_completions_max_tokens_limit() {
        // Create test app
        let app = create_test_app().await;

        // Create test request with very low max_tokens
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Write a long story."
                }
            ],
            "max_tokens": 5
        });
        let request = create_test_request(request_body);

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify response format
        assert!(body.get("id").is_some());
        assert_eq!(body["object"], "chat.completion");

        // Verify choices
        let choice = &body["choices"][0];
        assert_eq!(choice["index"], 0);
        assert!(choice["message"].is_object());

        // The content should be truncated
        let content = choice["message"]["content"].as_str().unwrap();
        let word_count = content.split_whitespace().count();
        assert!(word_count <= 6); // 5 words + possible ellipsis
        assert!(content.contains("..."));
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        // Create test app
        let app = create_test_app().await;

        // Create health check request
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body = parse_response_body(response).await;

        // Verify response format
        assert_eq!(body["status"], "ok");
    }
}
