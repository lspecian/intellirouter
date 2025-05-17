use crate::modules::memory::types::{Conversation, MemoryError};
use async_trait::async_trait;

/// Memory backend trait for different storage implementations
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Get a conversation by ID
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, MemoryError>;

    /// Save a conversation
    async fn save_conversation(&self, conversation: Conversation) -> Result<(), MemoryError>;

    /// Delete a conversation
    async fn delete_conversation(&self, id: &str) -> Result<(), MemoryError>;

    /// List all conversation IDs
    async fn list_conversations(&self) -> Result<Vec<String>, MemoryError>;
}
