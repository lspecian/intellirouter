//! IPC Utilities
//!
//! This module provides utility functions for IPC operations.

use crate::config::Config;
use crate::modules::ipc::{
    IpcResult, ResilientChainEngineClient, ResilientMemoryClient,
    ResilientModelRegistryClient, ResilientPersonaLayerClient, ResilientRAGManagerClient,
};

/// A collection of resilient clients for all services
pub struct ResilientClients {
    /// Chain Engine client
    pub chain_engine: Option<ResilientChainEngineClient>,
    /// Memory client
    pub memory: Option<ResilientMemoryClient>,
    /// Model Registry client
    pub model_registry: Option<ResilientModelRegistryClient>,
    /// Persona Layer client
    pub persona_layer: Option<ResilientPersonaLayerClient>,
    /// RAG Manager client
    pub rag_manager: Option<ResilientRAGManagerClient>,
}

// Since we can't derive Clone for ResilientClients because the client types don't implement Clone,
// we'll just implement a method to get a reference to the clients
impl ResilientClients {
    /// Get a reference to the chain engine client
    pub fn chain_engine(&self) -> Option<&ResilientChainEngineClient> {
        self.chain_engine.as_ref()
    }

    /// Get a reference to the memory client
    pub fn memory(&self) -> Option<&ResilientMemoryClient> {
        self.memory.as_ref()
    }

    /// Get a reference to the model registry client
    pub fn model_registry(&self) -> Option<&ResilientModelRegistryClient> {
        self.model_registry.as_ref()
    }

    /// Get a reference to the persona layer client
    pub fn persona_layer(&self) -> Option<&ResilientPersonaLayerClient> {
        self.persona_layer.as_ref()
    }

    /// Get a reference to the RAG manager client
    pub fn rag_manager(&self) -> Option<&ResilientRAGManagerClient> {
        self.rag_manager.as_ref()
    }
}

/// Create all resilient clients based on configuration
pub async fn create_all_resilient_clients(config: &Config) -> IpcResult<ResilientClients> {
    let host = config.server.host.to_string();
    let base_port = config.server.port;

    // Create clients with appropriate ports
    let chain_engine =
        super::create_resilient_chain_engine_client(&format!("http://{}:{}", host, base_port + 1))
            .await
            .ok();

    let memory = super::create_resilient_memory_client(&format!("http://{}:{}", host, base_port))
        .await
        .ok();

    let model_registry =
        super::create_resilient_model_registry_client(&format!("http://{}:{}", host, base_port))
            .await
            .ok();

    let persona_layer =
        super::create_resilient_persona_layer_client(&format!("http://{}:{}", host, base_port + 3))
            .await
            .ok();

    let rag_manager =
        super::create_resilient_rag_manager_client(&format!("http://{}:{}", host, base_port + 2))
            .await
            .ok();

    Ok(ResilientClients {
        chain_engine,
        memory,
        model_registry,
        persona_layer,
        rag_manager,
    })
}
