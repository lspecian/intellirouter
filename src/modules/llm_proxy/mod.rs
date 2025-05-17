//! LLM Proxy Module
//!
//! This module provides an OpenAI-compatible API interface for various LLM providers.
//! It handles request formatting, response parsing, and API compatibility layers.

pub mod conformance_tests;
pub mod formatting;
pub mod formatting_tests;
pub mod routes;
pub mod server;
pub mod telemetry_integration;
pub mod validation;
pub mod websocket;
pub mod websocket_tests;

use crate::config::Config;

/// LLM Provider types supported by the proxy
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Mistral,
    // Add more providers as needed
}

/// Initialize the LLM proxy with the specified provider and start the server
pub async fn init(provider: Provider, config: &Config) -> Result<(), String> {
    // Create server configuration from global config
    let server_config = server::ServerConfig::from_config(config);

    // Start the server
    server::start_server(server_config, provider).await
}

/// Initialize the LLM proxy with the specified provider but don't start the server
/// This is useful for testing or when you want to start the server manually
pub fn init_without_server(provider: Provider) -> Result<(), String> {
    // Initialize any provider-specific resources
    // This will be expanded in future implementations
    Ok(())
}

/// Send a request to the LLM provider
pub fn send_request() -> Result<String, String> {
    // TODO: Implement request sending logic
    Ok(String::new())
}

// Re-export key types from the server module
pub use server::{AppState, ServerConfig, SharedState};
