//! Chain Engine error types
//!
//! This module defines the error types for the Chain Engine.

use thiserror::Error;

/// Errors that can occur during chain execution
#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Step not found: {0}")]
    StepNotFound(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Step execution error: {0}")]
    StepExecutionError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Error: {0}")]
    Other(String),
}

/// Result type for Chain Engine operations
pub type ChainResult<T> = Result<T, ChainError>;
