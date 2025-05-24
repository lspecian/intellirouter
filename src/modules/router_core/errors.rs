//! Error types for the router core module

use thiserror::Error;

use crate::modules::model_registry::{ConnectorError, RegistryError};

/// Error types for the router core module
#[derive(Error, Debug, Clone)]
pub enum RouterError {
    /// No suitable model found for routing
    #[error("No suitable model found: {0}")]
    NoSuitableModel(String),

    /// Model registry error
    #[error("Model registry error: {0}")]
    RegistryError(#[from] RegistryError),

    /// Model connector error
    #[error("Model connector error: {0}")]
    ConnectorError(String),

    /// Strategy configuration error
    #[error("Strategy configuration error: {0}")]
    StrategyConfigError(String),

    /// Invalid request error
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Routing timeout error
    #[error("Routing timeout: {0}")]
    Timeout(String),

    /// Fallback error (when all fallbacks fail)
    #[error("All fallbacks failed: {0}")]
    FallbackError(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<ConnectorError> for RouterError {
    fn from(error: ConnectorError) -> Self {
        RouterError::ConnectorError(error.to_string())
    }
}
