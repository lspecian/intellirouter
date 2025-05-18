//! Model status types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of a model in the registry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model is available and ready to use
    Available,
    /// Model is unavailable (e.g., service down)
    Unavailable,
    /// Model is available but with limitations (e.g., rate limited)
    Limited,
    /// Model is in maintenance mode
    Maintenance,
    /// Model is deprecated and will be removed in the future
    Deprecated,
    /// Model is in an unknown state
    Unknown,
}

impl Default for ModelStatus {
    fn default() -> Self {
        ModelStatus::Unknown
    }
}

impl fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelStatus::Available => write!(f, "Available"),
            ModelStatus::Unavailable => write!(f, "Unavailable"),
            ModelStatus::Limited => write!(f, "Limited"),
            ModelStatus::Maintenance => write!(f, "Maintenance"),
            ModelStatus::Deprecated => write!(f, "Deprecated"),
            ModelStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ModelStatus {
    /// Check if the model status is considered available for use
    pub fn is_available(&self) -> bool {
        matches!(self, ModelStatus::Available | ModelStatus::Limited)
    }
}
