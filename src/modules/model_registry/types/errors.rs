//! Error types for the model registry

use std::fmt;

/// Error types for the model registry
#[derive(Debug, Clone)]
pub enum RegistryError {
    /// Model already exists in the registry
    AlreadyExists(String),
    /// Model not found in the registry
    NotFound(String),
    /// Invalid model metadata
    InvalidMetadata(String),
    /// Error communicating with the model
    CommunicationError(String),
    /// Storage-related error
    StorageError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::AlreadyExists(msg) => write!(f, "Model already exists: {}", msg),
            RegistryError::NotFound(msg) => write!(f, "Model not found: {}", msg),
            RegistryError::InvalidMetadata(msg) => write!(f, "Invalid model metadata: {}", msg),
            RegistryError::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            RegistryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            RegistryError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}
