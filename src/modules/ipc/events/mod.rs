//! Event definitions for IPC communication
//!
//! This module contains event definitions for asynchronous communication
//! between IntelliRouter modules using Redis pub/sub.

pub mod chain_engine_router_core;
pub mod memory_chain_engine;
pub mod rag_manager_persona_layer;
pub mod router_core_model_registry;

// Re-export common types
pub use chain_engine_router_core::{
    ChainEngineEvent, ChainEngineEventPublisher, ChainExecutionCompletedEvent,
    ChainExecutionFailedEvent, ChainStepCompletedEvent, RouterCoreEventSubscriber,
};

pub use memory_chain_engine::{
    ChainEngineMemorySubscriber, ConversationHistoryRetrievedEvent, ConversationMessage,
    ConversationUpdatedEvent, MemoryEvent, MemoryEventPublisher,
};

pub use rag_manager_persona_layer::{
    ContextAugmentationEvent, DocumentIndexedEvent, DocumentRetrievalEvent,
    PersonaLayerEventSubscriber, RagManagerEvent, RagManagerEventPublisher,
};

pub use router_core_model_registry::{
    ModelHealthCheckEvent, ModelRegistryEventSubscriber, ModelRoutingDecisionEvent,
    ModelUsageEvent, RouterCoreEvent, RouterCoreEventPublisher,
};
