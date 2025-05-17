use async_trait::async_trait;
use redis::AsyncCommands;
use serde_json;

use crate::modules::memory::backend::MemoryBackend;
use crate::modules::memory::types::{Conversation, MemoryError};

/// Redis backend implementation for persistent storage
pub struct RedisBackend {
    client: redis::Client,
    prefix: String,
}

impl RedisBackend {
    /// Create a new Redis backend
    pub fn new(redis_url: &str, prefix: &str) -> Result<Self, MemoryError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| MemoryError::StorageError(format!("Redis connection error: {}", e)))?;

        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }

    /// Generate a Redis key with the configured prefix
    fn get_key(&self, id: &str) -> String {
        format!("{}:{}", self.prefix, id)
    }
}

#[async_trait]
impl MemoryBackend for RedisBackend {
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis connection error: {}", e)))?;

        let key = self.get_key(id);
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis error: {}", e)))?;

        if !exists {
            return Ok(None);
        }

        let json: String = conn
            .get(&key)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis error: {}", e)))?;

        let conversation: Conversation = serde_json::from_str(&json).map_err(|e| {
            MemoryError::SerializationError(format!("Deserialization error: {}", e))
        })?;

        Ok(Some(conversation))
    }

    async fn save_conversation(&self, conversation: Conversation) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis connection error: {}", e)))?;

        let key = self.get_key(&conversation.id);
        let json = serde_json::to_string(&conversation)
            .map_err(|e| MemoryError::SerializationError(format!("Serialization error: {}", e)))?;

        conn.set(&key, json)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis error: {}", e)))?;

        Ok(())
    }

    async fn delete_conversation(&self, id: &str) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis connection error: {}", e)))?;

        let key = self.get_key(id);
        conn.del(&key)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis error: {}", e)))?;

        Ok(())
    }

    async fn list_conversations(&self) -> Result<Vec<String>, MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis connection error: {}", e)))?;

        let pattern = format!("{}:*", self.prefix);
        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(|e| MemoryError::StorageError(format!("Redis error: {}", e)))?;

        let ids = keys
            .iter()
            .map(|key| {
                key.strip_prefix(&format!("{}:", self.prefix))
                    .unwrap_or(key)
                    .to_string()
            })
            .collect();

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::memory::types::Message;
    use mockito;

    // This test is marked as ignore because it requires a Redis server
    // To run this test: cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn test_redis_backend() {
        // This test requires a running Redis server
        let redis_url = "redis://127.0.0.1:6379";
        let backend = RedisBackend::new(redis_url, "test").unwrap();

        // Create a test conversation
        let mut conversation = Conversation::new("redis-test-id".to_string());
        conversation.add_message(Message::new("user", "Hello from Redis test"));

        // Save the conversation
        backend
            .save_conversation(conversation.clone())
            .await
            .unwrap();

        // Retrieve the conversation
        let retrieved = backend
            .get_conversation("redis-test-id")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, "redis-test-id");
        assert_eq!(retrieved.messages.len(), 1);
        assert_eq!(retrieved.messages[0].role, "user");
        assert_eq!(retrieved.messages[0].content, "Hello from Redis test");

        // List conversations
        let conversations = backend.list_conversations().await.unwrap();
        assert!(conversations.contains(&"redis-test-id".to_string()));

        // Delete the conversation
        backend.delete_conversation("redis-test-id").await.unwrap();

        // Verify it's gone
        let result = backend.get_conversation("redis-test-id").await.unwrap();
        assert!(result.is_none());
    }
}
