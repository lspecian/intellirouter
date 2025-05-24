//! Data structures for the dashboard
//!
//! This module defines the data structures used by the dashboard to store and display metrics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::metrics::{
    CodeQualityMetrics, DocumentationMetrics, PerformanceMetrics, ProjectHealthMetrics,
    SecurityMetrics,
};

/// Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// Code quality metrics
    pub code_quality: CodeQualityMetrics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Security metrics
    pub security: SecurityMetrics,
    /// Documentation metrics
    pub documentation: DocumentationMetrics,
    /// Project health metrics
    pub project_health: ProjectHealthMetrics,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Label
    pub label: Option<String>,
}

/// Metric series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    /// Name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Unit
    pub unit: Option<String>,
    /// Data points
    pub data_points: Vec<MetricDataPoint>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Metric chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricChart {
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Type
    pub chart_type: String,
    /// Series
    pub series: Vec<MetricSeries>,
    /// Options
    pub options: HashMap<String, String>,
}

/// Status level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusLevel {
    /// Critical
    Critical,
    /// Error
    Error,
    /// Warning
    Warning,
    /// Info
    Info,
    /// Success
    Success,
}

impl StatusLevel {
    /// Get the color for this status level
    pub fn color(&self) -> &'static str {
        match self {
            StatusLevel::Critical => "#d9534f",
            StatusLevel::Error => "#f0ad4e",
            StatusLevel::Warning => "#f0ad4e",
            StatusLevel::Info => "#5bc0de",
            StatusLevel::Success => "#5cb85c",
        }
    }

    /// Get the name for this status level
    pub fn name(&self) -> &'static str {
        match self {
            StatusLevel::Critical => "Critical",
            StatusLevel::Error => "Error",
            StatusLevel::Warning => "Warning",
            StatusLevel::Info => "Info",
            StatusLevel::Success => "Success",
        }
    }
}

/// Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Level
    pub level: StatusLevel,
    /// Message
    pub message: String,
    /// Details
    pub details: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Priority
    pub priority: u32,
    /// Status
    pub status: Status,
    /// Related metrics
    pub related_metrics: Vec<String>,
}
