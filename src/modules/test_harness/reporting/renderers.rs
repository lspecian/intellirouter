//! Renderers module for rendering test reports
//!
//! This module provides functionality for rendering test reports in various formats.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::dashboard::Dashboard;
use crate::modules::test_harness::types::TestHarnessError;

/// Renderer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    /// Renderer name
    pub name: String,
    /// Renderer description
    pub description: Option<String>,
    /// Renderer options
    pub options: HashMap<String, String>,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            options: HashMap::new(),
        }
    }
}

/// Renderer trait
pub trait Renderer: Send + Sync {
    /// Get the renderer name
    fn name(&self) -> &str;

    /// Get the renderer description
    fn description(&self) -> Option<&str>;

    /// Get the renderer configuration
    fn config(&self) -> &RendererConfig;

    /// Set the renderer configuration
    fn set_config(&mut self, config: RendererConfig);

    /// Render a dashboard
    fn render(&self, dashboard: &Dashboard) -> Result<String, TestHarnessError>;
}

/// Console renderer
pub struct ConsoleRenderer {
    /// Renderer configuration
    config: RendererConfig,
}

impl ConsoleRenderer {
    /// Create a new console renderer
    pub fn new() -> Self {
        Self {
            config: RendererConfig {
                name: "console".to_string(),
                description: Some("Console renderer".to_string()),
                options: HashMap::new(),
            },
        }
    }
}

impl Default for ConsoleRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for ConsoleRenderer {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
    }

    fn render(&self, dashboard: &Dashboard) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        output.push_str(&format!("# {}\n\n", dashboard.config.title));

        if let Some(description) = &dashboard.config.description {
            output.push_str(&format!("{}\n\n", description));
        }

        output.push_str(&format!("Test Runs: {}\n\n", dashboard.test_runs.len()));

        for (i, run) in dashboard.test_runs.iter().enumerate() {
            output.push_str(&format!("## Run {}: {}\n\n", i + 1, run.name));

            if let Some(description) = &run.description {
                output.push_str(&format!("{}\n\n", description));
            }

            output.push_str(&format!("- Start Time: {}\n", run.start_time));
            output.push_str(&format!("- End Time: {}\n", run.end_time));
            output.push_str(&format!("- Duration: {:?}\n", run.duration));
            output.push_str(&format!("- Tests: {}\n", run.results.len()));
            output.push_str(&format!("- Passed: {}\n", run.passed_count));
            output.push_str(&format!("- Failed: {}\n", run.failed_count));
            output.push_str(&format!("- Skipped: {}\n", run.skipped_count));
            output.push_str(&format!("- Pass Rate: {:.2}%\n\n", run.pass_rate() * 100.0));
        }

        Ok(output)
    }
}

/// HTML renderer
pub struct HtmlRenderer {
    /// Renderer configuration
    config: RendererConfig,
}

impl HtmlRenderer {
    /// Create a new HTML renderer
    pub fn new() -> Self {
        Self {
            config: RendererConfig {
                name: "html".to_string(),
                description: Some("HTML renderer".to_string()),
                options: HashMap::new(),
            },
        }
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for HtmlRenderer {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
    }

    fn render(&self, dashboard: &Dashboard) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html>\n");
        output.push_str("<head>\n");
        output.push_str("  <meta charset=\"UTF-8\">\n");
        output.push_str(&format!("  <title>{}</title>\n", dashboard.config.title));
        output.push_str("  <style>\n");
        output.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
        output.push_str("    h1 { color: #333; }\n");
        output.push_str("    h2 { color: #666; }\n");
        output
            .push_str("    .run { margin-bottom: 20px; padding: 10px; border: 1px solid #ddd; }\n");
        output.push_str("    .stats { display: flex; flex-wrap: wrap; }\n");
        output.push_str("    .stat { margin-right: 20px; }\n");
        output.push_str("    .passed { color: green; }\n");
        output.push_str("    .failed { color: red; }\n");
        output.push_str("    .skipped { color: orange; }\n");
        output.push_str("  </style>\n");
        output.push_str("</head>\n");
        output.push_str("<body>\n");

        output.push_str(&format!("  <h1>{}</h1>\n", dashboard.config.title));

        if let Some(description) = &dashboard.config.description {
            output.push_str(&format!("  <p>{}</p>\n", description));
        }

        output.push_str(&format!(
            "  <p>Test Runs: {}</p>\n",
            dashboard.test_runs.len()
        ));

        for (i, run) in dashboard.test_runs.iter().enumerate() {
            output.push_str("  <div class=\"run\">\n");
            output.push_str(&format!("    <h2>Run {}: {}</h2>\n", i + 1, run.name));

            if let Some(description) = &run.description {
                output.push_str(&format!("    <p>{}</p>\n", description));
            }

            output.push_str("    <div class=\"stats\">\n");
            output.push_str(&format!(
                "      <div class=\"stat\">Start Time: {}</div>\n",
                run.start_time
            ));
            output.push_str(&format!(
                "      <div class=\"stat\">End Time: {}</div>\n",
                run.end_time
            ));
            output.push_str(&format!(
                "      <div class=\"stat\">Duration: {:?}</div>\n",
                run.duration
            ));
            output.push_str(&format!(
                "      <div class=\"stat\">Tests: {}</div>\n",
                run.results.len()
            ));
            output.push_str(&format!(
                "      <div class=\"stat passed\">Passed: {}</div>\n",
                run.passed_count
            ));
            output.push_str(&format!(
                "      <div class=\"stat failed\">Failed: {}</div>\n",
                run.failed_count
            ));
            output.push_str(&format!(
                "      <div class=\"stat skipped\">Skipped: {}</div>\n",
                run.skipped_count
            ));
            output.push_str(&format!(
                "      <div class=\"stat\">Pass Rate: {:.2}%</div>\n",
                run.pass_rate() * 100.0
            ));
            output.push_str("    </div>\n");
            output.push_str("  </div>\n");
        }

        output.push_str("</body>\n");
        output.push_str("</html>\n");

        Ok(output)
    }
}

/// Chart renderer
pub struct ChartRenderer {
    /// Renderer configuration
    config: RendererConfig,
}

impl ChartRenderer {
    /// Create a new chart renderer
    pub fn new() -> Self {
        Self {
            config: RendererConfig {
                name: "chart".to_string(),
                description: Some("Chart renderer".to_string()),
                options: HashMap::new(),
            },
        }
    }
}

impl Default for ChartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for ChartRenderer {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
    }

    fn render(&self, dashboard: &Dashboard) -> Result<String, TestHarnessError> {
        // This is a simplified implementation that generates a chart as SVG
        let mut output = String::new();

        output
            .push_str("<svg width=\"500\" height=\"300\" xmlns=\"http://www.w3.org/2000/svg\">\n");
        output.push_str("  <style>\n");
        output.push_str("    .passed { fill: green; }\n");
        output.push_str("    .failed { fill: red; }\n");
        output.push_str("    .skipped { fill: orange; }\n");
        output.push_str("    text { font-family: Arial, sans-serif; font-size: 12px; }\n");
        output.push_str("  </style>\n");

        output.push_str("  <text x=\"10\" y=\"20\" font-size=\"16\">Test Results</text>\n");

        let mut y = 50;
        for (i, run) in dashboard.test_runs.iter().enumerate() {
            let passed = run.passed_count;
            let failed = run.failed_count;
            let skipped = run.skipped_count;
            let total = run.results.len() as f64;

            if total > 0.0 {
                let passed_width = (passed as f64 / total) * 400.0;
                let failed_width = (failed as f64 / total) * 400.0;
                let skipped_width = (skipped as f64 / total) * 400.0;

                output.push_str(&format!(
                    "  <text x=\"10\" y=\"{}\">{}</text>\n",
                    y - 5,
                    run.name
                ));
                output.push_str(&format!(
                    "  <rect x=\"100\" y=\"{}\" width=\"{}\" height=\"20\" class=\"passed\" />\n",
                    y - 15,
                    passed_width
                ));
                output.push_str(&format!(
                    "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"20\" class=\"failed\" />\n",
                    100.0 + passed_width,
                    y - 15,
                    failed_width
                ));
                output.push_str(&format!(
                    "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"20\" class=\"skipped\" />\n",
                    100.0 + passed_width + failed_width,
                    y - 15,
                    skipped_width
                ));
                output.push_str(&format!(
                    "  <text x=\"510\" y=\"{}\">{} / {} / {}</text>\n",
                    y - 5,
                    passed,
                    failed,
                    skipped
                ));
            }

            y += 30;
        }

        output.push_str("</svg>\n");

        Ok(output)
    }
}

/// Table renderer
pub struct TableRenderer {
    /// Renderer configuration
    config: RendererConfig,
}

impl TableRenderer {
    /// Create a new table renderer
    pub fn new() -> Self {
        Self {
            config: RendererConfig {
                name: "table".to_string(),
                description: Some("Table renderer".to_string()),
                options: HashMap::new(),
            },
        }
    }
}

impl Default for TableRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TableRenderer {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
    }

    fn render(&self, dashboard: &Dashboard) -> Result<String, TestHarnessError> {
        let mut output = String::new();

        output.push_str("<table border=\"1\" cellpadding=\"5\" cellspacing=\"0\">\n");
        output.push_str("  <tr>\n");
        output.push_str("    <th>Run</th>\n");
        output.push_str("    <th>Start Time</th>\n");
        output.push_str("    <th>End Time</th>\n");
        output.push_str("    <th>Duration</th>\n");
        output.push_str("    <th>Tests</th>\n");
        output.push_str("    <th>Passed</th>\n");
        output.push_str("    <th>Failed</th>\n");
        output.push_str("    <th>Skipped</th>\n");
        output.push_str("    <th>Pass Rate</th>\n");
        output.push_str("  </tr>\n");

        for run in &dashboard.test_runs {
            output.push_str("  <tr>\n");
            output.push_str(&format!("    <td>{}</td>\n", run.name));
            output.push_str(&format!("    <td>{}</td>\n", run.start_time));
            output.push_str(&format!("    <td>{}</td>\n", run.end_time));
            output.push_str(&format!("    <td>{:?}</td>\n", run.duration));
            output.push_str(&format!("    <td>{}</td>\n", run.results.len()));
            output.push_str(&format!("    <td>{}</td>\n", run.passed_count));
            output.push_str(&format!("    <td>{}</td>\n", run.failed_count));
            output.push_str(&format!("    <td>{}</td>\n", run.skipped_count));
            output.push_str(&format!("    <td>{:.2}%</td>\n", run.pass_rate() * 100.0));
            output.push_str("  </tr>\n");
        }

        output.push_str("</table>\n");

        Ok(output)
    }
}

/// Multi renderer
pub struct MultiRenderer {
    /// Renderer configuration
    config: RendererConfig,
    /// Renderers
    renderers: Vec<Box<dyn Renderer>>,
}

impl MultiRenderer {
    /// Create a new multi renderer
    pub fn new(renderers: Vec<Box<dyn Renderer>>) -> Self {
        Self {
            config: RendererConfig {
                name: "multi".to_string(),
                description: Some("Multi renderer".to_string()),
                options: HashMap::new(),
            },
            renderers,
        }
    }

    /// Add a renderer
    pub fn add_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderers.push(renderer);
    }
}

impl Renderer for MultiRenderer {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
    }

    fn render(&self, dashboard: &Dashboard) -> Result<String, TestHarnessError> {
        // Get the renderer name from the options
        let renderer_name = self
            .config
            .options
            .get("renderer")
            .unwrap_or(&"html".to_string());

        // Find the renderer with the matching name
        for renderer in &self.renderers {
            if renderer.name() == renderer_name {
                return renderer.render(dashboard);
            }
        }

        // If no renderer was found, use the first one
        if let Some(renderer) = self.renderers.first() {
            return renderer.render(dashboard);
        }

        Err(TestHarnessError::ReportingError(format!(
            "No renderer found with name: {}",
            renderer_name
        )))
    }
}
