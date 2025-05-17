use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for memory operations
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Conversation not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Lock error")]
    LockError,

    #[error("Error: {0}")]
    Other(String),
}

/// Message structure with enhanced serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Conversation structure with enhanced serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Message {
    /// Create a new message
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to a message
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl Conversation {
    /// Create a new conversation with the given ID
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            messages: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Add metadata to the conversation
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }

    /// Get the last N messages from the conversation
    pub fn get_last_messages(&self, count: usize) -> Vec<Message> {
        if count >= self.messages.len() {
            return self.messages.clone();
        }
        self.messages[self.messages.len() - count..].to_vec()
    }
}
