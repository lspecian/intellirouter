//! Templates for generating reports
//!
//! This module provides functionality for loading and rendering templates.

use std::collections::HashMap;
use std::path::Path;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::Value;

use super::super::types::TestHarnessError;

/// Template context
#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    /// Template variables
    pub variables: HashMap<String, Value>,
}

impl TemplateContext {
    /// Create a new template context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable in the context
    pub fn set<T: Serialize>(&mut self, name: &str, value: T) -> Result<(), TestHarnessError> {
        let value = serde_json::to_value(value).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize value: {}", e))
        })?;
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Get a variable from the context
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Remove a variable from the context
    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Template trait
pub trait Template: Send + Sync {
    /// Render the template with the given context
    fn render(&self, context: &TemplateContext) -> Result<String, TestHarnessError>;

    /// Get the template name
    fn name(&self) -> &str;
}

/// Handlebars template
pub struct HandlebarsTemplate {
    /// Template name
    name: String,
    /// Template content
    content: String,
    /// Handlebars instance
    handlebars: Handlebars<'static>,
}

impl HandlebarsTemplate {
    /// Create a new Handlebars template
    pub fn new(
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Self, TestHarnessError> {
        let name = name.into();
        let content = content.into();
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string(&name, &content)
            .map_err(|e| {
                TestHarnessError::ReportingError(format!("Failed to register template: {}", e))
            })?;
        Ok(Self {
            name,
            content,
            handlebars,
        })
    }
}

impl Template for HandlebarsTemplate {
    fn render(&self, context: &TemplateContext) -> Result<String, TestHarnessError> {
        self.handlebars
            .render(&self.name, &context.variables)
            .map_err(|e| {
                TestHarnessError::ReportingError(format!("Failed to render template: {}", e))
            })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Template engine trait
pub trait TemplateEngine: Send + Sync {
    /// Render a template with the given context
    fn render(
        &self,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String, TestHarnessError>;

    /// Register a template
    fn register_template(&mut self, template: Box<dyn Template>) -> Result<(), TestHarnessError>;

    /// Get a template by name
    fn get_template(&self, name: &str) -> Option<&dyn Template>;

    /// Check if a template exists
    fn has_template(&self, name: &str) -> bool;

    /// Remove a template
    fn remove_template(&mut self, name: &str) -> Result<(), TestHarnessError>;
}

/// Handlebars template engine
pub struct HandlebarsTemplateEngine {
    /// Templates
    templates: HashMap<String, Box<dyn Template>>,
}

impl HandlebarsTemplateEngine {
    /// Create a new Handlebars template engine
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
}

impl Default for HandlebarsTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine for HandlebarsTemplateEngine {
    fn render(
        &self,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String, TestHarnessError> {
        let template = self.get_template(template_name).ok_or_else(|| {
            TestHarnessError::ReportingError(format!("Template not found: {}", template_name))
        })?;
        template.render(context)
    }

    fn register_template(&mut self, template: Box<dyn Template>) -> Result<(), TestHarnessError> {
        let name = template.name().to_string();
        if self.templates.contains_key(&name) {
            return Err(TestHarnessError::ReportingError(format!(
                "Template already exists: {}",
                name
            )));
        }
        self.templates.insert(name, template);
        Ok(())
    }

    fn get_template(&self, name: &str) -> Option<&dyn Template> {
        self.templates.get(name).map(|t| t.as_ref())
    }

    fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    fn remove_template(&mut self, name: &str) -> Result<(), TestHarnessError> {
        if !self.templates.contains_key(name) {
            return Err(TestHarnessError::ReportingError(format!(
                "Template not found: {}",
                name
            )));
        }
        self.templates.remove(name);
        Ok(())
    }
}

/// Template loader trait
pub trait TemplateLoader: Send + Sync {
    /// Load templates from a directory
    fn load_templates(&self, dir: &Path) -> Result<Vec<Box<dyn Template>>, TestHarnessError>;

    /// Load a template from a file
    fn load_template(&self, file: &Path) -> Result<Box<dyn Template>, TestHarnessError>;
}

/// Handlebars template loader
pub struct HandlebarsTemplateLoader;

impl HandlebarsTemplateLoader {
    /// Create a new Handlebars template loader
    pub fn new() -> Self {
        Self
    }
}

impl Default for HandlebarsTemplateLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateLoader for HandlebarsTemplateLoader {
    fn load_templates(&self, dir: &Path) -> Result<Vec<Box<dyn Template>>, TestHarnessError> {
        let mut templates = Vec::new();
        let entries = std::fs::read_dir(dir).map_err(|e| {
            TestHarnessError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read directory: {}", e),
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                TestHarnessError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read directory entry: {}", e),
                ))
            })?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "hbs") {
                let template = self.load_template(&path)?;
                templates.push(template);
            }
        }

        Ok(templates)
    }

    fn load_template(&self, file: &Path) -> Result<Box<dyn Template>, TestHarnessError> {
        let name = file
            .file_stem()
            .ok_or_else(|| TestHarnessError::ReportingError("Failed to get file stem".to_string()))?
            .to_string_lossy()
            .to_string();
        let content = std::fs::read_to_string(file).map_err(|e| {
            TestHarnessError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read file: {}", e),
            ))
        })?;
        let template = HandlebarsTemplate::new(name, content)?;
        Ok(Box::new(template))
    }
}
