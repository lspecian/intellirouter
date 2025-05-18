//! Persona Layer Module
//!
//! This module handles system prompt injection and persona management.
//! It allows for consistent personality and behavior across different LLM interactions.
//!
//! The persona layer provides:
//! - Templated system prompts with dynamic variable substitution
//! - Few-shot examples for in-context learning
//! - Guardrails for content filtering and response formatting
//! - Model-specific prompt formatting

mod error;
mod guardrails;
pub mod manager;
mod persona;

pub use error::PersonaError;
pub use guardrails::{ContentFilter, Guardrail, ResponseFormat, TopicRestriction};
pub use manager::PersonaManager;
pub use persona::{ExampleExchange, ModelSpecificFormat, Persona};

// Re-export the legacy API for backward compatibility
pub use persona::{apply_persona_to_string, create_persona, load_personas};
