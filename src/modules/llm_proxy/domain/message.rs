//! Message domain models for chat conversations
//!
//! This module defines the domain entities for chat messages,
//! supporting both simple text messages and multimodal content.

use super::content::MessageContent;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a message in a chat conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message author (system, user, assistant, etc.)
    pub role: MessageRole,

    /// The content of the message (can be text or multimodal)
    #[serde(flatten)]
    pub content: MessageContent,

    /// Optional name of the author for role disambiguation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Represents the role of a message author
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions to the model)
    System,
    /// User message (end user input)
    User,
    /// Assistant message (model-generated response)
    Assistant,
    /// Tool message (output from a tool)
    Tool,
    /// Function message (output from a function call)
    Function,
    /// Developer message (instructions to the model, newer alternative to system)
    #[serde(rename = "developer")]
    Developer,
    /// Unknown role (for forward compatibility)
    #[serde(other)]
    Unknown,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Tool => write!(f, "tool"),
            MessageRole::Function => write!(f, "function"),
            MessageRole::Developer => write!(f, "developer"),
            MessageRole::Unknown => write!(f, "unknown"),
        }
    }
}

impl Message {
    /// Create a new message with string content
    pub fn new(role: MessageRole, content: String, name: Option<String>) -> Self {
        Self {
            role,
            content: MessageContent::String(content),
            name,
        }
    }

    /// Create a new system message
    pub fn new_system(content: String) -> Self {
        Self::new(MessageRole::System, content, None)
    }

    /// Create a new user message
    pub fn new_user(content: String) -> Self {
        Self::new(MessageRole::User, content, None)
    }

    /// Create a new assistant message
    pub fn new_assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content, None)
    }

    /// Extract text content from a message, handling both string and array content
    pub fn extract_text_content(&self) -> String {
        self.content.extract_text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::llm_proxy::domain::content::{ContentPart, ImageUrl};

    #[test]
    fn test_message_deserialize_string_content() {
        let json = r#"{"role": "user", "content": "Hello world"}"#;
        let message: Message = serde_json::from_str(json).unwrap();

        assert_eq!(message.role, MessageRole::User);
        assert!(matches!(message.content, MessageContent::String(ref s) if s == "Hello world"));
    }

    #[test]
    fn test_message_deserialize_array_content() {
        let json = r#"{
            "role": "user", 
            "content": [
                {"type": "text", "text": "Hello world"},
                {"type": "image_url", "image_url": {"url": "https://example.com/image.jpg"}}
            ]
        }"#;

        let message: Message = serde_json::from_str(json).unwrap();

        assert_eq!(message.role, MessageRole::User);
        assert!(matches!(message.content, MessageContent::Array(ref parts) if parts.len() == 2));
    }

    #[test]
    fn test_extract_text_content_from_string() {
        let message = Message {
            role: MessageRole::User,
            content: MessageContent::String("Hello world".to_string()),
            name: None,
        };

        assert_eq!(message.extract_text_content(), "Hello world");
    }

    #[test]
    fn test_extract_text_content_from_array() {
        let message = Message {
            role: MessageRole::User,
            content: MessageContent::Array(vec![
                ContentPart::Text {
                    text: "Hello".to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "https://example.com/image.jpg".to_string(),
                        detail: "auto".to_string(),
                    },
                },
                ContentPart::Text {
                    text: "world".to_string(),
                },
            ]),
            name: None,
        };

        assert_eq!(message.extract_text_content(), "Hello world");
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::Tool.to_string(), "tool");
        assert_eq!(MessageRole::Function.to_string(), "function");
        assert_eq!(MessageRole::Developer.to_string(), "developer");
        assert_eq!(MessageRole::Unknown.to_string(), "unknown");
    }
}
