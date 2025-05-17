//! Error types for the persona layer

use thiserror::Error;

/// Error types for persona operations
#[derive(Error, Debug)]
pub enum PersonaError {
    /// Persona not found
    #[error("Persona not found: {0}")]
    PersonaNotFound(String),

    /// Template error
    #[error("Template error: {0}")]
    TemplateError(#[from] handlebars::RenderError),

    /// Template registration error
    #[error("Template registration error: {0}")]
    TemplateRegistrationError(#[from] handlebars::TemplateError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}
