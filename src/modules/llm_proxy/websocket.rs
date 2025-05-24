//! WebSocket support for LLM Proxy
//!
//! This module implements WebSocket support for the LLM Proxy, allowing
//! for streaming responses and bidirectional communication.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use super::{
    dto::{ApiError, ChatCompletionRequest},
    formatting,
    server::AppState,
    validation,
};

/// Handler for WebSocket connections
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Create a channel for sending messages back to the client
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
                    // Convert the error to a JSON message
                    let error_json = serde_json::to_string(&err).unwrap_or_else(|e| {
                        format!(
                            "{{\"error\":{{\"message\":\"Failed to serialize error: {}\"}}}}",
                            e
                        )
                    });
                    if let Err(e) = sender.send(Message::Text(error_json.into())).await {
                        error!("Error sending error message: {}", e);
                        break;
                    }
                }
            }
        }
    });

    // Process incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let should_close = match process_message(msg, &state, &tx).await {
                Ok(close) => close,
                Err(e) => {
                    error!("Error processing message: {}", e);
                    true
                }
            };

            if should_close {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    info!("WebSocket connection closed");
}

/// Process a WebSocket message
pub async fn process_message(
    msg: Message,
    _state: &AppState,
    tx: &mpsc::Sender<Result<Message, ApiError>>,
) -> Result<bool, String> {
    match msg {
        Message::Text(text) => {
            debug!("Received text message: {}", text);

            // Parse the message as a chat completion request
            let request: ChatCompletionRequest = match serde_json::from_str(&text) {
                Ok(req) => req,
                Err(e) => {
                    let error =
                        validation::create_validation_error(&format!("Invalid JSON: {}", e), None);
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
                validation::create_validation_error("Binary messages are not supported", None);
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
            Ok(true)
        }
    }
}

/// Handle a non-streaming request
async fn handle_non_streaming_request(
    request: &ChatCompletionRequest,
    tx: &mpsc::Sender<Result<Message, ApiError>>,
) -> Result<(), String> {
    // Extract the last user message to generate a contextual response
    let last_user_message = request
        .messages
        .iter()
        .filter(|m| m.role.to_string() == "user")
        .last()
        .map(|m| m.extract_text_content())
        .unwrap_or_else(|| "Hello".to_string());

    // Generate a response based on the user's message
    let response_content = formatting::generate_contextual_response(&last_user_message);

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

    // Send the response
    let response_json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
    tx.send(Ok(Message::Text(response_json.into())))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Handle a streaming request
async fn handle_streaming_request(
    request: &ChatCompletionRequest,
    tx: &mpsc::Sender<Result<Message, ApiError>>,
) -> Result<(), String> {
    // Extract the last user message to generate a contextual response
    let last_user_message = request
        .messages
        .iter()
        .filter(|m| m.role.to_string() == "user")
        .last()
        .map(|m| m.extract_text_content())
        .unwrap_or_else(|| "Hello".to_string());

    // Generate a response based on the user's message
    let response_content = formatting::generate_contextual_response(&last_user_message);

    // Apply temperature if specified
    let response_content =
        formatting::apply_temperature_effects(&response_content, request.temperature);

    // Apply max_tokens if specified
    let response_content =
        formatting::apply_max_tokens_truncation(&response_content, request.max_tokens);

    // Create streaming chunks
    let chunks = formatting::create_streaming_chunks(&request.model, &response_content, 5);

    // Send each chunk
    for chunk in chunks {
        let chunk_json = serde_json::to_string(&chunk).map_err(|e| e.to_string())?;
        tx.send(Ok(Message::Text(
            format!("data: {}\n\n", chunk_json).into(),
        )))
        .await
        .map_err(|e| e.to_string())?;
    }

    // Send the final [DONE] message
    tx.send(Ok(Message::Text("data: [DONE]\n\n".to_string().into())))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::llm_proxy::domain::message::Message as DomainMessage;
    use crate::modules::telemetry::{CostCalculator, TelemetryManager};
    use axum::extract::ws::Message as WsMessage;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_process_message_invalid_json() {
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

        // Create a channel for testing
        let (tx, mut rx) = mpsc::channel::<Result<WsMessage, ApiError>>(32);

        // Process an invalid JSON message
        let result =
            process_message(WsMessage::Text("invalid json".to_string()), &app_state, &tx).await;

        // Verify the result
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should not close the connection

        // Verify that an error was sent
        let response = rx.recv().await.unwrap();
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert!(error.error.message.contains("Invalid JSON"));
    }
}
