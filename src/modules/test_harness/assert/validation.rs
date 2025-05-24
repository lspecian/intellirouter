//! Validation framework for the assertion framework.
//!
//! This module provides a higher-level interface for validating values using the rules engine.
//! It includes validators, validation reports, and formatters for displaying validation results.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::modules::test_harness::types::TestHarnessError;

use super::rules::{
    create_rule_engine, create_rule_set, create_validation_context, RuleEngine, RuleSet,
    ValidationResult, ValidationSeverity, ValidationStatus,
};

/// Configuration for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to fail on the first error.
    pub fail_fast: bool,
    /// The maximum number of errors to report.
    pub max_errors: Option<usize>,
    /// The maximum number of warnings to report.
    pub max_warnings: Option<usize>,
    /// Whether to include passed validations in the report.
    pub include_passed: bool,
    /// Whether to include skipped validations in the report.
    pub include_skipped: bool,
    /// The minimum severity level to report.
    pub min_severity: ValidationSeverity,
}

impl ValidationConfig {
    /// Creates a new validation configuration.
    pub fn new() -> Self {
        Self {
            fail_fast: false,
            max_errors: None,
            max_warnings: None,
            include_passed: false,
            include_skipped: false,
            min_severity: ValidationSeverity::Warning,
        }
    }

    /// Sets whether to fail on the first error.
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Sets the maximum number of errors to report.
    pub fn with_max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = Some(max_errors);
        self
    }

    /// Sets the maximum number of warnings to report.
    pub fn with_max_warnings(mut self, max_warnings: usize) -> Self {
        self.max_warnings = Some(max_warnings);
        self
    }

    /// Sets whether to include passed validations in the report.
    pub fn with_include_passed(mut self, include_passed: bool) -> Self {
        self.include_passed = include_passed;
        self
    }

    /// Sets whether to include skipped validations in the report.
    pub fn with_include_skipped(mut self, include_skipped: bool) -> Self {
        self.include_skipped = include_skipped;
        self
    }

    /// Sets the minimum severity level to report.
    pub fn with_min_severity(mut self, min_severity: ValidationSeverity) -> Self {
        self.min_severity = min_severity;
        self
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// A report of validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The name of the validation.
    pub name: String,
    /// The description of the validation.
    pub description: String,
    /// The validation results.
    pub results: Vec<ValidationResult>,
    /// The time it took to execute the validation.
    pub duration: Duration,
    /// The number of errors.
    pub error_count: usize,
    /// The number of warnings.
    pub warning_count: usize,
    /// The number of passed validations.
    pub passed_count: usize,
    /// The number of skipped validations.
    pub skipped_count: usize,
    /// Additional metadata about the validation.
    pub metadata: serde_json::Value,
}

impl ValidationReport {
    /// Creates a new validation report.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "".to_string(),
            results: Vec::new(),
            duration: Duration::default(),
            error_count: 0,
            warning_count: 0,
            passed_count: 0,
            skipped_count: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the description of the validation.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Adds a validation result to the report.
    pub fn with_result(mut self, result: ValidationResult) -> Self {
        match result.status {
            ValidationStatus::Passed => self.passed_count += 1,
            ValidationStatus::Failed => match result.severity {
                ValidationSeverity::Warning => self.warning_count += 1,
                ValidationSeverity::Error | ValidationSeverity::Critical => self.error_count += 1,
                _ => {}
            },
            ValidationStatus::Skipped => self.skipped_count += 1,
        }

        self.results.push(result);
        self
    }

    /// Sets the time it took to execute the validation.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets additional metadata about the validation.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns whether the validation passed.
    pub fn passed(&self) -> bool {
        self.error_count == 0
    }

    /// Returns whether the validation failed.
    pub fn failed(&self) -> bool {
        self.error_count > 0
    }

    /// Returns the total number of validations.
    pub fn total_count(&self) -> usize {
        self.error_count + self.warning_count + self.passed_count + self.skipped_count
    }

    /// Returns the validation results with a specific status.
    pub fn results_with_status(&self, status: ValidationStatus) -> Vec<&ValidationResult> {
        self.results.iter().filter(|r| r.status == status).collect()
    }

    /// Returns the validation results with a specific severity.
    pub fn results_with_severity(&self, severity: ValidationSeverity) -> Vec<&ValidationResult> {
        self.results
            .iter()
            .filter(|r| r.severity == severity)
            .collect()
    }

    /// Returns the validation results with a specific rule name.
    pub fn results_with_rule_name(&self, rule_name: &str) -> Vec<&ValidationResult> {
        self.results
            .iter()
            .filter(|r| r.rule_name == rule_name)
            .collect()
    }

    /// Returns the validation results with a specific path.
    pub fn results_with_path(&self, path: &str) -> Vec<&ValidationResult> {
        self.results.iter().filter(|r| r.path == path).collect()
    }

    /// Returns the error results.
    pub fn error_results(&self) -> Vec<&ValidationResult> {
        self.results.iter().filter(|r| r.is_error()).collect()
    }

    /// Returns the warning results.
    pub fn warning_results(&self) -> Vec<&ValidationResult> {
        self.results.iter().filter(|r| r.is_warning()).collect()
    }

    /// Returns the passed results.
    pub fn passed_results(&self) -> Vec<&ValidationResult> {
        self.results.iter().filter(|r| r.passed()).collect()
    }

    /// Returns the skipped results.
    pub fn skipped_results(&self) -> Vec<&ValidationResult> {
        self.results.iter().filter(|r| r.skipped()).collect()
    }
}

/// Trait for types that can format validation reports.
pub trait ValidationFormatter {
    /// Formats a validation report.
    fn format(&self, report: &ValidationReport) -> Result<String, TestHarnessError>;
}

/// A formatter that formats validation reports as plain text.
#[derive(Debug, Clone)]
pub struct PlainTextFormatter;

impl ValidationFormatter for PlainTextFormatter {
    fn format(&self, report: &ValidationReport) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        output.push_str(&format!("Validation Report: {}\n", report.name));
        output.push_str(&format!("Description: {}\n", report.description));
        output.push_str(&format!("Duration: {:?}\n", report.duration));
        output.push_str(&format!("Total: {}\n", report.total_count()));
        output.push_str(&format!("Passed: {}\n", report.passed_count));
        output.push_str(&format!("Warnings: {}\n", report.warning_count));
        output.push_str(&format!("Errors: {}\n", report.error_count));
        output.push_str(&format!("Skipped: {}\n", report.skipped_count));
        output.push_str("\n");

        if report.error_count > 0 {
            output.push_str("Errors:\n");
            for result in report.error_results() {
                output.push_str(&format!(
                    "  - [{}] {}: {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        if report.warning_count > 0 {
            output.push_str("Warnings:\n");
            for result in report.warning_results() {
                output.push_str(&format!(
                    "  - [{}] {}: {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        if report.passed_count > 0 {
            output.push_str("Passed:\n");
            for result in report.passed_results() {
                output.push_str(&format!(
                    "  - [{}] {}: {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        if report.skipped_count > 0 {
            output.push_str("Skipped:\n");
            for result in report.skipped_results() {
                output.push_str(&format!(
                    "  - [{}] {}: {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        Ok(output)
    }
}

/// A formatter that formats validation reports as JSON.
#[derive(Debug, Clone)]
pub struct JsonFormatter;

impl ValidationFormatter for JsonFormatter {
    fn format(&self, report: &ValidationReport) -> Result<String, TestHarnessError> {
        match serde_json::to_string_pretty(report) {
            Ok(json) => Ok(json),
            Err(e) => Err(TestHarnessError::SerializationError(format!(
                "Failed to serialize validation report: {}",
                e
            ))),
        }
    }
}

/// A formatter that formats validation reports as Markdown.
#[derive(Debug, Clone)]
pub struct MarkdownFormatter;

impl ValidationFormatter for MarkdownFormatter {
    fn format(&self, report: &ValidationReport) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        output.push_str(&format!("# Validation Report: {}\n\n", report.name));
        output.push_str(&format!("**Description:** {}\n\n", report.description));
        output.push_str(&format!("**Duration:** {:?}\n\n", report.duration));
        output.push_str(&format!("**Total:** {}\n\n", report.total_count()));
        output.push_str(&format!("**Passed:** {}\n\n", report.passed_count));
        output.push_str(&format!("**Warnings:** {}\n\n", report.warning_count));
        output.push_str(&format!("**Errors:** {}\n\n", report.error_count));
        output.push_str(&format!("**Skipped:** {}\n\n", report.skipped_count));

        if report.error_count > 0 {
            output.push_str("## Errors\n\n");
            for result in report.error_results() {
                output.push_str(&format!(
                    "- **[{}] {}:** {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        if report.warning_count > 0 {
            output.push_str("## Warnings\n\n");
            for result in report.warning_results() {
                output.push_str(&format!(
                    "- **[{}] {}:** {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        if report.passed_count > 0 {
            output.push_str("## Passed\n\n");
            for result in report.passed_results() {
                output.push_str(&format!(
                    "- **[{}] {}:** {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        if report.skipped_count > 0 {
            output.push_str("## Skipped\n\n");
            for result in report.skipped_results() {
                output.push_str(&format!(
                    "- **[{}] {}:** {}\n",
                    result.rule_name, result.path, result.message
                ));
            }
            output.push_str("\n");
        }

        Ok(output)
    }
}

/// A reporter that reports validation results.
#[derive(Debug, Clone)]
pub struct ValidationReporter {
    /// The formatters for the reporter.
    formatters: HashMap<String, Arc<dyn ValidationFormatter + Send + Sync>>,
}

impl ValidationReporter {
    /// Creates a new validation reporter.
    pub fn new() -> Self {
        let mut formatters = HashMap::new();
        formatters.insert(
            "plain".to_string(),
            Arc::new(PlainTextFormatter) as Arc<dyn ValidationFormatter + Send + Sync>,
        );
        formatters.insert(
            "json".to_string(),
            Arc::new(JsonFormatter) as Arc<dyn ValidationFormatter + Send + Sync>,
        );
        formatters.insert(
            "markdown".to_string(),
            Arc::new(MarkdownFormatter) as Arc<dyn ValidationFormatter + Send + Sync>,
        );

        Self { formatters }
    }

    /// Adds a formatter to the reporter.
    pub fn with_formatter<F>(mut self, name: &str, formatter: F) -> Self
    where
        F: ValidationFormatter + Send + Sync + 'static,
    {
        self.formatters
            .insert(name.to_string(), Arc::new(formatter));
        self
    }

    /// Formats a validation report using a specific formatter.
    pub fn format(
        &self,
        report: &ValidationReport,
        format: &str,
    ) -> Result<String, TestHarnessError> {
        match self.formatters.get(format) {
            Some(formatter) => formatter.format(report),
            None => Err(TestHarnessError::ValidationError(format!(
                "Unknown formatter: {}",
                format
            ))),
        }
    }
}

impl Default for ValidationReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can validate values.
#[async_trait]
pub trait Validator: Send + Sync {
    /// Returns the name of the validator.
    fn name(&self) -> &str;

    /// Returns the description of the validator.
    fn description(&self) -> &str;

    /// Returns the configuration of the validator.
    fn config(&self) -> &ValidationConfig;

    /// Sets the configuration of the validator.
    fn set_config(&mut self, config: ValidationConfig);

    /// Validates a value.
    async fn validate(
        &self,
        value: serde_json::Value,
    ) -> Result<ValidationReport, TestHarnessError>;
}

/// A synchronous validator.
#[derive(Debug, Clone)]
pub struct SyncValidator {
    /// The name of the validator.
    name: String,
    /// The description of the validator.
    description: String,
    /// The configuration of the validator.
    config: ValidationConfig,
    /// The rule engine for the validator.
    rule_engine: RuleEngine,
}

impl SyncValidator {
    /// Creates a new synchronous validator.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "".to_string(),
            config: ValidationConfig::default(),
            rule_engine: create_rule_engine(),
        }
    }

    /// Sets the description of the validator.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the configuration of the validator.
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    /// Adds a rule set to the validator.
    pub fn with_rule_set(mut self, rule_set: RuleSet) -> Self {
        self.rule_engine = self.rule_engine.with_rule_set(rule_set);
        self
    }
}

#[async_trait]
impl Validator for SyncValidator {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn config(&self) -> &ValidationConfig {
        &self.config
    }

    fn set_config(&mut self, config: ValidationConfig) {
        self.config = config;
    }

    async fn validate(
        &self,
        value: serde_json::Value,
    ) -> Result<ValidationReport, TestHarnessError> {
        let start = Instant::now();
        let ctx = create_validation_context(value);
        let mut report = ValidationReport::new(&self.name).with_description(&self.description);

        let mut error_count = 0;
        let mut warning_count = 0;

        for rule_set in self.rule_engine.rule_sets() {
            let results = self
                .rule_engine
                .validate_with_rule_set(rule_set, &ctx)
                .await?;

            for result in results {
                let include_result = match result.status {
                    ValidationStatus::Passed => self.config.include_passed,
                    ValidationStatus::Failed => match result.severity {
                        ValidationSeverity::Warning => {
                            warning_count += 1;
                            self.config.min_severity <= ValidationSeverity::Warning
                                && (self.config.max_warnings.is_none()
                                    || warning_count <= self.config.max_warnings.unwrap())
                        }
                        ValidationSeverity::Error | ValidationSeverity::Critical => {
                            error_count += 1;
                            self.config.min_severity <= ValidationSeverity::Error
                                && (self.config.max_errors.is_none()
                                    || error_count <= self.config.max_errors.unwrap())
                        }
                        _ => false,
                    },
                    ValidationStatus::Skipped => self.config.include_skipped,
                };

                if include_result {
                    report = report.with_result(result.clone());
                }

                if self.config.fail_fast && result.is_error() {
                    break;
                }
            }

            if self.config.fail_fast && error_count > 0 {
                break;
            }
        }

        report = report.with_duration(start.elapsed());
        Ok(report)
    }
}

/// An asynchronous validator.
#[derive(Debug, Clone)]
pub struct AsyncValidator {
    /// The name of the validator.
    name: String,
    /// The description of the validator.
    description: String,
    /// The configuration of the validator.
    config: ValidationConfig,
    /// The rule engine for the validator.
    rule_engine: RuleEngine,
}

impl AsyncValidator {
    /// Creates a new asynchronous validator.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "".to_string(),
            config: ValidationConfig::default(),
            rule_engine: create_rule_engine(),
        }
    }

    /// Sets the description of the validator.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the configuration of the validator.
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    /// Adds a rule set to the validator.
    pub fn with_rule_set(mut self, rule_set: RuleSet) -> Self {
        self.rule_engine = self.rule_engine.with_rule_set(rule_set);
        self
    }
}

#[async_trait]
impl Validator for AsyncValidator {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn config(&self) -> &ValidationConfig {
        &self.config
    }

    fn set_config(&mut self, config: ValidationConfig) {
        self.config = config;
    }

    async fn validate(
        &self,
        value: serde_json::Value,
    ) -> Result<ValidationReport, TestHarnessError> {
        let start = Instant::now();
        let ctx = create_validation_context(value);
        let mut report = ValidationReport::new(&self.name).with_description(&self.description);

        let mut error_count = 0;
        let mut warning_count = 0;

        for rule_set in self.rule_engine.rule_sets() {
            let results = self
                .rule_engine
                .validate_with_rule_set(rule_set, &ctx)
                .await?;

            for result in results {
                let include_result = match result.status {
                    ValidationStatus::Passed => self.config.include_passed,
                    ValidationStatus::Failed => match result.severity {
                        ValidationSeverity::Warning => {
                            warning_count += 1;
                            self.config.min_severity <= ValidationSeverity::Warning
                                && (self.config.max_warnings.is_none()
                                    || warning_count <= self.config.max_warnings.unwrap())
                        }
                        ValidationSeverity::Error | ValidationSeverity::Critical => {
                            error_count += 1;
                            self.config.min_severity <= ValidationSeverity::Error
                                && (self.config.max_errors.is_none()
                                    || error_count <= self.config.max_errors.unwrap())
                        }
                        _ => false,
                    },
                    ValidationStatus::Skipped => self.config.include_skipped,
                };

                if include_result {
                    report = report.with_result(result.clone());
                }

                if self.config.fail_fast && result.is_error() {
                    break;
                }
            }

            if self.config.fail_fast && error_count > 0 {
                break;
            }
        }

        report = report.with_duration(start.elapsed());
        Ok(report)
    }
}

/// A builder for validators.
#[derive(Debug, Clone)]
pub struct ValidatorBuilder {
    /// The name of the validator.
    name: String,
    /// The description of the validator.
    description: String,
    /// The configuration of the validator.
    config: ValidationConfig,
    /// The rule sets for the validator.
    rule_sets: Vec<RuleSet>,
    /// Whether the validator is asynchronous.
    is_async: bool,
}

impl ValidatorBuilder {
    /// Creates a new validator builder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "".to_string(),
            config: ValidationConfig::default(),
            rule_sets: Vec::new(),
            is_async: false,
        }
    }

    /// Sets the description of the validator.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the configuration of the validator.
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    /// Adds a rule set to the validator.
    pub fn with_rule_set(mut self, rule_set: RuleSet) -> Self {
        self.rule_sets.push(rule_set);
        self
    }

    /// Sets whether the validator is asynchronous.
    pub fn with_async(mut self, is_async: bool) -> Self {
        self.is_async = is_async;
        self
    }

    /// Builds the validator.
    pub fn build(self) -> Box<dyn Validator + Send + Sync> {
        if self.is_async {
            let mut validator = AsyncValidator::new(&self.name)
                .with_description(&self.description)
                .with_config(self.config);

            for rule_set in self.rule_sets {
                validator = validator.with_rule_set(rule_set);
            }

            Box::new(validator)
        } else {
            let mut validator = SyncValidator::new(&self.name)
                .with_description(&self.description)
                .with_config(self.config);

            for rule_set in self.rule_sets {
                validator = validator.with_rule_set(rule_set);
            }

            Box::new(validator)
        }
    }
}

/// A registry of validators.
#[derive(Debug, Clone)]
pub struct ValidationRegistry {
    /// The validators in the registry.
    validators: HashMap<String, Arc<dyn Validator + Send + Sync>>,
}

impl ValidationRegistry {
    /// Creates a new validation registry.
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    /// Adds a validator to the registry.
    pub fn register<V>(&mut self, validator: V) -> Result<(), TestHarnessError>
    where
        V: Validator + Send + Sync + 'static,
    {
        let name = validator.name().to_string();
        if self.validators.contains_key(&name) {
            return Err(TestHarnessError::ValidationError(format!(
                "Validator already registered: {}",
                name
            )));
        }

        self.validators.insert(name, Arc::new(validator));
        Ok(())
    }

    /// Removes a validator from the registry.
    pub fn unregister(&mut self, name: &str) -> Result<(), TestHarnessError> {
        if !self.validators.contains_key(name) {
            return Err(TestHarnessError::ValidationError(format!(
                "Validator not registered: {}",
                name
            )));
        }

        self.validators.remove(name);
        Ok(())
    }

    /// Returns a validator from the registry.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Validator + Send + Sync>> {
        self.validators.get(name).cloned()
    }

    /// Returns whether a validator is registered.
    pub fn is_registered(&self, name: &str) -> bool {
        self.validators.contains_key(name)
    }

    /// Returns the names of all registered validators.
    pub fn validator_names(&self) -> Vec<String> {
        self.validators.keys().cloned().collect()
    }

    /// Returns the number of registered validators.
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
}

impl Default for ValidationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new validation config.
pub fn create_validation_config() -> ValidationConfig {
    ValidationConfig::new()
}

/// Creates a new validation report.
pub fn create_validation_report(name: &str) -> ValidationReport {
    ValidationReport::new(name)
}

/// Creates a new validation reporter.
pub fn create_validation_reporter() -> ValidationReporter {
    ValidationReporter::new()
}

/// Creates a new validator builder.
pub fn create_validator_builder(name: &str) -> ValidatorBuilder {
    ValidatorBuilder::new(name)
}

/// Creates a new validation registry.
pub fn create_validation_registry() -> ValidationRegistry {
    ValidationRegistry::new()
}
