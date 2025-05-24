//! Exporters module for exporting test reports
//!
//! This module provides functionality for exporting test reports in various formats.

use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::modules::test_harness::types::TestHarnessError;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    /// HTML format
    Html,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// CSV format
    Csv,
    /// XML format
    Xml,
    /// PDF format
    Pdf,
    /// Text format
    Text,
    /// Console format
    Console,
}

impl ExportFormat {
    /// Get the file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Html => "html",
            ExportFormat::Json => "json",
            ExportFormat::Markdown => "md",
            ExportFormat::Csv => "csv",
            ExportFormat::Xml => "xml",
            ExportFormat::Pdf => "pdf",
            ExportFormat::Text => "txt",
            ExportFormat::Console => "txt",
        }
    }

    /// Get the MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Html => "text/html",
            ExportFormat::Json => "application/json",
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Csv => "text/csv",
            ExportFormat::Xml => "application/xml",
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Text => "text/plain",
            ExportFormat::Console => "text/plain",
        }
    }
}

/// Exporter trait
#[async_trait]
pub trait Exporter: Send + Sync {
    /// Get the exporter name
    fn name(&self) -> &str;

    /// Get the exporter description
    fn description(&self) -> Option<&str>;

    /// Get the supported formats
    fn supported_formats(&self) -> Vec<ExportFormat>;

    /// Export a report
    async fn export(&self, content: String, path: &Path) -> Result<(), TestHarnessError>;
}

/// HTML exporter
pub struct HtmlExporter {
    /// Exporter name
    name: String,
    /// Exporter description
    description: Option<String>,
}

impl HtmlExporter {
    /// Create a new HTML exporter
    pub fn new() -> Self {
        Self {
            name: "html".to_string(),
            description: Some("HTML exporter".to_string()),
        }
    }
}

impl Default for HtmlExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Exporter for HtmlExporter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_formats(&self) -> Vec<ExportFormat> {
        vec![ExportFormat::Html]
    }

    async fn export(&self, content: String, path: &Path) -> Result<(), TestHarnessError> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| TestHarnessError::IoError(e))
    }
}

/// JSON exporter
pub struct JsonExporter {
    /// Exporter name
    name: String,
    /// Exporter description
    description: Option<String>,
}

impl JsonExporter {
    /// Create a new JSON exporter
    pub fn new() -> Self {
        Self {
            name: "json".to_string(),
            description: Some("JSON exporter".to_string()),
        }
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Exporter for JsonExporter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_formats(&self) -> Vec<ExportFormat> {
        vec![ExportFormat::Json]
    }

    async fn export(&self, content: String, path: &Path) -> Result<(), TestHarnessError> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| TestHarnessError::IoError(e))
    }
}

/// Markdown exporter
pub struct MarkdownExporter {
    /// Exporter name
    name: String,
    /// Exporter description
    description: Option<String>,
}

impl MarkdownExporter {
    /// Create a new Markdown exporter
    pub fn new() -> Self {
        Self {
            name: "markdown".to_string(),
            description: Some("Markdown exporter".to_string()),
        }
    }
}

impl Default for MarkdownExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Exporter for MarkdownExporter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_formats(&self) -> Vec<ExportFormat> {
        vec![ExportFormat::Markdown]
    }

    async fn export(&self, content: String, path: &Path) -> Result<(), TestHarnessError> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| TestHarnessError::IoError(e))
    }
}

/// CSV exporter
pub struct CsvExporter {
    /// Exporter name
    name: String,
    /// Exporter description
    description: Option<String>,
}

impl CsvExporter {
    /// Create a new CSV exporter
    pub fn new() -> Self {
        Self {
            name: "csv".to_string(),
            description: Some("CSV exporter".to_string()),
        }
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Exporter for CsvExporter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_formats(&self) -> Vec<ExportFormat> {
        vec![ExportFormat::Csv]
    }

    async fn export(&self, content: String, path: &Path) -> Result<(), TestHarnessError> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| TestHarnessError::IoError(e))
    }
}

/// Multi-format exporter
pub struct MultiFormatExporter {
    /// Exporter name
    name: String,
    /// Exporter description
    description: Option<String>,
    /// Supported formats
    formats: Vec<ExportFormat>,
    /// Exporters
    exporters: Vec<Box<dyn Exporter>>,
}

impl MultiFormatExporter {
    /// Create a new multi-format exporter
    pub fn new(name: impl Into<String>, exporters: Vec<Box<dyn Exporter>>) -> Self {
        let name = name.into();
        let description = Some(format!("Multi-format exporter: {}", name));

        // Collect supported formats from all exporters
        let mut formats = Vec::new();
        for exporter in &exporters {
            for format in exporter.supported_formats() {
                if !formats.contains(&format) {
                    formats.push(format);
                }
            }
        }

        Self {
            name,
            description,
            formats,
            exporters,
        }
    }

    /// Add an exporter
    pub fn add_exporter(&mut self, exporter: Box<dyn Exporter>) {
        // Add new supported formats
        for format in exporter.supported_formats() {
            if !self.formats.contains(&format) {
                self.formats.push(format);
            }
        }

        self.exporters.push(exporter);
    }
}

#[async_trait]
impl Exporter for MultiFormatExporter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_formats(&self) -> Vec<ExportFormat> {
        self.formats.clone()
    }

    async fn export(&self, content: String, path: &Path) -> Result<(), TestHarnessError> {
        // Determine the format from the file extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Find an exporter that supports this format
        for exporter in &self.exporters {
            let supported_formats = exporter.supported_formats();
            let format_match = supported_formats
                .iter()
                .find(|f| f.extension() == extension);

            if format_match.is_some() {
                return exporter.export(content.clone(), path).await;
            }
        }

        // If no exporter was found, use the first one
        if let Some(exporter) = self.exporters.first() {
            return exporter.export(content, path).await;
        }

        Err(TestHarnessError::ReportingError(format!(
            "No exporter found for format: {}",
            extension
        )))
    }
}
