//! Router HTTP Integration Tests
//!
//! These tests validate the router's HTTP endpoints by starting a test server
//! and sending real HTTP requests to it.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use intellirouter::{
    config::Config,
    modules::{
        llm_proxy::{
            domain::content::MessageContent,
            domain::message::{Message, MessageRole},
            dto::{ChatCompletionRequest, ChatCompletionResponse},
            routes::{chat_completions, chat_completions_stream},
            server::ServerConfig,
            telemetry_integration::AppState,
        },
        model_registry::api::ModelRegistryApi,
        router_core::{RouterConfig, RoutingStrategy},
        telemetry::TelemetryManager,
    },
    test_utils::init_test_logging,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower::ServiceExt;

/// Create a test app for the router
async fn create_test_app() -> Router {
    // Initialize test environment
    init_test_logging();

    // Create telemetry components
    let telemetry = Arc::new(TelemetryManager::new_for_testing());
    let cost_calculator = Arc::new(intellirouter::modules::telemetry::CostCalculator::new());

    // Create app state
    let app_state = AppState {
        telemetry: telemetry.clone(),
        cost_calculator: cost_calculator.clone(),
    };

    // Create router with routes
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/chat/completions/stream", post(chat_completions_stream))
        .with_state(app_state)
}

/// Spawn a test server and return its address
async fn spawn_test_server() -> (SocketAddr, oneshot::Sender<()>) {
    // Create the app
    let app = create_test_app().await;

    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Create a shutdown signal
    let (tx, rx) = oneshot::channel::<()>();

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await
            .unwrap();
    });

    // Return the address and shutdown signal
    (addr, tx)
}

#[tokio::test]
async fn test_chat_completions_endpoint() {
    // Spawn a test server
    let (addr, _shutdown) = spawn_test_server().await;

    // Create a client
    let client = reqwest::Client::new();

    // Create a request
    let request_body = json!({
        "model": "mock-llama",
        "messages": [
            {"role": "user", "content": "Hello from the integration test!"}
        ],
        "temperature": 0.7,
        "max_tokens": 100
    });

    // Send the request
    let response = client
        .post(format!("http://{}/v1/chat/completions", addr))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .unwrap();

    // Check the response status
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Response status should be 200 OK"
    );

    // Parse the response body
    let response_json: Value = response.json().await.unwrap();

    // Verify the response structure
    assert!(
        response_json.get("id").is_some(),
        "Response should have id field"
    );
    assert!(
        response_json.get("object").is_some(),
        "Response should have object field"
    );
    assert!(
        response_json.get("created").is_some(),
        "Response should have created field"
    );
    assert!(
        response_json.get("model").is_some(),
        "Response should have model field"
    );
    assert!(
        response_json.get("choices").is_some(),
        "Response should have choices field"
    );

    // Verify choices array
    let choices = response_json.get("choices").unwrap().as_array().unwrap();
    assert!(!choices.is_empty(), "Choices array should not be empty");

    // Verify first choice
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

    // Verify message
    let message = choice.get("message").unwrap();
    assert!(
        message.get("role").is_some(),
        "Message should have role field"
    );
    assert!(
        message.get("content").is_some(),
        "Message should have content field"
    );
    assert_eq!(
        message.get("role").unwrap(),
        "assistant",
        "Role should be assistant"
    );

    // Verify usage
    assert!(
        response_json.get("usage").is_some(),
        "Response should have usage field"
    );
    let usage = response_json.get("usage").unwrap();
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

#[tokio::test]
async fn test_chat_completions_invalid_request() {
    // Spawn a test server
    let (addr, _shutdown) = spawn_test_server().await;

    // Create a client
    let client = reqwest::Client::new();

    // Create an invalid request (missing messages)
    let request_body = json!({
        "model": "mock-llama",
        "temperature": 0.7,
        "max_tokens": 100
    });

    // Send the request
    let response = client
        .post(format!("http://{}/v1/chat/completions", addr))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .unwrap();

    // Check the response status
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Response status should be 400 Bad Request"
    );

    // Parse the response body
    let response_json: Value = response.json().await.unwrap();

    // Verify the error structure
    assert!(
        response_json.get("error").is_some(),
        "Response should have error field"
    );
    let error = response_json.get("error").unwrap();
    assert!(
        error.get("message").is_some(),
        "Error should have message field"
    );
    assert!(error.get("type").is_some(), "Error should have type field");
    assert_eq!(
        error.get("type").unwrap(),
        "invalid_request_error",
        "Error type should be invalid_request_error"
    );
}

#[tokio::test]
async fn test_chat_completions_streaming() {
    // Spawn a test server
    let (addr, _shutdown) = spawn_test_server().await;

    // Create a client
    let client = reqwest::Client::new();

    // Create a streaming request
    let request_body = json!({
        "model": "mock-llama",
        "messages": [
            {"role": "user", "content": "Hello from the streaming test!"}
        ],
        "temperature": 0.7,
        "max_tokens": 100,
        "stream": true
    });

    // Send the request
    let response = client
        .post(format!("http://{}/v1/chat/completions/stream", addr))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .unwrap();

    // Check the response status
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Response status should be 200 OK"
    );

    // Check the content type
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "text/event-stream",
        "Content type should be text/event-stream"
    );

    // Read the streaming response
    let body = response.bytes().await.unwrap();

    // Verify that we got some data
    assert!(!body.is_empty(), "Response body should not be empty");

    // The body should contain "data: " prefixes for SSE
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("data: "),
        "Response should contain SSE data markers"
    );
}

/// External API test that requires an OpenAI API key
/// This test is ignored by default and can be run with:
/// cargo test -- --ignored
#[tokio::test]
#[ignore]
async fn test_external_openai_api() {
    // Check if the OpenAI API key is set
    let api_key = std::env::var("OPENAI_API_KEY");
    if api_key.is_err() {
        println!("Skipping external API test: OPENAI_API_KEY not set");
        return;
    }

    // Create a client
    let client = reqwest::Client::new();

    // Create a request
    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "user", "content": "Say hello in one short sentence."}
        ],
        "temperature": 0.7,
        "max_tokens": 20
    });

    // Send the request to the actual OpenAI API
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key.unwrap()))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    // Check the response status
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Response status should be 200 OK"
    );

    // Parse the response body
    let response_json: Value = response.json().await.unwrap();

    // Verify the response structure matches our expected schema
    assert!(
        response_json.get("id").is_some(),
        "Response should have id field"
    );
    assert!(
        response_json.get("object").is_some(),
        "Response should have object field"
    );
    assert!(
        response_json.get("created").is_some(),
        "Response should have created field"
    );
    assert!(
        response_json.get("model").is_some(),
        "Response should have model field"
    );
    assert!(
        response_json.get("choices").is_some(),
        "Response should have choices field"
    );
    assert!(
        response_json.get("usage").is_some(),
        "Response should have usage field"
    );
}
