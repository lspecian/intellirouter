//! Memory Module
//!
//! This module handles conversation history and memory management.
//! It provides functionality for storing, retrieving, and managing
//! conversation context across multiple interactions.

mod backend;
mod in_memory;
mod manager;
mod redis;
mod types;

// Re-export the new types and implementations
pub use backend::MemoryBackend;
pub use in_memory::InMemoryBackend;
pub use manager::MemoryManager;
pub use redis::RedisBackend;
pub use types::{Conversation, MemoryError, Message};

use uuid::Uuid;

// Provide backward-compatible functions

/// Create a new conversation
pub fn create_conversation() -> types::Conversation {
    let id = Uuid::new_v4().to_string();
    types::Conversation::new(id)
}

/// Add a message to a conversation
pub fn add_message(conversation: &mut types::Conversation, role: &str, content: &str) {
    let message = types::Message::new(role, content);
    conversation.add_message(message);
}

/// Get the conversation history formatted for an LLM request
pub fn get_history(
    conversation: &types::Conversation,
    max_messages: Option<usize>,
) -> Vec<types::Message> {
    if let Some(max) = max_messages {
        conversation.get_last_messages(max)
    } else {
        conversation.messages.clone()
    }
}

/// Save a conversation to persistent storage
///
/// Note: This is a synchronous wrapper around the async API.
/// For production use, prefer using the MemoryManager directly.
pub fn save_conversation(_conversation: &types::Conversation) -> Result<(), String> {
    // This is a synchronous wrapper around the async API
    // In a real implementation, you might use tokio::runtime::Runtime to run the async code
    Err("Use MemoryManager for async storage operations".to_string())
}

/// Load a conversation from persistent storage
///
/// Note: This is a synchronous wrapper around the async API.
/// For production use, prefer using the MemoryManager directly.
pub fn load_conversation(_id: &str) -> Result<types::Conversation, String> {
    // This is a synchronous wrapper around the async API
    Err("Use MemoryManager for async storage operations".to_string())
}

#[cfg(all(test, not(feature = "production")))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    #[test]
    fn test_backward_compatibility() {
        // Test the backward-compatible functions
        let mut conversation = create_conversation();
        add_message(&mut conversation, "user", "Hello");
        add_message(&mut conversation, "assistant", "Hi there!");

        let messages = get_history(&conversation, Some(1));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "assistant");
        assert_eq!(messages[0].content, "Hi there!");
    }

    #[test]
    fn test_memory_manager_integration() {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
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

            // Delete conversation
            manager.delete_conversation(&id).await.unwrap();
            let result = manager.get_conversation(&id).await.unwrap();
            assert!(result.is_none());
        });
    }
}
