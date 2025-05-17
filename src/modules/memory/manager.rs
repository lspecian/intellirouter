use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::modules::memory::backend::MemoryBackend;
use crate::modules::memory::types::{Conversation, MemoryError, Message};

/// Memory manager for handling conversation history with windowing support
pub struct MemoryManager {
    backend: Arc<dyn MemoryBackend>,
    window_size: usize,
}

impl MemoryManager {
    /// Create a new memory manager with the specified backend and window size
    pub fn new(backend: Arc<dyn MemoryBackend>, window_size: usize) -> Self {
        Self {
            backend,
            window_size,
        }
    }

    /// Create a new conversation
    pub async fn create_conversation(&self) -> Result<Conversation, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let conversation = Conversation::new(id);

        self.backend.save_conversation(conversation.clone()).await?;

        Ok(conversation)
    }

    /// Get a conversation by ID
    pub async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, MemoryError> {
        self.backend.get_conversation(id).await
    }

    /// Add a message to a conversation
    pub async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), MemoryError> {
        let mut conversation = match self.backend.get_conversation(conversation_id).await? {
            Some(conv) => conv,
            None => return Err(MemoryError::NotFound(conversation_id.to_string())),
        };

        let message = Message::new(role, content);
        conversation.add_message(message);

        // Apply windowing if needed
        if self.window_size > 0 && conversation.messages.len() > self.window_size {
            conversation.messages = conversation
                .messages
                .split_off(conversation.messages.len() - self.window_size);
        }

        self.backend.save_conversation(conversation).await
    }

    /// Add a message with metadata to a conversation
    pub async fn add_message_with_metadata(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
    ) -> Result<(), MemoryError> {
        let mut conversation = match self.backend.get_conversation(conversation_id).await? {
            Some(conv) => conv,
            None => return Err(MemoryError::NotFound(conversation_id.to_string())),
        };

        let mut message = Message::new(role, content);
        message.metadata = metadata;

        conversation.add_message(message);

        // Apply windowing if needed
        if self.window_size > 0 && conversation.messages.len() > self.window_size {
            conversation.messages = conversation
                .messages
                .split_off(conversation.messages.len() - self.window_size);
        }

        self.backend.save_conversation(conversation).await
    }

    /// Get all messages from a conversation
    pub async fn get_messages(&self, conversation_id: &str) -> Result<Vec<Message>, MemoryError> {
        let conversation = match self.backend.get_conversation(conversation_id).await? {
            Some(conv) => conv,
            None => return Err(MemoryError::NotFound(conversation_id.to_string())),
        };

        Ok(conversation.messages)
    }

    /// Get the last N messages from a conversation
    pub async fn get_last_messages(
        &self,
        conversation_id: &str,
        count: usize,
    ) -> Result<Vec<Message>, MemoryError> {
        let conversation = match self.backend.get_conversation(conversation_id).await? {
            Some(conv) => conv,
            None => return Err(MemoryError::NotFound(conversation_id.to_string())),
        };

        Ok(conversation.get_last_messages(count))
    }

    /// Delete a conversation
    pub async fn delete_conversation(&self, id: &str) -> Result<(), MemoryError> {
        self.backend.delete_conversation(id).await
    }

    /// List all conversation IDs
    pub async fn list_conversations(&self) -> Result<Vec<String>, MemoryError> {
        self.backend.list_conversations().await
    }

    /// Add metadata to a conversation
    pub async fn add_metadata(
        &self,
        conversation_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), MemoryError> {
        let mut conversation = match self.backend.get_conversation(conversation_id).await? {
            Some(conv) => conv,
            None => return Err(MemoryError::NotFound(conversation_id.to_string())),
        };

        conversation.add_metadata(key, value);

        self.backend.save_conversation(conversation).await
    }

    /// Get the window size
    pub fn get_window_size(&self) -> usize {
        self.window_size
    }

    /// Set the window size
    pub fn set_window_size(&mut self, window_size: usize) {
        self.window_size = window_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::memory::in_memory::InMemoryBackend;

    #[tokio::test]
    async fn test_memory_manager() {
        let backend = Arc::new(InMemoryBackend::new());
        let manager = MemoryManager::new(backend, 5);

        // Create a conversation
        let conversation = manager.create_conversation().await.unwrap();
        let id = conversation.id.clone();

        // Add messages
        manager.add_message(&id, "user", "Hello").await.unwrap();
        manager
            .add_message(&id, "assistant", "Hi there!")
            .await
            .unwrap();

        // Get messages
        let messages = manager.get_messages(&id).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");

        // Test windowing
        for i in 0..10 {
            manager
                .add_message(&id, "user", &format!("Message {}", i))
                .await
                .unwrap();
        }

        let messages = manager.get_messages(&id).await.unwrap();
        assert_eq!(messages.len(), 5); // Window size is 5
        assert_eq!(messages[0].content, "Message 7");
        assert_eq!(messages[4].content, "Message 11");

        // Test get_last_messages
        let last_messages = manager.get_last_messages(&id, 3).await.unwrap();
        assert_eq!(last_messages.len(), 3);
        assert_eq!(last_messages[0].content, "Message 9");
        assert_eq!(last_messages[2].content, "Message 11");

        // Test metadata
        manager
            .add_metadata(&id, "test_key", "test_value")
            .await
            .unwrap();
        let conversation = manager.get_conversation(&id).await.unwrap().unwrap();
        assert_eq!(conversation.metadata.get("test_key").unwrap(), "test_value");

        // Delete conversation
        manager.delete_conversation(&id).await.unwrap();
        let result = manager.get_conversation(&id).await.unwrap();
        assert!(result.is_none());
    }
}
