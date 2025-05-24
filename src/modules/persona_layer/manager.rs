//! Persona manager
//!
//! This module provides functionality for managing personas and applying them to requests.

use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::modules::model_registry::connectors::{ChatCompletionRequest, ChatMessage, MessageRole};

use super::error::PersonaError;
use super::guardrails::{Guardrail, ResponseFormat, TopicRestriction};
use super::persona::Persona;

/// Manager for personas
#[derive(Debug)]
pub struct PersonaManager {
    /// Registered personas
    personas: HashMap<String, Persona>,

    /// Handlebars template engine
    handlebars: Handlebars<'static>,
}

impl PersonaManager {
    /// Create a new persona manager
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        Self {
            personas: HashMap::new(),
            handlebars,
        }
    }

    /// Register a persona
    pub fn register_persona(&mut self, persona: Persona) -> Result<(), PersonaError> {
        // Validate and register the template
        self.handlebars
            .register_template_string(&persona.id, &persona.system_prompt_template)?;

        // Store the persona
        self.personas.insert(persona.id.clone(), persona);

        Ok(())
    }

    /// Get a persona by ID
    pub fn get_persona(&self, id: &str) -> Option<&Persona> {
        self.personas.get(id)
    }

    /// List all registered personas
    pub fn list_personas(&self) -> Vec<&Persona> {
        self.personas.values().collect()
    }

    /// Remove a persona
    pub fn remove_persona(&mut self, id: &str) -> Option<Persona> {
        let persona = self.personas.remove(id);
        if let Some(ref p) = persona {
            self.handlebars.unregister_template(&p.id);
        }
        persona
    }

    /// Apply a persona to a chat completion request
    pub fn apply_persona(
        &self,
        persona_id: &str,
        request: &mut ChatCompletionRequest,
        context: &Value,
    ) -> Result<(), PersonaError> {
        let persona = self
            .personas
            .get(persona_id)
            .ok_or_else(|| PersonaError::PersonaNotFound(persona_id.to_string()))?;

        // Render the system prompt with the provided context
        let system_prompt = self.handlebars.render(persona_id, context)?;

        // Apply model-specific formatting if available
        let formatted_system_prompt =
            self.format_for_model(&system_prompt, &request.model, persona);

        // Insert the system prompt as the first message if it should be included
        let include_system_prompt = persona
            .get_model_format(&request.model)
            .and_then(|fmt| fmt.include_system_prompt)
            .unwrap_or(true);

        if include_system_prompt {
            request.messages.insert(
                0,
                ChatMessage {
                    role: MessageRole::System,
                    content: formatted_system_prompt,
                    name: None,
                    function_call: None,
                    tool_calls: None,
                },
            );
        }

        // Add few-shot examples if present
        if !persona.few_shot_examples.is_empty() {
            let mut index = if include_system_prompt { 1 } else { 0 };

            for example in &persona.few_shot_examples {
                // Insert user message
                request.messages.insert(
                    index,
                    ChatMessage {
                        role: MessageRole::User,
                        content: example.user.clone(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                );
                index += 1;

                // Insert assistant message
                request.messages.insert(
                    index,
                    ChatMessage {
                        role: MessageRole::Assistant,
                        content: example.assistant.clone(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                );
                index += 1;
            }
        }

        // Apply guardrails
        self.apply_guardrails(request, persona)?;

        Ok(())
    }

    /// Format a system prompt for a specific model
    fn format_for_model(&self, system_prompt: &str, model_id: &str, persona: &Persona) -> String {
        if let Some(format) = persona.get_model_format(model_id) {
            let prefix = format.system_prompt_prefix.as_deref().unwrap_or("");
            let suffix = format.system_prompt_suffix.as_deref().unwrap_or("");
            format!("{}{}{}", prefix, system_prompt, suffix)
        } else {
            system_prompt.to_string()
        }
    }

    /// Apply guardrails to a request
    fn apply_guardrails(
        &self,
        request: &mut ChatCompletionRequest,
        persona: &Persona,
    ) -> Result<(), PersonaError> {
        for guardrail in &persona.guardrails {
            match guardrail {
                Guardrail::ResponseFormat(ResponseFormat {
                    format_instructions,
                    format_example,
                    strict,
                }) => {
                    // Append format instructions to the system prompt
                    if let Some(system_message) = request.messages.first_mut() {
                        if matches!(system_message.role, MessageRole::System) {
                            let mut format_text =
                                format!("\n\nResponse Format: {}", format_instructions);

                            if let Some(example) = format_example {
                                format_text.push_str(&format!("\n\nExample Format:\n{}", example));
                            }

                            if *strict {
                                format_text
                                    .push_str("\n\nYou must strictly adhere to this format.");
                            }

                            system_message.content.push_str(&format_text);
                        }
                    }
                }
                Guardrail::ContentFilter(filter) => {
                    // Add content filter to additional params
                    let additional_params =
                        request.additional_params.get_or_insert_with(HashMap::new);
                    additional_params
                        .insert("content_filter".to_string(), serde_json::to_value(filter)?);
                }
                Guardrail::TopicRestriction(TopicRestriction {
                    forbidden_topics,
                    block_content,
                    block_message,
                }) => {
                    // Add topic restriction to additional params
                    let additional_params =
                        request.additional_params.get_or_insert_with(HashMap::new);

                    let restriction_value = serde_json::json!({
                        "forbidden_topics": forbidden_topics,
                        "block_content": block_content,
                        "block_message": block_message,
                    });

                    additional_params.insert("topic_restriction".to_string(), restriction_value);
                }
            }
        }

        Ok(())
    }

    /// Load personas from a file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PersonaError> {
        let content = fs::read_to_string(path)?;
        let personas: Vec<Persona> = serde_json::from_str(&content)?;

        for persona in personas {
            self.register_persona(persona)?;
        }

        Ok(())
    }

    /// Save personas to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), PersonaError> {
        let personas: Vec<Persona> = self.personas.values().cloned().collect();
        let content = serde_json::to_string_pretty(&personas)?;
        fs::write(path, content)?;

        Ok(())
    }

    /// List all guardrails across all personas
    ///
    /// # Returns
    ///
    /// A vector of guardrails with their associated persona
    pub fn list_guardrails(&self) -> Vec<(String, String, &Guardrail)> {
        let mut guardrails = Vec::new();

        for persona in self.personas.values() {
            for guardrail in &persona.guardrails {
                guardrails.push((persona.id.clone(), persona.name.clone(), guardrail));
            }
        }

        guardrails
    }

    /// Get usage statistics for personas
    ///
    /// # Returns
    ///
    /// A map of usage statistics
    pub fn get_usage_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();

        // In a real implementation, these would be tracked during apply_persona calls
        stats.insert("total_requests".to_string(), 0);
        stats.insert("guardrail_blocks".to_string(), 0);
        stats.insert("average_response_time_ms".to_string(), 0);

        stats
    }

    /// Get recent persona usage
    ///
    /// # Returns
    ///
    /// A vector of recent persona usage events
    pub fn get_recent_persona_usage(&self) -> Vec<HashMap<String, serde_json::Value>> {
        // In a real implementation, this would maintain a circular buffer of recent usage
        Vec::new()
    }

    /// Get recent guardrail blocks
    ///
    /// # Returns
    ///
    /// A vector of recent guardrail block events
    pub fn get_recent_guardrail_blocks(&self) -> Vec<HashMap<String, serde_json::Value>> {
        // In a real implementation, this would maintain a circular buffer of recent blocks
        Vec::new()
    }

    /// Get performance metrics
    ///
    /// # Returns
    ///
    /// A map of performance metrics
    pub fn get_performance_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        // In a real implementation, these would be tracked during apply_persona calls
        metrics.insert("average_prompt_tokens".to_string(), 0.0);
        metrics.insert("average_completion_tokens".to_string(), 0.0);
        metrics.insert("average_guardrail_check_time_ms".to_string(), 0.0);
        metrics.insert("average_persona_application_time_ms".to_string(), 0.0);

        metrics
    }
}

/// Thread-safe persona manager
pub type SharedPersonaManager = Arc<RwLock<PersonaManager>>;

/// Create a new shared persona manager
pub fn create_shared_manager() -> SharedPersonaManager {
    Arc::new(RwLock::new(PersonaManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_persona_creation() {
        let mut manager = PersonaManager::new();

        let mut persona = Persona::new(
            "test",
            "Test Persona",
            "A test persona",
            "You are a {{role}} named {{name}}",
        );

        persona.add_example("Hello", "Hi there!");

        manager.register_persona(persona).unwrap();

        let retrieved = manager.get_persona("test").unwrap();
        assert_eq!(retrieved.name, "Test Persona");
        assert_eq!(retrieved.few_shot_examples.len(), 1);
    }

    #[test]
    fn test_template_rendering() {
        let mut manager = PersonaManager::new();

        let persona = Persona::new(
            "test",
            "Test Persona",
            "A test persona",
            "You are a {{role}} named {{name}}",
        );

        manager.register_persona(persona).unwrap();

        let context = json!({
            "role": "assistant",
            "name": "TestBot"
        });

        let mut request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            }],
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        manager
            .apply_persona("test", &mut request, &context)
            .unwrap();

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(
            request.messages[0].content,
            "You are a assistant named TestBot"
        );
    }

    #[test]
    fn test_few_shot_examples() {
        let mut manager = PersonaManager::new();

        let mut persona = Persona::new(
            "test",
            "Test Persona",
            "A test persona",
            "You are a helpful assistant",
        );

        persona.add_example(
            "What is the capital of France?",
            "The capital of France is Paris.",
        );

        manager.register_persona(persona).unwrap();

        let mut request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "What is the capital of Italy?".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            }],
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        manager
            .apply_persona("test", &mut request, &json!({}))
            .unwrap();

        assert_eq!(request.messages.len(), 4);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert_eq!(
            request.messages[1].content,
            "What is the capital of France?"
        );
        assert_eq!(request.messages[2].role, MessageRole::Assistant);
        assert_eq!(
            request.messages[2].content,
            "The capital of France is Paris."
        );
        assert_eq!(request.messages[3].role, MessageRole::User);
        assert_eq!(request.messages[3].content, "What is the capital of Italy?");
    }

    #[test]
    fn test_guardrails() {
        let mut manager = PersonaManager::new();

        let mut persona = Persona::new(
            "test",
            "Test Persona",
            "A test persona",
            "You are a helpful assistant",
        );

        persona.add_guardrail(Guardrail::response_format(
            "Please respond in JSON format".to_string(),
            true,
        ));

        manager.register_persona(persona).unwrap();

        let mut request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            }],
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        manager
            .apply_persona("test", &mut request, &json!({}))
            .unwrap();

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert!(request.messages[0]
            .content
            .contains("Response Format: Please respond in JSON format"));
        assert!(request.messages[0]
            .content
            .contains("You must strictly adhere to this format"));
    }
}
