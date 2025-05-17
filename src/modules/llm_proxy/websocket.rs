//! LLM Proxy WebSocket Support
//!
//! This module implements WebSocket support for the LLM Proxy server,
//! providing an alternative to SSE streaming for clients that require
//! bidirectional communication.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use super::{
    formatting,
    routes::{ApiError, ChatCompletionRequest},
    server::AppState,
    validation,
};

/// Handler for WebSocket upgrade
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    info!("WebSocket connection requested");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    info!("WebSocket connection established");

    // Update active connections count
    {
        let mut shared = state.shared.lock().await;
        shared.active_connections += 1;
        debug!("Active connections: {}", shared.active_connections);
    }

    let (mut sender, mut receiver) = socket.split();

    // Create a channel for sending messages to the WebSocket
    let (tx, mut rx) = mpsc::channel::<Result<Message, ApiError>>(32);

    // Spawn a task to forward messages from the channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            match result {
                Ok(msg) => {
                    if let Err(e) = sender.send(msg).await {
                        error!("Error sending WebSocket message: {}", e);
                        break;
                    }
                }
                Err(err) => {
                    // Convert API error to WebSocket message
                    let error_json = serde_json::to_string(&err).unwrap_or_default();
                    if let Err(e) = sender.send(Message::Text(error_json)).await {
                        error!("Error sending error message: {}", e);
                        break;
                    }
                    break;
                }
            }
        }
    });

    // Process incoming WebSocket messages
    while let Some(Ok(msg)) = receiver.next().await {
        match process_message(msg, &state, &tx).await {
            Ok(should_break) => {
                if should_break {
                    break;
                }
            }
            Err(e) => {
                error!("Error processing WebSocket message: {}", e);
                break;
            }
        }
    }

    // Cancel the send task when the connection is closed
    send_task.abort();

    // Update active connections count
    {
        let mut shared = state.shared.lock().await;
        shared.active_connections -= 1;
        debug!(
            "WebSocket connection closed. Active connections: {}",
            shared.active_connections
        );
    }

    info!("WebSocket connection closed");
}

/// Process a WebSocket message
pub async fn process_message(
    msg: Message,
    state: &AppState,
    tx: &mpsc::Sender<Result<Message, ApiError>>,
) -> Result<bool, String> {
    match msg {
        Message::Text(text) => {
            debug!("Received text message: {}", text);

            // Parse the message as a chat completion request
            let request: ChatCompletionRequest = match serde_json::from_str(&text) {
                Ok(req) => req,
                Err(e) => {
                    let error = super::routes::handle_invalid_request(
                        &format!("Invalid JSON: {}", e),
                        None,
                    );
                    tx.send(Err(error)).await.map_err(|e| e.to_string())?;
                    return Ok(false);
                }
            };

            // Validate the request
            if let Err(error) = validation::validate_chat_completion_request(&request) {
                tx.send(Err(error)).await.map_err(|e| e.to_string())?;
                return Ok(false);
            }

            // Handle streaming vs non-streaming requests
            if request.stream {
                handle_streaming_request(&request, tx).await?;
            } else {
                handle_non_streaming_request(&request, tx).await?;
            }

            Ok(false)
        }
        Message::Binary(_) => {
            // Binary messages are not supported
            let error =
                super::routes::handle_invalid_request("Binary messages are not supported", None);
            tx.send(Err(error)).await.map_err(|e| e.to_string())?;
            Ok(false)
        }
        Message::Ping(data) => {
            // Respond to ping with pong
            tx.send(Ok(Message::Pong(data)))
                .await
                .map_err(|e| e.to_string())?;
            Ok(false)
        }
        Message::Pong(_) => {
            // Ignore pong messages
            Ok(false)
        }
        Message::Close(_) => {
            // Client requested close
            debug!("Client requested close");
            Ok(true)
        }
    }
}

/// Handle a non-streaming chat completion request
async fn handle_non_streaming_request(
    request: &ChatCompletionRequest,
    tx: &mpsc::Sender<Result<Message, ApiError>>,
) -> Result<(), String> {
    // Extract the last user message to generate a contextual response
    let last_user_message = request
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or("Hello");

    // Generate a response based on the user's message
    let response_content = formatting::generate_contextual_response(last_user_message);

    // Apply temperature if specified
    let response_content =
        formatting::apply_temperature_effects(&response_content, request.temperature);

    // Apply max_tokens if specified
    let response_content =
        formatting::apply_max_tokens_truncation(&response_content, request.max_tokens);

    // Format the response
    let response = formatting::format_completion_response(
        &request.model,
        &request.messages,
        &response_content,
        "stop",
    );

    // Send the response as a WebSocket message
    let response_json = serde_json::to_string(&response).unwrap_or_default();
    tx.send(Ok(Message::Text(response_json)))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Handle a streaming chat completion request
async fn handle_streaming_request(
    request: &ChatCompletionRequest,
    tx: &mpsc::Sender<Result<Message, ApiError>>,
) -> Result<(), String> {
    // Extract the last user message to generate a contextual response
    let last_user_message = request
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or("Hello");

    // Generate a response based on the user's message
    let response_content = formatting::generate_contextual_response(last_user_message);

    // Apply temperature if specified
    let response_content =
        formatting::apply_temperature_effects(&response_content, request.temperature);

    // Apply max_tokens if specified
    let response_content =
        formatting::apply_max_tokens_truncation(&response_content, request.max_tokens);

    // Create streaming chunks
    let chunks = formatting::create_streaming_chunks(&request.model, &response_content, 5);

    // Send each chunk as a separate WebSocket message
    for chunk in chunks {
        let chunk_json = serde_json::to_string(&chunk).unwrap_or_default();
        tx.send(Ok(Message::Text(chunk_json)))
            .await
            .map_err(|e| e.to_string())?;

        // Add a small delay between chunks to simulate streaming
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::llm_proxy::{Provider, ServerConfig, SharedState};
    use axum::body::Body;
    use axum::extract::ws::WebSocketUpgrade;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

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
            },
            shared: Arc::new(Mutex::new(SharedState::new())),
            telemetry: None,
            cost_calculator: None,
        }
    }

    #[tokio::test]
    async fn test_ws_handler_upgrade() {
        // Create a router with the WebSocket handler
        let app = Router::new()
            .route("/ws", get(ws_handler))
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
        assert_eq!(response.status(), 101); // 101 Switching Protocols
        assert_eq!(
            response.headers().get("upgrade").unwrap().to_str().unwrap(),
            "websocket"
        );
    }
}
