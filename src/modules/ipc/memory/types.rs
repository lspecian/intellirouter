//! Memory data types
//!
//! This module defines the data structures used by the Memory service.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Represents a message in a conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub id: String,
    pub parent_id: Option<String>,
    pub token_count: Option<u32>,
}

/// Represents a conversation
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}

/// Represents a message search result
#[derive(Debug, Clone)]
pub struct MessageSearchResult {
    pub message: Message,
    pub conversation_id: String,
    pub conversation_title: Option<String>,
    pub score: f32,
    pub highlighted_content: String,
}

/// Represents a step result for chain integration
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub step_name: String,
    pub input: String,
    pub output: String,
    pub model: String,
}
