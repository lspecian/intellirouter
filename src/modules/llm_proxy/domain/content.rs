//! Content domain models for chat messages
//!
//! This module defines the domain entities for message content,
//! supporting both simple text messages and multimodal content.

use serde::{Deserialize, Serialize};

/// Represents the content of a message, which can be either a string or an array of content parts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content as a string
    String(String),
    /// Multimodal content as an array of content parts
    Array(Vec<ContentPart>),
}

/// Represents a part of a message's content with a specific type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content part
    #[serde(rename = "text")]
    Text { text: String },

    /// Image URL content part
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    /// Audio content part
    #[serde(rename = "input_audio")]
    Audio { input_audio: AudioData },

    /// File content part
    #[serde(rename = "file")]
    File { file: FileData },
}

/// Represents an image URL in a content part
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    /// URL of the image (can be a web URL or base64 data URL)
    pub url: String,
    /// Detail level for image processing
    #[serde(default = "default_detail")]
    pub detail: String,
}

/// Represents audio data in a content part
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioData {
    /// Base64 encoded audio data
    pub data: String,
    /// Format of the audio data
    pub format: AudioFormat,
}

/// Represents a file in a content part
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileData {
    /// Optional filename
    pub filename: Option<String>,
    /// Optional base64 encoded file data
    pub file_data: Option<String>,
    /// Optional file ID for previously uploaded files
    pub file_id: Option<String>,
}

/// Represents the format of audio data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    /// WAV audio format
    Wav,
    /// MP3 audio format
    Mp3,
}

/// Default detail level for image URLs
fn default_detail() -> String {
    "auto".to_string()
}

impl ContentPart {
    /// Extract text content from a content part if available
    pub fn extract_text(&self) -> Option<String> {
        match self {
            ContentPart::Text { text } => Some(text.clone()),
            _ => None,
        }
    }
}

impl MessageContent {
    /// Create a new string content
    pub fn new_string(text: String) -> Self {
        MessageContent::String(text)
    }

    /// Create a new array content with a single text part
    pub fn new_text_array(text: String) -> Self {
        MessageContent::Array(vec![ContentPart::Text { text }])
    }

    /// Extract all text content from the message content
    pub fn extract_text(&self) -> String {
        match self {
            MessageContent::String(text) => text.clone(),
            MessageContent::Array(parts) => parts
                .iter()
                .filter_map(|part| part.extract_text())
                .collect::<Vec<String>>()
                .join(" "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_content_deserialize_string() {
        let json = r#""Hello world""#;
        let content: MessageContent = serde_json::from_str(json).unwrap();

        assert!(matches!(content, MessageContent::String(ref s) if s == "Hello world"));
    }

    #[test]
    fn test_message_content_deserialize_array() {
        let json = r#"[
            {"type": "text", "text": "Hello world"},
            {"type": "image_url", "image_url": {"url": "https://example.com/image.jpg"}}
        ]"#;

        let content: MessageContent = serde_json::from_str(json).unwrap();

        assert!(matches!(content, MessageContent::Array(ref parts) if parts.len() == 2));
    }

    #[test]
    fn test_extract_text_from_string_content() {
        let content = MessageContent::String("Hello world".to_string());
        assert_eq!(content.extract_text(), "Hello world");
    }

    #[test]
    fn test_extract_text_from_array_content() {
        let content = MessageContent::Array(vec![
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
        ]);

        assert_eq!(content.extract_text(), "Hello world");
    }
}
