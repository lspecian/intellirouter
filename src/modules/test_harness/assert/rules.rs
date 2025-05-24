//! Validation rules engine for the assertion framework.
//!
//! This module provides a framework for defining and applying validation rules.
//! Rules can be combined into rule sets and applied to values using a rule engine.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::modules::test_harness::types::TestHarnessError;

/// Represents the severity of a validation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// The validation passed.
    Info,
    /// The validation failed with a warning.
    Warning,
    /// The validation failed with an error.
    Error,
    /// The validation failed with a critical error.
    Critical,
}

/// Represents the status of a validation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// The validation passed.
    Passed,
    /// The validation failed.
    Failed,
    /// The validation was skipped.
    Skipped,
}

/// Represents the result of a validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// The name of the rule that produced the result.
    pub rule_name: String,
    /// The status of the validation.
    pub status: ValidationStatus,
    /// The severity of the validation.
    pub severity: ValidationSeverity,
    /// The message associated with the validation.
    pub message: String,
    /// The path to the validated value.
    pub path: String,
    /// Additional metadata about the validation.
    pub metadata: serde_json::Value,
}

impl ValidationResult {
    /// Creates a new validation result.
    pub fn new(
        rule_name: &str,
        status: ValidationStatus,
        severity: ValidationSeverity,
        message: &str,
    ) -> Self {
        Self {
            rule_name: rule_name.to_string(),
            status,
            severity,
            message: message.to_string(),
            path: "".to_string(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the path to the validated value.
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    /// Sets additional metadata about the validation.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns whether the validation passed.
    pub fn passed(&self) -> bool {
        self.status == ValidationStatus::Passed
    }

    /// Returns whether the validation failed.
    pub fn failed(&self) -> bool {
        self.status == ValidationStatus::Failed
    }

    /// Returns whether the validation was skipped.
    pub fn skipped(&self) -> bool {
        self.status == ValidationStatus::Skipped
    }

    /// Returns whether the validation is an error.
    pub fn is_error(&self) -> bool {
        self.status == ValidationStatus::Failed
            && (self.severity == ValidationSeverity::Error
                || self.severity == ValidationSeverity::Critical)
    }

    /// Returns whether the validation is a warning.
    pub fn is_warning(&self) -> bool {
        self.status == ValidationStatus::Failed && self.severity == ValidationSeverity::Warning
    }

    /// Returns whether the validation is informational.
    pub fn is_info(&self) -> bool {
        self.status == ValidationStatus::Passed && self.severity == ValidationSeverity::Info
    }

    /// Returns whether the validation is critical.
    pub fn is_critical(&self) -> bool {
        self.status == ValidationStatus::Failed && self.severity == ValidationSeverity::Critical
    }
}

/// Trait for types that can validate values.
#[async_trait]
pub trait Rule: Send + Sync {
    /// Returns the name of the rule.
    fn name(&self) -> &str;

    /// Returns the description of the rule.
    fn description(&self) -> &str;

    /// Returns the severity of the rule.
    fn severity(&self) -> ValidationSeverity;

    /// Validates a value.
    async fn validate(&self, ctx: &ValidationContext)
        -> Result<ValidationResult, TestHarnessError>;

    /// Returns whether the rule is enabled.
    fn is_enabled(&self) -> bool;

    /// Enables or disables the rule.
    fn set_enabled(&mut self, enabled: bool);
}

/// A context for validation.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// The value being validated.
    pub value: serde_json::Value,
    /// The path to the value being validated.
    pub path: String,
    /// Additional context for validation.
    pub context: HashMap<String, serde_json::Value>,
}

impl ValidationContext {
    /// Creates a new validation context.
    pub fn new(value: serde_json::Value) -> Self {
        Self {
            value,
            path: "".to_string(),
            context: HashMap::new(),
        }
    }

    /// Sets the path to the value being validated.
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    /// Adds additional context for validation.
    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context.insert(key.to_string(), value);
        self
    }

    /// Gets a value from the context.
    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }

    /// Gets a value from the context as a specific type.
    pub fn get_context_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.get_context(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// A set of validation rules.
#[derive(Debug, Clone)]
pub struct RuleSet {
    /// The name of the rule set.
    name: String,
    /// The description of the rule set.
    description: String,
    /// The rules in the rule set.
    rules: Vec<Arc<dyn Rule>>,
}

impl RuleSet {
    /// Creates a new rule set.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "".to_string(),
            rules: Vec::new(),
        }
    }

    /// Sets the description of the rule set.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Adds a rule to the rule set.
    pub fn with_rule(mut self, rule: Arc<dyn Rule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Returns the name of the rule set.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description of the rule set.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the rules in the rule set.
    pub fn rules(&self) -> &[Arc<dyn Rule>] {
        &self.rules
    }
}

/// An engine for applying validation rules.
#[derive(Debug, Clone)]
pub struct RuleEngine {
    /// The rule sets in the engine.
    rule_sets: Vec<RuleSet>,
}

impl RuleEngine {
    /// Creates a new rule engine.
    pub fn new() -> Self {
        Self {
            rule_sets: Vec::new(),
        }
    }

    /// Adds a rule set to the engine.
    pub fn with_rule_set(mut self, rule_set: RuleSet) -> Self {
        self.rule_sets.push(rule_set);
        self
    }

    /// Returns the rule sets in the engine.
    pub fn rule_sets(&self) -> &[RuleSet] {
        &self.rule_sets
    }

    /// Validates a value using all rule sets.
    pub async fn validate(
        &self,
        value: serde_json::Value,
    ) -> Result<Vec<ValidationResult>, TestHarnessError> {
        let mut results = Vec::new();

        for rule_set in &self.rule_sets {
            let ctx = ValidationContext::new(value.clone());
            let rule_results = self.validate_with_rule_set(rule_set, &ctx).await?;
            results.extend(rule_results);
        }

        Ok(results)
    }

    /// Validates a value using a specific rule set.
    pub async fn validate_with_rule_set(
        &self,
        rule_set: &RuleSet,
        ctx: &ValidationContext,
    ) -> Result<Vec<ValidationResult>, TestHarnessError> {
        let mut results = Vec::new();

        for rule in rule_set.rules() {
            if rule.is_enabled() {
                let result = rule.validate(ctx).await?;
                results.push(result);
            }
        }

        Ok(results)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A base implementation of a validation rule.
#[derive(Debug, Clone)]
pub struct BaseRule {
    /// The name of the rule.
    name: String,
    /// The description of the rule.
    description: String,
    /// The severity of the rule.
    severity: ValidationSeverity,
    /// Whether the rule is enabled.
    enabled: bool,
    /// The validation function.
    #[allow(clippy::type_complexity)]
    validator:
        Arc<dyn Fn(&ValidationContext) -> Result<ValidationResult, TestHarnessError> + Send + Sync>,
}

impl BaseRule {
    /// Creates a new base rule.
    pub fn new<F>(name: &str, severity: ValidationSeverity, validator: F) -> Self
    where
        F: Fn(&ValidationContext) -> Result<ValidationResult, TestHarnessError>
            + Send
            + Sync
            + 'static,
    {
        Self {
            name: name.to_string(),
            description: "".to_string(),
            severity,
            enabled: true,
            validator: Arc::new(validator),
        }
    }

    /// Sets the description of the rule.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
}

#[async_trait]
impl Rule for BaseRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn severity(&self) -> ValidationSeverity {
        self.severity.clone()
    }

    async fn validate(
        &self,
        ctx: &ValidationContext,
    ) -> Result<ValidationResult, TestHarnessError> {
        (self.validator)(ctx)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Creates a new rule set.
pub fn create_rule_set(name: &str) -> RuleSet {
    RuleSet::new(name)
}

/// Creates a new rule engine.
pub fn create_rule_engine() -> RuleEngine {
    RuleEngine::new()
}

/// Creates a new validation context.
pub fn create_validation_context(value: serde_json::Value) -> ValidationContext {
    ValidationContext::new(value)
}

/// Creates a new validation result.
pub fn create_validation_result(
    rule_name: &str,
    status: ValidationStatus,
    severity: ValidationSeverity,
    message: &str,
) -> ValidationResult {
    ValidationResult::new(rule_name, status, severity, message)
}

/// Creates a new base rule.
pub fn create_rule<F>(name: &str, severity: ValidationSeverity, validator: F) -> BaseRule
where
    F: Fn(&ValidationContext) -> Result<ValidationResult, TestHarnessError> + Send + Sync + 'static,
{
    BaseRule::new(name, severity, validator)
}
