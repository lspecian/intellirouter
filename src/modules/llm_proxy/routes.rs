//! LLM Proxy Routes
//!
//! This module implements the route handlers for the LLM Proxy server,
//! providing OpenAI-compatible API endpoints.

use axum::{
    extract::State,
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    Json,
};
use futures::stream;
use std::convert::Infallible;
use std::time::Duration;

use super::dto::{ApiError, ChatCompletionRequest, ChatCompletionResponse};
use super::server::AppState;
use super::service::ChatCompletionService;
use super::validation;
use crate::modules::router_core::RouterError;

/// Validate service health before handling requests
async fn validate_service_health(state: &AppState) -> Result<(), ApiError> {
    // Check if the service is shutting down
    let shared_state = state.shared.lock().await;
    if shared_state.shutting_down {
        return Err(ApiError {
            error: super::dto::ApiErrorDetail {
                message: "Service is shutting down".to_string(),
                r#type: "service_unavailable".to_string(),
                param: None,
                code: None,
            },
        });
    }

    // Check if the service has reached max connections
    if shared_state.active_connections >= state.config.max_connections {
        return Err(ApiError {
            error: super::dto::ApiErrorDetail {
                message: "Service is at maximum capacity".to_string(),
                r#type: "service_unavailable".to_string(),
                param: None,
                code: None,
            },
        });
    }

    // Additional health checks could be added here
    // For example, checking if dependent services are available

    Ok(())
}

/// Route handler for /v1/chat/completions
#[axum::debug_handler]
pub async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, ApiError> {
    // Removed debug log

    // Validate service health before processing the request
    validate_service_health(&state).await?;

    // Check if streaming is requested and redirect to streaming handler
    if request.stream {
        return Err(validation::create_validation_error(
            "Streaming requests should be sent to /v1/chat/completions/stream endpoint",
            Some("stream"),
        ));
    }

    // Validate the request
    validation::validate_chat_completion_request(&request)?;

    // Create service with appropriate router
    #[cfg(feature = "test-utils")]
    let service = ChatCompletionService::new_with_mock_router();

    #[cfg(not(feature = "test-utils"))]
    {
        // In a real implementation, we would create a router service here
        // For now, use the legacy method
        return Ok(Json(
            ChatCompletionService::legacy_process_completion_request(&request),
        ));
    }

    // Process the request using the service (only reached when test-utils is enabled)
    #[cfg(feature = "test-utils")]
    match service.process_completion_request(&request).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            error!("Error processing completion request: {}", err);
            Err(convert_router_error_to_api_error(err))
        }
    }
}

/// Route handler for /v1/chat/completions/stream
#[axum::debug_handler]
pub async fn chat_completions_stream(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, ApiError> {
    // Removed debug log

    // Validate service health before processing the request
    validate_service_health(&state).await?;

    // Validate the request
    validation::validate_chat_completion_request(&request)?;

    // Create service with appropriate router (not used directly in this implementation)
    #[cfg(feature = "test-utils")]
    let _service = ChatCompletionService::new_with_mock_router();

    #[cfg(not(feature = "test-utils"))]
    let _service = {
        // In a real implementation, we would create a router service here
        // But for streaming, we're using the legacy method anyway
    };

    // For now, use the legacy method for streaming
    // In a real implementation, we would use the router service
    let chunks = ChatCompletionService::legacy_generate_streaming_chunks(&request, 5);

    // Create a stream from the chunks
    let stream = futures::StreamExt::map(stream::iter(chunks.into_iter()), move |chunk| {
        let json = serde_json::to_string(&chunk).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(json))
    });

    // Apply throttling and boxing
    let stream = tokio_stream::StreamExt::throttle(stream, Duration::from_millis(300));
    let stream = futures::StreamExt::boxed(stream);

    // Return the SSE stream wrapped in a Response
    Ok(Sse::new(stream).into_response())
}

/// Convert a router error to an API error
fn _convert_router_error_to_api_error(err: RouterError) -> ApiError {
    match err {
        RouterError::NoSuitableModel(msg) => validation::create_validation_error(
            &format!("No suitable model found: {}", msg),
            Some("model"),
        ),
        RouterError::ConnectorError(msg) => ApiError {
            error: super::dto::ApiErrorDetail {
                message: format!("Model connector error: {}", msg),
                r#type: "model_connector_error".to_string(),
                param: None,
                code: None,
            },
        },
        RouterError::Timeout(msg) => ApiError {
            error: super::dto::ApiErrorDetail {
                message: format!("Request timed out: {}", msg),
                r#type: "timeout".to_string(),
                param: None,
                code: None,
            },
        },
        _ => ApiError {
            error: super::dto::ApiErrorDetail {
                message: format!("Router error: {}", err),
                r#type: "router_error".to_string(),
                param: None,
                code: None,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::llm_proxy::domain::message::{Message, MessageRole};
    use crate::modules::telemetry::{CostCalculator, TelemetryManager};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_chat_completions() {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            provider: super::Provider::OpenAI,
            config: super::server::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 1000,
                request_timeout_secs: 30,
                cors_enabled: false,
                cors_allowed_origins: vec!["*".to_string()],
                redis_url: None,
            },
            shared: std::sync::Arc::new(tokio::sync::Mutex::new(super::server::SharedState::new())),
            telemetry: Some(telemetry),
            cost_calculator: Some(cost_calculator),
        };

        // Create test request
        let request = ChatCompletionRequest {
            model: "claude-3-sonnet".to_string(),
            messages: vec![Message::new_user("Hello!".to_string())],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: false,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        // Call the handler
        let result = chat_completions(State(app_state), Json(request)).await;

        // Verify the result
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
    }

    #[tokio::test]
    async fn test_chat_completions_stream() {
        // Create test app state
        let telemetry = Arc::new(TelemetryManager::new_for_testing());
        let cost_calculator = Arc::new(CostCalculator::new());

        let app_state = AppState {
            provider: super::Provider::OpenAI,
            config: super::server::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 1000,
                request_timeout_secs: 30,
                cors_enabled: false,
                cors_allowed_origins: vec!["*".to_string()],
                redis_url: None,
            },
            shared: std::sync::Arc::new(tokio::sync::Mutex::new(super::server::SharedState::new())),
            telemetry: Some(telemetry),
            cost_calculator: Some(cost_calculator),
        };

        // Create test request
        let request = ChatCompletionRequest {
            model: "claude-3-sonnet".to_string(),
            messages: vec![Message::new_user("Hello!".to_string())],
            temperature: Some(0.7),
            top_p: None,
            n: None,
            stream: true,
            max_tokens: Some(100),
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        // Call the handler
        let result = chat_completions_stream(State(app_state), Json(request)).await;

        // Verify the result
        assert!(result.is_ok());
    }
}
