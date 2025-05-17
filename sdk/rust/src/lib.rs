//! # IntelliRouter
//!
//! The IntelliRouter Rust SDK provides a clean, idiomatic interface for interacting with IntelliRouter,
//! including support for chat completions, streaming, and chain execution.

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;

/// Error types for the IntelliRouter SDK
#[derive(Debug, Error)]
pub enum Error {
    /// API error returned by the IntelliRouter server
    #[error("API error: {message} ({code})")]
    ApiError {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },
    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(StatusCode),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    /// Request error
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

/// Result type for the IntelliRouter SDK
pub type Result<T> = std::result::Result<T, Error>;

/// Configuration for the IntelliRouter client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the IntelliRouter API
    pub base_url: String,
    /// Timeout for requests in seconds
    pub timeout: u64,
    /// Maximum number of retries
    pub max_retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "http://localhost:8000".to_string(),
            timeout: 60,
            max_retries: 3,
        }
    }
}

/// Main client for the IntelliRouter SDK
pub struct IntelliRouter {
    client: Arc<Client>,
    config: ClientConfig,
}

impl IntelliRouter {
    /// Create a new IntelliRouter client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(ClientConfig {
            api_key: api_key.into(),
            ..Default::default()
        })
    }

    /// Create a new IntelliRouter client with the given configuration
    pub fn with_config(config: ClientConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client: Arc::new(client),
            config,
        }
    }

    /// Get the chat completions API
    pub fn chat_completions(&self) -> ChatCompletions {
        ChatCompletions {
            client: Arc::clone(&self.client),
            config: self.config.clone(),
        }
    }

    /// Get the chains API
    pub fn chains(&self) -> Chains {
        Chains {
            client: Arc::clone(&self.client),
            config: self.config.clone(),
        }
    }
}

/// Chat completions API
pub struct ChatCompletions {
    client: Arc<Client>,
    config: ClientConfig,
}

/// Chains API
pub struct Chains {
    client: Arc<Client>,
    config: ClientConfig,
}

// This is a basic skeleton - the actual implementation would include
// methods for creating chat completions, streaming responses, etc.
