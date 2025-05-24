//! Metrics collection and processing
//!
//! This module defines the metrics structures and functions for collecting and processing metrics.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::data::{MetricDataPoint, MetricSeries, Recommendation, Status, StatusLevel};

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    /// Total warnings
    pub total_warnings: u32,
    /// Warning density (warnings per 1000 lines of code)
    pub warning_density: f64,
    /// Test coverage percentage
    pub test_coverage: f64,
    /// Documentation coverage percentage
    pub doc_coverage: f64,
    /// Lines of code
    pub lines_of_code: u32,
    /// Warning trends
    pub warning_trends: MetricSeries,
    /// Test coverage trends
    pub test_coverage_trends: MetricSeries,
    /// Documentation coverage trends
    pub doc_coverage_trends: MetricSeries,
    /// Status
    pub status: Status,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl Default for CodeQualityMetrics {
    fn default() -> Self {
        Self {
            total_warnings: 0,
            warning_density: 0.0,
            test_coverage: 0.0,
            doc_coverage: 0.0,
            lines_of_code: 0,
            warning_trends: MetricSeries {
                name: "Warning Trends".to_string(),
                description: Some("Trend of warnings over time".to_string()),
                unit: Some("count".to_string()),
                data_points: Vec::new(),
                metadata: HashMap::new(),
            },
            test_coverage_trends: MetricSeries {
                name: "Test Coverage Trends".to_string(),
                description: Some("Trend of test coverage over time".to_string()),
                unit: Some("%".to_string()),
                data_points: Vec::new(),
                metadata: HashMap::new(),
            },
            doc_coverage_trends: MetricSeries {
                name: "Documentation Coverage Trends".to_string(),
                description: Some("Trend of documentation coverage over time".to_string()),
                unit: Some("%".to_string()),
                data_points: Vec::new(),
                metadata: HashMap::new(),
            },
            status: Status {
                level: StatusLevel::Info,
                message: "No data available".to_string(),
                details: None,
                timestamp: Utc::now(),
            },
            recommendations: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Benchmark results
    pub benchmark_results: HashMap<String, BenchmarkResult>,
    /// Performance trends
    pub performance_trends: HashMap<String, MetricSeries>,
    /// Regressions
    pub regressions: Vec<PerformanceRegression>,
    /// Status
    pub status: Status,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            benchmark_results: HashMap::new(),
            performance_trends: HashMap::new(),
            regressions: Vec::new(),
            status: Status {
                level: StatusLevel::Info,
                message: "No data available".to_string(),
                details: None,
                timestamp: Utc::now(),
            },
            recommendations: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name
    pub name: String,
    /// Component
    pub component: String,
    /// Median (ms)
    pub median_ms: f64,
    /// Mean (ms)
    pub mean_ms: f64,
    /// Standard deviation (ms)
    pub std_dev_ms: f64,
    /// Minimum (ms)
    pub min_ms: f64,
    /// Maximum (ms)
    pub max_ms: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Component
    pub component: String,
    /// Benchmark
    pub benchmark: String,
    /// Previous value (ms)
    pub previous_ms: f64,
    /// Current value (ms)
    pub current_ms: f64,
    /// Change percentage
    pub change_percentage: f64,
    /// Status
    pub status: Status,
}

/// Security metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Total issues
    pub total_issues: u32,
    /// Critical issues
    pub critical_issues: u32,
    /// High issues
    pub high_issues: u32,
    /// Medium issues
    pub medium_issues: u32,
    /// Low issues
    pub low_issues: u32,
    /// Issue trends
    pub issue_trends: MetricSeries,
    /// Vulnerabilities
    pub vulnerabilities: Vec<Vulnerability>,
    /// Status
    pub status: Status,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            total_issues: 0,
            critical_issues: 0,
            high_issues: 0,
            medium_issues: 0,
            low_issues: 0,
            issue_trends: MetricSeries {
                name: "Security Issue Trends".to_string(),
                description: Some("Trend of security issues over time".to_string()),
                unit: Some("count".to_string()),
                data_points: Vec::new(),
                metadata: HashMap::new(),
            },
            vulnerabilities: Vec::new(),
            status: Status {
                level: StatusLevel::Info,
                message: "No data available".to_string(),
                details: None,
                timestamp: Utc::now(),
            },
            recommendations: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    /// Critical
    Critical,
    /// High
    High,
    /// Medium
    Medium,
    /// Low
    Low,
}

/// Vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Severity
    pub severity: VulnerabilitySeverity,
    /// Location
    pub location: Option<String>,
    /// Line
    pub line: Option<u32>,
    /// CVE ID
    pub cve_id: Option<String>,
    /// CVSS score
    pub cvss_score: Option<f64>,
    /// CWE ID
    pub cwe_id: Option<String>,
    /// Remediation
    pub remediation: Option<String>,
}

/// Documentation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationMetrics {
    /// Overall coverage percentage
    pub overall_coverage: f64,
    /// API documentation coverage percentage
    pub api_coverage: f64,
    /// User guides coverage percentage
    pub user_guides_coverage: f64,
    /// Architecture documentation coverage percentage
    pub architecture_coverage: f64,
    /// Examples coverage percentage
    pub examples_coverage: f64,
    /// Coverage trends
    pub coverage_trends: HashMap<String, MetricSeries>,
    /// Status
    pub status: Status,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Project health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealthMetrics {
    /// Overall health score (0-100)
    pub overall_health: f64,
    /// Code quality score (0-100)
    pub code_quality_score: f64,
    /// Performance score (0-100)
    pub performance_score: f64,
    /// Security score (0-100)
    pub security_score: f64,
    /// Documentation score (0-100)
    pub documentation_score: f64,
    /// Health trends
    pub health_trends: MetricSeries,
    /// Status
    pub status: Status,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Collect code quality metrics
pub async fn collect_code_quality_metrics(data_dir: &Path) -> CodeQualityMetrics {
    // In a real implementation, this would parse the metrics files
    // For now, we'll return a placeholder with some sample data
    let mut metrics = CodeQualityMetrics::default();

    // Add sample data
    metrics.total_warnings = 42;
    metrics.warning_density = 3.5;
    metrics.test_coverage = 78.5;
    metrics.doc_coverage = 65.2;
    metrics.lines_of_code = 12000;

    // Add sample data points for trends
    for i in 0..10 {
        let timestamp = Utc::now() - chrono::Duration::days(i);

        metrics.warning_trends.data_points.push(MetricDataPoint {
            timestamp,
            value: 42.0 * (1.0 + (i as f64 * 0.05)),
            label: None,
        });

        metrics
            .test_coverage_trends
            .data_points
            .push(MetricDataPoint {
                timestamp,
                value: 78.5 * (1.0 - (i as f64 * 0.01)),
                label: None,
            });

        metrics
            .doc_coverage_trends
            .data_points
            .push(MetricDataPoint {
                timestamp,
                value: 65.2 * (1.0 - (i as f64 * 0.01)),
                label: None,
            });
    }

    // Reverse data points to have chronological order
    metrics.warning_trends.data_points.reverse();
    metrics.test_coverage_trends.data_points.reverse();
    metrics.doc_coverage_trends.data_points.reverse();

    metrics.status = Status {
        level: StatusLevel::Info,
        message: "Code quality metrics collected".to_string(),
        details: Some(format!(
            "{} warnings, {:.1}% test coverage",
            metrics.total_warnings, metrics.test_coverage
        )),
        timestamp: Utc::now(),
    };

    metrics.last_updated = Utc::now();

    metrics
}

/// Collect performance metrics
pub async fn collect_performance_metrics(data_dir: &Path) -> PerformanceMetrics {
    // In a real implementation, this would parse the metrics files
    // For now, we'll return a placeholder with some sample data
    let mut metrics = PerformanceMetrics::default();

    // Add sample benchmark results
    let components = [
        "router",
        "model_registry",
        "chain_engine",
        "memory",
        "rag_manager",
    ];
    let benchmarks = ["throughput", "latency", "memory_usage"];

    for component in components.iter() {
        for benchmark in benchmarks.iter() {
            let key = format!("{}/{}", component, benchmark);
            let result = BenchmarkResult {
                name: benchmark.to_string(),
                component: component.to_string(),
                median_ms: 10.0 + (rand::random::<f64>() * 20.0),
                mean_ms: 12.0 + (rand::random::<f64>() * 20.0),
                std_dev_ms: 1.0 + (rand::random::<f64>() * 2.0),
                min_ms: 8.0 + (rand::random::<f64>() * 10.0),
                max_ms: 15.0 + (rand::random::<f64>() * 30.0),
                timestamp: Utc::now(),
            };

            metrics.benchmark_results.insert(key.clone(), result);

            // Add sample performance trends
            let mut data_points = Vec::new();
            for i in 0..10 {
                let timestamp = Utc::now() - chrono::Duration::days(i);
                data_points.push(MetricDataPoint {
                    timestamp,
                    value: 12.0 * (1.0 + (i as f64 * 0.02) * (if i % 2 == 0 { 1.0 } else { -1.0 })),
                    label: None,
                });
            }
            data_points.reverse();

            let series = MetricSeries {
                name: key.clone(),
                description: Some(format!("Performance trend for {}", key)),
                unit: Some("ms".to_string()),
                data_points,
                metadata: HashMap::new(),
            };

            metrics.performance_trends.insert(key, series);
        }
    }

    // Add sample regression
    metrics.regressions.push(PerformanceRegression {
        component: "chain_engine".to_string(),
        benchmark: "latency".to_string(),
        previous_ms: 15.2,
        current_ms: 18.5,
        change_percentage: 21.7,
        status: Status {
            level: StatusLevel::Warning,
            message: "Performance regression detected".to_string(),
            details: None,
            timestamp: Utc::now(),
        },
    });

    metrics.status = Status {
        level: StatusLevel::Warning,
        message: "Performance metrics collected".to_string(),
        details: Some(format!(
            "{} benchmarks, {} regressions",
            metrics.benchmark_results.len(),
            metrics.regressions.len()
        )),
        timestamp: Utc::now(),
    };

    metrics.last_updated = Utc::now();

    metrics
}

/// Collect security metrics
pub async fn collect_security_metrics(data_dir: &Path) -> SecurityMetrics {
    // In a real implementation, this would parse the metrics files
    // For now, we'll return a placeholder with some sample data
    let mut metrics = SecurityMetrics::default();

    // Add sample data
    metrics.total_issues = 5;
    metrics.critical_issues = 1;
    metrics.high_issues = 2;
    metrics.medium_issues = 1;
    metrics.low_issues = 1;

    // Add sample vulnerabilities
    metrics.vulnerabilities.push(Vulnerability {
        id: "SEC-001".to_string(),
        name: "Hardcoded API Key".to_string(),
        description: "API key hardcoded in source code".to_string(),
        severity: VulnerabilitySeverity::Critical,
        location: Some("src/config.rs".to_string()),
        line: Some(123),
        cve_id: None,
        cvss_score: None,
        cwe_id: Some("CWE-798".to_string()),
        remediation: Some("Move to environment variables or secure storage".to_string()),
    });

    // Add sample data points for trends
    for i in 0..10 {
        let timestamp = Utc::now() - chrono::Duration::days(i);

        metrics.issue_trends.data_points.push(MetricDataPoint {
            timestamp,
            value: 5.0 + (i as f64 * 0.5),
            label: None,
        });
    }

    // Reverse data points to have chronological order
    metrics.issue_trends.data_points.reverse();

    metrics.status = Status {
        level: StatusLevel::Warning,
        message: "Security metrics collected".to_string(),
        details: Some(format!(
            "{} total issues, {} critical",
            metrics.total_issues, metrics.critical_issues
        )),
        timestamp: Utc::now(),
    };

    metrics.last_updated = Utc::now();

    metrics
}

/// Collect documentation metrics
pub async fn collect_documentation_metrics(data_dir: &Path) -> DocumentationMetrics {
    // In a real implementation, this would parse the metrics files
    // For now, we'll return a placeholder with some sample data
    let mut metrics = DocumentationMetrics::default();

    // Add sample data
    metrics.overall_coverage = 72.5;
    metrics.api_coverage = 85.2;
    metrics.user_guides_coverage = 65.8;
    metrics.architecture_coverage = 78.3;
    metrics.examples_coverage = 60.7;

    // Add sample trends
    let categories = ["overall", "api", "user_guides", "architecture", "examples"];
    let base_values = [72.5, 85.2, 65.8, 78.3, 60.7];

    for (i, category) in categories.iter().enumerate() {
        let mut data_points = Vec::new();

        for j in 0..10 {
            let timestamp = Utc::now() - chrono::Duration::days(j);

            data_points.push(MetricDataPoint {
                timestamp,
                value: base_values[i] * (1.0 - (j as f64 * 0.01)),
                label: None,
            });
        }

        // Reverse data points to have chronological order
        data_points.reverse();

        let series = MetricSeries {
            name: category.to_string(),
            description: Some(format!("{} documentation coverage trend", category)),
            unit: Some("%".to_string()),
            data_points,
            metadata: HashMap::new(),
        };

        metrics.coverage_trends.insert(category.to_string(), series);
    }

    metrics.status = Status {
        level: StatusLevel::Info,
        message: "Documentation metrics collected".to_string(),
        details: Some(format!(
            "Overall coverage: {:.1}%",
            metrics.overall_coverage
        )),
        timestamp: Utc::now(),
    };

    metrics.last_updated = Utc::now();

    metrics
}

/// Calculate project health metrics
pub fn calculate_project_health(
    code_quality: &CodeQualityMetrics,
    performance: &PerformanceMetrics,
    security: &SecurityMetrics,
    documentation: &DocumentationMetrics,
) -> ProjectHealthMetrics {
    let mut metrics = ProjectHealthMetrics::default();

    // Calculate scores
    metrics.code_quality_score = calculate_code_quality_score(code_quality);
    metrics.performance_score = calculate_performance_score(performance);
    metrics.security_score = calculate_security_score(security);
    metrics.documentation_score = calculate_documentation_score(documentation);

    // Calculate overall health
    metrics.overall_health = (metrics.code_quality_score * 0.25
        + metrics.performance_score * 0.25
        + metrics.security_score * 0.3
        + metrics.documentation_score * 0.2);

    // Add sample data points for trends
    for i in 0..10 {
        let timestamp = Utc::now() - chrono::Duration::days(i);

        metrics.health_trends.data_points.push(MetricDataPoint {
            timestamp,
            value: metrics.overall_health * (1.0 - (i as f64 * 0.01)),
            label: None,
        });
    }

    // Reverse data points to have chronological order
    metrics.health_trends.data_points.reverse();

    // Set status based on overall health
    let status_level = if metrics.overall_health >= 80.0 {
        StatusLevel::Success
    } else if metrics.overall_health >= 60.0 {
        StatusLevel::Info
    } else if metrics.overall_health >= 40.0 {
        StatusLevel::Warning
    } else {
        StatusLevel::Error
    };

    metrics.status = Status {
        level: status_level,
        message: "Project health metrics calculated".to_string(),
        details: Some(format!("Overall health: {:.1}%", metrics.overall_health)),
        timestamp: Utc::now(),
    };

    metrics.last_updated = Utc::now();

    metrics
}

/// Calculate code quality score
fn calculate_code_quality_score(metrics: &CodeQualityMetrics) -> f64 {
    // In a real implementation, this would use a more sophisticated algorithm
    let warning_score = 100.0 * (1.0 - (metrics.warning_density / 20.0).min(1.0));
    let test_coverage_score = metrics.test_coverage;
    let doc_coverage_score = metrics.doc_coverage;

    (warning_score * 0.3 + test_coverage_score * 0.4 + doc_coverage_score * 0.3)
        .min(100.0)
        .max(0.0)
}

/// Calculate performance score
fn calculate_performance_score(metrics: &PerformanceMetrics) -> f64 {
    // In a real implementation, this would use a more sophisticated algorithm
    let regression_penalty = metrics
        .regressions
        .iter()
        .filter(|r| r.status.level == StatusLevel::Warning)
        .map(|r| r.change_percentage)
        .sum::<f64>();

    (100.0 - regression_penalty).min(100.0).max(0.0)
}

/// Calculate security score
fn calculate_security_score(metrics: &SecurityMetrics) -> f64 {
    // In a real implementation, this would use a more sophisticated algorithm
    let critical_penalty = metrics.critical_issues as f64 * 20.0;
    let high_penalty = metrics.high_issues as f64 * 10.0;
    let medium_penalty = metrics.medium_issues as f64 * 5.0;
    let low_penalty = metrics.low_issues as f64 * 1.0;

    (100.0 - critical_penalty - high_penalty - medium_penalty - low_penalty)
        .min(100.0)
        .max(0.0)
}

/// Calculate documentation score
fn calculate_documentation_score(metrics: &DocumentationMetrics) -> f64 {
    // In a real implementation, this would use a more sophisticated algorithm
    metrics.overall_coverage
}
