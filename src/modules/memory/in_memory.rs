use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::memory::backend::MemoryBackend;
use crate::modules::memory::types::{Conversation, MemoryError};

/// In-memory backend implementation using a HashMap protected by a Mutex
pub struct InMemoryBackend {
    conversations: Arc<Mutex<HashMap<String, Conversation>>>,
}

impl InMemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            conversations: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MemoryBackend for InMemoryBackend {
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, MemoryError> {
        let conversations = self
            .conversations
            .lock()
            .map_err(|_| MemoryError::LockError)?;

        Ok(conversations.get(id).cloned())
    }

    async fn save_conversation(&self, conversation: Conversation) -> Result<(), MemoryError> {
        let mut conversations = self
            .conversations
            .lock()
            .map_err(|_| MemoryError::LockError)?;

        conversations.insert(conversation.id.clone(), conversation);
        Ok(())
    }

    async fn delete_conversation(&self, id: &str) -> Result<(), MemoryError> {
        let mut conversations = self
            .conversations
            .lock()
            .map_err(|_| MemoryError::LockError)?;

        conversations.remove(id);
        Ok(())
    }

    async fn list_conversations(&self) -> Result<Vec<String>, MemoryError> {
        let conversations = self
            .conversations
            .lock()
            .map_err(|_| MemoryError::LockError)?;

        Ok(conversations.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::memory::types::Message;
    use chrono::Utc;

    #[tokio::test]
    async fn test_in_memory_backend() {
        let backend = InMemoryBackend::new();

        // Create a test conversation
        let mut conversation = Conversation::new("test-id".to_string());
        conversation.add_message(Message::new("user", "Hello"));

        // Save the conversation
        backend
            .save_conversation(conversation.clone())
            .await
            .unwrap();

        // Retrieve the conversation
        let retrieved = backend.get_conversation("test-id").await.unwrap().unwrap();
        assert_eq!(retrieved.id, "test-id");
        assert_eq!(retrieved.messages.len(), 1);
        assert_eq!(retrieved.messages[0].role, "user");
        assert_eq!(retrieved.messages[0].content, "Hello");

        // List conversations
        let conversations = backend.list_conversations().await.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0], "test-id");

        // Delete the conversation
        backend.delete_conversation("test-id").await.unwrap();

        // Verify it's gone
        let result = backend.get_conversation("test-id").await.unwrap();
        assert!(result.is_none());
    }
}
