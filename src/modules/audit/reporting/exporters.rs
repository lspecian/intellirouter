//! Report Exporters
//!
//! This module provides functionality to export reports in various formats.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};

#[cfg(feature = "pdf-export")]
use wkhtmltopdf::{pdf::*, PdfApplication};

use crate::modules::audit::report::AuditReport;
use crate::modules::audit::types::AuditError;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
    /// PDF format
    Pdf,
}

/// Report exporter
#[derive(Debug)]
pub struct ReportExporter {
    /// Handlebars template registry
    handlebars: Handlebars<'static>,
}

impl ReportExporter {
    /// Create a new report exporter
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // Register HTML template
        handlebars
            .register_template_string("html_report", include_str!("../templates/report.html"))
            .expect("Failed to register HTML template");

        // Register Markdown template
        handlebars
            .register_template_string("markdown_report", include_str!("../templates/report.md"))
            .expect("Failed to register Markdown template");

        Self { handlebars }
    }

    /// Export report to a file
    pub async fn export(
        &self,
        report: &AuditReport,
        path: &str,
        format: ExportFormat,
    ) -> Result<(), AuditError> {
        info!("Exporting report in {:?} format to {}", format, path);

        let path = Path::new(path);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to create directories: {}", e))
            })?;
        }

        // Generate the report content
        let content = match format {
            ExportFormat::Json => self.to_json(report)?,
            ExportFormat::Markdown => self.to_markdown(report)?,
            ExportFormat::Html => self.to_html(report)?,
            ExportFormat::Pdf => {
                #[cfg(feature = "pdf-export")]
                {
                    // For PDF, we first generate HTML, then convert it to PDF
                    let html_content = self.to_html(report)?;
                    self.html_to_pdf(&html_content, path)?;
                    return Ok(());
                }

                #[cfg(not(feature = "pdf-export"))]
                {
                    error!("PDF export is not available. Compile with the 'pdf-export' feature to enable it.");
                    return Err(AuditError::ReportGenerationError(
                        "PDF export is not available. Compile with the 'pdf-export' feature to enable it.".to_string()
                    ));
                }
            }
        };

        // Write the content to the file
        let mut file = File::create(path).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to create file: {}", e))
        })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to write to file: {}", e))
        })?;

        info!("Report exported successfully to {}", path.display());

        Ok(())
    }

    /// Convert report to JSON
    fn to_json(&self, report: &AuditReport) -> Result<String, AuditError> {
        serde_json::to_string_pretty(report).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to serialize report to JSON: {}", e))
        })
    }

    /// Convert report to Markdown
    fn to_markdown(&self, report: &AuditReport) -> Result<String, AuditError> {
        // Use the handlebars template
        let data = json!({
            "report": report,
            "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });

        self.handlebars
            .render("markdown_report", &data)
            .map_err(|e| {
                AuditError::ReportGenerationError(format!(
                    "Failed to render Markdown template: {}",
                    e
                ))
            })
    }

    /// Convert report to HTML
    fn to_html(&self, report: &AuditReport) -> Result<String, AuditError> {
        // Use the handlebars template
        let data = json!({
            "report": report,
            "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });

        self.handlebars.render("html_report", &data).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to render HTML template: {}", e))
        })
    }

    /// Convert HTML to PDF
    #[cfg(feature = "pdf-export")]
    fn html_to_pdf(&self, html_content: &str, output_path: &Path) -> Result<(), AuditError> {
        // Initialize PDF application
        let pdf_app = match PdfApplication::new() {
            Ok(app) => app,
            Err(e) => {
                return Err(AuditError::ReportGenerationError(format!(
                    "Failed to initialize PDF application: {}",
                    e
                )));
            }
        };

        // Create PDF builder
        let mut builder = pdf_app.builder();

        // Set options
        builder
            .orientation(Orientation::Portrait)
            .margin(Size::Millimeters(10))
            .title("IntelliRouter Audit Report");

        // Generate PDF from HTML
        let mut pdf = builder.build_from_html(html_content).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to generate PDF: {}", e))
        })?;

        // Save PDF to file
        pdf.save(output_path)
            .map_err(|e| AuditError::ReportGenerationError(format!("Failed to save PDF: {}", e)))?;

        Ok(())
    }
}
