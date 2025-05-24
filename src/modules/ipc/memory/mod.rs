//! Memory IPC module
//!
//! This module provides trait-based abstractions for the Memory service,
//! ensuring a clear separation between interface and transport logic.

// Private module declarations
mod client;
mod grpc;
mod responses;
mod service;
mod types;

// Re-export specific types for public API
pub use client::{MemoryChainIntegrationClient, MemoryClient};
pub use grpc::GrpcMemoryClient;
pub use responses::{
    CreateConversationFromChainExecutionResponse, GetConversationHistoryForChainResponse,
    GetHistoryResponse, ListConversationsResponse, SearchMessagesResponse,
    StoreChainResultInConversationResponse,
};
pub use service::{MemoryChainIntegrationService, MemoryService};
pub use types::{Conversation, Message, MessageSearchResult, StepResult};

// TODO: Add in-memory implementation of the Memory service
// TODO: Add Redis implementation of the Memory service
// TODO: Add tests for the Memory service implementations
