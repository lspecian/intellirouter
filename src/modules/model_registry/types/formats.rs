//! Input and output format types

use serde::{Deserialize, Serialize};

/// Input format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputFormat {
    /// Plain text
    Text,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
    /// Image (base64 encoded)
    Image,
    /// Audio (base64 encoded)
    Audio,
    /// Other format
    Other(String),
}

/// Output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text
    Text,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
    /// Image (base64 encoded)
    Image,
    /// Audio (base64 encoded)
    Audio,
    /// Other format
    Other(String),
}
