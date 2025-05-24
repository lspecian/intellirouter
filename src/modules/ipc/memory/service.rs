//! Memory service interfaces
//!
//! This module defines the service interfaces for the Memory service.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::modules::ipc::memory::responses::{
    CreateConversationFromChainExecutionResponse, GetConversationHistoryForChainResponse,
    GetHistoryResponse, ListConversationsResponse, SearchMessagesResponse,
    StoreChainResultInConversationResponse,
};
use crate::modules::ipc::memory::types::{Conversation, Message, StepResult};
use crate::modules::ipc::IpcResult;

/// Server interface for the Memory service
#[async_trait]
pub trait MemoryService: Send + Sync {
    /// Create a new conversation
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<Conversation>;

    /// Get a conversation by ID
    async fn get_conversation(&self, conversation_id: &str) -> IpcResult<Conversation>;

    /// Add a message to a conversation
    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<&str>,
    ) -> IpcResult<Message>;

    /// Get the conversation history formatted for an LLM request
    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
    ) -> IpcResult<GetHistoryResponse>;

    /// Save a conversation to persistent storage
    async fn save_conversation(&self, conversation_id: &str) -> IpcResult<()>;

    /// Load a conversation from persistent storage
    async fn load_conversation(&self, conversation_id: &str) -> IpcResult<Conversation>;

    /// Delete a conversation
    async fn delete_conversation(&self, conversation_id: &str) -> IpcResult<()>;

    /// List all conversations
    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<&str>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse>;

    /// Search for messages across conversations
    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<&str>,
        user_id: Option<&str>,
        role: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse>;
}

/// Server interface for the Memory-Chain integration service
#[async_trait]
pub trait MemoryChainIntegrationService: Send + Sync {
    /// Get conversation history formatted for a chain
    async fn get_conversation_history_for_chain(
        &self,
        conversation_id: &str,
        chain_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
        additional_context: Option<&str>,
    ) -> IpcResult<GetConversationHistoryForChainResponse>;

    /// Store a chain result in a conversation
    async fn store_chain_result_in_conversation(
        &self,
        conversation_id: &str,
        chain_id: &str,
        execution_id: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        metadata: HashMap<String, String>,
    ) -> IpcResult<StoreChainResultInConversationResponse>;

    /// Create a new conversation from a chain execution
    async fn create_conversation_from_chain_execution(
        &self,
        chain_id: &str,
        execution_id: &str,
        input: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> IpcResult<CreateConversationFromChainExecutionResponse>;
}
