//! Integration tests for the LLM Proxy module
//!
//! This module contains integration tests for the LLM Proxy module,
//! verifying that the router correctly routes requests to the appropriate
//! model backend.

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::modules::llm_proxy::{
        domain::message::Message,
        dto::ChatCompletionRequest,
        router_integration::create_mock_router_service,
        routes::{chat_completions, chat_completions_stream},
        service::ChatCompletionService,
        telemetry_integration::AppState,
    };
    use crate::modules::telemetry::{CostCalculator, TelemetryManager};

    /// Create a test app with the chat completions routes
    fn create_test_app() -> Router {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            telemetry,
            cost_calculator,
        };

        // Create router
        // Create router with inline handlers to avoid Handler trait issues
        Router::new()
            .route(
                "/v1/chat/completions",
                post(|state, json| async move { chat_completions(state, json).await }),
            )
            .route(
                "/v1/chat/completions/stream",
                post(|state, json| async move { chat_completions_stream(state, json).await }),
            )
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_chat_completions_integration() {
        // Create test app
        let app = create_test_app();

        // Create test request
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ],
            "temperature": 0.7,
            "max_tokens": 100
        });

        // Create HTTP request
        let request = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Send request to app
        let response = app.oneshot(request).await.unwrap();

        // Verify response status
        assert_eq!(response.status(), StatusCode::OK);

        // Get response body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        // Verify response structure
        assert!(body.get("id").is_some());
        assert_eq!(body["choices"].as_array().unwrap().len(), 1);
        assert_eq!(body["choices"][0]["message"]["role"], "assistant");
        assert!(body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_router_service_directly() {
        // Create router service
        let service = create_mock_router_service();

        // Create test request
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![Message::new_user("Hello, world!".to_string())],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: None,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        // Create service
        let chat_service = ChatCompletionService::new(service);

        // Process request
        let response = chat_service
            .process_completion_request(&request)
            .await
            .unwrap();

        // Verify response
        assert_eq!(response.choices.len(), 1);
        assert!(response.choices[0]
            .message
            .content
            .contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_model_selection() {
        // Create router service
        let service = create_mock_router_service();

        // Test with different models
        let models = vec!["gpt-3.5-turbo", "gpt-4", "claude-3-sonnet"];

        for model in models {
            // Create test request
            let request = ChatCompletionRequest {
                model: model.to_string(),
                messages: vec![Message::new_user("Hello from test!".to_string())],
                temperature: None,
                top_p: None,
                n: None,
                stream: None,
                max_tokens: None,
                presence_penalty: None,
                frequency_penalty: None,
                user: None,
            };

            // Create service
            let chat_service = ChatCompletionService::new(service.clone());

            // Process request
            let response = chat_service
                .process_completion_request(&request)
                .await
                .unwrap();

            // Verify response
            assert_eq!(response.choices.len(), 1);
            assert!(response.choices[0]
                .message
                .content
                .contains("Hello from test!"));
            assert!(response.choices[0].message.content.contains(model));
        }
    }
}
