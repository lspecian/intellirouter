//! Reporting and Dashboard System
//!
//! This module provides functionality for generating detailed reports of test results
//! and visualizing test metrics and statistics.

mod dashboard;
mod exporters;
mod formatters;
mod renderers;
mod templates;
mod web_server;

pub use dashboard::{Dashboard, DashboardConfig, DashboardPanel, DashboardView};
pub use exporters::{ExportFormat, Exporter};
pub use formatters::{
    ConsoleFormatter, Formatter, FormatterConfig, HtmlFormatter, JsonFormatter, MarkdownFormatter,
};
pub use renderers::{Renderer, RendererConfig};
pub use templates::{Template, TemplateContext, TemplateEngine, TemplateLoader};
pub use web_server::{DashboardServer, DashboardServerConfig, DashboardServerState};

use super::types::{TestHarnessError, TestResult, TestSuiteResult};
use async_trait::async_trait;

/// Reporter trait for reporting test results
#[async_trait]
pub trait Reporter: Send + Sync {
    /// Report a test result
    async fn report_test_result(&self, result: &TestResult);

    /// Report a test suite result
    async fn report_suite_result(&self, result: &TestSuiteResult);

    /// Report a warning
    async fn report_warning(&self, message: &str);

    /// Report an error
    async fn report_error(&self, message: &str);

    /// Report a message
    async fn report_message(&self, message: &str);
}

/// Console reporter
pub struct ConsoleReporter {
    /// Formatter
    formatter: Box<dyn Formatter>,
}

impl ConsoleReporter {
    /// Create a new console reporter
    pub fn new() -> Self {
        Self {
            formatter: Box::new(ConsoleFormatter::new()),
        }
    }

    /// Create a new console reporter with a custom formatter
    pub fn with_formatter(formatter: Box<dyn Formatter>) -> Self {
        Self { formatter }
    }
}

#[async_trait]
impl Reporter for ConsoleReporter {
    async fn report_test_result(&self, result: &TestResult) {
        let formatted = self
            .formatter
            .format_test_result(result, &FormatterConfig::default())
            .unwrap_or_else(|e| format!("Error formatting test result: {}", e));
        println!("{}", formatted);
    }

    async fn report_suite_result(&self, result: &TestSuiteResult) {
        let formatted = self
            .formatter
            .format_suite_result(result, &FormatterConfig::default())
            .unwrap_or_else(|e| format!("Error formatting suite result: {}", e));
        println!("{}", formatted);
    }

    async fn report_warning(&self, message: &str) {
        println!("⚠️ WARNING: {}", message);
    }

    async fn report_error(&self, message: &str) {
        println!("❌ ERROR: {}", message);
    }

    async fn report_message(&self, message: &str) {
        println!("ℹ️ {}", message);
    }
}

/// File reporter
pub struct FileReporter {
    /// Formatter
    formatter: Box<dyn Formatter>,
    /// Output directory
    output_dir: std::path::PathBuf,
}

impl FileReporter {
    /// Create a new file reporter
    pub fn new(output_dir: impl Into<std::path::PathBuf>, formatter: Box<dyn Formatter>) -> Self {
        Self {
            formatter,
            output_dir: output_dir.into(),
        }
    }
}

#[async_trait]
impl Reporter for FileReporter {
    async fn report_test_result(&self, result: &TestResult) {
        let formatted = match self
            .formatter
            .format_test_result(result, &FormatterConfig::default())
        {
            Ok(formatted) => formatted,
            Err(e) => {
                eprintln!("Error formatting test result: {}", e);
                return;
            }
        };

        let file_name = format!("test_result_{}.txt", result.name.replace(" ", "_"));
        let file_path = self.output_dir.join(file_name);

        if let Err(e) = tokio::fs::create_dir_all(&self.output_dir).await {
            eprintln!("Error creating output directory: {}", e);
            return;
        }

        if let Err(e) = tokio::fs::write(&file_path, formatted).await {
            eprintln!("Error writing test result to file: {}", e);
        }
    }

    async fn report_suite_result(&self, result: &TestSuiteResult) {
        let formatted = match self
            .formatter
            .format_suite_result(result, &FormatterConfig::default())
        {
            Ok(formatted) => formatted,
            Err(e) => {
                eprintln!("Error formatting suite result: {}", e);
                return;
            }
        };

        let file_name = format!("suite_result_{}.txt", result.name.replace(" ", "_"));
        let file_path = self.output_dir.join(file_name);

        if let Err(e) = tokio::fs::create_dir_all(&self.output_dir).await {
            eprintln!("Error creating output directory: {}", e);
            return;
        }

        if let Err(e) = tokio::fs::write(&file_path, formatted).await {
            eprintln!("Error writing suite result to file: {}", e);
        }
    }

    async fn report_warning(&self, message: &str) {
        let file_path = self.output_dir.join("warnings.log");

        if let Err(e) = tokio::fs::create_dir_all(&self.output_dir).await {
            eprintln!("Error creating output directory: {}", e);
            return;
        }

        let mut content = String::new();
        if let Ok(existing) = tokio::fs::read_to_string(&file_path).await {
            content = existing;
        }

        content.push_str(&format!("[{}] WARNING: {}\n", chrono::Utc::now(), message));

        if let Err(e) = tokio::fs::write(&file_path, content).await {
            eprintln!("Error writing warning to file: {}", e);
        }
    }

    async fn report_error(&self, message: &str) {
        let file_path = self.output_dir.join("errors.log");

        if let Err(e) = tokio::fs::create_dir_all(&self.output_dir).await {
            eprintln!("Error creating output directory: {}", e);
            return;
        }

        let mut content = String::new();
        if let Ok(existing) = tokio::fs::read_to_string(&file_path).await {
            content = existing;
        }

        content.push_str(&format!("[{}] ERROR: {}\n", chrono::Utc::now(), message));

        if let Err(e) = tokio::fs::write(&file_path, content).await {
            eprintln!("Error writing error to file: {}", e);
        }
    }

    async fn report_message(&self, message: &str) {
        let file_path = self.output_dir.join("messages.log");

        if let Err(e) = tokio::fs::create_dir_all(&self.output_dir).await {
            eprintln!("Error creating output directory: {}", e);
            return;
        }

        let mut content = String::new();
        if let Ok(existing) = tokio::fs::read_to_string(&file_path).await {
            content = existing;
        }

        content.push_str(&format!("[{}] INFO: {}\n", chrono::Utc::now(), message));

        if let Err(e) = tokio::fs::write(&file_path, content).await {
            eprintln!("Error writing message to file: {}", e);
        }
    }
}

/// Multi reporter that forwards reports to multiple reporters
pub struct MultiReporter {
    /// Reporters
    reporters: Vec<Box<dyn Reporter>>,
}

impl MultiReporter {
    /// Create a new multi reporter
    pub fn new(reporters: Vec<Box<dyn Reporter>>) -> Self {
        Self { reporters }
    }

    /// Add a reporter
    pub fn add_reporter(&mut self, reporter: Box<dyn Reporter>) {
        self.reporters.push(reporter);
    }
}

#[async_trait]
impl Reporter for MultiReporter {
    async fn report_test_result(&self, result: &TestResult) {
        for reporter in &self.reporters {
            reporter.report_test_result(result).await;
        }
    }

    async fn report_suite_result(&self, result: &TestSuiteResult) {
        for reporter in &self.reporters {
            reporter.report_suite_result(result).await;
        }
    }

    async fn report_warning(&self, message: &str) {
        for reporter in &self.reporters {
            reporter.report_warning(message).await;
        }
    }

    async fn report_error(&self, message: &str) {
        for reporter in &self.reporters {
            reporter.report_error(message).await;
        }
    }

    async fn report_message(&self, message: &str) {
        for reporter in &self.reporters {
            reporter.report_message(message).await;
        }
    }
}
