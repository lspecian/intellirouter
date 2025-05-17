//! Persona data structures
//!
//! This module defines the core data structures for personas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::error::PersonaError;
use super::guardrails::Guardrail;

/// Example exchange for few-shot learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleExchange {
    /// User message
    pub user: String,

    /// Assistant response
    pub assistant: String,
}

/// Model-specific formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpecificFormat {
    /// Prefix to add before the system prompt
    pub system_prompt_prefix: Option<String>,

    /// Suffix to add after the system prompt
    pub system_prompt_suffix: Option<String>,

    /// Separator to use between examples
    pub example_separator: Option<String>,

    /// Whether to include the system prompt at all
    pub include_system_prompt: Option<bool>,

    /// How to format few-shot examples for this model
    pub few_shot_format: Option<String>,
}

/// Persona configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Unique identifier for the persona
    pub id: String,

    /// Display name of the persona
    pub name: String,

    /// Description of the persona
    pub description: String,

    /// System prompt template (using Handlebars syntax)
    pub system_prompt_template: String,

    /// Few-shot examples for in-context learning
    pub few_shot_examples: Vec<ExampleExchange>,

    /// Guardrails for content control
    pub guardrails: Vec<Guardrail>,

    /// Model-specific formatting options
    pub model_specific_formats: HashMap<String, ModelSpecificFormat>,

    /// Response format (for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

impl Persona {
    /// Create a new persona with the specified parameters
    pub fn new(id: &str, name: &str, description: &str, system_prompt_template: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            system_prompt_template: system_prompt_template.to_string(),
            few_shot_examples: Vec::new(),
            guardrails: Vec::new(),
            model_specific_formats: HashMap::new(),
            response_format: None,
        }
    }

    /// Add a few-shot example to the persona
    pub fn add_example(&mut self, user: &str, assistant: &str) {
        self.few_shot_examples.push(ExampleExchange {
            user: user.to_string(),
            assistant: assistant.to_string(),
        });
    }

    /// Add a guardrail to the persona
    pub fn add_guardrail(&mut self, guardrail: Guardrail) {
        self.guardrails.push(guardrail);
    }

    /// Add model-specific formatting for a model
    pub fn add_model_format(&mut self, model_id: &str, format: ModelSpecificFormat) {
        self.model_specific_formats
            .insert(model_id.to_string(), format);
    }

    /// Get model-specific formatting for a model if available
    pub fn get_model_format(&self, model_id: &str) -> Option<&ModelSpecificFormat> {
        self.model_specific_formats.get(model_id)
    }
}

/// Create a new persona with the specified parameters (legacy API)
pub fn create_persona(name: &str, description: &str, system_prompt: &str) -> Persona {
    Persona {
        id: name.to_lowercase().replace(' ', "_"),
        name: name.to_string(),
        description: description.to_string(),
        system_prompt_template: system_prompt.to_string(),
        few_shot_examples: Vec::new(),
        guardrails: Vec::new(),
        model_specific_formats: HashMap::new(),
        response_format: None,
    }
}

/// Apply a persona to a request string (legacy API)
pub fn apply_persona_to_string(persona: &Persona, request: &str) -> String {
    format!("{}\n\n{}", persona.system_prompt_template, request)
}

/// Load personas from a configuration file (legacy API)
pub fn load_personas<P: AsRef<Path>>(path: P) -> Result<Vec<Persona>, PersonaError> {
    let content = fs::read_to_string(path)?;
    let personas: Vec<Persona> = serde_json::from_str(&content)?;
    Ok(personas)
}
