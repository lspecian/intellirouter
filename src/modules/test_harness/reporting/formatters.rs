//! Test Result Formatters
//!
//! This module provides formatters for test results.

use std::collections::HashMap;

use super::super::types::{TestHarnessError, TestResult, TestSuiteResult};

/// Formatter configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Include passed tests
    pub include_passed: bool,
    /// Include skipped tests
    pub include_skipped: bool,
    /// Include test output
    pub include_output: bool,
    /// Include assertions
    pub include_assertions: bool,
    /// Include timestamps
    pub include_timestamps: bool,
    /// Include duration
    pub include_duration: bool,
    /// Include metadata
    pub include_metadata: bool,
    /// Include file and line
    pub include_file_and_line: bool,
    /// Include category and suite
    pub include_category_and_suite: bool,
    /// Include children
    pub include_children: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            include_passed: true,
            include_skipped: true,
            include_output: true,
            include_assertions: true,
            include_timestamps: true,
            include_duration: true,
            include_metadata: true,
            include_file_and_line: true,
            include_category_and_suite: true,
            include_children: true,
        }
    }
}

/// Formatter trait for formatting test results
pub trait Formatter: Send + Sync {
    /// Format a test result
    fn format_test_result(
        &self,
        result: &TestResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError>;

    /// Format a test suite result
    fn format_suite_result(
        &self,
        result: &TestSuiteResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError>;
}

/// Console formatter
pub struct ConsoleFormatter {
    /// Configuration
    config: FormatterConfig,
}

impl ConsoleFormatter {
    /// Create a new console formatter
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Create a new console formatter with custom configuration
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl Formatter for ConsoleFormatter {
    fn format_test_result(
        &self,
        result: &TestResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        // Format test name and outcome
        let status = match result.outcome {
            super::super::types::TestOutcome::Passed => "✅ PASS",
            super::super::types::TestOutcome::Failed => "❌ FAIL",
            super::super::types::TestOutcome::Skipped => "⏭️ SKIP",
            super::super::types::TestOutcome::TimedOut => "⏱️ TIMEOUT",
            super::super::types::TestOutcome::Panicked => "💥 PANIC",
        };
        output.push_str(&format!("{} {}\n", status, result.name));

        // Format error message if any
        if let Some(error) = &result.error {
            output.push_str(&format!("  Error: {}\n", error));
        }

        // Format duration if requested
        if config.include_duration {
            output.push_str(&format!("  Duration: {:?}\n", result.duration));
        }

        // Format timestamps if requested
        if config.include_timestamps {
            output.push_str(&format!("  Start: {}\n", result.start_time));
            output.push_str(&format!("  End: {}\n", result.end_time));
        }

        // Format category and suite if requested
        if config.include_category_and_suite {
            output.push_str(&format!("  Category: {:?}\n", result.category));
        }

        // Format artifacts if any
        if !result.artifacts.is_empty() {
            output.push_str("  Artifacts:\n");
            for (key, value) in &result.artifacts {
                output.push_str(&format!("    {}: {}\n", key, value));
            }
        }

        // Format metrics if any
        if !result.metrics.is_empty() {
            output.push_str("  Metrics:\n");
            for (key, value) in &result.metrics {
                output.push_str(&format!("    {}: {}\n", key, value));
            }
        }

        // Format logs if requested and any
        if config.include_output && !result.logs.is_empty() {
            output.push_str("  Logs:\n");
            for log in &result.logs {
                output.push_str(&format!("    {}\n", log));
            }
        }

        Ok(output)
    }

    fn format_suite_result(
        &self,
        result: &TestSuiteResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        // Format suite name
        output.push_str(&format!("Test Suite: {}\n", result.name));

        // Format duration if requested
        if config.include_duration {
            output.push_str(&format!("Duration: {:?}\n", result.duration));
        }

        // Format timestamps if requested
        if config.include_timestamps {
            output.push_str(&format!("Start: {}\n", result.start_time));
            output.push_str(&format!("End: {}\n", result.end_time));
        }

        // Format summary
        output.push_str(&format!("Total Tests: {}\n", result.total_tests()));
        output.push_str(&format!("Passed: {}\n", result.passed));
        output.push_str(&format!("Failed: {}\n", result.failed));
        output.push_str(&format!("Skipped: {}\n", result.skipped));
        output.push_str(&format!("Timed Out: {}\n", result.timed_out));
        output.push_str(&format!("Panicked: {}\n", result.panicked));

        // Format test results if requested
        if config.include_children {
            output.push_str("\nTest Results:\n");
            for test_result in &result.test_results {
                let formatted = self.format_test_result(test_result, config)?;
                output.push_str(&formatted);
                output.push('\n');
            }
        }

        Ok(output)
    }
}

/// JSON formatter
pub struct JsonFormatter {
    /// Configuration
    config: FormatterConfig,
}

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Create a new JSON formatter with custom configuration
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl Formatter for JsonFormatter {
    fn format_test_result(
        &self,
        result: &TestResult,
        _config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        serde_json::to_string_pretty(result).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize test result: {}", e))
        })
    }

    fn format_suite_result(
        &self,
        result: &TestSuiteResult,
        _config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        serde_json::to_string_pretty(result).map_err(|e| {
            TestHarnessError::SerializationError(format!(
                "Failed to serialize test suite result: {}",
                e
            ))
        })
    }
}

/// Markdown formatter
pub struct MarkdownFormatter {
    /// Configuration
    config: FormatterConfig,
}

impl MarkdownFormatter {
    /// Create a new Markdown formatter
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Create a new Markdown formatter with custom configuration
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl Formatter for MarkdownFormatter {
    fn format_test_result(
        &self,
        result: &TestResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        // Format test name and outcome
        let status = match result.outcome {
            super::super::types::TestOutcome::Passed => "✅ PASS",
            super::super::types::TestOutcome::Failed => "❌ FAIL",
            super::super::types::TestOutcome::Skipped => "⏭️ SKIP",
            super::super::types::TestOutcome::TimedOut => "⏱️ TIMEOUT",
            super::super::types::TestOutcome::Panicked => "💥 PANIC",
        };
        output.push_str(&format!("### {} {}\n\n", status, result.name));

        // Format error message if any
        if let Some(error) = &result.error {
            output.push_str(&format!("**Error:** {}\n\n", error));
        }

        // Format duration if requested
        if config.include_duration {
            output.push_str(&format!("**Duration:** {:?}\n\n", result.duration));
        }

        // Format timestamps if requested
        if config.include_timestamps {
            output.push_str(&format!("**Start:** {}\n\n", result.start_time));
            output.push_str(&format!("**End:** {}\n\n", result.end_time));
        }

        // Format category and suite if requested
        if config.include_category_and_suite {
            output.push_str(&format!("**Category:** {:?}\n\n", result.category));
        }

        // Format artifacts if any
        if !result.artifacts.is_empty() {
            output.push_str("#### Artifacts\n\n");
            output.push_str("| Key | Value |\n");
            output.push_str("|-----|-------|\n");
            for (key, value) in &result.artifacts {
                output.push_str(&format!("| {} | {} |\n", key, value));
            }
            output.push('\n');
        }

        // Format metrics if any
        if !result.metrics.is_empty() {
            output.push_str("#### Metrics\n\n");
            output.push_str("| Metric | Value |\n");
            output.push_str("|--------|-------|\n");
            for (key, value) in &result.metrics {
                output.push_str(&format!("| {} | {} |\n", key, value));
            }
            output.push('\n');
        }

        // Format logs if requested and any
        if config.include_output && !result.logs.is_empty() {
            output.push_str("#### Logs\n\n");
            output.push_str("```\n");
            for log in &result.logs {
                output.push_str(&format!("{}\n", log));
            }
            output.push_str("```\n\n");
        }

        Ok(output)
    }

    fn format_suite_result(
        &self,
        result: &TestSuiteResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        // Format suite name
        output.push_str(&format!("# Test Suite: {}\n\n", result.name));

        // Format duration if requested
        if config.include_duration {
            output.push_str(&format!("**Duration:** {:?}\n\n", result.duration));
        }

        // Format timestamps if requested
        if config.include_timestamps {
            output.push_str(&format!("**Start:** {}\n\n", result.start_time));
            output.push_str(&format!("**End:** {}\n\n", result.end_time));
        }

        // Format summary
        output.push_str("## Summary\n\n");
        output.push_str("| Metric | Value |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!("| Total Tests | {} |\n", result.total_tests()));
        output.push_str(&format!("| Passed | {} |\n", result.passed));
        output.push_str(&format!("| Failed | {} |\n", result.failed));
        output.push_str(&format!("| Skipped | {} |\n", result.skipped));
        output.push_str(&format!("| Timed Out | {} |\n", result.timed_out));
        output.push_str(&format!("| Panicked | {} |\n\n", result.panicked));

        // Format test results if requested
        if config.include_children {
            output.push_str("## Test Results\n\n");
            for test_result in &result.test_results {
                let formatted = self.format_test_result(test_result, config)?;
                output.push_str(&formatted);
                output.push('\n');
            }
        }

        Ok(output)
    }
}

/// HTML formatter
pub struct HtmlFormatter {
    /// Configuration
    config: FormatterConfig,
}

impl HtmlFormatter {
    /// Create a new HTML formatter
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Create a new HTML formatter with custom configuration
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl Formatter for HtmlFormatter {
    fn format_test_result(
        &self,
        result: &TestResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        // Format test name and outcome
        let (status_class, status_text) = match result.outcome {
            super::super::types::TestOutcome::Passed => ("passed", "PASS"),
            super::super::types::TestOutcome::Failed => ("failed", "FAIL"),
            super::super::types::TestOutcome::Skipped => ("skipped", "SKIP"),
            super::super::types::TestOutcome::TimedOut => ("timeout", "TIMEOUT"),
            super::super::types::TestOutcome::Panicked => ("panic", "PANIC"),
        };
        output.push_str(&format!("<div class=\"test-result {}\">\n", status_class));
        output.push_str(&format!(
            "  <h3><span class=\"status\">{}</span> {}</h3>\n",
            status_text, result.name
        ));

        // Format error message if any
        if let Some(error) = &result.error {
            output.push_str(&format!(
                "  <div class=\"error\"><strong>Error:</strong> {}</div>\n",
                error
            ));
        }

        // Format duration if requested
        if config.include_duration {
            output.push_str(&format!(
                "  <div><strong>Duration:</strong> {:?}</div>\n",
                result.duration
            ));
        }

        // Format timestamps if requested
        if config.include_timestamps {
            output.push_str(&format!(
                "  <div><strong>Start:</strong> {}</div>\n",
                result.start_time
            ));
            output.push_str(&format!(
                "  <div><strong>End:</strong> {}</div>\n",
                result.end_time
            ));
        }

        // Format category and suite if requested
        if config.include_category_and_suite {
            output.push_str(&format!(
                "  <div><strong>Category:</strong> {:?}</div>\n",
                result.category
            ));
        }

        // Format artifacts if any
        if !result.artifacts.is_empty() {
            output.push_str("  <div class=\"artifacts\">\n");
            output.push_str("    <h4>Artifacts</h4>\n");
            output.push_str("    <table>\n");
            output.push_str("      <tr><th>Key</th><th>Value</th></tr>\n");
            for (key, value) in &result.artifacts {
                output.push_str(&format!(
                    "      <tr><td>{}</td><td>{}</td></tr>\n",
                    key, value
                ));
            }
            output.push_str("    </table>\n");
            output.push_str("  </div>\n");
        }

        // Format metrics if any
        if !result.metrics.is_empty() {
            output.push_str("  <div class=\"metrics\">\n");
            output.push_str("    <h4>Metrics</h4>\n");
            output.push_str("    <table>\n");
            output.push_str("      <tr><th>Metric</th><th>Value</th></tr>\n");
            for (key, value) in &result.metrics {
                output.push_str(&format!(
                    "      <tr><td>{}</td><td>{}</td></tr>\n",
                    key, value
                ));
            }
            output.push_str("    </table>\n");
            output.push_str("  </div>\n");
        }

        // Format logs if requested and any
        if config.include_output && !result.logs.is_empty() {
            output.push_str("  <div class=\"logs\">\n");
            output.push_str("    <h4>Logs</h4>\n");
            output.push_str("    <pre>\n");
            for log in &result.logs {
                output.push_str(&format!("{}\n", log));
            }
            output.push_str("    </pre>\n");
            output.push_str("  </div>\n");
        }

        output.push_str("</div>\n");

        Ok(output)
    }

    fn format_suite_result(
        &self,
        result: &TestSuiteResult,
        config: &FormatterConfig,
    ) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        // Start HTML document
        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html>\n");
        output.push_str("<head>\n");
        output.push_str("  <meta charset=\"UTF-8\">\n");
        output.push_str(&format!("  <title>Test Suite: {}</title>\n", result.name));
        output.push_str("  <style>\n");
        output.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
        output.push_str("    .passed { background-color: #dff0d8; }\n");
        output.push_str("    .failed { background-color: #f2dede; }\n");
        output.push_str("    .skipped { background-color: #fcf8e3; }\n");
        output.push_str("    .timeout { background-color: #f2dede; }\n");
        output.push_str("    .panic { background-color: #f2dede; }\n");
        output.push_str(
            "    .test-result { margin-bottom: 20px; padding: 10px; border-radius: 5px; }\n",
        );
        output.push_str("    .status { font-weight: bold; }\n");
        output.push_str("    .error { color: red; margin-bottom: 10px; }\n");
        output.push_str("    table { border-collapse: collapse; width: 100%; }\n");
        output.push_str("    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        output.push_str("    th { background-color: #f2f2f2; }\n");
        output.push_str("    pre { background-color: #f5f5f5; padding: 10px; overflow: auto; }\n");
        output.push_str("  </style>\n");
        output.push_str("</head>\n");
        output.push_str("<body>\n");

        // Format suite name
        output.push_str(&format!("<h1>Test Suite: {}</h1>\n", result.name));

        // Format duration if requested
        if config.include_duration {
            output.push_str(&format!(
                "<p><strong>Duration:</strong> {:?}</p>\n",
                result.duration
            ));
        }

        // Format timestamps if requested
        if config.include_timestamps {
            output.push_str(&format!(
                "<p><strong>Start:</strong> {}</p>\n",
                result.start_time
            ));
            output.push_str(&format!(
                "<p><strong>End:</strong> {}</p>\n",
                result.end_time
            ));
        }

        // Format summary
        output.push_str("<h2>Summary</h2>\n");
        output.push_str("<table>\n");
        output.push_str("  <tr><th>Metric</th><th>Value</th></tr>\n");
        output.push_str(&format!(
            "  <tr><td>Total Tests</td><td>{}</td></tr>\n",
            result.total_tests()
        ));
        output.push_str(&format!(
            "  <tr><td>Passed</td><td>{}</td></tr>\n",
            result.passed
        ));
        output.push_str(&format!(
            "  <tr><td>Failed</td><td>{}</td></tr>\n",
            result.failed
        ));
        output.push_str(&format!(
            "  <tr><td>Skipped</td><td>{}</td></tr>\n",
            result.skipped
        ));
        output.push_str(&format!(
            "  <tr><td>Timed Out</td><td>{}</td></tr>\n",
            result.timed_out
        ));
        output.push_str(&format!(
            "  <tr><td>Panicked</td><td>{}</td></tr>\n",
            result.panicked
        ));
        output.push_str("</table>\n");

        // Format test results if requested
        if config.include_children {
            output.push_str("<h2>Test Results</h2>\n");
            for test_result in &result.test_results {
                let formatted = self.format_test_result(test_result, config)?;
                output.push_str(&formatted);
            }
        }

        // End HTML document
        output.push_str("</body>\n");
        output.push_str("</html>\n");

        Ok(output)
    }
}
