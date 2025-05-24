//! Resilient IPC Client
//!
//! This module provides a wrapper around IPC clients to add retry and circuit breaker
//! functionality for improved reliability in inter-service communication.

pub mod circuit_breaker;
pub mod config;
pub mod retry;
pub mod traits;

// Client-specific modules
pub mod chain_engine_client;
pub mod memory_client;
pub mod model_registry_client;
pub mod persona_layer_client;
pub mod rag_manager_client;

// Re-export configuration functions
pub use config::{default_circuit_breaker_config, default_retry_policy};

// Re-export client implementations
pub use chain_engine_client::ResilientChainEngineClient;
pub use memory_client::ResilientMemoryClient;
pub use model_registry_client::ResilientModelRegistryClient;
pub use persona_layer_client::ResilientPersonaLayerClient;
pub use rag_manager_client::ResilientRAGManagerClient;

// Re-export traits
pub use traits::ResilientClient;
