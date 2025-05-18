//! OpenAI API Compatibility Tests
//!
//! This module contains integration tests to verify that the IntelliRouter
//! correctly handles the OpenAI API format, including multimodal content.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use intellirouter::modules::llm_proxy::{
    domain::content::{ContentPart, ImageUrl, MessageContent},
    domain::message::{Message, MessageRole},
    dto::{ChatCompletionRequest, ChatCompletionResponse},
    routes::{chat_completions, chat_completions_stream},
    telemetry_integration::AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

/// Create a test app for the chat completions endpoint
fn create_test_app() -> Router {
    // Create mock telemetry and cost calculator
    let telemetry =
        Arc::new(intellirouter::modules::telemetry::TelemetryManager::new_for_testing());
    let cost_calculator = Arc::new(intellirouter::modules::telemetry::CostCalculator::new());

    let app_state = AppState {
        telemetry,
        cost_calculator,
    };

    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/chat/completions/stream", post(chat_completions_stream))
        .with_state(app_state)
}

#[tokio::test]
async fn test_chat_completions_with_string_content() {
    // Create the test app
    let app = create_test_app();

    // Create a request with string content
    let request_body = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {"role": "user", "content": "Hello, how are you?"}
        ]
    });

    // Send the request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(response_json.get("id").is_some());
    assert_eq!(response_json["object"], "chat.completion");
    assert!(response_json.get("created").is_some());
    assert_eq!(response_json["model"], "claude-3-sonnet");
    assert!(response_json["choices"].is_array());
    assert_eq!(response_json["choices"][0]["message"]["role"], "assistant");
    assert!(response_json["choices"][0]["message"]["content"].is_string());
}

#[tokio::test]
async fn test_chat_completions_with_array_content() {
    // Create the test app
    let app = create_test_app();

    // Create a request with array content
    let request_body = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Hello, how are you?"}
                ]
            }
        ]
    });

    // Send the request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(response_json.get("id").is_some());
    assert_eq!(response_json["object"], "chat.completion");
    assert!(response_json.get("created").is_some());
    assert_eq!(response_json["model"], "claude-3-sonnet");
    assert!(response_json["choices"].is_array());
    assert_eq!(response_json["choices"][0]["message"]["role"], "assistant");
    assert!(response_json["choices"][0]["message"]["content"].is_string());
}

#[tokio::test]
async fn test_chat_completions_with_multimodal_content() {
    // Create the test app
    let app = create_test_app();

    // Create a request with multimodal content (text + image)
    let request_body = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "What's in this image?"},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wAARCAABAAEDASIAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwD9/KKKKAP/2Q==",
                            "detail": "auto"
                        }
                    }
                ]
            }
        ]
    });

    // Send the request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(response_json.get("id").is_some());
    assert_eq!(response_json["object"], "chat.completion");
    assert!(response_json.get("created").is_some());
    assert_eq!(response_json["model"], "claude-3-sonnet");
    assert!(response_json["choices"].is_array());
    assert_eq!(response_json["choices"][0]["message"]["role"], "assistant");
    assert!(response_json["choices"][0]["message"]["content"].is_string());
}

#[tokio::test]
async fn test_chat_completions_with_invalid_content() {
    // Create the test app
    let app = create_test_app();

    // Create a request with invalid content (empty array)
    let request_body = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {
                "role": "user",
                "content": []
            }
        ]
    });

    // Send the request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response (should be a 400 Bad Request)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the error structure
    assert!(response_json.get("error").is_some());
    assert_eq!(response_json["error"]["type"], "invalid_request_error");
    assert!(response_json["error"]["message"].is_string());
}

#[tokio::test]
async fn test_chat_completions_stream() {
    // Create the test app
    let app = create_test_app();

    // Create a request for streaming
    let request_body = json!({
        "model": "claude-3-sonnet",
        "messages": [
            {"role": "user", "content": "Hello, how are you?"}
        ],
        "stream": true
    });

    // Send the request to the wrong endpoint (should get a redirect error)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response (should be a 400 Bad Request with a specific message)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the error structure
    assert!(response_json.get("error").is_some());
    assert_eq!(response_json["error"]["type"], "invalid_request_error");
    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("/v1/chat/completions/stream"));

    // Now send to the correct streaming endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions/stream")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
}
