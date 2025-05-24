//! Memory response types
//!
//! This module defines the response structures used by the Memory service.

use crate::modules::ipc::memory::types::{Conversation, Message, MessageSearchResult};

/// Response for get_history
#[derive(Debug, Clone)]
pub struct GetHistoryResponse {
    pub messages: Vec<Message>,
    pub total_tokens: u32,
    pub formatted_history: Option<String>,
}

/// Response for list_conversations
#[derive(Debug, Clone)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
    pub total_count: u32,
}

/// Response for search_messages
#[derive(Debug, Clone)]
pub struct SearchMessagesResponse {
    pub results: Vec<MessageSearchResult>,
    pub total_count: u32,
}

/// Response for get_conversation_history_for_chain
#[derive(Debug, Clone)]
pub struct GetConversationHistoryForChainResponse {
    pub formatted_history: String,
    pub messages: Vec<Message>,
    pub total_tokens: u32,
}

/// Response for store_chain_result_in_conversation
#[derive(Debug, Clone)]
pub struct StoreChainResultInConversationResponse {
    pub message_id: String,
    pub step_message_ids: Vec<String>,
}

/// Response for create_conversation_from_chain_execution
#[derive(Debug, Clone)]
pub struct CreateConversationFromChainExecutionResponse {
    pub conversation_id: String,
}
