//! IPC infrastructure for multi-role deployment
//!
//! This module provides trait-based abstractions for gRPC service interfaces,
//! ensuring a clear separation between interface and transport logic.

pub mod chain_engine;
pub mod events;
pub mod memory;
pub mod model_registry;
pub mod persona_layer;
pub mod rag_manager;
pub mod redis_pubsub;
pub mod security;

#[cfg(test)]
pub mod tests;

/// Common error type for IPC operations
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Security error: {0}")]
    Security(String),
}

/// Result type for IPC operations
pub type IpcResult<T> = Result<T, IpcError>;
