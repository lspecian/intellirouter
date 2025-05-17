//! Guardrails for persona-based interactions
//!
//! This module defines various guardrails that can be applied to personas
//! to control the behavior of the LLM.

use serde::{Deserialize, Serialize};

/// Guardrail for controlling LLM behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Guardrail {
    /// Filter content based on patterns
    ContentFilter(ContentFilter),

    /// Restrict topics
    TopicRestriction(TopicRestriction),

    /// Format responses
    ResponseFormat(ResponseFormat),
}

/// Content filter guardrail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilter {
    /// Patterns to filter (regex strings)
    pub patterns: Vec<String>,

    /// Whether to block or just warn
    pub block_content: bool,

    /// Custom message to return when content is blocked
    pub block_message: Option<String>,
}

/// Topic restriction guardrail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicRestriction {
    /// List of forbidden topics
    pub forbidden_topics: Vec<String>,

    /// Whether to block or just warn
    pub block_content: bool,

    /// Custom message to return when a topic is forbidden
    pub block_message: Option<String>,
}

/// Response format guardrail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Format instructions
    pub format_instructions: String,

    /// Example of the expected format
    pub format_example: Option<String>,

    /// Whether to enforce the format strictly
    pub strict: bool,
}

impl Guardrail {
    /// Create a new content filter guardrail
    pub fn content_filter(patterns: Vec<String>, block_content: bool) -> Self {
        Guardrail::ContentFilter(ContentFilter {
            patterns,
            block_content,
            block_message: None,
        })
    }

    /// Create a new topic restriction guardrail
    pub fn topic_restriction(forbidden_topics: Vec<String>, block_content: bool) -> Self {
        Guardrail::TopicRestriction(TopicRestriction {
            forbidden_topics,
            block_content,
            block_message: None,
        })
    }

    /// Create a new response format guardrail
    pub fn response_format(format_instructions: String, strict: bool) -> Self {
        Guardrail::ResponseFormat(ResponseFormat {
            format_instructions,
            format_example: None,
            strict,
        })
    }
}
