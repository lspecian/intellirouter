//! Reporting and Continuous Improvement
//!
//! This module provides functionality for generating reports and facilitating
//! continuous improvement of the testing strategy.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::modules::orchestrator::continuous_improvement::OrchestratorReporting;
use crate::modules::orchestrator::types::{
    OrchestratorError, ReportingError, TaskStatus,
};

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Plain text
    PlainText,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
}

/// Report configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Report format
    pub format: ReportFormat,
    /// Include task details
    pub include_task_details: bool,
    /// Include task results
    pub include_task_results: bool,
    /// Include task events
    pub include_task_events: bool,
    /// Include task durations
    pub include_task_durations: bool,
    /// Include workflow details
    pub include_workflow_details: bool,
    /// Include workflow results
    pub include_workflow_results: bool,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::Markdown,
            include_task_details: true,
            include_task_results: true,
            include_task_events: false,
            include_task_durations: true,
            include_workflow_details: true,
            include_workflow_results: true,
            options: HashMap::new(),
        }
    }
}

impl ReportConfig {
    /// Create a new report configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the report format
    pub fn with_format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// Set whether to include task details
    pub fn with_task_details(mut self, include: bool) -> Self {
        self.include_task_details = include;
        self
    }

    /// Set whether to include task results
    pub fn with_task_results(mut self, include: bool) -> Self {
        self.include_task_results = include;
        self
    }

    /// Set whether to include task events
    pub fn with_task_events(mut self, include: bool) -> Self {
        self.include_task_events = include;
        self
    }

    /// Set whether to include task durations
    pub fn with_task_durations(mut self, include: bool) -> Self {
        self.include_task_durations = include;
        self
    }

    /// Set whether to include workflow details
    pub fn with_workflow_details(mut self, include: bool) -> Self {
        self.include_workflow_details = include;
        self
    }

    /// Set whether to include workflow results
    pub fn with_workflow_results(mut self, include: bool) -> Self {
        self.include_workflow_results = include;
        self
    }

    /// Add an option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Report generator for generating reports
pub struct ReportGenerator {
    /// Report configurations
    configs: Mutex<HashMap<String, ReportConfig>>,
    /// Report cache
    cache: Mutex<HashMap<String, (String, Instant)>>,
    /// Cache TTL (in seconds)
    cache_ttl: u64,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new() -> Self {
        Self {
            configs: Mutex::new(HashMap::new()),
            cache: Mutex::new(HashMap::new()),
            cache_ttl: 60, // 1 minute
        }
    }

    /// Register a report configuration
    pub fn register_config(
        &self,
        report_id: impl Into<String>,
        config: ReportConfig,
    ) -> Result<(), OrchestratorError> {
        let mut configs = self.configs.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on configs".to_string())
        })?;

        configs.insert(report_id.into(), config);
        Ok(())
    }

    /// Get a report configuration
    pub fn get_config(&self, report_id: &str) -> Result<Option<ReportConfig>, OrchestratorError> {
        let configs = self.configs.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on configs".to_string())
        })?;

        Ok(configs.get(report_id).cloned())
    }

    /// Set the cache TTL
    pub fn set_cache_ttl(&mut self, ttl: u64) {
        self.cache_ttl = ttl;
    }

    /// Clear the cache
    pub fn clear_cache(&self) -> Result<(), OrchestratorError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| OrchestratorError::Other("Failed to acquire lock on cache".to_string()))?;

        cache.clear();
        Ok(())
    }

    /// Add an improvement suggestion to the report
    pub fn add_improvement_suggestion(
        &self,
        suggestion: ImprovementSuggestion,
    ) -> Result<(), OrchestratorError> {
        // Implementation would store the suggestion in the report
        // For now, we'll just log it
        tracing::info!("Adding improvement suggestion: {}", suggestion.title);
        Ok(())
    }

    /// Add a metric to the report
    pub fn add_metric(&self, key: &str, value: f64) -> Result<(), OrchestratorError> {
        // Implementation would store the metric in the report
        // For now, we'll just log it
        tracing::info!("Adding metric: {} = {}", key, value);
        Ok(())
    }

    /// Add a finding to the report
    pub fn add_finding(
        &self,
        id: &str,
        title: &str,
        description: &str,
        severity: String,
    ) -> Result<(), OrchestratorError> {
        // Implementation would store the finding in the report
        // For now, we'll just log it
        tracing::info!("Adding finding: {} - {} ({})", id, title, severity);
        Ok(())
    }

    /// Generate a report
    pub fn generate_report(
        &self,
        report_id: &str,
        orchestrator: &dyn OrchestratorReporting,
    ) -> Result<String, OrchestratorError> {
        // Check the cache
        {
            let cache = self.cache.lock().map_err(|_| {
                OrchestratorError::Other("Failed to acquire lock on cache".to_string())
            })?;

            if let Some((report, timestamp)) = cache.get(report_id) {
                if timestamp.elapsed().as_secs() < self.cache_ttl {
                    return Ok(report.clone());
                }
            }
        }

        // Get the report configuration
        let config = self.get_config(report_id)?.ok_or_else(|| {
            OrchestratorError::Reporting(ReportingError::InvalidReportFormat(format!(
                "Report configuration not found: {}",
                report_id
            )))
        })?;

        // Generate the report
        let report = match config.format {
            ReportFormat::PlainText => {
                self.generate_plain_text_report(report_id, &config, orchestrator)?
            }
            ReportFormat::Markdown => {
                self.generate_markdown_report(report_id, &config, orchestrator)?
            }
            ReportFormat::Html => self.generate_html_report(report_id, &config, orchestrator)?,
            ReportFormat::Json => self.generate_json_report(report_id, &config, orchestrator)?,
        };

        // Cache the report
        {
            let mut cache = self.cache.lock().map_err(|_| {
                OrchestratorError::Other("Failed to acquire lock on cache".to_string())
            })?;

            cache.insert(report_id.to_string(), (report.clone(), Instant::now()));
        }

        Ok(report)
    }

    /// Generate a plain text report
    fn generate_plain_text_report(
        &self,
        report_id: &str,
        config: &ReportConfig,
        orchestrator: &dyn OrchestratorReporting,
    ) -> Result<String, OrchestratorError> {
        let mut report = String::new();

        // Add report header
        report.push_str(&format!("Report: {}\n", report_id));
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));

        // Add task details
        if config.include_task_details {
            report.push_str("Tasks:\n");

            let tasks = orchestrator.get_all_tasks()?;
            for task in tasks {
                report.push_str(&format!(
                    "- {} ({}): {}\n",
                    task.id, task.status, task.title
                ));

                if config.include_task_results {
                    if let Some(result) = &task.result {
                        report.push_str(&format!("  Result: {}\n", result.message));
                    }
                }
            }

            report.push('\n');
        }

        // Add workflow details
        if config.include_workflow_details {
            // This would require access to the workflow manager
            // For now, we'll just add a placeholder
            report.push_str("Workflows: Not implemented in plain text format\n\n");
        }

        Ok(report)
    }

    /// Generate a markdown report
    fn generate_markdown_report(
        &self,
        report_id: &str,
        config: &ReportConfig,
        orchestrator: &dyn OrchestratorReporting,
    ) -> Result<String, OrchestratorError> {
        let mut report = String::new();

        // Add report header
        report.push_str(&format!("# Report: {}\n\n", report_id));
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));

        // Add task details
        if config.include_task_details {
            report.push_str("## Tasks\n\n");

            let tasks = orchestrator.get_all_tasks()?;

            // Add task summary
            let total_tasks = tasks.len();
            let completed_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .count();
            let failed_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Failed)
                .count();
            let pending_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Pending)
                .count();
            let in_progress_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::InProgress)
                .count();

            report.push_str(&format!("### Summary\n\n"));
            report.push_str(&format!("- Total tasks: {}\n", total_tasks));
            report.push_str(&format!("- Completed tasks: {}\n", completed_tasks));
            report.push_str(&format!("- Failed tasks: {}\n", failed_tasks));
            report.push_str(&format!("- Pending tasks: {}\n", pending_tasks));
            report.push_str(&format!("- In-progress tasks: {}\n\n", in_progress_tasks));

            // Add task details
            report.push_str("### Details\n\n");

            for task in tasks {
                report.push_str(&format!("#### Task: {} ({})\n\n", task.id, task.status));
                report.push_str(&format!("- Title: {}\n", task.title));
                report.push_str(&format!("- Description: {}\n", task.description));
                report.push_str(&format!("- Mode: {}\n", task.mode));

                if !task.dependencies.is_empty() {
                    report.push_str(&format!(
                        "- Dependencies: {}\n",
                        task.dependencies.join(", ")
                    ));
                }

                if config.include_task_results {
                    if let Some(result) = &task.result {
                        report.push_str(&format!("- Result: {}\n", result.message));

                        if !result.data.is_empty() {
                            report.push_str("- Data:\n");
                            for (key, value) in &result.data {
                                report.push_str(&format!("  - {}: {}\n", key, value));
                            }
                        }
                    }
                }

                report.push('\n');
            }
        }

        // Add workflow details
        if config.include_workflow_details {
            // This would require access to the workflow manager
            // For now, we'll just add a placeholder
            report.push_str("## Workflows\n\n");
            report.push_str("Workflow details not implemented in markdown format\n\n");
        }

        Ok(report)
    }

    /// Generate an HTML report
    fn generate_html_report(
        &self,
        report_id: &str,
        config: &ReportConfig,
        orchestrator: &dyn OrchestratorReporting,
    ) -> Result<String, OrchestratorError> {
        let mut report = String::new();

        // Add HTML header
        report.push_str("<!DOCTYPE html>\n");
        report.push_str("<html>\n");
        report.push_str("<head>\n");
        report.push_str(&format!("<title>Report: {}</title>\n", report_id));
        report.push_str("<style>\n");
        report.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
        report.push_str("h1 { color: #333; }\n");
        report.push_str("h2 { color: #666; }\n");
        report.push_str("h3 { color: #999; }\n");
        report.push_str("table { border-collapse: collapse; width: 100%; }\n");
        report.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        report.push_str("th { background-color: #f2f2f2; }\n");
        report.push_str("tr:nth-child(even) { background-color: #f9f9f9; }\n");
        report.push_str(".completed { color: green; }\n");
        report.push_str(".failed { color: red; }\n");
        report.push_str(".pending { color: orange; }\n");
        report.push_str(".in-progress { color: blue; }\n");
        report.push_str("</style>\n");
        report.push_str("</head>\n");
        report.push_str("<body>\n");

        // Add report header
        report.push_str(&format!("<h1>Report: {}</h1>\n", report_id));
        report.push_str(&format!("<p>Generated: {}</p>\n", chrono::Utc::now()));

        // Add task details
        if config.include_task_details {
            report.push_str("<h2>Tasks</h2>\n");

            let tasks = orchestrator.get_all_tasks()?;

            // Add task summary
            let total_tasks = tasks.len();
            let completed_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .count();
            let failed_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Failed)
                .count();
            let pending_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Pending)
                .count();
            let in_progress_tasks = tasks
                .iter()
                .filter(|t| t.status == TaskStatus::InProgress)
                .count();

            report.push_str("<h3>Summary</h3>\n");
            report.push_str("<ul>\n");
            report.push_str(&format!("<li>Total tasks: {}</li>\n", total_tasks));
            report.push_str(&format!("<li>Completed tasks: {}</li>\n", completed_tasks));
            report.push_str(&format!("<li>Failed tasks: {}</li>\n", failed_tasks));
            report.push_str(&format!("<li>Pending tasks: {}</li>\n", pending_tasks));
            report.push_str(&format!(
                "<li>In-progress tasks: {}</li>\n",
                in_progress_tasks
            ));
            report.push_str("</ul>\n");

            // Add task details
            report.push_str("<h3>Details</h3>\n");
            report.push_str("<table>\n");
            report.push_str(
                "<tr><th>ID</th><th>Title</th><th>Description</th><th>Mode</th><th>Status</th>",
            );

            if config.include_task_results {
                report.push_str("<th>Result</th>");
            }

            report.push_str("</tr>\n");

            for task in tasks {
                let status_class = match task.status {
                    TaskStatus::Completed => "completed",
                    TaskStatus::Failed => "failed",
                    TaskStatus::Pending => "pending",
                    TaskStatus::InProgress => "in-progress",
                    TaskStatus::Cancelled => "failed",
                };

                report.push_str("<tr>");
                report.push_str(&format!("<td>{}</td>", task.id));
                report.push_str(&format!("<td>{}</td>", task.title));
                report.push_str(&format!("<td>{}</td>", task.description));
                report.push_str(&format!("<td>{}</td>", task.mode));
                report.push_str(&format!(
                    "<td class=\"{}\">{}</td>",
                    status_class, task.status
                ));

                if config.include_task_results {
                    if let Some(result) = &task.result {
                        report.push_str(&format!("<td>{}</td>", result.message));
                    } else {
                        report.push_str("<td></td>");
                    }
                }

                report.push_str("</tr>\n");
            }

            report.push_str("</table>\n");
        }

        // Add workflow details
        if config.include_workflow_details {
            // This would require access to the workflow manager
            // For now, we'll just add a placeholder
            report.push_str("<h2>Workflows</h2>\n");
            report.push_str("<p>Workflow details not implemented in HTML format</p>\n");
        }

        // Add HTML footer
        report.push_str("</body>\n");
        report.push_str("</html>\n");

        Ok(report)
    }

    /// Generate a JSON report
    fn generate_json_report(
        &self,
        report_id: &str,
        config: &ReportConfig,
        orchestrator: &dyn OrchestratorReporting,
    ) -> Result<String, OrchestratorError> {
        use serde_json::{json, to_string_pretty};

        let tasks = orchestrator.get_all_tasks()?;

        let task_details = if config.include_task_details {
            let task_json = tasks
                .iter()
                .map(|task| {
                    let mut task_json = json!({
                        "id": task.id,
                        "title": task.title,
                        "description": task.description,
                        "mode": format!("{}", task.mode),
                        "status": format!("{}", task.status),
                        "dependencies": task.dependencies,
                    });

                    if config.include_task_results {
                        if let Some(result) = &task.result {
                            let result_json = json!({
                                "message": result.message,
                                "data": result.data,
                            });

                            if let serde_json::Value::Object(ref mut obj) = task_json {
                                obj.insert("result".to_string(), result_json);
                            }
                        }
                    }

                    task_json
                })
                .collect::<Vec<_>>();

            Some(task_json)
        } else {
            None
        };

        let report_json = json!({
            "report_id": report_id,
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "tasks": task_details,
            "workflows": config.include_workflow_details,
        });

        to_string_pretty(&report_json).map_err(|e| {
            OrchestratorError::Reporting(ReportingError::GenerationFailed(format!(
                "Failed to generate JSON report: {}",
                e
            )))
        })
    }
}

/// Continuous improvement for facilitating continuous improvement of the testing strategy
pub struct ContinuousImprovement {
    /// Improvement suggestions
    suggestions: Mutex<Vec<ImprovementSuggestion>>,
}

/// Improvement suggestion
#[derive(Debug, Clone)]
pub struct ImprovementSuggestion {
    /// Suggestion ID
    pub id: String,
    /// Suggestion title
    pub title: String,
    /// Suggestion description
    pub description: String,
    /// Affected tasks
    pub affected_tasks: Vec<String>,
    /// Affected workflows
    pub affected_workflows: Vec<String>,
    /// Suggestion priority
    pub priority: SuggestionPriority,
    /// Suggestion status
    pub status: SuggestionStatus,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated at
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Suggestion priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
}

/// Suggestion status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionStatus {
    /// Pending
    Pending,
    /// Implemented
    Implemented,
    /// Rejected
    Rejected,
}

impl ContinuousImprovement {
    /// Create a new continuous improvement
    pub fn new() -> Self {
        Self {
            suggestions: Mutex::new(Vec::new()),
        }
    }

    /// Add a suggestion
    pub fn add_suggestion(
        &self,
        suggestion: ImprovementSuggestion,
    ) -> Result<(), OrchestratorError> {
        let mut suggestions = self.suggestions.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on suggestions".to_string())
        })?;

        suggestions.push(suggestion);
        Ok(())
    }

    /// Get all suggestions
    pub fn get_all_suggestions(&self) -> Result<Vec<ImprovementSuggestion>, OrchestratorError> {
        let suggestions = self.suggestions.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on suggestions".to_string())
        })?;

        Ok(suggestions.clone())
    }

    /// Get suggestions by status
    pub fn get_suggestions_by_status(
        &self,
        status: SuggestionStatus,
    ) -> Result<Vec<ImprovementSuggestion>, OrchestratorError> {
        let suggestions = self.suggestions.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on suggestions".to_string())
        })?;

        Ok(suggestions
            .iter()
            .filter(|s| s.status == status)
            .cloned()
            .collect())
    }

    /// Update suggestion status
    pub fn update_suggestion_status(
        &self,
        suggestion_id: &str,
        status: SuggestionStatus,
    ) -> Result<(), OrchestratorError> {
        let mut suggestions = self.suggestions.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on suggestions".to_string())
        })?;

        for suggestion in &mut *suggestions {
            if suggestion.id == suggestion_id {
                suggestion.status = status;
                suggestion.updated_at = chrono::Utc::now();
                return Ok(());
            }
        }

        Err(OrchestratorError::Reporting(ReportingError::Other(
            format!("Suggestion not found: {}", suggestion_id),
        )))
    }

    /// Generate improvement suggestions
    pub fn generate_suggestions(
        &self,
        orchestrator: &dyn OrchestratorReporting,
    ) -> Result<(), OrchestratorError> {
        // This would analyze the tasks and workflows to generate improvement suggestions
        // For now, we'll just add a placeholder suggestion
        let suggestion = ImprovementSuggestion {
            id: "suggestion-1".to_string(),
            title: "Improve test coverage".to_string(),
            description: "Add more tests to improve coverage".to_string(),
            affected_tasks: Vec::new(),
            affected_workflows: Vec::new(),
            priority: SuggestionPriority::Medium,
            status: SuggestionStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.add_suggestion(suggestion)?;

        Ok(())
    }
}
