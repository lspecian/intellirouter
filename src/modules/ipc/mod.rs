//! IPC Module
//!
//! This module provides inter-process communication functionality for the IntelliRouter system.

pub mod chain_engine;
pub mod memory;
pub mod model_registry;
pub mod persona_layer;
pub mod rag_manager;
pub mod redis_pubsub;
pub mod resilient;
pub mod security;
pub mod utils;


// Re-export resilient clients
pub use resilient::{
    ResilientChainEngineClient, ResilientMemoryClient, ResilientModelRegistryClient,
    ResilientPersonaLayerClient, ResilientRAGManagerClient,
};

// Re-export client implementations
pub use chain_engine::ChainEngineClient;
pub use memory::MemoryClient;
pub use model_registry::ModelRegistryClient;
pub use persona_layer::PersonaLayerClient;
pub use rag_manager::RAGManagerClient;
pub use redis_pubsub::{ChannelName, EventPayload, Message, RedisClient, Subscription};

// Re-export security
pub use security::{JwtAuthenticator, JwtConfig, TlsConfig};

/// IPC Error
#[derive(Debug, Clone)]
pub enum IpcError {
    /// Connection error
    ConnectionError(String),
    /// Authentication error
    AuthenticationError(String),
    /// Authorization error
    AuthorizationError(String),
    /// Timeout error
    Timeout(String),
    /// Service unavailable
    Unavailable(String),
    /// Transport error
    TransportError(String),
    /// Circuit breaker open
    CircuitOpen(String),
    /// Invalid request
    InvalidRequest(String),
    /// Internal error
    InternalError(String),
    /// Connection error (used by redis_pubsub.rs)
    Connection(String),
    /// Serialization error (used by security.rs)
    Serialization(String),
    /// Security error
    Security(String),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            IpcError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            IpcError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            IpcError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            IpcError::Unavailable(msg) => write!(f, "Service unavailable: {}", msg),
            IpcError::TransportError(msg) => write!(f, "Transport error: {}", msg),
            IpcError::CircuitOpen(msg) => write!(f, "Circuit breaker open: {}", msg),
            IpcError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            IpcError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            IpcError::Connection(msg) => write!(f, "Connection error: {}", msg),
            IpcError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            IpcError::Security(msg) => write!(f, "Security error: {}", msg),
        }
    }
}

impl std::error::Error for IpcError {}

/// IPC Result
pub type IpcResult<T> = Result<T, IpcError>;

/// Create a resilient chain engine client
pub async fn create_resilient_chain_engine_client(
    addr: &str,
) -> IpcResult<ResilientChainEngineClient> {
    let client = chain_engine::GrpcChainEngineClient::new(addr)
        .await
        .map_err(|e| IpcError::ConnectionError(e.to_string()))?;

    // Create a new resilient client with default configuration
    let resilient_client = ResilientChainEngineClient::new(
        client,
        resilient::config::ResilientClientConfig::default(),
    );
    Ok(resilient_client)
}

/// Create a resilient memory client
pub async fn create_resilient_memory_client(addr: &str) -> IpcResult<ResilientMemoryClient> {
    let client = memory::GrpcMemoryClient::new(addr)
        .await
        .map_err(|e| IpcError::ConnectionError(e.to_string()))?;

    // Create a new resilient client with default configuration
    let resilient_client =
        ResilientMemoryClient::new(client, resilient::config::ResilientClientConfig::default());
    Ok(resilient_client)
}

/// Create a resilient model registry client
pub async fn create_resilient_model_registry_client(
    addr: &str,
) -> IpcResult<ResilientModelRegistryClient> {
    let client = model_registry::GrpcModelRegistryClient::new(addr)
        .await
        .map_err(|e| IpcError::ConnectionError(e.to_string()))?;

    // Create a new resilient client with default configuration
    let resilient_client = ResilientModelRegistryClient::new(
        client,
        resilient::config::ResilientClientConfig::default(),
    );
    Ok(resilient_client)
}

/// Create a resilient persona layer client
pub async fn create_resilient_persona_layer_client(
    addr: &str,
) -> IpcResult<ResilientPersonaLayerClient> {
    let client = persona_layer::GrpcPersonaLayerClient::new(addr)
        .await
        .map_err(|e| IpcError::ConnectionError(e.to_string()))?;

    // Create a new resilient client with default configuration
    let resilient_client = ResilientPersonaLayerClient::new(
        client,
        resilient::config::ResilientClientConfig::default(),
    );
    Ok(resilient_client)
}

/// Create a resilient RAG manager client
pub async fn create_resilient_rag_manager_client(
    addr: &str,
) -> IpcResult<ResilientRAGManagerClient> {
    let client = rag_manager::GrpcRAGManagerClient::new(addr)
        .await
        .map_err(|e| IpcError::ConnectionError(e.to_string()))?;

    // Create a new resilient client with default configuration
    let resilient_client =
        ResilientRAGManagerClient::new(client, resilient::config::ResilientClientConfig::default());
    Ok(resilient_client)
}
