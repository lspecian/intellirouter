//! Documentation Generator Module
//!
//! This module provides functionality for generating documentation for the test harness.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::TestHarnessError;

/// Documentation format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentationFormat {
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
    /// PDF format
    Pdf,
    /// AsciiDoc format
    AsciiDoc,
    /// reStructuredText format
    ReStructuredText,
}

impl fmt::Display for DocumentationFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentationFormat::Markdown => write!(f, "Markdown"),
            DocumentationFormat::Html => write!(f, "HTML"),
            DocumentationFormat::Pdf => write!(f, "PDF"),
            DocumentationFormat::AsciiDoc => write!(f, "AsciiDoc"),
            DocumentationFormat::ReStructuredText => write!(f, "reStructuredText"),
        }
    }
}

impl DocumentationFormat {
    /// Get the file extension for the documentation format
    pub fn extension(&self) -> &'static str {
        match self {
            DocumentationFormat::Markdown => "md",
            DocumentationFormat::Html => "html",
            DocumentationFormat::Pdf => "pdf",
            DocumentationFormat::AsciiDoc => "adoc",
            DocumentationFormat::ReStructuredText => "rst",
        }
    }
}

/// Documentation section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSection {
    /// Section ID
    pub id: String,
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
    /// Section subsections
    pub subsections: Vec<DocumentationSection>,
    /// Section metadata
    pub metadata: HashMap<String, String>,
}

impl DocumentationSection {
    /// Create a new documentation section
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            subsections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a subsection to the section
    pub fn with_subsection(mut self, subsection: DocumentationSection) -> Self {
        self.subsections.push(subsection);
        self
    }

    /// Add multiple subsections to the section
    pub fn with_subsections(mut self, subsections: Vec<DocumentationSection>) -> Self {
        self.subsections.extend(subsections);
        self
    }

    /// Add metadata to the section
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries to the section
    pub fn with_metadata_entries(
        mut self,
        entries: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in entries {
            self.metadata.insert(key.into(), value.into());
        }
        self
    }

    /// Add a subsection to the section
    pub fn add_subsection(&mut self, subsection: DocumentationSection) {
        self.subsections.push(subsection);
    }

    /// Add metadata to the section
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Render the section to markdown
    pub fn to_markdown(&self, level: usize) -> String {
        let mut result = String::new();

        // Add section title
        let heading = "#".repeat(level);
        result.push_str(&format!("{} {}\n\n", heading, self.title));

        // Add section content
        result.push_str(&self.content);
        result.push_str("\n\n");

        // Add subsections
        for subsection in &self.subsections {
            result.push_str(&subsection.to_markdown(level + 1));
        }

        result
    }

    /// Render the section to HTML
    pub fn to_html(&self, level: usize) -> String {
        let mut result = String::new();

        // Add section title
        let heading_level = std::cmp::min(level, 6);
        result.push_str(&format!(
            "<h{}>{}</h{}>\n\n",
            heading_level, self.title, heading_level
        ));

        // Add section content
        result.push_str(&format!(
            "<div class=\"section-content\">\n{}\n</div>\n\n",
            self.content
        ));

        // Add subsections
        if !self.subsections.is_empty() {
            result.push_str("<div class=\"subsections\">\n");

            for subsection in &self.subsections {
                result.push_str(&subsection.to_html(level + 1));
            }

            result.push_str("</div>\n\n");
        }

        result
    }
}

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    /// Documentation title
    pub title: String,
    /// Documentation description
    pub description: Option<String>,
    /// Documentation version
    pub version: String,
    /// Documentation author
    pub author: Option<String>,
    /// Documentation output directory
    pub output_dir: PathBuf,
    /// Documentation formats
    pub formats: Vec<DocumentationFormat>,
    /// Documentation template directory
    pub template_dir: Option<PathBuf>,
    /// Documentation assets directory
    pub assets_dir: Option<PathBuf>,
    /// Documentation metadata
    pub metadata: HashMap<String, String>,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            title: "IntelliRouter Test Harness Documentation".to_string(),
            description: Some("Documentation for the IntelliRouter test harness".to_string()),
            version: "1.0.0".to_string(),
            author: Some("IntelliRouter Team".to_string()),
            output_dir: PathBuf::from("docs"),
            formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
            template_dir: None,
            assets_dir: None,
            metadata: HashMap::new(),
        }
    }
}

/// Documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    /// Documentation configuration
    pub config: DocumentationConfig,
    /// Documentation sections
    pub sections: Vec<DocumentationSection>,
    /// Documentation generation time
    pub generation_time: DateTime<Utc>,
}

impl Documentation {
    /// Create a new documentation
    pub fn new(config: DocumentationConfig) -> Self {
        Self {
            config,
            sections: Vec::new(),
            generation_time: Utc::now(),
        }
    }

    /// Add a section to the documentation
    pub fn with_section(mut self, section: DocumentationSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add multiple sections to the documentation
    pub fn with_sections(mut self, sections: Vec<DocumentationSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add a section to the documentation
    pub fn add_section(&mut self, section: DocumentationSection) {
        self.sections.push(section);
    }

    /// Render the documentation to markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // Add title
        result.push_str(&format!("# {}\n\n", self.config.title));

        // Add description
        if let Some(description) = &self.config.description {
            result.push_str(&format!("{}\n\n", description));
        }

        // Add metadata
        result.push_str("## Metadata\n\n");
        result.push_str(&format!("- **Version:** {}\n", self.config.version));

        if let Some(author) = &self.config.author {
            result.push_str(&format!("- **Author:** {}\n", author));
        }

        result.push_str(&format!(
            "- **Generated:** {}\n\n",
            self.generation_time.format("%Y-%m-%d %H:%M:%S")
        ));

        // Add table of contents
        result.push_str("## Table of Contents\n\n");

        for (i, section) in self.sections.iter().enumerate() {
            result.push_str(&format!(
                "{}. [{}](#{})\n",
                i + 1,
                section.title,
                section.id
            ));

            for (j, subsection) in section.subsections.iter().enumerate() {
                result.push_str(&format!(
                    "   {}. [{}](#{})\n",
                    j + 1,
                    subsection.title,
                    subsection.id
                ));
            }
        }

        result.push_str("\n");

        // Add sections
        for section in &self.sections {
            result.push_str(&section.to_markdown(2));
        }

        result
    }

    /// Render the documentation to HTML
    pub fn to_html(&self) -> String {
        let mut result = String::new();

        // Add HTML header
        result.push_str("<!DOCTYPE html>\n");
        result.push_str("<html lang=\"en\">\n");
        result.push_str("<head>\n");
        result.push_str("  <meta charset=\"UTF-8\">\n");
        result.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        result.push_str(&format!("  <title>{}</title>\n", self.config.title));
        result.push_str("  <style>\n");
        result.push_str("    body { font-family: Arial, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
        result.push_str("    h1, h2, h3, h4, h5, h6 { color: #333; }\n");
        result.push_str(
            "    code { background-color: #f4f4f4; padding: 2px 5px; border-radius: 3px; }\n",
        );
        result.push_str("    pre { background-color: #f4f4f4; padding: 10px; border-radius: 5px; overflow-x: auto; }\n");
        result.push_str("    .toc { background-color: #f9f9f9; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
        result.push_str("    .metadata { background-color: #f0f0f0; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
        result.push_str("  </style>\n");
        result.push_str("</head>\n");
        result.push_str("<body>\n");

        // Add title
        result.push_str(&format!("<h1>{}</h1>\n\n", self.config.title));

        // Add description
        if let Some(description) = &self.config.description {
            result.push_str(&format!("<p>{}</p>\n\n", description));
        }

        // Add metadata
        result.push_str("<div class=\"metadata\">\n");
        result.push_str("<h2>Metadata</h2>\n");
        result.push_str("<ul>\n");
        result.push_str(&format!(
            "  <li><strong>Version:</strong> {}</li>\n",
            self.config.version
        ));

        if let Some(author) = &self.config.author {
            result.push_str(&format!("  <li><strong>Author:</strong> {}</li>\n", author));
        }

        result.push_str(&format!(
            "  <li><strong>Generated:</strong> {}</li>\n",
            self.generation_time.format("%Y-%m-%d %H:%M:%S")
        ));
        result.push_str("</ul>\n");
        result.push_str("</div>\n\n");

        // Add table of contents
        result.push_str("<div class=\"toc\">\n");
        result.push_str("<h2>Table of Contents</h2>\n");
        result.push_str("<ol>\n");

        for (i, section) in self.sections.iter().enumerate() {
            result.push_str(&format!(
                "  <li><a href=\"#{}\">{}</a>\n",
                section.id, section.title
            ));

            if !section.subsections.is_empty() {
                result.push_str("    <ol>\n");

                for subsection in &section.subsections {
                    result.push_str(&format!(
                        "      <li><a href=\"#{}\">{}</a></li>\n",
                        subsection.id, subsection.title
                    ));
                }

                result.push_str("    </ol>\n");
            }

            result.push_str("  </li>\n");
        }

        result.push_str("</ol>\n");
        result.push_str("</div>\n\n");

        // Add sections
        for section in &self.sections {
            result.push_str(&format!("<div id=\"{}\" class=\"section\">\n", section.id));
            result.push_str(&section.to_html(2));
            result.push_str("</div>\n\n");
        }

        // Add HTML footer
        result.push_str("</body>\n");
        result.push_str("</html>\n");

        result
    }
}

/// Documentation generator
pub struct DocumentationGenerator {
    /// Documentation configuration
    config: DocumentationConfig,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new(config: DocumentationConfig) -> Self {
        Self { config }
    }

    /// Generate documentation for the test harness
    pub async fn generate(&self) -> Result<Documentation, TestHarnessError> {
        info!("Generating documentation for the test harness");

        // Create documentation
        let mut documentation = Documentation::new(self.config.clone());

        // Add overview section
        let overview_section = DocumentationSection::new(
            "overview",
            "Overview",
            "The IntelliRouter test harness provides a comprehensive framework for testing, benchmarking, and security scanning of IntelliRouter components.",
        )
        .with_subsection(DocumentationSection::new(
            "overview-purpose",
            "Purpose",
            "The purpose of the test harness is to provide a unified framework for testing all aspects of IntelliRouter, from unit tests to end-to-end tests, performance benchmarks, and security scans.",
        ))
        .with_subsection(DocumentationSection::new(
            "overview-architecture",
            "Architecture",
            "The test harness is built with a modular architecture, consisting of several key components that work together to provide a comprehensive testing solution.",
        ));

        documentation.add_section(overview_section);

        // Add components section
        let components_section = DocumentationSection::new(
            "components",
            "Components",
            "The test harness consists of several key components, each responsible for a specific aspect of testing.",
        )
        .with_subsection(DocumentationSection::new(
            "components-execution-engine",
            "Test Execution Engine",
            "The test execution engine is responsible for discovering, filtering, and executing tests. It provides a plugin-based architecture for extending the test harness with new test types.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-environment-management",
            "Environment Management",
            "The environment management component is responsible for provisioning and tearing down test environments. It provides support for containerized environments and configuration via TOML files.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-test-data",
            "Test Data Management",
            "The test data management component is responsible for generating and managing test data. It provides factories for creating test entities and support for data fixtures and seeding.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-mocking",
            "Mocking/Stubbing Framework",
            "The mocking/stubbing framework is responsible for creating mock objects and stubs for testing. It provides a flexible behavior system for defining mock responses and a recorder for tracking interactions.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-assertions",
            "Assertion Libraries",
            "The assertion libraries provide a fluent API for making assertions in tests. They support various matcher types and provide detailed error reporting for failed assertions.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-reporting",
            "Reporting and Dashboard",
            "The reporting and dashboard component is responsible for generating reports and dashboards for test results. It supports multiple output formats and provides detailed metrics and statistics.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-benchmarking",
            "Performance Benchmarking",
            "The performance benchmarking component is responsible for measuring and analyzing the performance of IntelliRouter components. It supports different benchmark types and provides detailed metrics and analysis.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-security",
            "Security Testing",
            "The security testing component is responsible for identifying and analyzing security vulnerabilities in IntelliRouter components. It supports different security test types and provides detailed vulnerability tracking and reporting.",
        ))
        .with_subsection(DocumentationSection::new(
            "components-ci",
            "CI Integration",
            "The CI integration component is responsible for integrating the test harness with CI/CD pipelines. It supports different CI providers and provides automated test execution and reporting.",
        ));

        documentation.add_section(components_section);

        // Add usage section
        let usage_section = DocumentationSection::new(
            "usage",
            "Usage",
            "This section provides examples of how to use the test harness for different testing scenarios.",
        )
        .with_subsection(DocumentationSection::new(
            "usage-unit-tests",
            "Unit Tests",
            "Unit tests focus on testing individual components in isolation. The test harness provides a fluent API for writing unit tests with expressive assertions.",
        ))
        .with_subsection(DocumentationSection::new(
            "usage-integration-tests",
            "Integration Tests",
            "Integration tests focus on testing the interaction between components. The test harness provides support for setting up test environments and managing test data for integration tests.",
        ))
        .with_subsection(DocumentationSection::new(
            "usage-end-to-end-tests",
            "End-to-End Tests",
            "End-to-end tests focus on testing the entire system from a user's perspective. The test harness provides support for setting up complete test environments and simulating user interactions.",
        ))
        .with_subsection(DocumentationSection::new(
            "usage-performance-tests",
            "Performance Tests",
            "Performance tests focus on measuring the performance characteristics of the system. The test harness provides a benchmarking framework for measuring throughput, latency, and resource usage.",
        ))
        .with_subsection(DocumentationSection::new(
            "usage-security-tests",
            "Security Tests",
            "Security tests focus on identifying security vulnerabilities in the system. The test harness provides a security testing framework for scanning dependencies, detecting secrets, and identifying vulnerabilities.",
        ));

        documentation.add_section(usage_section);

        // Add API reference section
        let api_section = DocumentationSection::new(
            "api",
            "API Reference",
            "This section provides a reference for the test harness API.",
        )
        .with_subsection(DocumentationSection::new(
            "api-execution-engine",
            "Test Execution Engine API",
            "The test execution engine API provides functions for discovering, filtering, and executing tests.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-environment-management",
            "Environment Management API",
            "The environment management API provides functions for provisioning and tearing down test environments.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-test-data",
            "Test Data Management API",
            "The test data management API provides functions for generating and managing test data.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-mocking",
            "Mocking/Stubbing API",
            "The mocking/stubbing API provides functions for creating mock objects and stubs for testing.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-assertions",
            "Assertion API",
            "The assertion API provides functions for making assertions in tests.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-reporting",
            "Reporting and Dashboard API",
            "The reporting and dashboard API provides functions for generating reports and dashboards for test results.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-benchmarking",
            "Performance Benchmarking API",
            "The performance benchmarking API provides functions for measuring and analyzing the performance of IntelliRouter components.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-security",
            "Security Testing API",
            "The security testing API provides functions for identifying and analyzing security vulnerabilities in IntelliRouter components.",
        ))
        .with_subsection(DocumentationSection::new(
            "api-ci",
            "CI Integration API",
            "The CI integration API provides functions for integrating the test harness with CI/CD pipelines.",
        ));

        documentation.add_section(api_section);

        // Add examples section
        let examples_section = DocumentationSection::new(
            "examples",
            "Examples",
            "This section provides examples of how to use the test harness for different testing scenarios.",
        )
        .with_subsection(DocumentationSection::new(
            "examples-unit-tests",
            "Unit Test Examples",
            "Examples of how to write unit tests using the test harness.",
        ))
        .with_subsection(DocumentationSection::new(
            "examples-integration-tests",
            "Integration Test Examples",
            "Examples of how to write integration tests using the test harness.",
        ))
        .with_subsection(DocumentationSection::new(
            "examples-end-to-end-tests",
            "End-to-End Test Examples",
            "Examples of how to write end-to-end tests using the test harness.",
        ))
        .with_subsection(DocumentationSection::new(
            "examples-performance-tests",
            "Performance Test Examples",
            "Examples of how to write performance tests using the test harness.",
        ))
        .with_subsection(DocumentationSection::new(
            "examples-security-tests",
            "Security Test Examples",
            "Examples of how to write security tests using the test harness.",
        ));

        documentation.add_section(examples_section);

        // Generate documentation files
        self.generate_files(&documentation).await?;

        Ok(documentation)
    }

    /// Generate documentation files
    async fn generate_files(&self, documentation: &Documentation) -> Result<(), TestHarnessError> {
        // Create output directory
        fs::create_dir_all(&self.config.output_dir)
            .await
            .map_err(|e| {
                TestHarnessError::IoError(format!("Failed to create output directory: {}", e))
            })?;

        // Generate files for each format
        for format in &self.config.formats {
            match format {
                DocumentationFormat::Markdown => {
                    let markdown = documentation.to_markdown();
                    let path = self
                        .config
                        .output_dir
                        .join(format!("index.{}", format.extension()));

                    fs::write(&path, markdown).await.map_err(|e| {
                        TestHarnessError::IoError(format!("Failed to write markdown file: {}", e))
                    })?;

                    info!("Generated markdown documentation: {:?}", path);
                }
                DocumentationFormat::Html => {
                    let html = documentation.to_html();
                    let path = self
                        .config
                        .output_dir
                        .join(format!("index.{}", format.extension()));

                    fs::write(&path, html).await.map_err(|e| {
                        TestHarnessError::IoError(format!("Failed to write HTML file: {}", e))
                    })?;

                    info!("Generated HTML documentation: {:?}", path);
                }
                _ => {
                    warn!("Documentation format not implemented: {}", format);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_section() {
        let section =
            DocumentationSection::new("test-section", "Test Section", "This is a test section.")
                .with_subsection(DocumentationSection::new(
                    "test-subsection",
                    "Test Subsection",
                    "This is a test subsection.",
                ))
                .with_metadata("key", "value");

        assert_eq!(section.id, "test-section");
        assert_eq!(section.title, "Test Section");
        assert_eq!(section.content, "This is a test section.");
        assert_eq!(section.subsections.len(), 1);
        assert_eq!(section.subsections[0].id, "test-subsection");
        assert_eq!(section.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_documentation_to_markdown() {
        let config = DocumentationConfig {
            title: "Test Documentation".to_string(),
            description: Some("This is a test documentation.".to_string()),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            output_dir: PathBuf::from("docs"),
            formats: vec![DocumentationFormat::Markdown],
            template_dir: None,
            assets_dir: None,
            metadata: HashMap::new(),
        };

        let section =
            DocumentationSection::new("test-section", "Test Section", "This is a test section.")
                .with_subsection(DocumentationSection::new(
                    "test-subsection",
                    "Test Subsection",
                    "This is a test subsection.",
                ));

        let documentation = Documentation::new(config).with_section(section);

        let markdown = documentation.to_markdown();

        assert!(markdown.contains("# Test Documentation"));
        assert!(markdown.contains("This is a test documentation."));
        assert!(markdown.contains("## Test Section"));
        assert!(markdown.contains("This is a test section."));
        assert!(markdown.contains("### Test Subsection"));
        assert!(markdown.contains("This is a test subsection."));
    }
}
